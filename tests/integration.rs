//! End-to-end tests: scanner walks JSONL fixtures, parser extracts records,
//! storage persists them, and aggregate queries return the expected shape.

use claude_cost::{scanner, storage::Store, types::UsageRecord};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const EPS: f64 = 1e-9;

// Expected per-message costs derived by hand from the fixtures. Keeping them
// here as constants makes regressions in pricing.rs surface immediately.
const COST_MSG_A1: f64 = 0.100_500; // sonnet-4-6  : 1k in + 2k out + 10k c5m + 5k c1h
const COST_MSG_A2: f64 = 0.050_000; // opus-4-7    : 500 in + 1.5k out + 20k c_read
const COST_MSG_A3: f64 = 0.051_000; // sonnet-4-6  : 2k in + 3k out
const COST_MSG_B1: f64 = 0.035_000; // haiku-4-5   : 10k in + 5k out
const COST_MSG_B2: f64 = 0.018_300; // sonnet-4-6  : 100 in + 200 out + 50k c_read
const COST_MSG_B5: f64 = 0.000_000; // unknown model -> $0 (still recorded)

const PROJ_A_COST: f64 = COST_MSG_A1 + COST_MSG_A2 + COST_MSG_A3;
const PROJ_B_COST: f64 = COST_MSG_B1 + COST_MSG_B2 + COST_MSG_B5;
const TOTAL_COST: f64 = PROJ_A_COST + PROJ_B_COST;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("projects")
}

/// Each test gets a fresh on-disk SQLite file. Tests run in parallel, so the
/// name must be unique per call — combine PID, a monotonic counter, and nanos.
fn tmp_db_path() -> PathBuf {
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("claude-cost-it-{pid}-{seq}-{nanos:x}.sqlite"))
}

struct TestDb {
    path: PathBuf,
    pub store: Store,
}

impl TestDb {
    fn new() -> Self {
        let path = tmp_db_path();
        let store = Store::open(&path).expect("open store");
        Self { path, store }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // Best-effort cleanup; SQLite WAL files share the prefix.
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(self.path.with_extension("sqlite-shm"));
    }
}

#[test]
fn scan_report_matches_hand_computed_totals() {
    let db = TestDb::new();
    let report = scanner::scan_all(&fixtures_dir(), &db.store).expect("scan");

    assert_eq!(report.sessions_seen, 2, "two .jsonl files in fixtures");
    assert_eq!(report.unique_calls, 6, "6 billable assistant lines after dedup/skip");
    assert_eq!(report.new_calls, 6, "first scan: all rows are new");
    assert!(
        (report.total_cost_usd - TOTAL_COST).abs() < EPS,
        "total cost {} vs expected {}",
        report.total_cost_usd,
        TOTAL_COST
    );
}

#[test]
fn second_scan_is_idempotent() {
    let db = TestDb::new();
    let r1 = scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let r2 = scanner::scan_all(&fixtures_dir(), &db.store).unwrap();

    assert_eq!(r1.new_calls, 6);
    assert_eq!(r2.new_calls, 0, "rescan must not insert duplicates");
    // Avec le scan incrémental, les fichiers inchangés sont sautés entièrement
    assert_eq!(r2.unique_calls, 0, "unchanged files are skipped by incremental scanner");
    assert_eq!(r2.sessions_seen, 0, "no files re-parsed when mtime unchanged");
}

#[test]
fn summary_aggregates_match_scan_totals() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let s = db.store.summary(None, None).unwrap();

    assert_eq!(s.calls, 6);
    assert_eq!(s.sessions, 2);
    assert!((s.total_cost_usd - TOTAL_COST).abs() < EPS);

    // Tokens: a1(1000) + a2(500) + a3(2000) + b1(10000) + b2(100) + b5(1000) = 14600 input
    //         a1(2000) + a2(1500) + a3(3000) + b1(5000) + b2(200) + b5(1000) = 12700 output
    assert_eq!(s.input_tokens, 14_600);
    assert_eq!(s.output_tokens, 12_700);
    assert_eq!(s.cache_read_tokens, 70_000, "20k (a2) + 50k (b2)");
    assert_eq!(s.cache_5m_tokens, 10_000, "only a1 had a 5m breakdown");
    assert_eq!(s.cache_1h_tokens, 5_000, "only a1 had a 1h breakdown");
}

