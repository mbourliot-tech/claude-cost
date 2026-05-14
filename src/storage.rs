use crate::pricing::ModelPrice;
use crate::types::UsageRecord;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS file_cache (
                path        TEXT PRIMARY KEY,
                mtime_secs  INTEGER NOT NULL,
                file_size   INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS model_prices (
                model               TEXT PRIMARY KEY,
                input_per_mtok      REAL NOT NULL,
                output_per_mtok     REAL NOT NULL,
                cache_read_per_mtok REAL
            );
            CREATE TABLE IF NOT EXISTS alerts (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                period          TEXT NOT NULL,
                project_path    TEXT,
                threshold_usd   REAL NOT NULL,
                enabled         INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS usage (
                message_id        TEXT PRIMARY KEY,
                session_id        TEXT NOT NULL,
                project_path      TEXT NOT NULL,
                ts                TEXT NOT NULL,
                model             TEXT NOT NULL,
                input_tokens      INTEGER NOT NULL,
                output_tokens     INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_5m_tokens   INTEGER NOT NULL,
                cache_1h_tokens   INTEGER NOT NULL,
                cost_usd          REAL NOT NULL,
                service_tier      TEXT,
                speed             TEXT,
                web_search_requests INTEGER NOT NULL DEFAULT 0,
                web_fetch_requests  INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_usage_ts      ON usage(ts);
            CREATE INDEX IF NOT EXISTS idx_usage_model   ON usage(model);
            CREATE INDEX IF NOT EXISTS idx_usage_project ON usage(project_path);
            CREATE INDEX IF NOT EXISTS idx_usage_session ON usage(session_id);
            "#,
        )?;
        let _ = conn.execute_batch("ALTER TABLE usage ADD COLUMN web_search_requests INTEGER NOT NULL DEFAULT 0");
        let _ = conn.execute_batch("ALTER TABLE usage ADD COLUMN web_fetch_requests INTEGER NOT NULL DEFAULT 0");
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Insert a batch. Each row is identified by `message_id`. On conflict the
    /// `cost_usd` is refreshed only when it actually changes — this lets the
    /// scanner re-price old rows after `pricing.rs` is updated (e.g. when a new
    /// model is added or a tier is adjusted) without losing idempotence: a
    /// rescan with unchanged prices touches zero rows.
    ///
    /// Returns the number of rows inserted OR repriced.
    pub fn insert_batch(&self, records: &[UsageRecord]) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let mut touched_rows = 0usize;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO usage \
                 (message_id, session_id, project_path, ts, model, \
                  input_tokens, output_tokens, cache_read_tokens, cache_5m_tokens, cache_1h_tokens, \
                  cost_usd, service_tier, speed, web_search_requests, web_fetch_requests) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15) \
                 ON CONFLICT(message_id) DO UPDATE SET \
                   cost_usd = excluded.cost_usd, \
                   web_search_requests = excluded.web_search_requests, \
                   web_fetch_requests = excluded.web_fetch_requests \
                 WHERE usage.cost_usd IS NOT excluded.cost_usd \
                    OR usage.web_search_requests != excluded.web_search_requests \
                    OR usage.web_fetch_requests != excluded.web_fetch_requests",
            )?;
            for r in records {
                let inserted = stmt.execute(params![
                    r.message_id,
                    r.session_id,
                    r.project_path,
                    r.timestamp,
                    r.model,
                    r.input_tokens as i64,
                    r.output_tokens as i64,
                    r.cache_read_tokens as i64,
                    r.cache_5m_tokens as i64,
                    r.cache_1h_tokens as i64,
                    r.cost_usd,
                    r.service_tier,
                    r.speed,
                    r.web_search_requests as i64,
                    r.web_fetch_requests as i64,
                ])?;
                touched_rows += inserted;
            }
        }
        tx.commit()?;
        Ok(touched_rows)
    }

    pub fn list_price_overrides(&self) -> Result<Vec<ModelPriceOverride>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT model, input_per_mtok, output_per_mtok, cache_read_per_mtok FROM model_prices ORDER BY model",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(ModelPriceOverride {
                    model: r.get(0)?,
                    input_per_mtok: r.get(1)?,
                    output_per_mtok: r.get(2)?,
                    cache_read_per_mtok: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn price_overrides_map(&self) -> Result<HashMap<String, ModelPrice>> {
        let overrides = self.list_price_overrides()?;
        Ok(overrides
            .into_iter()
            .map(|o| {
                (o.model, ModelPrice { input_per_mtok: o.input_per_mtok, output_per_mtok: o.output_per_mtok, cache_read_per_mtok: o.cache_read_per_mtok })
            })
            .collect())
    }

    pub fn upsert_model_price(&self, model: &str, input: f64, output: f64, cache_read: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO model_prices (model, input_per_mtok, output_per_mtok, cache_read_per_mtok) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(model) DO UPDATE SET \
               input_per_mtok = excluded.input_per_mtok, \
               output_per_mtok = excluded.output_per_mtok, \
               cache_read_per_mtok = excluded.cache_read_per_mtok",
            params![model, input, output, cache_read],
        )?;
        Ok(())
    }

    pub fn delete_model_price(&self, model: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM model_prices WHERE model = ?1", params![model])?;
        Ok(())
    }

    pub fn distinct_models(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT DISTINCT model FROM usage ORDER BY model")?;
        let rows = stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok(rows)
    }

    pub fn summary(&self, since: Option<&str>, until: Option<&str>) -> Result<Summary> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT COALESCE(SUM(cost_usd),0), \
                    COALESCE(SUM(input_tokens),0), \
                    COALESCE(SUM(output_tokens),0), \
                    COALESCE(SUM(cache_read_tokens),0), \
                    COALESCE(SUM(cache_5m_tokens),0), \
                    COALESCE(SUM(cache_1h_tokens),0), \
                    COUNT(*), \
                    COUNT(DISTINCT session_id) \
             FROM usage{clause}"
        );
        let row: Summary = conn.query_row(&sql, rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
            Ok(Summary {
                total_cost_usd: r.get(0)?,
                input_tokens: r.get::<_, i64>(1)? as u64,
                output_tokens: r.get::<_, i64>(2)? as u64,
                cache_read_tokens: r.get::<_, i64>(3)? as u64,
                cache_5m_tokens: r.get::<_, i64>(4)? as u64,
                cache_1h_tokens: r.get::<_, i64>(5)? as u64,
                calls: r.get::<_, i64>(6)? as u64,
                sessions: r.get::<_, i64>(7)? as u64,
            })
        })?;
        Ok(row)
    }

    pub fn by_model(&self, since: Option<&str>, until: Option<&str>) -> Result<Vec<ByModel>> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT model, \
                    SUM(cost_usd), \
                    SUM(input_tokens), SUM(output_tokens), \
                    SUM(cache_read_tokens), SUM(cache_5m_tokens), SUM(cache_1h_tokens), \
                    COUNT(*) \
             FROM usage{clause} GROUP BY model ORDER BY SUM(cost_usd) DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
                Ok(ByModel {
                    model: r.get(0)?,
                    cost_usd: r.get(1)?,
                    input_tokens: r.get::<_, i64>(2)? as u64,
                    output_tokens: r.get::<_, i64>(3)? as u64,
                    cache_read_tokens: r.get::<_, i64>(4)? as u64,
                    cache_5m_tokens: r.get::<_, i64>(5)? as u64,
                    cache_1h_tokens: r.get::<_, i64>(6)? as u64,
                    calls: r.get::<_, i64>(7)? as u64,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Group costs by calendar day (UTC since timestamps are ISO Z). `days` = window length.
    pub fn by_month(&self, months: i64) -> Result<Vec<ByMonth>> {
        let conn = self.conn.lock().unwrap();
        let sql = "\
            SELECT strftime('%Y-%m', ts) AS month, SUM(cost_usd), COUNT(*) \
            FROM usage \
            WHERE ts >= date('now', ?1) \
            GROUP BY month ORDER BY month ASC";
        let offset = format!("-{} months", months.max(1));
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt
            .query_map(params![offset], |r| {
                Ok(ByMonth {
                    month: r.get(0)?,
                    cost_usd: r.get(1)?,
                    calls: r.get::<_, i64>(2)? as u64,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn by_day(&self, days: i64) -> Result<Vec<ByDay>> {
        let conn = self.conn.lock().unwrap();
        let sql = "\
            SELECT substr(ts, 1, 10) AS day, SUM(cost_usd), SUM(input_tokens + output_tokens + cache_read_tokens + cache_5m_tokens + cache_1h_tokens) \
            FROM usage \
            WHERE ts >= date('now', ?1) \
            GROUP BY day ORDER BY day ASC";
        let offset = format!("-{} days", days.max(1));
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt
            .query_map(params![offset], |r| {
                Ok(ByDay {
                    date: r.get(0)?,
                    cost_usd: r.get(1)?,
                    tokens: r.get::<_, i64>(2)? as u64,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn by_project(&self, since: Option<&str>, until: Option<&str>) -> Result<Vec<ByProject>> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT project_path, SUM(cost_usd), COUNT(DISTINCT session_id), COUNT(*) \
             FROM usage{clause} GROUP BY project_path ORDER BY SUM(cost_usd) DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
                Ok(ByProject {
                    project_path: r.get(0)?,
                    cost_usd: r.get(1)?,
                    sessions: r.get::<_, i64>(2)? as u64,
                    calls: r.get::<_, i64>(3)? as u64,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn by_session(&self, project: Option<&str>, limit: i64) -> Result<Vec<BySession>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT session_id, project_path, MIN(ts), MAX(ts), SUM(cost_usd), COUNT(*), \
             SUM(cache_read_tokens), SUM(input_tokens), \
             MAX(input_tokens + cache_read_tokens + cache_5m_tokens + cache_1h_tokens), \
             COALESCE(SUM(web_search_requests), 0), COALESCE(SUM(web_fetch_requests), 0) \
             FROM usage",
        );
        let mut params_owned: Vec<String> = Vec::new();
        if let Some(p) = project {
            sql.push_str(" WHERE project_path = ?1");
            params_owned.push(p.to_string());
        }
        sql.push_str(" GROUP BY session_id ORDER BY MAX(ts) DESC LIMIT ?");
        sql.push_str(&format!("{}", params_owned.len() + 1));
        params_owned.push(limit.to_string());
        let mut stmt = conn.prepare(&sql)?;
        let p_refs: Vec<&dyn rusqlite::ToSql> = params_owned.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = stmt
            .query_map(p_refs.as_slice(), |r| {
                Ok(BySession {
                    session_id: r.get(0)?,
                    project_path: r.get(1)?,
                    started_at: r.get(2)?,
                    ended_at: r.get(3)?,
                    cost_usd: r.get(4)?,
                    calls: r.get::<_, i64>(5)? as u64,
                    cache_read_tokens: r.get::<_, i64>(6)? as u64,
                    input_tokens: r.get::<_, i64>(7)? as u64,
                    peak_context_tokens: r.get::<_, i64>(8)? as u64,
                    web_search_requests: r.get::<_, i64>(9)? as u32,
                    web_fetch_requests: r.get::<_, i64>(10)? as u32,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn by_hour(&self, since: Option<&str>, until: Option<&str>) -> Result<Vec<ByHour>> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT substr(ts, 1, 13) AS h, SUM(cost_usd), \
                    SUM(input_tokens), SUM(output_tokens), SUM(cache_read_tokens) \
             FROM usage{clause} GROUP BY h ORDER BY h"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
                Ok(ByHour {
                    hour: r.get::<_, String>(0)? + ":00:00Z",
                    cost_usd: r.get(1)?,
                    input_tokens: r.get::<_, i64>(2)? as u64,
                    output_tokens: r.get::<_, i64>(3)? as u64,
                    cache_read_tokens: r.get::<_, i64>(4)? as u64,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn recent_calls(&self, since: Option<&str>, limit: i64) -> Result<Vec<RecentCall>> {
        let conn = self.conn.lock().unwrap();
        let (where_clause, p_since) = match since {
            Some(s) => (" WHERE ts > ?1", Some(s.to_string())),
            None => ("", None),
        };
        let sql = format!(
            "SELECT message_id, session_id, project_path, ts, model, \
                    input_tokens, output_tokens, cache_read_tokens, \
                    cache_5m_tokens, cache_1h_tokens, cost_usd, service_tier, speed \
             FROM usage{where_clause} ORDER BY ts DESC, message_id DESC LIMIT ?{}",
            if p_since.is_some() { 2 } else { 1 }
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = if let Some(s) = p_since {
            stmt.query_map(rusqlite::params![s, limit], row_to_recent_call)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map(rusqlite::params![limit], row_to_recent_call)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        Ok(rows)
    }

    pub fn list_alerts(&self) -> Result<Vec<Alert>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, period, project_path, threshold_usd, enabled FROM alerts ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Alert {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    period: r.get(2)?,
                    project_path: r.get(3)?,
                    threshold_usd: r.get(4)?,
                    enabled: r.get::<_, i64>(5)? != 0,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn insert_alert(&self, name: &str, period: &str, project_path: Option<&str>, threshold_usd: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO alerts (name, period, project_path, threshold_usd) VALUES (?1, ?2, ?3, ?4)",
            params![name, period, project_path, threshold_usd],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_alert(&self, id: i64, name: &str, period: &str, project_path: Option<&str>, threshold_usd: f64, enabled: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE alerts SET name=?2, period=?3, project_path=?4, threshold_usd=?5, enabled=?6 WHERE id=?1",
            params![id, name, period, project_path, threshold_usd, enabled as i64],
        )?;
        Ok(())
    }

    pub fn delete_alert(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM alerts WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn alert_spend(&self, since: &str, project_path: Option<&str>) -> Result<f64> {
        let conn = self.conn.lock().unwrap();
        let cost: f64 = match project_path {
            Some(p) => conn.query_row(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM usage WHERE ts >= ?1 AND project_path = ?2",
                params![since, p],
                |r| r.get(0),
            )?,
            None => conn.query_row(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM usage WHERE ts >= ?1",
                params![since],
                |r| r.get(0),
            )?,
        };
        Ok(cost)
    }

    pub fn file_needs_scan(&self, path: &str, mtime_secs: u64, file_size: u64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let result: Option<(u64, u64)> = conn
            .query_row(
                "SELECT mtime_secs, file_size FROM file_cache WHERE path = ?1",
                params![path],
                |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)? as u64)),
            )
            .optional()?;
        Ok(match result {
            None => true,
            Some((m, s)) => m != mtime_secs || s != file_size,
        })
    }

    pub fn update_file_cache(&self, path: &str, mtime_secs: u64, file_size: u64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO file_cache (path, mtime_secs, file_size) VALUES (?1, ?2, ?3) \
             ON CONFLICT(path) DO UPDATE SET mtime_secs = excluded.mtime_secs, file_size = excluded.file_size",
            params![path, mtime_secs as i64, file_size as i64],
        )?;
        Ok(())
    }

    pub fn by_weekday(&self, since: Option<&str>, until: Option<&str>) -> Result<Vec<ByWeekday>> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT CAST(strftime('%w', ts) AS INTEGER) AS dow, SUM(cost_usd), COUNT(*) \
             FROM usage{clause} GROUP BY dow ORDER BY dow"
        );
        let mut stmt = conn.prepare(&sql)?;
        let raw = stmt
            .query_map(rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?, r.get::<_, i64>(2)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let labels = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"];
        let mut result: Vec<ByWeekday> = (0u8..7)
            .map(|i| ByWeekday { weekday: i, label: labels[i as usize].to_string(), cost_usd: 0.0, calls: 0 })
            .collect();
        for (dow_sqlite, cost, calls) in raw {
            let iso = ((dow_sqlite + 6) % 7) as usize;
            result[iso].cost_usd = cost;
            result[iso].calls = calls as u64;
        }
        Ok(result)
    }

    pub fn by_hourofday(&self, since: Option<&str>, until: Option<&str>) -> Result<Vec<ByHourOfDay>> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT CAST(strftime('%H', ts) AS INTEGER) AS h, SUM(cost_usd), COUNT(*) \
             FROM usage{clause} GROUP BY h ORDER BY h"
        );
        let mut stmt = conn.prepare(&sql)?;
        let raw = stmt
            .query_map(rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?, r.get::<_, i64>(2)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut result: Vec<ByHourOfDay> = (0u8..24)
            .map(|h| ByHourOfDay { hour: h, cost_usd: 0.0, calls: 0 })
            .collect();
        for (h, cost, calls) in raw {
            result[h as usize].cost_usd = cost;
            result[h as usize].calls = calls as u64;
        }
        Ok(result)
    }

    pub fn all_usage_for_export(&self, since: Option<&str>, until: Option<&str>) -> Result<Vec<ExportRow>> {
        let conn = self.conn.lock().unwrap();
        let (clause, p1, p2) = range_clause(since, until);
        let sql = format!(
            "SELECT message_id, session_id, project_path, ts, model, \
                    input_tokens, output_tokens, cache_read_tokens, cache_5m_tokens, cache_1h_tokens, \
                    cost_usd, COALESCE(service_tier,''), COALESCE(speed,''), \
                    COALESCE(web_search_requests,0), COALESCE(web_fetch_requests,0) \
             FROM usage{clause} ORDER BY ts"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(opt_params(&p1, &p2)), |r| {
                Ok(ExportRow {
                    message_id:           r.get(0)?,
                    session_id:           r.get(1)?,
                    project_path:         r.get(2)?,
                    ts:                   r.get(3)?,
                    model:                r.get(4)?,
                    input_tokens:         r.get::<_, i64>(5)? as u64,
                    output_tokens:        r.get::<_, i64>(6)? as u64,
                    cache_read_tokens:    r.get::<_, i64>(7)? as u64,
                    cache_5m_tokens:      r.get::<_, i64>(8)? as u64,
                    cache_1h_tokens:      r.get::<_, i64>(9)? as u64,
                    cost_usd:             r.get(10)?,
                    service_tier:         r.get(11)?,
                    speed:                r.get(12)?,
                    web_search_requests:  r.get::<_, i64>(13)? as u32,
                    web_fetch_requests:   r.get::<_, i64>(14)? as u32,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn waste_stats(&self) -> Result<WasteStats> {
        let conn = self.conn.lock().unwrap();
        let (wasted_sessions, wasted_cost_usd): (i64, f64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(s_cost), 0.0) FROM (
               SELECT session_id,
                      SUM(cost_usd) AS s_cost,
                      SUM(cache_5m_tokens + cache_1h_tokens) AS writes,
                      SUM(cache_read_tokens) AS reads,
                      COUNT(*) AS calls
               FROM usage
               GROUP BY session_id
               HAVING writes > 0 AND reads = 0 AND calls <= 5
             )",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let sessions_near_limit: i64 = conn.query_row(
            "SELECT COUNT(*) FROM (
               SELECT session_id,
                      MAX(input_tokens + cache_read_tokens + cache_5m_tokens + cache_1h_tokens) AS peak
               FROM usage GROUP BY session_id HAVING peak > 150000
             )",
            [],
            |r| r.get(0),
        )?;
        let (cache_read_tokens, input_tokens): (i64, i64) = conn.query_row(
            "SELECT COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(input_tokens),0) FROM usage",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let hit_rate = {
            let total = cache_read_tokens + input_tokens;
            if total > 0 { cache_read_tokens as f64 / total as f64 } else { 0.0 }
        };
        Ok(WasteStats {
            wasted_sessions: wasted_sessions as u64,
            wasted_cost_usd,
            sessions_near_limit: sessions_near_limit as u64,
            alltime_hit_rate: hit_rate,
        })
    }

    pub fn last_timestamp(&self) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let ts: Option<String> = conn
            .query_row("SELECT MAX(ts) FROM usage", [], |r| r.get(0))
            .optional()?
            .flatten();
        Ok(ts)
    }
}

fn row_to_recent_call(r: &rusqlite::Row) -> rusqlite::Result<RecentCall> {
    Ok(RecentCall {
        message_id: r.get(0)?,
        session_id: r.get(1)?,
        project_path: r.get(2)?,
        ts: r.get(3)?,
        model: r.get(4)?,
        input_tokens: r.get::<_, i64>(5)? as u64,
        output_tokens: r.get::<_, i64>(6)? as u64,
        cache_read_tokens: r.get::<_, i64>(7)? as u64,
        cache_5m_tokens: r.get::<_, i64>(8)? as u64,
        cache_1h_tokens: r.get::<_, i64>(9)? as u64,
        cost_usd: r.get(10)?,
        service_tier: r.get(11)?,
        speed: r.get(12)?,
    })
}

fn range_clause(since: Option<&str>, until: Option<&str>) -> (String, Option<String>, Option<String>) {
    match (since, until) {
        (Some(a), Some(b)) => (" WHERE ts >= ?1 AND ts <= ?2".into(), Some(a.into()), Some(b.into())),
        (Some(a), None) => (" WHERE ts >= ?1".into(), Some(a.into()), None),
        (None, Some(b)) => (" WHERE ts <= ?1".into(), Some(b.into()), None),
        (None, None) => (String::new(), None, None),
    }
}

fn opt_params<'a>(p1: &'a Option<String>, p2: &'a Option<String>) -> Vec<&'a dyn rusqlite::ToSql> {
    let mut v: Vec<&dyn rusqlite::ToSql> = Vec::new();
    if let Some(s) = p1 {
        v.push(s as &dyn rusqlite::ToSql);
    }
    if let Some(s) = p2 {
        v.push(s as &dyn rusqlite::ToSql);
    }
    v
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_5m_tokens: u64,
    pub cache_1h_tokens: u64,
    pub calls: u64,
    pub sessions: u64,
}

#[derive(Debug, Serialize)]
pub struct ByModel {
    pub model: String,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_5m_tokens: u64,
    pub cache_1h_tokens: u64,
    pub calls: u64,
}

#[derive(Debug, Serialize)]
pub struct ByMonth {
    pub month: String,
    pub cost_usd: f64,
    pub calls: u64,
}

#[derive(Debug, Serialize)]
pub struct ByDay {
    pub date: String,
    pub cost_usd: f64,
    pub tokens: u64,
}

#[derive(Debug, Serialize)]
pub struct ByHour {
    pub hour: String,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
}

#[derive(Debug, Serialize)]
pub struct ByProject {
    pub project_path: String,
    pub cost_usd: f64,
    pub sessions: u64,
    pub calls: u64,
}

#[derive(Debug, Serialize)]
pub struct RecentCall {
    pub message_id: String,
    pub session_id: String,
    pub project_path: String,
    pub ts: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_5m_tokens: u64,
    pub cache_1h_tokens: u64,
    pub cost_usd: f64,
    pub service_tier: Option<String>,
    pub speed: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelPriceOverride {
    pub model: String,
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_read_per_mtok: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct WasteStats {
    pub wasted_sessions: u64,
    pub wasted_cost_usd: f64,
    pub sessions_near_limit: u64,
    pub alltime_hit_rate: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    pub id: i64,
    pub name: String,
    pub period: String,
    pub project_path: Option<String>,
    pub threshold_usd: f64,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ByWeekday {
    pub weekday: u8,
    pub label: String,
    pub cost_usd: f64,
    pub calls: u64,
}

#[derive(Debug, Serialize)]
pub struct ByHourOfDay {
    pub hour: u8,
    pub cost_usd: f64,
    pub calls: u64,
}

#[derive(Debug, Serialize)]
pub struct ExportRow {
    pub message_id: String,
    pub session_id: String,
    pub project_path: String,
    pub ts: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_5m_tokens: u64,
    pub cache_1h_tokens: u64,
    pub cost_usd: f64,
    pub service_tier: String,
    pub speed: String,
    pub web_search_requests: u32,
    pub web_fetch_requests: u32,
}

#[derive(Debug, Serialize)]
pub struct BySession {
    pub session_id: String,
    pub project_path: String,
    pub started_at: String,
    pub ended_at: String,
    pub cost_usd: f64,
    pub calls: u64,
    pub cache_read_tokens: u64,
    pub input_tokens: u64,
    pub peak_context_tokens: u64,
    pub web_search_requests: u32,
    pub web_fetch_requests: u32,
}
