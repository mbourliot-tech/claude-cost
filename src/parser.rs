use crate::pricing::{self, ModelPrice};
use crate::types::{RawLine, UsageRecord};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{trace, warn};

const SYNTHETIC_MODEL: &str = "<synthetic>";

/// Estimated input tokens for the away_summary prompt (system + recent turns).
const AWAY_SUMMARY_INPUT_TOKENS: u64 = 500;
/// Flat input token estimate for agent-name and ai-title generation (lightweight task).
const INTERNAL_LIGHTWEIGHT_INPUT: u64 = 500;
/// Lightweight model used for agent-name / ai-title generation (Haiku-class task).
const INTERNAL_LIGHTWEIGHT_MODEL: &str = "claude-haiku-4-5-20251001";

/// Parse one JSONL transcript file and return the deduplicated usage records.
/// The `seen_ids` set is updated in place so the caller can deduplicate across files
/// (rare but possible if Claude Code copies state between sessions).
///
/// Within a single file, the LAST occurrence of each `message_id` is used. Claude Code
/// emits the same id multiple times during streaming: the first occurrence carries only
/// the initial partial token count; the final occurrence has the complete billable count.
/// Using the first occurrence would systematically undercount output tokens and cost.
pub fn parse_file(
    path: &Path,
    seen_ids: &mut HashSet<String>,
    overrides: &HashMap<String, ModelPrice>,
) -> Result<Vec<UsageRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // First pass: collect only the last occurrence of each message_id in this file.
    let mut last_by_id: HashMap<String, RawLine> = HashMap::new();

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
        if raw.kind.as_deref() != Some("assistant") {
            continue;
        }
        match raw.message.as_ref().and_then(|m| m.id.as_ref()) {
            Some(id) => { last_by_id.insert(id.clone(), raw); }
            None => {} // build_record requires message.id — will return None
        }
    }

    // Second pass: build records from the last occurrences (cross-file dedup via seen_ids).
    let mut records = Vec::new();
    for (_, raw) in last_by_id {
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
        is_estimate: false,
    })
}