#[test]
fn by_model_groups_and_sums_correctly() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let rows = db.store.by_model(None, None).unwrap();

    let model = |name: &str| {
        rows.iter()
            .find(|r| r.model == name)
            .unwrap_or_else(|| panic!("missing model {name} in by_model"))
    };

    // 4 distinct models survive parsing (synthetic + zero-usage are filtered out)
    assert_eq!(rows.len(), 4, "expected 4 distinct models, got {rows:?}");

    let sonnet = model("claude-sonnet-4-6");
    assert_eq!(sonnet.calls, 3);
    assert!((sonnet.cost_usd - (COST_MSG_A1 + COST_MSG_A3 + COST_MSG_B2)).abs() < EPS);

    let opus = model("claude-opus-4-7");
    assert_eq!(opus.calls, 1);
    assert!((opus.cost_usd - COST_MSG_A2).abs() < EPS);

    let haiku = model("claude-haiku-4-5");
    assert_eq!(haiku.calls, 1);
    assert!((haiku.cost_usd - COST_MSG_B1).abs() < EPS);

    let unknown = model("claude-future-99");
    assert_eq!(unknown.calls, 1, "unknown model is still recorded");
    assert_eq!(unknown.cost_usd, 0.0, "but priced at zero");
}

#[test]
fn by_project_separates_proj_a_and_proj_b() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let rows = db.store.by_project(None, None).unwrap();

    assert_eq!(rows.len(), 2, "two distinct cwd values in fixtures");
    let a = rows.iter().find(|r| r.project_path == "C:\\proj-a").expect("proj-a");
    let b = rows.iter().find(|r| r.project_path == "C:\\proj-b").expect("proj-b");

    assert_eq!(a.calls, 3);
    assert_eq!(a.sessions, 1);
    assert!((a.cost_usd - PROJ_A_COST).abs() < EPS);

    assert_eq!(b.calls, 3);
    assert_eq!(b.sessions, 1);
    assert!((b.cost_usd - PROJ_B_COST).abs() < EPS);
}

#[test]
fn by_session_returns_both_sessions_ordered_by_recency() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let rows = db.store.by_session(None, 50).unwrap();

    assert_eq!(rows.len(), 2);
    // sess-bbb is on 2026-05-11, sess-aaa on 2026-05-10 → bbb first (DESC by MAX(ts))
    assert_eq!(rows[0].session_id, "sess-bbb");
    assert_eq!(rows[1].session_id, "sess-aaa");

    assert_eq!(rows[0].calls, 3);
    assert!((rows[0].cost_usd - PROJ_B_COST).abs() < EPS);
    assert_eq!(rows[1].calls, 3);
    assert!((rows[1].cost_usd - PROJ_A_COST).abs() < EPS);
}

#[test]
fn by_session_filters_by_project() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let rows = db.store.by_session(Some("C:\\proj-a"), 50).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].session_id, "sess-aaa");
}

#[test]
fn since_filter_excludes_older_project() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let s = db.store.summary(Some("2026-05-11T00:00:00Z"), None).unwrap();
    assert_eq!(s.calls, 3, "only proj-b records on or after 2026-05-11");
    assert!((s.total_cost_usd - PROJ_B_COST).abs() < EPS);
}

#[test]
fn until_filter_excludes_newer_project() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let s = db.store.summary(None, Some("2026-05-10T23:59:59Z")).unwrap();
    assert_eq!(s.calls, 3, "only proj-a records on or before 2026-05-10");
    assert!((s.total_cost_usd - PROJ_A_COST).abs() < EPS);
}

#[test]
fn last_timestamp_is_max_in_fixtures() {
    let db = TestDb::new();
    scanner::scan_all(&fixtures_dir(), &db.store).unwrap();
    let ts = db.store.last_timestamp().unwrap().expect("non-empty after scan");
    assert_eq!(ts, "2026-05-11T14:04:00Z");
}

#[test]
fn last_timestamp_is_none_on_empty_store() {
    let db = TestDb::new();
    // no scan
    assert!(db.store.last_timestamp().unwrap().is_none());
}

