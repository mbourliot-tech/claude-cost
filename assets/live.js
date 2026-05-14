const fmtUsd = (n) => "$" + (n ?? 0).toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 });
const fmtUsd2 = (n) => "$" + (n ?? 0).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const fmtNum = (n) => (n ?? 0).toLocaleString();
const fmtTok = (n) => {
  n = n ?? 0;
  if (n >= 1e6) return (n / 1e6).toFixed(2) + "M";
  if (n >= 1e3) return (n / 1e3).toFixed(1) + "k";
  return n.toString();
};
const escHtml = (s) => String(s).replace(/[&<>"']/g, (c) => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
const shortPath = (p) => { if (!p) return ""; const parts = p.replace(/\\/g,"/").split("/").filter(Boolean); return parts.slice(-2).join("/") || p; };

// Color palette per model family
const modelColor = (m) => {
  m = (m || "").toLowerCase();
  if (m.includes("opus"))    return { bg: "rgba(201,139,255,0.15)", color: "#c98bff" };
  if (m.includes("sonnet"))  return { bg: "rgba(107,209,255,0.12)", color: "#6bd1ff" };
  if (m.includes("haiku"))   return { bg: "rgba(94,224,138,0.12)",  color: "#5ee08a" };
  if (m.includes("deepseek"))return { bg: "rgba(255,184,107,0.12)", color: "#ffb86b" };
  if (m.includes("mimo"))    return { bg: "rgba(255,127,156,0.12)", color: "#ff7f9c" };
  return { bg: "rgba(160,163,255,0.12)", color: "#a0a3ff" };
};

const shortModel = (m) => {
  if (!m) return m;
  return m.replace(/^claude-/, "").replace(/-\d{8}$/, "");
};

const relTime = (ts) => {
  const diff = Math.floor((Date.now() - new Date(ts)) / 1000);
  if (diff < 5)   return "à l'instant";
  if (diff < 60)  return `il y a ${diff}s`;
  if (diff < 3600)return `il y a ${Math.floor(diff/60)}min`;
  return new Date(ts).toLocaleTimeString();
};

let knownIds = new Set();
let lastTs = null;
let todayCalls = 0;
let todayCost = 0;
let pollInterval = null;

async function init() {
  // Load initial batch (last 100 calls)
  const rows = await fetch("/api/recent-calls?limit=100").then(r => r.json());
  rows.reverse(); // oldest first for initial render
  for (const r of rows) {
    knownIds.add(r.message_id);
    appendRow(r, false);
    if (!lastTs || r.ts > lastTs) lastTs = r.ts;
  }

  // Load today's summary for stats
  await refreshStats();

  // Update relative timestamps every 30s
  setInterval(updateRelTimes, 30_000);

  // Poll for new calls every 5s when page is visible
  pollInterval = setInterval(poll, 5_000);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      clearInterval(pollInterval);
    } else {
      poll();
      pollInterval = setInterval(poll, 5_000);
    }
  });
}

async function poll() {
  const url = "/api/recent-calls?limit=50" + (lastTs ? "&since=" + encodeURIComponent(lastTs) : "");
  const rows = await fetch(url).then(r => r.json()).catch(() => []);
  let added = 0;
  for (const r of rows) {
    if (knownIds.has(r.message_id)) continue;
    knownIds.add(r.message_id);
    prependRow(r);
    if (!lastTs || r.ts > lastTs) lastTs = r.ts;
    added++;
  }
  if (added > 0) await refreshStats();
  document.getElementById("last-update").textContent = "màj " + new Date().toLocaleTimeString();
}

async function refreshStats() {
  const now = new Date();
  const todayStart = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate())).toISOString();
  const sum = await fetch("/api/summary?since=" + encodeURIComponent(todayStart)).then(r => r.json()).catch(() => null);
  if (sum) {
    document.getElementById("sk-calls").textContent = fmtNum(sum.calls);
    document.getElementById("sk-cost").textContent  = fmtUsd2(sum.total_cost_usd);
  }
  const tbody = document.getElementById("tbl-live-body");
  const count = tbody.rows.length;
  document.getElementById("row-count").textContent = count + " appels affichés";
  if (tbody.rows.length > 0) {
    const first = tbody.rows[0];
    document.getElementById("sk-model").textContent   = first.dataset.model || "—";
    document.getElementById("sk-last-ts").textContent = relTime(first.dataset.ts) || "—";
  }
}

function buildRowHtml(r, isNew) {
  const c = modelColor(r.model);
  const badge = `<span class="model-badge" style="background:${c.bg};color:${c.color}">${escHtml(shortModel(r.model))}</span>`;
  const totalIn = (r.input_tokens || 0) + (r.cache_read_tokens || 0);
  const hitPct = totalIn > 0 ? Math.round((r.cache_read_tokens / totalIn) * 100) + "%" : "—";
  const hitColor = totalIn > 0 && r.cache_read_tokens / totalIn >= 0.8 ? "var(--good)" : "var(--muted)";
  const tier = [r.service_tier, r.speed].filter(Boolean).join(" / ");
  const tierBadge = tier ? `<span class="tier-badge">${escHtml(tier)}</span>` : "—";
  const cacheWrite = (r.cache_5m_tokens || 0) + (r.cache_1h_tokens || 0);

  return `
    <td class="mono" data-ts="${escHtml(r.ts)}">${relTime(r.ts)}</td>
    <td>${badge}</td>
    <td style="max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="${escHtml(r.project_path)}">${escHtml(shortPath(r.project_path))}</td>
    <td class="right" style="font-variant-numeric:tabular-nums">${fmtUsd(r.cost_usd)}</td>
    <td class="right">${fmtTok(r.input_tokens)}</td>
    <td class="right">${fmtTok(r.output_tokens)}</td>
    <td class="right">${fmtTok(r.cache_read_tokens)}${cacheWrite > 0 ? `<br><span style="font-size:10px;color:var(--muted)">+${fmtTok(cacheWrite)} écrit</span>` : ""}</td>
    <td class="right" style="color:${hitColor}">${hitPct}</td>
    <td>${tierBadge}</td>
  `;
}

function appendRow(r, isNew) {
  const tbody = document.getElementById("tbl-live-body");
  const tr = document.createElement("tr");
  tr.dataset.id = r.message_id;
  tr.dataset.model = shortModel(r.model);
  tr.dataset.ts = r.ts;
  if (isNew) tr.classList.add("row-new");
  tr.innerHTML = buildRowHtml(r, isNew);
  tbody.appendChild(tr);
}

function prependRow(r) {
  const tbody = document.getElementById("tbl-live-body");
  const tr = document.createElement("tr");
  tr.dataset.id = r.message_id;
  tr.dataset.model = shortModel(r.model);
  tr.dataset.ts = r.ts;
  tr.classList.add("row-new");
  tr.innerHTML = buildRowHtml(r, true);
  tbody.insertBefore(tr, tbody.firstChild);
  // Cap display at 500 rows
  while (tbody.rows.length > 500) tbody.deleteRow(tbody.rows.length - 1);
}

function updateRelTimes() {
  document.querySelectorAll("#tbl-live-body td[data-ts]").forEach(td => {
    td.textContent = relTime(td.dataset.ts);
  });
  const first = document.getElementById("tbl-live-body").rows[0];
  if (first) document.getElementById("sk-last-ts").textContent = relTime(first.dataset.ts);
}

init().catch(e => { document.getElementById("last-update").textContent = "Erreur: " + e.message; });
