use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use claude_cost::{api, pricing, scanner, storage};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

// ── ANSI ──────────────────────────────────────────────────────────────────────
const R:   &str = "\x1b[0m";
const B:   &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[32m";
const CYN: &str = "\x1b[36m";
const MGT: &str = "\x1b[35m";
const YLW: &str = "\x1b[33m";
const BLU: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";

fn model_color(m: &str) -> &'static str {
    let m = m.to_lowercase();
    if m.contains("opus")     { MGT }
    else if m.contains("sonnet")  { CYN }
    else if m.contains("haiku")   { GRN }
    else if m.contains("deepseek"){ YLW }
    else if m.contains("mimo")    { BLU }
    else                          { R   }
}

fn fmt_tok(n: u64) -> String {
    if n >= 1_000_000_000 { format!("{:.1}G", n as f64 / 1e9) }
    else if n >= 1_000_000 { format!("{:.2}M", n as f64 / 1e6) }
    else if n >= 1_000     { format!("{:.1}k", n as f64 / 1e3) }
    else                   { n.to_string() }
}
fn fmt_usd(n: f64)  -> String { format!("${:.2}", n) }
fn fmt_usd4(n: f64) -> String { format!("${:.4}", n) }

fn hit_str(cache_read: u64, input: u64) -> String {
    let tot = cache_read + input;
    if tot == 0 { "—".to_string() }
    else { format!("{:.1}%", cache_read as f64 / tot as f64 * 100.0) }
}

fn short_path(p: &str) -> &str {
    let norm = p.replace('\\', "/");
    // keep last segment
    norm.split('/').filter(|s| !s.is_empty()).last()
        .map(|_| p.split(|c| c == '/' || c == '\\').filter(|s| !s.is_empty()).last().unwrap_or(p))
        .unwrap_or(p)
}

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "claude-cost", version, about = "Track Claude Code token usage and cost")]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,

    /// Chemin vers ~/.claude/projects
    #[arg(long, global = true)]
    projects_dir: Option<PathBuf>,

    /// Chemin vers la base SQLite
    #[arg(long, global = true)]
    db: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Lance le serveur web (comportement par défaut)
    Serve {
        #[arg(long, default_value_t = 3737)]
        port: u16,
        #[arg(long, default_value_t = 60)]
        rescan_interval: u64,
        #[arg(long)]
        no_open: bool,
    },
    /// Résumé global des coûts
    Summary {
        #[arg(long)] since: Option<String>,
        #[arg(long)] until: Option<String>,
    },
    /// Breakdown des coûts par modèle
    #[command(name = "by-model")]
    ByModel {
        #[arg(long)] since: Option<String>,
        #[arg(long)] until: Option<String>,
    },
    /// Stats du jour en cours, par heure
    Today,
    /// Flux temps réel des derniers appels (Ctrl+C pour quitter)
    Live {
        /// Intervalle de rafraîchissement en secondes
        #[arg(long, default_value_t = 5)]
        interval: u64,
        /// Nombre d'appels à afficher au démarrage
        #[arg(long, default_value_t = 20)]
        limit: i64,
    },
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_serve = matches!(cli.command, None | Some(Cmd::Serve { .. }));

    if is_serve {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("claude_cost=info,warn")))
            .init();
    }

    let projects_dir = match cli.projects_dir {
        Some(p) => p,
        None => default_projects_dir().context("could not locate ~/.claude/projects")?,
    };
    let db_path = match cli.db {
        Some(p) => p,
        None => default_db_path().context("could not locate data directory")?,
    };

    let store = Arc::new(storage::Store::open(&db_path).context("opening SQLite store")?);

    let cmd = cli.command.unwrap_or(Cmd::Serve { port: 3737, rescan_interval: 60, no_open: false });

    // Toutes les commandes CLI sauf serve font un scan rapide d'abord
    if !matches!(cmd, Cmd::Serve { .. }) {
        scanner::scan_all(&projects_dir, &store).context("scan")?;
    }

    match cmd {
        Cmd::Serve { port, rescan_interval, no_open } =>
            run_serve(store, projects_dir, port, rescan_interval, no_open).await?,
        Cmd::Summary { since, until } =>
            cmd_summary(&store, since.as_deref(), until.as_deref())?,
        Cmd::ByModel { since, until } =>
            cmd_by_model(&store, since.as_deref(), until.as_deref())?,
        Cmd::Today =>
            cmd_today(&store)?,
        Cmd::Live { interval, limit } =>
            cmd_live(store, projects_dir, interval, limit).await?,
    }
    Ok(())
}

// ── Serve ─────────────────────────────────────────────────────────────────────