#[test]
fn empty_projects_dir_yields_zero_report() {
    let db = TestDb::new();
    let empty = std::env::temp_dir().join(format!("claude-cost-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let report = scanner::scan_all(&empty, &db.store).unwrap();
    assert_eq!(report.sessions_seen, 0);
    assert_eq!(report.unique_calls, 0);
    assert_eq!(report.new_calls, 0);
    assert_eq!(report.total_cost_usd, 0.0);
    let _ = std::fs::remove_dir(&empty);
}

/// When `pricing.rs` changes (new model, tier adjustment), the next scan must
/// refresh `cost_usd` on rows already in the DB — otherwise historical totals
/// stay stuck at the price that was in effect at first insertion.
#[test]
fn reinsert_with_different_cost_repricies_existing_row() {
    let db = TestDb::new();
    let mut rec = UsageRecord {
        message_id: "msg-reprice-1".into(),
        session_id: "sess-x".into(),
        project_path: "C:\\proj-x".into(),
        timestamp: "2026-05-14T12:00:00Z".into(),
        model: "mimo-v2.5-pro".into(),
        input_tokens: 1_000_000,
        output_tokens: 1_000_000,
        cache_read_tokens: 0,
        cache_5m_tokens: 0,
        cache_1h_tokens: 0,
        cost_usd: 4.0, // old tier ($1/$3)
        service_tier: None,
        speed: None,
        web_search_requests: 0,
        web_fetch_requests: 0,
    };
    let n1 = db.store.insert_batch(&[rec.clone()]).unwrap();
    assert_eq!(n1, 1, "first insert counts as a touched row");
    assert!((db.store.summary(None, None).unwrap().total_cost_usd - 4.0).abs() < EPS);

    // Re-insert same message_id with new cost (e.g. after pricing.rs tier bump)
    rec.cost_usd = 8.0;
    let n2 = db.store.insert_batch(&[rec.clone()]).unwrap();
    assert_eq!(n2, 1, "reprice must count as touched");
    assert!(
        (db.store.summary(None, None).unwrap().total_cost_usd - 8.0).abs() < EPS,
        "summary must reflect the new cost, not the old one"
    );

    // Third re-insert with identical cost is a no-op (preserves idempotence).
    let n3 = db.store.insert_batch(&[rec]).unwrap();
    assert_eq!(n3, 0, "no-op rescan must not count as touched");
}

#[test]
fn missing_projects_dir_does_not_error() {
    let db = TestDb::new();
    let ghost = std::env::temp_dir().join(format!("claude-cost-ghost-{}-x", std::process::id()));
    let report = scanner::scan_all(&ghost, &db.store).expect("scan should tolerate missing dir");
    assert_eq!(report.sessions_seen, 0);
}

// ── Tests API HTTP ────────────────────────────────────────────────────────────

use axum::{body::Body, http::Request};
use claude_cost::api;
use std::sync::Arc;
use tower::ServiceExt;

async fn build_app() -> (axum::Router, TestDb) {
    let db = TestDb::new();
    let store = Arc::new(claude_cost::storage::Store::open(&db.path).unwrap());
    let app = api::router(store, std::env::temp_dir());
    (app, db)
}

async fn get_json(app: axum::Router, uri: &str) -> serde_json::Value {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "GET {uri} should return 200");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn api_summary_empty_store() {
    let (app, _db) = build_app().await;
    let v = get_json(app, "/api/summary").await;
    assert_eq!(v["total_cost_usd"].as_f64().unwrap(), 0.0);
    assert_eq!(v["calls"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn api_by_weekday_returns_seven_entries() {
    let (app, _db) = build_app().await;
    let v = get_json(app, "/api/by-weekday").await;
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 7, "by-weekday must return exactly 7 entries");
    assert_eq!(arr[0]["label"].as_str().unwrap(), "Lun");
    assert_eq!(arr[6]["label"].as_str().unwrap(), "Dim");
}

#[tokio::test]
async fn api_by_hourofday_returns_24_entries() {
    let (app, _db) = build_app().await;
    let v = get_json(app, "/api/by-hourofday").await;
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 24, "by-hourofday must return exactly 24 entries");
    assert_eq!(arr[0]["hour"].as_u64().unwrap(), 0);
    assert_eq!(arr[23]["hour"].as_u64().unwrap(), 23);
}

#[tokio::test]
async fn api_export_csv_returns_header_line() {
    let (app, _db) = build_app().await;
    let resp = app
        .oneshot(Request::builder().uri("/api/export.csv").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.starts_with("message_id,"), "CSV must start with header");
    assert!(text.contains("cost_usd"), "CSV header must contain cost_usd");
}

#[tokio::test]
async fn api_alerts_crud() {
    let (app, _db) = build_app().await;
    // Initially empty
    let v = get_json(app.clone(), "/api/alerts").await;
    assert_eq!(v.as_array().unwrap().len(), 0);

    // Create
    let body = serde_json::json!({"name":"test","period":"month","project_path":null,"threshold_usd":50.0});
    let resp = app.clone()
        .oneshot(Request::builder().uri("/api/alerts").method("POST")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string())).unwrap())
        .await.unwrap();
    assert_eq!(resp.status(), 200);

    // List — one entry
    let v = get_json(app, "/api/alerts").await;
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"].as_str().unwrap(), "test");
    assert_eq!(arr[0]["threshold_usd"].as_f64().unwrap(), 50.0);
}

#[tokio::test]
async fn api_model_prices_returns_array() {
    let (app, _db) = build_app().await;
    let v = get_json(app, "/api/model-prices").await;
    assert!(v.is_array(), "model-prices must return an array");
}

#[tokio::test]
async fn api_by_month_returns_array() {
    let (app, _db) = build_app().await;
    let v = get_json(app, "/api/by-month?months=3").await;
    assert!(v.is_array(), "by-month must return an array");
}
