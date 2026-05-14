const $ = (s) => document.querySelector(s);
const fmtUsd = (n) => "$" + (n ?? 0).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const fmtNum = (n) => (n ?? 0).toLocaleString();
const fmtTok = (n) => {
  n = n ?? 0;
  if (n >= 1e9) return (n / 1e9).toFixed(2) + "B";
  if (n >= 1e6) return (n / 1e6).toFixed(2) + "M";
  if (n >= 1e3) return (n / 1e3).toFixed(1) + "k";
  return n.toString();
};

let dayChart = null;
let modelChart = null;

function periodToRange(value) {
  const now = new Date();
  const iso = (d) => d.toISOString();
  switch (value) {
    case "today": {
      const start = new Date(now); start.setUTCHours(0, 0, 0, 0);
      return { since: iso(start), until: null, days: 1, hourly: true };
    }
    case "day": {
      const val = $("#date-picker").value; // "YYYY-MM-DD"
      if (!val) return { since: null, until: null, days: 1, hourly: true };
      const start = new Date(val + "T00:00:00Z");
      const end   = new Date(val + "T23:59:59Z");
      return { since: iso(start), until: iso(end), days: 1, hourly: true };
    }
    case "7d": {
      const s = new Date(now); s.setUTCDate(s.getUTCDate() - 7);
      return { since: iso(s), until: null, days: 7 };
    }
    case "30d": {
      const s = new Date(now); s.setUTCDate(s.getUTCDate() - 30);
      return { since: iso(s), until: null, days: 30 };
    }
    case "month": {
      const s = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
      return { since: iso(s), until: null, days: 31 };
    }
    case "all":
    default:
      return { since: null, until: null, days: 365 };
  }
}

async function jget(url) {
  const r = await fetch(url);
  if (!r.ok) throw new Error(url + " " + r.status);
  return r.json();
}

function qs(params) {
  const u = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) if (v != null) u.set(k, v);
  const s = u.toString();
  return s ? "?" + s : "";
}

async function refresh() {
  const period = $("#period").value;
  const range = periodToRange(period);
  const params = { since: range.since, until: range.until };
  const isToday = range.hourly === true;

  const fetches = [
    jget("/api/summary" + qs(params)),
    jget("/api/by-model" + qs(params)),
    jget("/api/by-project" + qs(params)),
    jget("/api/by-session?limit=20"),
    jget("/api/cache-stats" + qs(params)),
    isToday
      ? jget("/api/by-hour" + qs(params))
      : jget("/api/by-day?days=" + range.days),
    jget("/api/model-prices"),
  ];
  const [summary, byModel, byProject, bySession, cacheStats, timeData, modelPrices] = await Promise.all(fetches);
  priceRows = modelPrices;
  renderModelWarnings(modelPrices);

  refreshAlerts().catch(() => {});

  $("#kpi-cost").textContent = fmtUsd(summary.total_cost_usd);
  $("#kpi-calls").textContent = fmtNum(summary.calls);
  $("#kpi-sessions").textContent = fmtNum(summary.sessions);
  $("#kpi-tokens").textContent = fmtTok((summary.input_tokens || 0) + (summary.output_tokens || 0));

  renderCacheStats(cacheStats);
  if (isToday) {
    const label = period === "day" ? `Coût par heure — ${$("#date-picker").value}` : "Coût par heure — aujourd'hui";
    $("#chart-time-title").textContent = label;
    renderHourChart(timeData);
  } else {
    $("#chart-time-title").textContent = "Coût par jour";
    renderDayChart(timeData);
  }
  renderModelChart(byModel);
  renderProjects(byProject);
  renderSessions(bySession);

  $("#footer-meta").textContent =
    `Tokens — input ${fmtTok(summary.input_tokens)} · output ${fmtTok(summary.output_tokens)} · cache_read ${fmtTok(summary.cache_read_tokens)} · cache_write ${fmtTok((summary.cache_5m_tokens || 0) + (summary.cache_1h_tokens || 0))}`;
}

