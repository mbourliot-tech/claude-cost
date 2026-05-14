use crate::{parser, storage::Store};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, warn};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct ScanReport {
    pub sessions_seen: usize,
    pub unique_calls: usize,
    pub new_calls: usize,
    pub elapsed_ms: u128,
    pub total_cost_usd: f64,
}

pub fn scan_all(projects_dir: &Path, store: &Store) -> Result<ScanReport> {
    let started = Instant::now();
    let mut report = ScanReport::default();
    if !projects_dir.exists() {
        warn!(dir = %projects_dir.display(), "projects directory does not exist");
        report.elapsed_ms = started.elapsed().as_millis();
        return Ok(report);
    }

    let overrides = store.price_overrides_map()?;
    let mut seen_ids: HashSet<String> = HashSet::new();
    for entry in WalkDir::new(projects_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        report.sessions_seen += 1;
        match parser::parse_file(path, &mut seen_ids, &overrides) {
            Ok(records) => {
                report.unique_calls += records.len();
                report.total_cost_usd += records.iter().map(|r| r.cost_usd).sum::<f64>();
                let new_rows = store.insert_batch(&records)?;
                report.new_calls += new_rows;
                debug!(file = %path.display(), records = records.len(), new = new_rows, "parsed");
            }
            Err(e) => {
                warn!(file = %path.display(), error = %e, "failed to parse file");
            }
        }
    }
    report.elapsed_ms = started.elapsed().as_millis();
    Ok(report)
}