async fn run_serve(store: Arc<storage::Store>, projects_dir: PathBuf, port: u16, rescan_interval: u64, no_open: bool) -> Result<()> {
    info!(?projects_dir, "starting claude-cost");
    let report = scanner::scan_all(&projects_dir, &store).context("scanning projects")?;
    info!(
        sessions = report.sessions_seen,
        unique_calls = report.unique_calls,
        new_calls = report.new_calls,
        elapsed_ms = report.elapsed_ms,
        total_cost = format!("${:.2}", report.total_cost_usd),
        "scan complete"
    );

    if rescan_interval > 0 {
        let bg_store = store.clone();
        let bg_dir = projects_dir.clone();
        let dur = Duration::from_secs(rescan_interval);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(dur).await;
                match scanner::scan_all(&bg_dir, &bg_store) {
                    Ok(r) if r.new_calls > 0 => info!(new_calls = r.new_calls, elapsed_ms = r.elapsed_ms, "auto-rescan: new data ingested"),
                    Ok(_) => {}
                    Err(e) => warn!(error = %e, "auto-rescan failed"),
                }
            }
        });
        info!(interval_secs = rescan_interval, "auto-rescan enabled");
    }

    let app = api::router(store, projects_dir);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.with_context(|| format!("bind {addr}"))?;
    info!("listening on http://localhost:{port}");

    if !no_open {
        let url = format!("http://localhost:{port}");
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            if let Err(e) = open::that(&url) {
                warn!(error = %e, "could not open browser");
            }
        });
    }

    axum::serve(listener, app).await.context("axum serve")?;
    Ok(())
}

// ── CLI commands ──────────────────────────────────────────────────────────────

fn cmd_summary(store: &storage::Store, since: Option<&str>, until: Option<&str>) -> Result<()> {
    let s = store.summary(since, until)?;
    let overrides = store.price_overrides_map()?;
    let by_model = store.by_model(since, until)?;

    let mut savings = 0f64;
    let mut write_premium = 0f64;
    for row in &by_model {
        if let Some(p) = pricing::effective_price(&row.model, &overrides) {
            let ir = p.input_per_mtok / 1e6;
            let cr = p.cache_read_per_mtok.unwrap_or(p.input_per_mtok * 0.10) / 1e6;
            savings += row.cache_read_tokens as f64 * (ir - cr);
            write_premium += row.cache_5m_tokens as f64 * ir * 0.25;
            write_premium += row.cache_1h_tokens as f64 * ir * 1.0;
        }
    }
    let net = savings - write_premium;
    let net_col = if net >= 0.0 { GRN } else { RED };

    let total_eff = s.cache_read_tokens + s.input_tokens;
    let hit = if total_eff > 0 { format!("{:.1}%", s.cache_read_tokens as f64 / total_eff as f64 * 100.0) } else { "—".to_string() };

    let period = match (since, until) {
        (None, None)       => "tout".to_string(),
        (Some(s), None)    => format!("depuis {}", &s[..10.min(s.len())]),
        (Some(s), Some(u)) => format!("{} → {}", &s[..10.min(s.len())], &u[..10.min(u.len())]),
        (None, Some(u))    => format!("jusqu'au {}", &u[..10.min(u.len())]),
    };

    println!();
    println!("  {B}claude-cost — Résumé{R}  {DIM}{period}{R}");
    println!("  {}", "─".repeat(50));
    println!("  {B}Coût total{R}      {GRN}{:>10}{R}    {B}Appels{R}      {:>8}", fmt_usd(s.total_cost_usd), s.calls);
    println!("  {B}Sessions{R}        {:>10}    {B}Input{R}       {:>8}", s.sessions, fmt_tok(s.input_tokens));
    println!("  {B}Output{R}          {:>10}    {B}Cache write{R} {:>8}", fmt_tok(s.output_tokens), fmt_tok(s.cache_5m_tokens + s.cache_1h_tokens));
    println!("  {}", "─".repeat(50));
    println!("  {B}Cache hit{R}       {:>10}    {B}Tokens read{R} {:>8}", hit, fmt_tok(s.cache_read_tokens));
    println!("  {B}Économies brutes{R} {GRN}{:>+9}{R}    {B}Surcoût write{R} {YLW}{:>+6}{R}", fmt_usd(savings), fmt_usd(-write_premium));
    println!("  {B}Gain net cache{R}  {net_col}{:>+10}{R}", fmt_usd(net));
    println!();
    Ok(())
}

fn cmd_by_model(store: &storage::Store, since: Option<&str>, until: Option<&str>) -> Result<()> {
    let rows = store.by_model(since, until)?;
    if rows.is_empty() {
        println!("  {DIM}Aucune donnée.{R}");
        return Ok(());
    }

    println!();
    println!("  {B}claude-cost — Par modèle{R}");
    println!("  {DIM}{:<34} {:>9}  {:>7}  {:>6}  {:>10}  {:>8}{R}",
        "Modèle", "Coût", "Appels", "Hit%", "Cache read", "Output");
    println!("  {}", "─".repeat(84));

    let total_cost: f64 = rows.iter().map(|r| r.cost_usd).sum();
    let total_calls: u64 = rows.iter().map(|r| r.calls).sum();

    for r in &rows {
        let col = model_color(&r.model);
        let hit = hit_str(r.cache_read_tokens, r.input_tokens);
        // shorten: strip "claude-" prefix and date suffix
        let m = r.model.replace("claude-", "");
        let m = if m.len() > 34 { &m[..34] } else { &m };
        println!("  {col}{B}{:<34}{R} {:>9}  {:>7}  {:>6}  {:>10}  {:>8}",
            m, fmt_usd(r.cost_usd), r.calls, hit,
            fmt_tok(r.cache_read_tokens), fmt_tok(r.output_tokens));
    }
    println!("  {}", "─".repeat(84));
    println!("  {B}{:<34} {:>9}  {:>7}{R}", "Total", fmt_usd(total_cost), total_calls);
    println!();
    Ok(())
}