function renderCacheStats(s) {
  const hitPct = ((s.hit_rate || 0) * 100).toFixed(1) + "%";
  $("#ck-hitrate").textContent = hitPct;
  $("#ck-read").textContent = fmtTok(s.cache_read_tokens);
  $("#ck-savings").textContent = fmtUsd(s.savings_usd);
  $("#ck-write-cost").textContent = fmtUsd(s.write_premium_usd);
  const net = s.net_savings_usd || 0;
  const netEl = $("#ck-net");
  netEl.textContent = fmtUsd(net);
  netEl.className = "value " + (net >= 0 ? "good" : "warn");
}

function renderDayChart(rows) {
  const ctx = $("#chart-day").getContext("2d");
  const labels = rows.map((r) => r.date);
  const costs = rows.map((r) => r.cost_usd);
  if (dayChart) dayChart.destroy();
  dayChart = new Chart(ctx, {
    type: "bar",
    data: {
      labels,
      datasets: [{ label: "Coût USD", data: costs, backgroundColor: "rgba(201, 139, 255, 0.7)", borderColor: "#c98bff", borderWidth: 1 }],
    },
    options: {
      responsive: true,
      plugins: { legend: { labels: { color: "#e6e8ee" } } },
      scales: {
        x: { ticks: { color: "#8a93a6" }, grid: { color: "#2a2f3a" } },
        y: { ticks: { color: "#8a93a6", callback: (v) => "$" + v }, grid: { color: "#2a2f3a" } },
      },
    },
  });
}

function renderHourChart(rows) {
  const ctx = $("#chart-day").getContext("2d");
  // Format "2026-05-14T18:00:00Z" → "18h"
  const labels = rows.map((r) => r.hour.substring(11, 13) + "h");
  const costs = rows.map((r) => r.cost_usd);
  const cacheRead = rows.map((r) => r.cache_read_tokens);
  if (dayChart) dayChart.destroy();
  dayChart = new Chart(ctx, {
    type: "bar",
    data: {
      labels,
      datasets: [
        { label: "Coût USD", data: costs, backgroundColor: "rgba(201, 139, 255, 0.7)", borderColor: "#c98bff", borderWidth: 1, yAxisID: "yCost" },
        { label: "Cache read (tok)", data: cacheRead, type: "line", borderColor: "#6bd1ff", backgroundColor: "rgba(107,209,255,0.15)", borderWidth: 2, pointRadius: 3, tension: 0.3, yAxisID: "yTok" },
      ],
    },
    options: {
      responsive: true,
      plugins: { legend: { labels: { color: "#e6e8ee" } } },
      scales: {
        x: { ticks: { color: "#8a93a6" }, grid: { color: "#2a2f3a" } },
        yCost: { position: "left",  ticks: { color: "#c98bff", callback: (v) => "$" + v }, grid: { color: "#2a2f3a" } },
        yTok:  { position: "right", ticks: { color: "#6bd1ff", callback: (v) => fmtTok(v) }, grid: { display: false } },
      },
    },
  });
}

function renderModelChart(rows) {
  const ctx = $("#chart-model").getContext("2d");
  const labels = rows.map((r) => r.model);
  const costs = rows.map((r) => r.cost_usd);
  const palette = ["#c98bff", "#6bd1ff", "#5ee08a", "#ffb86b", "#ff7f9c", "#a0a3ff", "#ffd166"];
  if (modelChart) modelChart.destroy();
  modelChart = new Chart(ctx, {
    type: "doughnut",
    data: { labels, datasets: [{ data: costs, backgroundColor: labels.map((_, i) => palette[i % palette.length]) }] },
    options: { responsive: true, plugins: { legend: { position: "right", labels: { color: "#e6e8ee" } } } },
  });
}

