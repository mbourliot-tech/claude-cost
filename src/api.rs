use crate::assets::StaticAssets;
use crate::pricing;
use crate::scanner;
use crate::storage::Store;
use chrono::{Datelike, Utc};
use axum::{
    extract::{Path as AxPath, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
    projects_dir: PathBuf,
}

pub fn router(store: Arc<Store>, projects_dir: PathBuf) -> Router {
    let state = AppState { store, projects_dir };
    Router::new()
        .route("/", get(serve_index))
        .route("/live", get(serve_live))
        .route("/assets/{*path}", get(serve_asset))
        .route("/api/summary", get(api_summary))
        .route("/api/by-model", get(api_by_model))
        .route("/api/by-day", get(api_by_day))
        .route("/api/by-project", get(api_by_project))
        .route("/api/by-session", get(api_by_session))
        .route("/api/rescan", post(api_rescan))
        .route("/api/last-timestamp", get(api_last_timestamp))
        .route("/api/recent-calls", get(api_recent_calls))
        .route("/api/cache-stats", get(api_cache_stats))
        .route("/api/by-hour", get(api_by_hour))
        .route("/api/model-prices", get(api_model_prices))
        .route("/api/model-prices/{model}", put(api_put_model_price).delete(api_delete_model_price))
        .route("/api/by-month", get(api_by_month))
        .route("/api/alerts", get(api_list_alerts).post(api_create_alert))
        .route("/api/alerts/{id}", put(api_update_alert).delete(api_delete_alert))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct Range {
    since: Option<String>,
    until: Option<String>,
}

async fn serve_index() -> Response {
    serve_embedded("index.html")
}

async fn serve_live() -> Response {
    serve_embedded("live.html")
}

async fn serve_asset(AxPath(path): AxPath<String>) -> Response {
    serve_embedded(&path)
}

fn serve_embedded(name: &str) -> Response {
    match StaticAssets::get(name) {
        Some(file) => {
            let mime = mime_guess::from_path(name).first_or_octet_stream();
            let mut resp = (StatusCode::OK, file.data.into_owned()).into_response();
            resp.headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref()).unwrap());
            resp
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn api_summary(State(s): State<AppState>, Query(r): Query<Range>) -> Result<Json<serde_json::Value>, ApiErr> {
    let sum = s.store.summary(r.since.as_deref(), r.until.as_deref())?;
    Ok(Json(serde_json::to_value(sum)?))
}

async fn api_by_model(State(s): State<AppState>, Query(r): Query<Range>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.by_model(r.since.as_deref(), r.until.as_deref())?;
    Ok(Json(serde_json::to_value(rows)?))
}

#[derive(Debug, Deserialize)]
struct DaysQuery {
    days: Option<i64>,
}

async fn api_by_day(State(s): State<AppState>, Query(q): Query<DaysQuery>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.by_day(q.days.unwrap_or(30))?;
    Ok(Json(serde_json::to_value(rows)?))
}

#[derive(Debug, Deserialize)]
struct MonthsQuery {
    months: Option<i64>,
}

async fn api_by_month(State(s): State<AppState>, Query(q): Query<MonthsQuery>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.by_month(q.months.unwrap_or(12))?;
    Ok(Json(serde_json::to_value(rows)?))
}

async fn api_by_project(State(s): State<AppState>, Query(r): Query<Range>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.by_project(r.since.as_deref(), r.until.as_deref())?;
    Ok(Json(serde_json::to_value(rows)?))
}

#[derive(Debug, Deserialize)]
struct SessionQuery {
    project: Option<String>,
    limit: Option<i64>,
}

async fn api_by_session(State(s): State<AppState>, Query(q): Query<SessionQuery>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.by_session(q.project.as_deref(), q.limit.unwrap_or(50))?;
    Ok(Json(serde_json::to_value(rows)?))
}

#[derive(Debug, Serialize)]
struct ModelPriceRow {
    model: String,
    input_per_mtok: f64,
    output_per_mtok: f64,
    cache_read_per_mtok: Option<f64>,
    is_override: bool,
    is_known: bool,
}

#[derive(Debug, Deserialize)]
struct ModelPriceBody {
    input_per_mtok: f64,
    output_per_mtok: f64,
    cache_read_per_mtok: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RecentQuery {
    since: Option<String>,
    limit: Option<i64>,
}

async fn api_recent_calls(State(s): State<AppState>, Query(q): Query<RecentQuery>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.recent_calls(q.since.as_deref(), q.limit.unwrap_or(100))?;
    Ok(Json(serde_json::to_value(rows)?))
}

async fn api_cache_stats(State(s): State<AppState>, Query(r): Query<Range>) -> Result<Json<serde_json::Value>, ApiErr> {
    let by_model = s.store.by_model(r.since.as_deref(), r.until.as_deref())?;
    let overrides = s.store.price_overrides_map()?;

    let mut total_cache_read: u64 = 0;
    let mut total_input: u64 = 0;
    let mut total_cache_5m: u64 = 0;
    let mut total_cache_1h: u64 = 0;
    let mut savings_usd: f64 = 0.0;
    let mut write_premium_usd: f64 = 0.0;

    for row in &by_model {
        total_cache_read += row.cache_read_tokens;
        total_input += row.input_tokens;
        total_cache_5m += row.cache_5m_tokens;
        total_cache_1h += row.cache_1h_tokens;

        if let Some(p) = pricing::effective_price(&row.model, &overrides) {
            let input_rate = p.input_per_mtok / 1_000_000.0;
            let cache_read_rate = p.cache_read_per_mtok.unwrap_or(p.input_per_mtok * 0.10) / 1_000_000.0;
            savings_usd += row.cache_read_tokens as f64 * (input_rate - cache_read_rate);
            write_premium_usd += row.cache_5m_tokens as f64 * input_rate * 0.25;
            write_premium_usd += row.cache_1h_tokens as f64 * input_rate * 1.0;
        }
    }

    let total_effective_input = total_cache_read + total_input;
    let hit_rate = if total_effective_input > 0 {
        total_cache_read as f64 / total_effective_input as f64
    } else {
        0.0
    };

    Ok(Json(json!({
        "cache_read_tokens":  total_cache_read,
        "cache_write_5m_tokens": total_cache_5m,
        "cache_write_1h_tokens": total_cache_1h,
        "input_tokens": total_input,
        "hit_rate": hit_rate,
        "savings_usd": savings_usd,
        "write_premium_usd": write_premium_usd,
        "net_savings_usd": savings_usd - write_premium_usd,
    })))
}

async fn api_by_hour(State(s): State<AppState>, Query(r): Query<Range>) -> Result<Json<serde_json::Value>, ApiErr> {
    let rows = s.store.by_hour(r.since.as_deref(), r.until.as_deref())?;
    Ok(Json(serde_json::to_value(rows)?))
}

async fn api_model_prices(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiErr> {
    let models = s.store.distinct_models()?;
    let overrides = s.store.price_overrides_map()?;
    let empty = std::collections::HashMap::new();
    let rows: Vec<ModelPriceRow> = models
        .into_iter()
        .map(|model| {
            let is_override = overrides.contains_key(&model);
            let effective = pricing::effective_price(&model, &overrides);
            let default_p = pricing::effective_price(&model, &empty);
            ModelPriceRow {
                input_per_mtok: effective.map(|p| p.input_per_mtok).unwrap_or(0.0),
                output_per_mtok: effective.map(|p| p.output_per_mtok).unwrap_or(0.0),
                cache_read_per_mtok: effective.and_then(|p| p.cache_read_per_mtok),
                is_override,
                is_known: default_p.is_some(),
                model,
            }
        })
        .collect();
    Ok(Json(serde_json::to_value(rows)?))
}

async fn api_put_model_price(
    State(s): State<AppState>,
    AxPath(model): AxPath<String>,
    Json(body): Json<ModelPriceBody>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    s.store.upsert_model_price(&model, body.input_per_mtok, body.output_per_mtok, body.cache_read_per_mtok)?;
    let report = scanner::scan_all(&s.projects_dir, &s.store)?;
    Ok(Json(json!({ "ok": true, "repriced": report.new_calls })))
}

async fn api_delete_model_price(
    State(s): State<AppState>,
    AxPath(model): AxPath<String>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    s.store.delete_model_price(&model)?;
    let report = scanner::scan_all(&s.projects_dir, &s.store)?;
    Ok(Json(json!({ "ok": true, "repriced": report.new_calls })))
}

async fn api_last_timestamp(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiErr> {
    let ts = s.store.last_timestamp()?;
    Ok(Json(json!({ "last_timestamp": ts })))
}

fn period_start(period: &str) -> String {
    let now = Utc::now();
    match period {
        "week" => {
            let days_from_monday = now.weekday().num_days_from_monday() as i64;
            let monday = now - chrono::Duration::days(days_from_monday);
            monday.format("%Y-%m-%dT00:00:00Z").to_string()
        }
        "month" => format!("{:04}-{:02}-01T00:00:00Z", now.year(), now.month()),
        _ => "1970-01-01T00:00:00Z".to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct AlertBody {
    name: String,
    period: String,
    project_path: Option<String>,
    threshold_usd: f64,
    enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
struct AlertStatus {
    id: i64,
    name: String,
    period: String,
    project_path: Option<String>,
    threshold_usd: f64,
    enabled: bool,
    current_usd: f64,
    is_triggered: bool,
}

async fn api_list_alerts(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiErr> {
    let alerts = s.store.list_alerts()?;
    let mut statuses: Vec<AlertStatus> = Vec::with_capacity(alerts.len());
    for a in alerts {
        let since = period_start(&a.period);
        let current_usd = s.store.alert_spend(&since, a.project_path.as_deref())?;
        let is_triggered = a.enabled && current_usd >= a.threshold_usd;
        statuses.push(AlertStatus {
            id: a.id,
            name: a.name,
            period: a.period,
            project_path: a.project_path,
            threshold_usd: a.threshold_usd,
            enabled: a.enabled,
            current_usd,
            is_triggered,
        });
    }
    Ok(Json(serde_json::to_value(statuses)?))
}

async fn api_create_alert(
    State(s): State<AppState>,
    Json(body): Json<AlertBody>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    let id = s.store.insert_alert(
        &body.name,
        &body.period,
        body.project_path.as_deref(),
        body.threshold_usd,
    )?;
    Ok(Json(json!({ "id": id })))
}

async fn api_update_alert(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
    Json(body): Json<AlertBody>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    s.store.update_alert(
        id,
        &body.name,
        &body.period,
        body.project_path.as_deref(),
        body.threshold_usd,
        body.enabled.unwrap_or(true),
    )?;
    Ok(Json(json!({ "ok": true })))
}

async fn api_delete_alert(
    State(s): State<AppState>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    s.store.delete_alert(id)?;
    Ok(Json(json!({ "ok": true })))
}

async fn api_rescan(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiErr> {
    let report = scanner::scan_all(&s.projects_dir, &s.store)?;
    Ok(Json(json!({
        "sessions_seen": report.sessions_seen,
        "unique_calls": report.unique_calls,
        "new_calls": report.new_calls,
        "elapsed_ms": report.elapsed_ms,
        "total_cost_usd": report.total_cost_usd,
    })))
}

struct ApiErr(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for ApiErr {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for ApiErr {
    fn into_response(self) -> Response {
        tracing::warn!(error = %self.0, "api error");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": self.0.to_string()}))).into_response()
    }
}