fn cmd_today(store: &storage::Store) -> Result<()> {
    let now = Utc::now();
    let since = now.format("%Y-%m-%dT00:00:00Z").to_string();
    let rows = store.by_hour(Some(&since), None)?;
    let sum = store.summary(Some(&since), None)?;

    if rows.is_empty() {
        println!("  {DIM}Aucun appel aujourd'hui.{R}");
        return Ok(());
    }

    println!();
    println!("  {B}claude-cost — Aujourd'hui{R}  {DIM}{}{R}", now.format("%Y-%m-%d"));
    println!("  {DIM}{:<6}  {:>9}  {:>8}  {:>8}  {:>10}  {:>6}{R}",
        "Heure", "Coût", "Input", "Output", "Cache read", "Hit%");
    println!("  {}", "─".repeat(58));

    for r in &rows {
        let h = &r.hour[11..13];
        let hit = hit_str(r.cache_read_tokens, r.input_tokens);
        println!("  {B}{h}h{R}     {:>9}  {:>8}  {:>8}  {:>10}  {:>6}",
            fmt_usd(r.cost_usd), fmt_tok(r.input_tokens),
            fmt_tok(r.output_tokens), fmt_tok(r.cache_read_tokens), hit);
    }
    println!("  {}", "─".repeat(58));
    println!("  {B}{:<6}  {:>9}  {:>8}  {:>8}  {:>10}  {:>6}{R}",
        "Total", fmt_usd(sum.total_cost_usd), fmt_tok(sum.input_tokens),
        fmt_tok(sum.output_tokens), fmt_tok(sum.cache_read_tokens),
        hit_str(sum.cache_read_tokens, sum.input_tokens));
    println!();
    Ok(())
}

async fn cmd_live(store: Arc<storage::Store>, projects_dir: PathBuf, interval: u64, limit: i64) -> Result<()> {
    println!();
    println!("  {B}{GRN}● LIVE{R}  {B}claude-cost — Flux temps réel{R}  {DIM}Ctrl+C pour quitter — rafraîchissement {interval}s{R}");
    println!("  {}", "─".repeat(100));
    println!("  {DIM}{:<19}  {:<26}  {:<18}  {:>9}  {:>7}  {:>7}  {:>10}  {:>5}{R}",
        "Horodatage", "Modèle", "Projet", "Coût", "Input", "Output", "Cache read", "Hit%");
    println!("  {}", "─".repeat(100));

    // Chargement initial
    let initial = store.recent_calls(None, limit)?;
    let mut last_ts: Option<String> = initial.iter().map(|r| r.ts.clone()).max();
    let mut seen: std::collections::HashSet<String> =
        initial.iter().map(|r| r.message_id.clone()).collect();

    for r in initial.iter().rev() {
        print_live_row(r);
    }

    let dur = Duration::from_secs(interval);
    loop {
        tokio::select! {
            _ = tokio::time::sleep(dur) => {}
            _ = tokio::signal::ctrl_c() => {
                println!("\n  {DIM}Arrêt.{R}");
                break;
            }
        }
        let _ = scanner::scan_all(&projects_dir, &store);
        let new_rows = store.recent_calls(last_ts.as_deref(), 50)?;
        for r in new_rows.iter().rev() {
            if seen.insert(r.message_id.clone()) {
                print_live_row(r);
                if last_ts.as_deref().map_or(true, |lt| r.ts.as_str() > lt) {
                    last_ts = Some(r.ts.clone());
                }
            }
        }
    }
    Ok(())
}

fn print_live_row(r: &storage::RecentCall) {
    let col = model_color(&r.model);
    let m = r.model.replace("claude-", "");
    let m = if m.len() > 26 { &m[..26] } else { &m };
    let p = short_path(&r.project_path);
    let p = if p.len() > 18 { &p[..18] } else { p };
    let ts = r.ts.get(..19).unwrap_or(&r.ts).replace('T', " ");
    let hit = hit_str(r.cache_read_tokens, r.input_tokens);
    println!("  {DIM}{ts}{R}  {col}{B}{:<26}{R}  {:<18}  {:>9}  {:>7}  {:>7}  {:>10}  {:>5}",
        m, p, fmt_usd4(r.cost_usd),
        fmt_tok(r.input_tokens), fmt_tok(r.output_tokens),
        fmt_tok(r.cache_read_tokens), hit);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_projects_dir() -> Option<PathBuf> {
    directories::UserDirs::new().map(|d| d.home_dir().join(".claude").join("projects"))
}

fn default_db_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "claude-cost", "claude-cost")?;
    let data = dirs.data_dir().to_path_buf();
    std::fs::create_dir_all(&data).ok()?;
    Some(data.join("tracker.db"))
}