function renderProjects(rows) {
  const tbody = $("#tbl-projects tbody");
  tbody.innerHTML = "";
  for (const r of rows.slice(0, 15)) {
    const tr = document.createElement("tr");
    tr.innerHTML = `<td>${escapeHtml(shortenPath(r.project_path))}</td><td class="right">${fmtUsd(r.cost_usd)}</td><td class="right">${fmtNum(r.sessions)}</td><td class="right">${fmtNum(r.calls)}</td>`;
    tbody.appendChild(tr);
  }
}

function renderSessions(rows) {
  const tbody = $("#tbl-sessions tbody");
  tbody.innerHTML = "";
  for (const r of rows) {
    const hitRate = (r.input_tokens + r.cache_read_tokens) > 0
      ? ((r.cache_read_tokens / (r.input_tokens + r.cache_read_tokens)) * 100).toFixed(0) + "%"
      : "—";
    const hitClass = parseFloat(hitRate) >= 80 ? "good" : parseFloat(hitRate) >= 40 ? "" : "warn";
    const tr = document.createElement("tr");
    tr.innerHTML = `<td>${r.session_id.slice(0, 8)}…</td><td>${escapeHtml(shortenPath(r.project_path))}</td><td>${shortTs(r.started_at)}</td><td>${shortTs(r.ended_at)}</td><td class="right">${fmtUsd(r.cost_usd)}</td><td class="right">${fmtNum(r.calls)}</td><td class="right" style="color:var(--${hitClass || 'text'})">${hitRate}</td>`;
    tbody.appendChild(tr);
  }
}

function shortenPath(p) {
  if (!p) return "";
  const parts = p.replace(/\\/g, "/").split("/").filter(Boolean);
  return parts.slice(-2).join("/") || p;
}