/// Traverse the parentUuid chain to find the nearest preceding assistant message.
/// Returns (cache_read_input_tokens, model_string).
fn find_preceding_assistant(
    start_uuid: &str,
    by_uuid: &HashMap<String, serde_json::Value>,
) -> Option<(u64, String)> {
    let mut cur = start_uuid.to_string();
    for _ in 0..10 {
        let node = by_uuid.get(&cur)?;
        if node.get("type").and_then(|t| t.as_str()) == Some("assistant") {
            let model = node
                .pointer("/message/model")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // Skip <synthetic> entries — they are not real API calls.
            if model == SYNTHETIC_MODEL || model.is_empty() {
                cur = node.get("parentUuid")?.as_str()?.to_string();
                continue;
            }
            let cache_read = node
                .pointer("/message/usage/cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Some((cache_read, model.to_string()));
        }
        cur = node.get("parentUuid")?.as_str()?.to_string();
    }
    None
}

/// Estimate costs for internal Claude Code API calls that do not appear as
/// `type: "assistant"` messages in the JSONL but ARE billed by Anthropic.
/// Records are tagged `is_estimate = true` so they can be filtered out.
///
/// Sources estimated:
///   away_summary  — session summarisation call triggered when the user leaves.
///     cache_read  = total session cache_creation tokens (full accumulated context).
///     output      = content.chars().count() / 4
///     input       = AWAY_SUMMARY_INPUT_TOKENS (flat prompt)
///     model       = same as the preceding assistant message in the UUID chain
///
///   agent-name    — lightweight call to name a spawned sub-agent.
///   ai-title      — lightweight call to title a conversation turn.
///     Both use INTERNAL_LIGHTWEIGHT_MODEL, flat input/output token counts.
pub fn parse_internal_estimates(
    path: &Path,
    seen_ids: &mut HashSet<String>,
    overrides: &HashMap<String, ModelPrice>,
) -> Result<Vec<UsageRecord>> {
    use crate::types::RawUsage;
    use serde_json::Value;

    let content = std::fs::read_to_string(path)?;

    // Single pass: build UUID index, collect internal entries, accumulate session stats.
    // cache_by_id keeps only the LAST cache_creation value per message_id so that
    // streaming duplicates don't inflate the away_summary cache_read estimate.
    // away_summary entries appear at the tail of the file (after all assistant messages),
    // so we sum the deduplicated totals after the pass and assign to every summary.
    let mut by_uuid: HashMap<String, Value> = HashMap::new();
    let mut away_summaries: Vec<Value> = Vec::new();
    let mut agent_names: Vec<Value> = Vec::new();
    let mut ai_titles: Vec<Value> = Vec::new();
    // Deduplicated cache_creation per message_id (last occurrence wins).
    let mut cache_by_id: HashMap<String, u64> = HashMap::new();
    let mut session_id_default = String::new();
    let mut project_path_default = project_path_from_file(path);
    let mut primary_model = String::from("claude-sonnet-4-6");

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let v: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(uuid) = v.get("uuid").and_then(|u| u.as_str()) {
            by_uuid.insert(uuid.to_string(), v.clone());
        }
        match v.get("type").and_then(|t| t.as_str()) {
            Some("system") if v.get("subtype").and_then(|s| s.as_str()) == Some("away_summary") => {
                away_summaries.push(v);
            }
            Some("agent-name") => { agent_names.push(v); }
            Some("ai-title")   => { ai_titles.push(v); }
            Some("assistant") => {
                if let (Some(msg_id), Some(cc)) = (
                    v.pointer("/message/id").and_then(|x| x.as_str()),
                    v.pointer("/message/usage/cache_creation_input_tokens").and_then(|x| x.as_u64()),
                ) {
                    cache_by_id.insert(msg_id.to_string(), cc); // last wins
                }
                if session_id_default.is_empty() {
                    if let Some(sid) = v.get("sessionId").and_then(|s| s.as_str()) {
                        session_id_default = sid.to_string();
                    }
                }
                if let Some(cwd) = v.get("cwd").and_then(|c| c.as_str()) {
                    project_path_default = cwd.to_string();
                }
                if let Some(m) = v.pointer("/message/model").and_then(|x| x.as_str()) {
                    if m != SYNTHETIC_MODEL { primary_model = m.to_string(); }
                }
            }
            _ => {}
        }
    }

    if away_summaries.is_empty() && agent_names.is_empty() && ai_titles.is_empty() {
        return Ok(Vec::new());
    }

    // Total unique cache_creation for this file (deduplicated).
    let total_cache: u64 = cache_by_id.values().sum();

    let mut records = Vec::new();

    // ── away_summary ──────────────────────────────────────────────────────────
    for summary in &away_summaries {
        let uuid = match summary.get("uuid").and_then(|u| u.as_str()) {
            Some(u) => u, None => continue,
        };
        let estimate_id = format!("estimate:{uuid}");
        if !seen_ids.insert(estimate_id.clone()) { continue; }

        let parent_uuid = match summary.get("parentUuid").and_then(|p| p.as_str()) {
            Some(p) => p, None => continue,
        };
        // Model from the preceding assistant turn; fall back to file's primary model.
        let (_, model) = find_preceding_assistant(parent_uuid, &by_uuid)
            .unwrap_or((0, primary_model.clone()));

        let text = summary.get("content").and_then(|c| c.as_str()).unwrap_or("");
        let output_tokens = (text.chars().count() as u64 / 4).max(1);
        let timestamp  = summary.get("timestamp").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let session_id = summary.get("sessionId").and_then(|s| s.as_str()).unwrap_or(&session_id_default).to_string();
        let project_path = summary.get("cwd").and_then(|c| c.as_str())
            .map(|s| s.to_string()).unwrap_or_else(|| project_path_default.clone());

        let raw_usage = RawUsage {
            input_tokens: AWAY_SUMMARY_INPUT_TOKENS,
            output_tokens,
            cache_read_input_tokens: total_cache,
            cache_creation_input_tokens: 0,
            cache_creation: None,
            service_tier: None, speed: None, server_tool_use: None,
        };
        let cost_usd = pricing::effective_cost(&raw_usage, &model, overrides);
        records.push(UsageRecord {
            message_id: estimate_id, session_id, project_path, timestamp, model,
            input_tokens: AWAY_SUMMARY_INPUT_TOKENS, output_tokens,
            cache_read_tokens: total_cache, cache_5m_tokens: 0, cache_1h_tokens: 0,
            cost_usd, service_tier: None, speed: None,
            web_search_requests: 0, web_fetch_requests: 0, is_estimate: true,
        });
    }

    // ── agent-name  (Haiku-class, flat tokens, no cache read) ─────────────────
    for (idx, entry) in agent_names.iter().enumerate() {
        let name = entry.get("agentName").and_then(|n| n.as_str()).unwrap_or("?");
        let sid  = entry.get("sessionId").and_then(|s| s.as_str()).unwrap_or(&session_id_default);
        let estimate_id = format!("estimate:agent-name:{sid}:{idx}:{name}");
        if !seen_ids.insert(estimate_id.clone()) { continue; }
        let output_tokens = (name.chars().count() as u64 / 4).max(1);
        let raw = RawUsage {
            input_tokens: INTERNAL_LIGHTWEIGHT_INPUT, output_tokens,
            cache_read_input_tokens: 0, cache_creation_input_tokens: 0,
            cache_creation: None, service_tier: None, speed: None, server_tool_use: None,
        };
        let cost_usd = pricing::effective_cost(&raw, INTERNAL_LIGHTWEIGHT_MODEL, overrides);
        records.push(UsageRecord {
            message_id: estimate_id,
            session_id: sid.to_string(),
            project_path: project_path_default.clone(),
            timestamp: String::new(),
            model: INTERNAL_LIGHTWEIGHT_MODEL.to_string(),
            input_tokens: INTERNAL_LIGHTWEIGHT_INPUT, output_tokens,
            cache_read_tokens: 0, cache_5m_tokens: 0, cache_1h_tokens: 0,
            cost_usd, service_tier: None, speed: None,
            web_search_requests: 0, web_fetch_requests: 0, is_estimate: true,
        });
    }

    // ── ai-title  (Haiku-class, flat tokens, no cache read) ───────────────────
    for (idx, entry) in ai_titles.iter().enumerate() {
        let title = entry.get("aiTitle").and_then(|t| t.as_str()).unwrap_or("?");
        let sid   = entry.get("sessionId").and_then(|s| s.as_str()).unwrap_or(&session_id_default);
        let estimate_id = format!("estimate:ai-title:{sid}:{idx}:{title}");
        if !seen_ids.insert(estimate_id.clone()) { continue; }
        let output_tokens = (title.chars().count() as u64 / 4).max(1);
        let raw = RawUsage {
            input_tokens: INTERNAL_LIGHTWEIGHT_INPUT, output_tokens,
            cache_read_input_tokens: 0, cache_creation_input_tokens: 0,
            cache_creation: None, service_tier: None, speed: None, server_tool_use: None,
        };
        let cost_usd = pricing::effective_cost(&raw, INTERNAL_LIGHTWEIGHT_MODEL, overrides);
        records.push(UsageRecord {
            message_id: estimate_id,
            session_id: sid.to_string(),
            project_path: project_path_default.clone(),
            timestamp: String::new(),
            model: INTERNAL_LIGHTWEIGHT_MODEL.to_string(),
            input_tokens: INTERNAL_LIGHTWEIGHT_INPUT, output_tokens,
            cache_read_tokens: 0, cache_5m_tokens: 0, cache_1h_tokens: 0,
            cost_usd, service_tier: None, speed: None,
            web_search_requests: 0, web_fetch_requests: 0, is_estimate: true,
        });
    }

    Ok(records)
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
    fn uses_last_occurrence_of_message_id_for_streaming_completion() {
        // Claude Code emits the same message_id twice during streaming:
        // first with partial output_tokens, then with the final complete count.
        // The last occurrence must win so the billed cost is not understated.
        let partial = r#"{"type":"assistant","sessionId":"s","cwd":"/p","message":{"id":"msg_stream","model":"claude-sonnet-4-6","usage":{"input_tokens":1,"output_tokens":63}}}"#;
        let complete = r#"{"type":"assistant","sessionId":"s","cwd":"/p","message":{"id":"msg_stream","model":"claude-sonnet-4-6","usage":{"input_tokens":1,"output_tokens":213}}}"#;
        let content = format!("{partial}\n{complete}\n");
        let (_d, p) = write_tmp(&content);
        let mut seen = HashSet::new();
        let recs = parse_file(&p, &mut seen, &Default::default()).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].output_tokens, 213, "must use final (last) output count");
        let expected_cost = (1.0 * 3.0 + 213.0 * 15.0) / 1_000_000.0;
        assert!((recs[0].cost_usd - expected_cost).abs() < 1e-9);
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
