# claude·cost

> Local dashboard tracking Claude Code token usage and costs by parsing JSONL transcripts.

[🇫🇷 Version française](README.fr.md)

---

## What it is

**claude-cost** is a self-hosted Rust web application that reads the JSONL conversation logs written by Claude Code (`~/.claude/projects/**/*.jsonl`), computes the billing cost of every API call, stores the results in a local SQLite database, and serves a dashboard on `http://localhost:3737`.

No data leaves your machine. No API key required.

---

## Features

| Tab | Contents |
|-----|----------|
| **Overview** | Cost KPIs with trend vs previous period · Cache stats · Time chart · Model donut · Top projects · Weekday & hourly patterns |
| **Sessions** | All sessions with peak context window, cache hit %, web tool usage · Project & limit filters · CSV export |
| **Plans** | API billing vs Pro / Max 5× / Max 20× comparison over the last N months · Regularity score |
| **Insights** | Actionable cost-reduction recommendations with estimated monthly savings |
| **Alerts** | Configurable budget thresholds (weekly / monthly, global or per project) |

**Additional features**
- 🌙 / ☀️ / 🌌 Three themes: Dark, Light, Midnight
- 🇫🇷 / 🇬🇧 Two languages: French, English
- ⚡ Incremental scanning — unchanged JSONL files are skipped
- 🔔 Auto-refresh every 30 s (polls `/api/last-timestamp`)
- 📺 Live view at `/live` (5 s polling, animated table)
- 🖥️ Full CLI: `summary`, `by-model`, `today`, `live` sub-commands
- 🕹️ Hidden Easter egg (try the Konami code on the dashboard…)

---

## Requirements

- **Rust 1.75+** (stable)
- Windows, macOS or Linux

---

## Installation

### Download binary (recommended)

Grab the latest pre-compiled binary from the [Releases](https://github.com/mbourliot-tech/claude-cost/releases) page — no Rust toolchain required.

| Platform | File |
|----------|------|
| Windows  | `claude-cost-windows-x86_64.exe` |
| macOS (Apple Silicon) | `claude-cost-macos-aarch64` |
| macOS (Intel) | `claude-cost-macos-x86_64` |
| Linux | `claude-cost-linux-x86_64` |

On macOS / Linux, make the binary executable:
```bash
chmod +x claude-cost-macos-aarch64
./claude-cost-macos-aarch64
```

### From source

Requires [Rust](https://rustup.rs) 1.75+.

```bash
git clone https://github.com/mbourliot-tech/claude-cost.git
cd claude-cost
cargo install --path .
```

The binary is installed to `~/.cargo/bin/claude-cost`.

### Update

```bash
cd claude-cost
git pull
cargo install --path .
```

---

## Usage

### Web dashboard (default)

```bash
claude-cost
# or
claude-cost serve
```

Opens `http://localhost:3737` automatically in your browser.

**Options**

```
--port <PORT>               Port to listen on [default: 3737]
--projects-dir <DIR>        Override ~/.claude/projects
--rescan-interval <SECS>    Background rescan interval in seconds [default: 60]
--no-open                   Don't open the browser automatically
```

### CLI commands

```bash
claude-cost summary                    # Global cost summary (coloured)
claude-cost summary --since 2026-05-01 # Filtered by date
claude-cost by-model                   # Cost breakdown by model
claude-cost today                      # Hourly breakdown for today
claude-cost live                       # Real-time watch mode
claude-cost live --interval 3 --limit 20
```

---

## Dashboard tabs

### Overview
Four KPIs (total cost, API calls, sessions, tokens) with a trend indicator compared to the previous equivalent period. Below: a cache statistics bar, a time chart (hourly for today, daily otherwise), a model donut chart, a top-projects table, and two pattern charts (cost by weekday and by hour of day).

### Sessions
Full session table with: peak context window (colour-coded against 200 K / 256 K / 1 M limits), cache hit rate, web-search and web-fetch counts. Filters by project and limit (20 / 50 / 100 / 500). Direct CSV export button.

### Plans
Monthly bar chart overlaid with $20 / $100 / $200 subscription thresholds. Table with total savings or loss per plan, profitable-month count and a consistency bar. Period selector: 3, 6 or 12 months.

### Insights
Automatic analysis of five cost-reduction levers:
1. **Cache hit rate** — suggests a `CLAUDE.md` if rate is low
2. **Wasted short sessions** — sessions that wrote to cache but never read from it
3. **Model mix** — flags high Opus usage and estimates savings if switched to Sonnet
4. **Context near limit** — sessions approaching 200 K tokens, recommends `/compact`
5. **Subscription** — recommends Pro if monthly API spend consistently exceeds $20

Each recommendation shows an estimated monthly saving in USD.

### Alerts
Create budget alerts with a name, period (week / month), optional project scope, and a USD threshold. The dashboard shows a warning banner and a badge on the tab when a threshold is exceeded.

---

## Model pricing

Built-in prices follow Anthropic's official list (May 2026). Non-Anthropic models routed through Claude Code Router (MiMo, DeepSeek) are also included.

Custom prices can be set per model through the **Model prices** modal (header button). Changes trigger an immediate rescan that reprices historical records.

---

## Architecture

```
src/
  main.rs       — CLI (clap), server setup, auto-rescan loop
  lib.rs        — public module re-exports
  api.rs        — Axum router + all HTTP handlers
  scanner.rs    — incremental JSONL walker (mtime-based)
  parser.rs     — line-by-line JSONL deserialisation
  pricing.rs    — ModelPrice, price_for(), effective_cost()
  storage.rs    — SQLite via rusqlite (WAL mode)
  types.rs      — RawLine, RawUsage, UsageRecord
  assets.rs     — rust-embed static files
assets/
  index.html    — main dashboard
  app.js        — dashboard logic (vanilla JS)
  game.js       — Easter egg
  live.html / live.js — real-time view
  style.css     — dark / light / midnight themes
tests/
  integration.rs — 41 tests (storage, scanner, API HTTP)
```

---

## Development

```bash
cargo build                # debug build
cargo test                 # run all 41 tests
cargo install --path .     # install / update the binary
```

After editing assets, touch `src/assets.rs` to force rust-embed to re-embed them:

```bash
touch src/assets.rs && cargo install --path .
```

---

## License

MIT