function shortTs(t) {
  if (!t) return "—";
  return t.replace("T", " ").replace(/\..*$/, "").replace("Z", "");
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

$("#period").addEventListener("change", () => {
  const picker = $("#date-picker");
  if ($("#period").value === "day") {
    // pré-remplir avec aujourd'hui si vide
    if (!picker.value) picker.value = new Date().toISOString().slice(0, 10);
    picker.classList.remove("hidden");
  } else {
    picker.classList.add("hidden");
  }
  refresh();
});
$("#date-picker").addEventListener("change", refresh);
$("#rescan").addEventListener("click", async () => {
  $("#rescan").disabled = true;
  $("#rescan").textContent = "…";
  try {
    await fetch("/api/rescan", { method: "POST" });
    await refresh();
  } finally {
    $("#rescan").disabled = false;
    $("#rescan").textContent = "Rescan";
  }
});

refresh().catch((e) => {
  console.error(e);
  $("#footer-meta").textContent = "Erreur: " + e.message;
});

// ── Modal "Prix des modèles" ─────────────────────────────────────────────────

let priceRows = [];

$("#open-prices").addEventListener("click", openPricesModal);
$("#close-prices").addEventListener("click", closePricesModal);
$("#modal-prices").addEventListener("click", (e) => { if (e.target === $("#modal-prices")) closePricesModal(); });
document.addEventListener("keydown", (e) => { if (e.key === "Escape") closePricesModal(); });

async function openPricesModal() {
  await loadPrices();
  $("#modal-prices").classList.remove("hidden");
}
function closePricesModal() {
  $("#modal-prices").classList.add("hidden");
}

async function loadPrices() {
  priceRows = await jget("/api/model-prices");
  renderPrices();
}

function renderPrices() {
  const tbody = $("#tbl-prices-body");
  tbody.innerHTML = "";
  for (const r of priceRows) {
    const tr = document.createElement("tr");
    tr.dataset.model = r.model;
    tr.innerHTML = buildPriceRowHtml(r);
    tbody.appendChild(tr);
  }
}

function buildPriceRowHtml(r) {
  const badge = r.is_override
    ? `<span class="badge">Personnalisé</span>`
    : (!r.is_known ? `<span class="badge badge-unknown">Inconnu</span>` : "");
  const cacheDisplay = r.cache_read_per_mtok != null ? r.cache_read_per_mtok : "";
  return `
    <td>${escapeHtml(r.model)}${badge}</td>
    <td class="right"><input class="price-input" data-field="input" type="number" step="any" min="0" value="${r.input_per_mtok}" /></td>
    <td class="right"><input class="price-input" data-field="output" type="number" step="any" min="0" value="${r.output_per_mtok}" /></td>
    <td class="right"><input class="price-input" data-field="cache_read" type="number" step="any" min="0" placeholder="×0.1" value="${cacheDisplay}" /></td>
    <td class="right" style="white-space:nowrap;gap:4px;display:flex;justify-content:flex-end;">
      <button class="btn-sm primary" onclick="savePrice(this)">Sauvegarder</button>
      ${r.is_override ? `<button class="btn-sm danger" onclick="resetPrice(this)">Réinitialiser</button>` : ""}
    </td>
  `;
}

async function savePrice(btn) {
  const tr = btn.closest("tr");
  const model = tr.dataset.model;
  const inp = (sel) => tr.querySelector(`input[data-field="${sel}"]`);
  const body = {
    input_per_mtok: parseFloat(inp("input").value) || 0,
    output_per_mtok: parseFloat(inp("output").value) || 0,
    cache_read_per_mtok: inp("cache_read").value.trim() !== "" ? parseFloat(inp("cache_read").value) : null,
  };
  btn.disabled = true;
  btn.textContent = "…";
  try {
    await fetch(`/api/model-prices/${encodeURIComponent(model)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    await loadPrices();
    await refresh();
  } finally {
    btn.disabled = false;
    btn.textContent = "Sauvegarder";
  }
}

async function resetPrice(btn) {
  const tr = btn.closest("tr");
  const model = tr.dataset.model;
  btn.disabled = true;
  btn.textContent = "…";
  try {
    await fetch(`/api/model-prices/${encodeURIComponent(model)}`, { method: "DELETE" });
    await loadPrices();
    await refresh();
  } finally {
    btn.disabled = false;
    btn.textContent = "Réinitialiser";
  }
}

// ── Avertissement modèles inconnus ───────────────────────────────────────────

function renderModelWarnings(models) {
  const unknowns = models.filter((m) => !m.is_known && !m.is_override);
  const banner = $("#model-warning-banner");
  const badge = $("#unknown-badge");
  if (unknowns.length === 0) {
    banner.classList.add("hidden");
    badge.classList.add("hidden");
    return;
  }
  badge.textContent = unknowns.length;
  badge.classList.remove("hidden");
  banner.classList.remove("hidden");
  const names = unknowns.map((m) => escapeHtml(m.model)).join(", ");
  $("#model-warning-text").textContent =
    `${unknowns.length} modèle(s) sans prix configuré — appels comptabilisés à $0 : ${names}.`;
}

$("#model-warning-configure").addEventListener("click", openPricesModal);

// ── Alertes budget ───────────────────────────────────────────────────────────

let allProjects = [];
let cachedAlerts = [];

async function refreshAlerts() {
  cachedAlerts = await jget("/api/alerts");
  renderAlertBanner(cachedAlerts);
  renderAlertsTable(cachedAlerts);
}

function renderAlertBanner(alerts) {
  const triggered = alerts.filter((a) => a.is_triggered);
  const banner = $("#alert-banner");
  if (triggered.length === 0) {
    banner.classList.add("hidden");
    banner.classList.remove("critical");
    return;
  }
  banner.classList.remove("hidden");
  const msgs = triggered.map((a) => {
    const period = a.period === "week" ? "cette semaine" : "ce mois";
    const proj = a.project_path ? ` [${shortenPath(a.project_path)}]` : "";
    return `${escapeHtml(a.name)}${proj} : ${fmtUsd(a.current_usd)} / ${fmtUsd(a.threshold_usd)} ${period}`;
  });
  $("#alert-banner-text").textContent = msgs.join("  ·  ");
}

function renderAlertsTable(alerts) {
  const empty = $("#alerts-empty");
  const tbl = $("#tbl-alerts");
  const tbody = $("#tbl-alerts-body");
  if (alerts.length === 0) {
    empty.classList.remove("hidden");
    tbl.classList.add("hidden");
    return;
  }
  empty.classList.add("hidden");
  tbl.classList.remove("hidden");
  tbody.innerHTML = "";
  for (const a of alerts) {
    const tr = document.createElement("tr");
    tr.dataset.id = a.id;
    const periodLabel = a.period === "week" ? "Semaine" : "Mois";
    const proj = a.project_path ? escapeHtml(shortenPath(a.project_path)) : "<em style='color:var(--muted)'>global</em>";
    const pct = a.threshold_usd > 0 ? Math.min(100, (a.current_usd / a.threshold_usd) * 100).toFixed(0) : 0;
    const statusClass = a.is_triggered ? "status-warn" : "status-ok";
    const statusText = a.is_triggered ? `⚠ ${pct}%` : `✓ ${pct}%`;
    tr.innerHTML = `
      <td><input type="checkbox" class="toggle-enabled" ${a.enabled ? "checked" : ""} title="Activer/désactiver" onchange="toggleAlert(${a.id}, this)" /> ${escapeHtml(a.name)}</td>
      <td>${periodLabel}</td>
      <td>${proj}</td>
      <td class="right">${fmtUsd(a.threshold_usd)}</td>
      <td class="right">${fmtUsd(a.current_usd)}</td>
      <td class="right ${statusClass}">${statusText}</td>
      <td class="right"><button class="btn-sm danger" onclick="deleteAlert(${a.id})">×</button></td>
    `;
    tbody.appendChild(tr);
  }
}

async function deleteAlert(id) {
  await fetch(`/api/alerts/${id}`, { method: "DELETE" });
  await refreshAlerts();
}

async function toggleAlert(id, checkbox) {
  const a = cachedAlerts.find((x) => x.id === id);
  if (!a) return;
  await fetch(`/api/alerts/${id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      name: a.name,
      period: a.period,
      project_path: a.project_path,
      threshold_usd: a.threshold_usd,
      enabled: checkbox.checked,
    }),
  });
  await refreshAlerts();
}

