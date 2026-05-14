use crate::pricing::{self, ModelPrice};
use crate::types::{RawLine, UsageRecord};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{trace, warn};

const SYNTHETIC_MODEL: &str = "<synthetic>";

/// Parse one JSONL transcript file and return the deduplicated usage records.
/// The `seen_ids` set is updated in place so the caller can deduplicate across files
/// (rare but possible if Claude Code copies state between sessions).
pub fn parse_file(
    path: &Path,
    seen_ids: &mut HashSet<String>,
    overrides: &HashMap<String, ModelPrice>,
) -> Result<Vec<UsageRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                warn!(file = %path.display(), line = idx + 1, error = %e, "io error reading line, stopping file");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let raw: RawLine = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                trace!(file = %path.display(), line = idx + 1, error = %e, "skipping unparseable line");
                continue;
            }
        };
        if let Some(rec) = build_record(raw, path, seen_ids, overrides) {
            records.push(rec);
        }
    }

    Ok(records)
}

fn build_record(raw: RawLine, path: &Path, seen_ids: &mut HashSet<String>, overrides: &HashMap<String, ModelPrice>) -> Option<UsageRecord> {
    if raw.kind.as_deref() != Some("assistant") {
        return None;
    }
    let message = raw.message?;
    let model = message.model?;
    if model == SYNTHETIC_MODEL {
        return None;
    }
    let usage = message.usage?;
    if usage.is_empty() {
        return None;
    }
    let message_id = message.id?;
    if !seen_ids.insert(message_id.clone()) {
        return None;
    }

    let (c5m, c1h) = usage.cache_split();
    let session_id = raw.session_id.unwrap_or_else(|| session_id_from_path(path));
    let project_path = raw.cwd.unwrap_or_else(|| project_path_from_file(path));
    let timestamp = raw.timestamp.unwrap_or_default();
    let cost_usd = pricing::effective_cost(&usage, &model, overrides);

    if pricing::effective_price(&model, overrides).is_none() {
        warn!(model = %model, "unknown model — pricing 0; update pricing.rs or add an override via the dashboard");
    }

    let (web_search, web_fetch) = usage.server_tool_use.as_ref()
        .map(|s| (s.web_search_requests, s.web_fetch_requests))
        .unwrap_or((0, 0));

    Some(UsageRecord {
        message_id,
        session_id,
        project_path,
        timestamp,
        model,
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cache_read_tokens: usage.cache_read_input_tokens,
        cache_5m_tokens: c5m,
        cache_1h_tokens: c1h,
        cost_usd,
        service_tier: usage.service_tier,
        speed: usage.speed,
        web_search_requests: web_search,
        web_fetch_requests: web_fetch,
    })
}

fn session_id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn project_path_from_file(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(decode_project_dir)
        .unwrap_or_else(|| "unknown".to_string())
}

/// Decode a Claude Code project directory name like `C--Users-miche-kDrive-Dev-CCR-test`
/// back to a path-like form. This is a fallback only — prefer `cwd` from the JSONL line.
fn decode_project_dir(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '-' && chars.peek() == Some(&'-') {
            chars.next();
            out.push(':');
            out.push('\\');
        } else if c == '-' {
            out.push('\\');
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(content: &str) -> (tempdir::Path, std::path::PathBuf) {
        // Use std::env::temp_dir to avoid a dep on tempfile.
        let dir = std::env::temp_dir();
        let file_name = format!("cc-test-{}.jsonl", uuid_like());
        let path = dir.join(&file_name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (dir, path)
    }

    fn uuid_like() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        format!("{}-{seq}-{nanos:x}", std::process::id())
    }

    mod tempdir {
        pub type Path = std::path::PathBuf;
    }

    #[test]
    fn parses_assistant_line_with_usage() {
        // Must stay on one physical line — parse_file is line-oriented (JSONL).
        let json = r#"{"type":"assistant","uuid":"u1","timestamp":"2026-05-01T15:49:28.838Z","sessionId":"sess-1","cwd":"C:\\proj","message":{"id":"msg_abc","model":"claude-sonnet-4-6","role":"assistant","usage":{"input_tokens":3,"output_tokens":19,"cache_creation_input_tokens":28307,"cache_read_input_tokens":0,"cache_creation":{"ephemeral_5m_input_tokens":28307,"ephemeral_1h_input_tokens":0}}}}"#;
        let (_d, p) = write_tmp(json);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 1);
        let r = &recs[0];
        assert_eq!(r.message_id, "msg_abc");
        assert_eq!(r.model, "claude-sonnet-4-6");
        assert_eq!(r.session_id, "sess-1");
        assert_eq!(r.project_path, "C:\\proj");
        assert_eq!(r.input_tokens, 3);
        assert_eq!(r.output_tokens, 19);
        assert_eq!(r.cache_5m_tokens, 28307);
        assert_eq!(r.cache_1h_tokens, 0);
        // cost = (3 * 3 + 28307 * 3 * 1.25 + 19 * 15) / 1e6
        let expected = (3.0 * 3.0 + 28307.0 * 3.0 * 1.25 + 19.0 * 15.0) / 1_000_000.0;
        assert!((r.cost_usd - expected).abs() < 1e-9);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn deduplicates_repeated_message_id_within_file() {
        // Same message.id on two consecutive lines (thinking + text blocks) must count once.
        let line = r#"{"type":"assistant","sessionId":"s","cwd":"/p","message":{"id":"msg_dup","model":"claude-sonnet-4-6","usage":{"input_tokens":1,"output_tokens":1}}}"#;
        let content = format!("{line}\n{line}\n");
        let (_d, p) = write_tmp(&content);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn skips_synthetic_model() {
        let json = r#"{"type":"assistant","message":{"id":"m1","model":"<synthetic>","usage":{"input_tokens":1}}}"#;
        let (_d, p) = write_tmp(json);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn skips_user_messages_and_attachments() {
        let content = "\
{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"hi\"}}\n\
{\"type\":\"attachment\",\"subtype\":\"skill_listing\"}\n\
{\"type\":\"permission-mode\",\"mode\":\"plan\"}\n";
        let (_d, p) = write_tmp(content);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn tolerates_corrupt_line_and_continues() {
        let content = "\
not valid json at all\n\
{\"type\":\"assistant\",\"message\":{\"id\":\"m_ok\",\"model\":\"claude-sonnet-4-6\",\"usage\":{\"output_tokens\":1}}}\n";
        let (_d, p) = write_tmp(content);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].message_id, "m_ok");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn skips_assistant_with_all_zero_usage() {
        let json = r#"{"type":"assistant","message":{"id":"m_zero","model":"claude-sonnet-4-6","usage":{"input_tokens":0,"output_tokens":0}}}"#;
        let (_d, p) = write_tmp(json);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 0);
        let _ = std::fs::remove_file(&p);
    }
}
