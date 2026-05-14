use crate::{parser, storage::Store};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use std::time::{Instant, UNIX_EPOCH};
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

        // Scan incrémental : ne relit que les fichiers modifiés
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!(file = %path.display(), error = %e, "cannot read metadata, scanning anyway");
                // Scan quand même si métadonnées indisponibles
                let _ = do_scan(path, &mut seen_ids, &overrides, store, &mut report);
                continue;
            }
        };
        let mtime_secs = meta.modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let file_size = meta.len();
        let path_str = path.to_string_lossy().to_string();

        match store.file_needs_scan(&path_str, mtime_secs, file_size) {
            Ok(true) => {
                report.sessions_seen += 1;
                if do_scan(path, &mut seen_ids, &overrides, store, &mut report).is_ok() {
                    let _ = store.update_file_cache(&path_str, mtime_secs, file_size);
                }
            }
            Ok(false) => {
                debug!(file = %path.display(), "skipped (unchanged)");
            }
            Err(e) => {
                warn!(file = %path.display(), error = %e, "file_cache check failed, scanning anyway");
                report.sessions_seen += 1;
                let _ = do_scan(path, &mut seen_ids, &overrides, store, &mut report);
            }
        }
    }
    report.elapsed_ms = started.elapsed().as_millis();
    Ok(report)
}

fn do_scan(
    path: &Path,
    seen_ids: &mut HashSet<String>,
    overrides: &std::collections::HashMap<String, crate::pricing::ModelPrice>,
    store: &Store,
    report: &mut ScanReport,
) -> Result<()> {
    match parser::parse_file(path, seen_ids, overrides) {
        Ok(records) => {
            report.unique_calls += records.len();
            report.total_cost_usd += records.iter().map(|r| r.cost_usd).sum::<f64>();
            let new_rows = store.insert_batch(&records)?;
            report.new_calls += new_rows;
            debug!(file = %path.display(), records = records.len(), new = new_rows, "parsed");
            Ok(())
        }
        Err(e) => {
            warn!(file = %path.display(), error = %e, "failed to parse file");
            Err(e)
        }
    }
}