// ── Modal alerte ─────────────────────────────────────────────────────────────

$("#btn-add-alert").addEventListener("click", openAlertModal);
$("#close-alert").addEventListener("click", closeAlertModal);
$("#cancel-alert").addEventListener("click", closeAlertModal);
$("#modal-alert").addEventListener("click", (e) => { if (e.target === $("#modal-alert")) closeAlertModal(); });
$("#submit-alert").addEventListener("click", submitAlert);
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") { closePricesModal(); closeAlertModal(); }
});

async function openAlertModal() {
  // Populate project list
  const projects = await jget("/api/by-project");
  allProjects = projects;
  const sel = $("#alert-project");
  sel.innerHTML = `<option value="">Global (tous projets)</option>`;
  for (const p of projects) {
    const opt = document.createElement("option");
    opt.value = p.project_path;
    opt.textContent = shortenPath(p.project_path);
    sel.appendChild(opt);
  }
  // Reset form
  $("#alert-name").value = "";
  $("#alert-period").value = "month";
  $("#alert-project").value = "";
  $("#alert-threshold").value = "";
  $("#modal-alert").classList.remove("hidden");
  $("#alert-name").focus();
}

function closeAlertModal() {
  $("#modal-alert").classList.add("hidden");
}

async function submitAlert() {
  const name = $("#alert-name").value.trim();
  const period = $("#alert-period").value;
  const project_path = $("#alert-project").value || null;
  const threshold_usd = parseFloat($("#alert-threshold").value);
  if (!name || isNaN(threshold_usd) || threshold_usd <= 0) {
    $("#alert-name").focus();
    return;
  }
  const btn = $("#submit-alert");
  btn.disabled = true;
  btn.textContent = "…";
  try {
    await fetch("/api/alerts", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, period, project_path, threshold_usd }),
    });
    closeAlertModal();
    await refreshAlerts();
  } finally {
    btn.disabled = false;
    btn.textContent = "Créer";
  }
}
