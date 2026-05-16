// ── Internationalisation ──────────────────────────────────────────────────────

const TRANSLATIONS = {
  fr: {
    'tab.overview': "Vue d'ensemble", 'tab.sessions': "Sessions", 'tab.plans': "Plans", 'tab.alerts': "Alertes",
    'ctrl.period': "Période", 'ctrl.today': "Aujourd'hui", 'ctrl.7d': "7 jours", 'ctrl.30d': "30 jours",
    'ctrl.month': "Ce mois", 'ctrl.all': "Tout", 'ctrl.day': "Jour précis…",
    'ctrl.rescan': "Rescan", 'ctrl.prices': "Prix des modèles", 'ctrl.live': "Temps réel", 'ctrl.estimates': "+overhead",
    'kpi.cost': "Coût total", 'kpi.calls': "Appels API", 'kpi.sessions': "Sessions", 'kpi.tokens': "Tokens (entrée + sortie)",
    'cache.hitrate': "Cache hit rate", 'cache.read': "Tokens lus depuis cache", 'cache.savings': "Économies brutes",
    'cache.write': "Surcoût écriture", 'cache.net': "Gain net cache",
    'chart.day': "Coût par jour", 'chart.hour.today': "Coût par heure — aujourd'hui",
    'chart.hour.day': "Coût par heure — ", 'chart.model': "Répartition par modèle",
    'chart.projects': "Top projets", 'chart.weekday': "Coût par jour de semaine", 'chart.hourofday': "Coût par heure de la journée",
    'chart.cost_usd': "Coût USD", 'chart.cache_tok': "Cache read (tok)",
    'sessions.limit': "Limite", 'sessions.project': "Projet", 'sessions.all': "Tous", 'sessions.export': "↓ Export CSV",
    'sessions.col.session': "Session", 'sessions.col.project': "Projet", 'sessions.col.start': "Début",
    'sessions.col.end': "Fin", 'sessions.col.cost': "Coût", 'sessions.col.calls': "Appels",
    'sessions.col.cache': "Cache hit", 'sessions.col.ctx': "Ctx max",
    'plans.title': "Comparaison de plans", 'plans.period': "Période", 'plans.months_label': "mois",
    'plans.col.plan': "Plan", 'plans.col.price': "Prix/mois", 'plans.col.total': "Coût sur période",
    'plans.col.api': "Votre API", 'plans.col.delta': "Δ économie", 'plans.col.months': "Mois rentables", 'plans.col.regularity': "Régularité",
    'plans.saved': "économisé", 'plans.lost': "perdu", 'plans.week_label': "Semaine", 'plans.month_label': "Mois",
    'alerts.title': "Alertes budget", 'alerts.add': "+ Nouvelle alerte", 'alerts.empty': "Aucune alerte configurée.",
    'alerts.col.name': "Nom", 'alerts.col.period': "Période", 'alerts.col.project': "Projet",
    'alerts.col.threshold': "Seuil", 'alerts.col.current': "Actuel", 'alerts.col.status': "Statut",
    'modal.prices.title': "Prix des modèles", 'modal.prices.model': "Modèle", 'modal.prices.input': "Input",
    'modal.prices.output': "Output", 'modal.prices.cache': "Cache read",
    'modal.prices.note': "Cache read vide = multiplicateur Anthropic ×0.1 sur l'input.",
    'modal.alert.title': "Nouvelle alerte", 'modal.alert.name': "Nom", 'modal.alert.period': "Période",
    'modal.alert.week': "Cette semaine", 'modal.alert.month': "Ce mois", 'modal.alert.project': "Projet",
    'modal.alert.project_hint': "(vide = global)", 'modal.alert.global': "Global (tous projets)",
    'modal.alert.threshold': "Seuil ($)", 'modal.alert.cancel': "Annuler", 'modal.alert.create': "Créer",
    'banner.configure': "Configurer les prix →", 'banner.goto': "Voir les alertes →",
    'banner.models_a': "modèle(s) sans prix configuré — appels comptabilisés à $0 :",
    'trend.today': "vs hier", 'trend.7d': "vs 7j préc.", 'trend.30d': "vs 30j préc.", 'trend.month': "vs mois préc.", 'trend.day': "vs j-1",
    'btn.save': "Sauvegarder", 'btn.reset': "Réinitialiser", 'badge.custom': "Personnalisé", 'badge.unknown': "Inconnu",
    'theme.dark': "🌙 Sombre", 'theme.light': "☀️ Clair", 'theme.midnight': "🌌 Minuit",
    'global': "global", 'week': "cette semaine", 'month_period': "ce mois",
    'tab.insights': "Optimisation",
    'insights.title': "Réduction des coûts",
    'insights.avoidable': "Économie potentielle estimée",
    'insights.none': "✅ Aucune optimisation majeure détectée — votre utilisation est efficace.",
    'insights.cache.good.title': "Excellent taux de cache",
    'insights.cache.good.desc': "Votre taux de cache ({rate}%) est optimal. Le contexte est bien réutilisé entre les appels.",
    'insights.cache.med.title': "Taux de cache perfectible",
    'insights.cache.med.desc': "Votre taux de cache est de {rate}%. Un CLAUDE.md structuré avec vos instructions permanentes en début de conversation peut le porter à 80%+.",
    'insights.cache.low.title': "Taux de cache faible — action recommandée",
    'insights.cache.low.desc': "Seulement {rate}% des tokens d'entrée sont lus depuis le cache. Chaque conversation recrée le contexte depuis zéro.",
    'insights.cache.action': "Placez vos instructions système et le contexte projet dans un CLAUDE.md. Claude mettra automatiquement ce bloc en cache dès le 2ème appel.",
    'insights.waste.title': "Sessions courtes sans réutilisation du cache",
    'insights.waste.desc': "{n} session(s) ont écrit dans le cache ({cost}) sans jamais le relire (≤5 appels). Le coût d'écriture du cache est perdu.",
    'insights.waste.action': "Regroupez vos questions en une session plus longue, ou utilisez /clear entre les sujets distincts pour ne pas écrire de cache inutilement.",
    'insights.model.title': "Opus représente une grande part du coût",
    'insights.model.desc': "claude-opus coûte $5/Mtok en input contre $3 pour Sonnet (1,7× plus cher). Opus représente {pct}% de votre dépense totale.",
    'insights.model.action': "Réservez Opus aux tâches complexes (architecture, débogage difficile). Utilisez Sonnet pour l'explication de code, le refactoring simple et la génération de tests.",
    'insights.ctx.title': "Plusieurs sessions proches de la limite de contexte",
    'insights.ctx.desc': "{n} session(s) ont dépassé 150 000 tokens de contexte. Au-delà de 200K, Claude ne peut plus traiter la demande et le cache doit être recréé.",
    'insights.ctx.action': "Utilisez /compact régulièrement dans les longues sessions pour résumer le contexte et réduire la taille de la fenêtre.",
    'insights.plan.title': "L'abonnement Pro réduirait vos coûts",
    'insights.plan.desc': "Votre dépense API moyenne sur les derniers mois complets est de {avg}/mois. L'abonnement Pro à $20/mois vous économiserait {save}/mois.",
    'insights.plan.action': "Passez au plan Pro sur claude.ai. Si votre rythme mensuel dépasse $100, envisagez Max 5×.",
  },
  en: {
    'tab.overview': "Overview", 'tab.sessions': "Sessions", 'tab.plans': "Plans", 'tab.alerts': "Alerts",
    'ctrl.period': "Period", 'ctrl.today': "Today", 'ctrl.7d': "7 days", 'ctrl.30d': "30 days",
    'ctrl.month': "This month", 'ctrl.all': "All", 'ctrl.day': "Specific day…",
    'ctrl.rescan': "Rescan", 'ctrl.prices': "Model prices", 'ctrl.live': "Live", 'ctrl.estimates': "+overhead",
    'kpi.cost': "Total cost", 'kpi.calls': "API calls", 'kpi.sessions': "Sessions", 'kpi.tokens': "Tokens (in + out)",
    'cache.hitrate': "Cache hit rate", 'cache.read': "Tokens read from cache", 'cache.savings': "Gross savings",
    'cache.write': "Write overhead", 'cache.net': "Net cache gain",
    'chart.day': "Cost per day", 'chart.hour.today': "Cost per hour — today",
    'chart.hour.day': "Cost per hour — ", 'chart.model': "By model",
    'chart.projects': "Top projects", 'chart.weekday': "Cost by weekday", 'chart.hourofday': "Cost by hour of day",
    'chart.cost_usd': "Cost USD", 'chart.cache_tok': "Cache read (tok)",
    'sessions.limit': "Limit", 'sessions.project': "Project", 'sessions.all': "All", 'sessions.export': "↓ Export CSV",
    'sessions.col.session': "Session", 'sessions.col.project': "Project", 'sessions.col.start': "Start",
    'sessions.col.end': "End", 'sessions.col.cost': "Cost", 'sessions.col.calls': "Calls",
    'sessions.col.cache': "Cache hit", 'sessions.col.ctx': "Max ctx",
    'plans.title': "Plan comparison", 'plans.period': "Period", 'plans.months_label': "months",
    'plans.col.plan': "Plan", 'plans.col.price': "Price/month", 'plans.col.total': "Cost over period",
    'plans.col.api': "Your API", 'plans.col.delta': "Δ savings", 'plans.col.months': "Profitable months", 'plans.col.regularity': "Consistency",
    'plans.saved': "saved", 'plans.lost': "lost", 'plans.week_label': "Week", 'plans.month_label': "Month",
    'alerts.title': "Budget alerts", 'alerts.add': "+ New alert", 'alerts.empty': "No alerts configured.",
    'alerts.col.name': "Name", 'alerts.col.period': "Period", 'alerts.col.project': "Project",
    'alerts.col.threshold': "Threshold", 'alerts.col.current': "Current", 'alerts.col.status': "Status",
    'modal.prices.title': "Model prices", 'modal.prices.model': "Model", 'modal.prices.input': "Input",
    'modal.prices.output': "Output", 'modal.prices.cache': "Cache read",
    'modal.prices.note': "Empty cache read = Anthropic ×0.1 multiplier on input.",
    'modal.alert.title': "New alert", 'modal.alert.name': "Name", 'modal.alert.period': "Period",
    'modal.alert.week': "This week", 'modal.alert.month': "This month", 'modal.alert.project': "Project",
    'modal.alert.project_hint': "(empty = global)", 'modal.alert.global': "Global (all projects)",
    'modal.alert.threshold': "Threshold ($)", 'modal.alert.cancel': "Cancel", 'modal.alert.create': "Create",
    'banner.configure': "Configure prices →", 'banner.goto': "View alerts →",
    'banner.models_a': "model(s) without configured price — billed at $0:",
    'trend.today': "vs yesterday", 'trend.7d': "vs prev 7d", 'trend.30d': "vs prev 30d", 'trend.month': "vs prev month", 'trend.day': "vs day-1",
    'btn.save': "Save", 'btn.reset': "Reset", 'badge.custom': "Custom", 'badge.unknown': "Unknown",
    'theme.dark': "🌙 Dark", 'theme.light': "☀️ Light", 'theme.midnight': "🌌 Midnight",
    'global': "global", 'week': "this week", 'month_period': "this month",
    'tab.insights': "Insights",
    'insights.title': "Cost reduction",
    'insights.avoidable': "Estimated potential savings",
    'insights.none': "✅ No major optimizations found — your usage is efficient.",
    'insights.cache.good.title': "Excellent cache hit rate",
    'insights.cache.good.desc': "Your cache hit rate ({rate}%) is optimal. Context is well reused across calls.",
    'insights.cache.med.title': "Cache hit rate could improve",
    'insights.cache.med.desc': "Your cache hit rate is {rate}%. A structured CLAUDE.md with your permanent instructions at the start of conversations can bring it to 80%+.",
    'insights.cache.low.title': "Low cache hit rate — action recommended",
    'insights.cache.low.desc': "Only {rate}% of input tokens are read from cache. Each conversation recreates context from scratch.",
    'insights.cache.action': "Place your system instructions and project context in a CLAUDE.md file. Claude will automatically cache this block from the 2nd call onwards.",
    'insights.waste.title': "Short sessions with wasted cache",
    'insights.waste.desc': "{n} session(s) wrote to cache ({cost}) without ever reading from it (≤5 calls). Cache write cost was lost.",
    'insights.waste.action': "Group your questions into longer sessions, or use /clear between unrelated topics to avoid writing useless cache.",
    'insights.model.title': "Opus represents a large share of costs",
    'insights.model.desc': "claude-opus costs $5/Mtok input vs $3 for Sonnet (1.7× more expensive). Opus accounts for {pct}% of your total spend.",
    'insights.model.action': "Reserve Opus for complex tasks (architecture, hard debugging). Use Sonnet for code explanation, simple refactoring and test generation.",
    'insights.ctx.title': "Several sessions near context limit",
    'insights.ctx.desc': "{n} session(s) exceeded 150,000 context tokens. Above 200K, Claude can no longer process requests and cache must be rebuilt.",
    'insights.ctx.action': "Use /compact regularly in long sessions to summarize context and reduce the window size.",
    'insights.plan.title': "Pro subscription would reduce your costs",
    'insights.plan.desc': "Your average API spend over recent complete months is {avg}/month. The Pro plan at $20/month would save you {save}/month.",
    'insights.plan.action': "Switch to the Pro plan on claude.ai. If your monthly spend regularly exceeds $100, consider Max 5×.",
  },
};

let currentLang      = localStorage.getItem('cc-lang')      || 'fr';
let currentTheme     = localStorage.getItem('cc-theme')     || 'dark';
let currentEstimates = localStorage.getItem('cc-estimates') === 'true';

function t(key) { return TRANSLATIONS[currentLang][key] ?? TRANSLATIONS['fr'][key] ?? key; }

function applyTranslations() {
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  document.querySelectorAll('[data-i18n-opt]').forEach((el) => {
    el.textContent = t(el.dataset.i18nOpt);
  });
  document.documentElement.lang = currentLang;
}

function applyTheme(theme) {
  currentTheme = theme;
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('cc-theme', theme);
}

function applyLang(lang) {
  currentLang = lang;
  localStorage.setItem('cc-lang', lang);
  applyTranslations();
}

function chartColors() {
  const s = getComputedStyle(document.documentElement);
  const v = (n) => s.getPropertyValue(n).trim();
  return { muted: v('--muted'), border: v('--border'), text: v('--text'), surface2: v('--surface-2') };
}

// Initialise thème et langue au chargement
applyTheme(currentTheme);
applyTranslations();

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
let weekdayChart = null;
let hourOfDayChart = null;
let lastAutoTs = null;

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

function prevPeriodParams(period, range) {
  const now = new Date();
  const iso = (d) => d.toISOString();
  switch (period) {
    case "today": {
      const y = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() - 1));
      return { since: iso(y), until: iso(new Date(Date.UTC(y.getUTCFullYear(), y.getUTCMonth(), y.getUTCDate(), 23, 59, 59))) };
    }
    case "day": {
      if (!range.since) return null;
      const d = new Date(range.since); d.setUTCDate(d.getUTCDate() - 1);
      return { since: iso(new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate()))),
               until: iso(new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate(), 23, 59, 59))) };
    }
    case "7d": {
      const end = new Date(now); end.setUTCDate(end.getUTCDate() - 7);
      const start = new Date(end); start.setUTCDate(start.getUTCDate() - 7);
      return { since: iso(start), until: iso(end) };
    }
    case "30d": {
      const end = new Date(now); end.setUTCDate(end.getUTCDate() - 30);
      const start = new Date(end); start.setUTCDate(start.getUTCDate() - 30);
      return { since: iso(start), until: iso(end) };
    }
    case "month": {
      const first = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
      const lastPrev = new Date(first); lastPrev.setUTCDate(0);
      const firstPrev = new Date(Date.UTC(lastPrev.getUTCFullYear(), lastPrev.getUTCMonth(), 1));
      return { since: iso(firstPrev), until: iso(lastPrev) };
    }
    default: return null;
  }
}

async function refresh() {
  const period = $("#period").value;
  const range = periodToRange(period);
  const params = { since: range.since, until: range.until, estimate_internal: currentEstimates || undefined };
  const isToday = range.hourly === true;
  const prevP = prevPeriodParams(period, range);
  const sessLimit = parseInt($("#sessions-limit")?.value, 10) || 50;
  const sessProject = $("#sessions-project")?.value || undefined;

  const fetches = [
    jget("/api/summary" + qs(params)),
    jget("/api/by-model" + qs(params)),
    jget("/api/by-project" + qs(params)),
    jget("/api/by-session" + qs({ limit: sessLimit, project: sessProject })),
    jget("/api/cache-stats" + qs(params)),
    isToday ? jget("/api/by-hour" + qs(params)) : jget("/api/by-day?days=" + range.days),
    jget("/api/model-prices"),
    prevP ? jget("/api/summary" + qs(prevP)) : Promise.resolve(null),
    jget("/api/by-weekday" + qs(params)),
    jget("/api/by-hourofday" + qs(params)),
  ];
  const [summary, byModel, byProject, bySession, cacheStats, timeData, modelPrices, prevSummary, weekdayData, hourData] =
    await Promise.all(fetches);

  priceRows = modelPrices;
  renderModelWarnings(modelPrices);
  refreshAlerts().catch(() => {});
  refreshPlans().catch(() => {});
  refreshInsights().catch(() => {});

  $("#kpi-cost").textContent = fmtUsd(summary.total_cost_usd);
  $("#kpi-calls").textContent = fmtNum(summary.calls);
  $("#kpi-sessions").textContent = fmtNum(summary.sessions);
  $("#kpi-tokens").textContent = fmtTok((summary.input_tokens || 0) + (summary.output_tokens || 0));
  renderTrends(summary, prevSummary, period);

  renderCacheStats(cacheStats);
  if (isToday) {
    const label = period === "day" ? t('chart.hour.day') + $("#date-picker").value : t('chart.hour.today');
    $("#chart-time-title").textContent = label;
    renderHourChart(timeData);
  } else {
    $("#chart-time-title").textContent = "Coût par jour";
    renderDayChart(timeData);
  }
  renderModelChart(byModel);
  renderProjects(byProject);
  renderSessions(bySession);
  renderWeekdayChart(weekdayData);
  renderHourOfDayChart(hourData);

  const now = new Date();
  $("#footer-meta").textContent =
    `Tokens — input ${fmtTok(summary.input_tokens)} · output ${fmtTok(summary.output_tokens)} · cache_read ${fmtTok(summary.cache_read_tokens)} · cache_write ${fmtTok((summary.cache_5m_tokens || 0) + (summary.cache_1h_tokens || 0))}`;
  $("#footer-refresh").textContent = `Mis à jour ${now.toLocaleTimeString()}`;
}

function renderTrends(curr, prev, period) {
  const fields = [
    { id: "kpi-cost",     curr: curr.total_cost_usd,                                   prev: prev?.total_cost_usd,     costDir: true },
    { id: "kpi-calls",    curr: curr.calls,                                             prev: prev?.calls,              costDir: false },
    { id: "kpi-sessions", curr: curr.sessions,                                          prev: prev?.sessions,           costDir: false },
    { id: "kpi-tokens",   curr: (curr.input_tokens||0)+(curr.output_tokens||0),         prev: prev ? (prev.input_tokens||0)+(prev.output_tokens||0) : null, costDir: false },
  ];
  const prevLabel = { today: t('trend.today'), "7d": t('trend.7d'), "30d": t('trend.30d'), month: t('trend.month'), day: t('trend.day') }[period] || "";
  for (const f of fields) {
    const el = document.getElementById(f.id + "-trend");
    if (!el) continue;
    if (!prev || !f.prev || f.prev === 0) { el.textContent = ""; el.className = "kpi-trend"; continue; }
    const delta = (f.curr - f.prev) / f.prev;
    const pct = (Math.abs(delta) * 100).toFixed(0) + "%";
    const up = delta >= 0;
    el.textContent = `${up ? "▲" : "▼"} ${pct} ${prevLabel}`;
    el.className = "kpi-trend " + (f.costDir ? (up ? "trend-up" : "trend-down") : (up ? "trend-neutral" : "trend-neutral"));
  }
}

function miniChartOpts() {
  const c = chartColors();
  return {
    responsive: true,
    plugins: { legend: { display: false } },
    scales: {
      x: { ticks: { color: c.muted, font: { size: 11 } }, grid: { color: c.border } },
      y: { ticks: { color: c.muted, callback: (v) => "$" + v, font: { size: 11 } }, grid: { color: c.border } },
    },
  };
}

function renderWeekdayChart(rows) {
  const ctx = $("#chart-weekday")?.getContext("2d");
  if (!ctx) return;
  if (weekdayChart) weekdayChart.destroy();
  weekdayChart = new Chart(ctx, {
    type: "bar",
    data: { labels: rows.map((r) => r.label), datasets: [{ data: rows.map((r) => r.cost_usd), backgroundColor: "rgba(107,209,255,0.65)", borderColor: "#6bd1ff", borderWidth: 1 }] },
    options: miniChartOpts(),
  });
}

function renderHourOfDayChart(rows) {
  const ctx = $("#chart-hourofday")?.getContext("2d");
  if (!ctx) return;
  if (hourOfDayChart) hourOfDayChart.destroy();
  hourOfDayChart = new Chart(ctx, {
    type: "bar",
    data: { labels: rows.map((r) => r.hour + "h"), datasets: [{ data: rows.map((r) => r.cost_usd), backgroundColor: "rgba(94,224,138,0.65)", borderColor: "#5ee08a", borderWidth: 1 }] },
    options: miniChartOpts(),
  });
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

function chartScales(c) {
  return {
    x: { ticks: { color: c.muted }, grid: { color: c.border } },
    y: { ticks: { color: c.muted, callback: (v) => "$" + v }, grid: { color: c.border } },
  };
}

function renderDayChart(rows) {
  const c = chartColors();
  const ctx = $("#chart-day").getContext("2d");
  if (dayChart) dayChart.destroy();
  dayChart = new Chart(ctx, {
    type: "bar",
    data: {
      labels: rows.map((r) => r.date),
      datasets: [{ label: t('chart.cost_usd'), data: rows.map((r) => r.cost_usd), backgroundColor: "rgba(201,139,255,0.7)", borderColor: "#c98bff", borderWidth: 1 }],
    },
    options: { responsive: true, plugins: { legend: { labels: { color: c.text } } }, scales: chartScales(c) },
  });
}

function renderHourChart(rows) {
  const c = chartColors();
  const ctx = $("#chart-day").getContext("2d");
  if (dayChart) dayChart.destroy();
  dayChart = new Chart(ctx, {
    type: "bar",
    data: {
      labels: rows.map((r) => r.hour.substring(11, 13) + "h"),
      datasets: [
        { label: t('chart.cost_usd'), data: rows.map((r) => r.cost_usd), backgroundColor: "rgba(201,139,255,0.7)", borderColor: "#c98bff", borderWidth: 1, yAxisID: "yCost" },
        { label: t('chart.cache_tok'), data: rows.map((r) => r.cache_read_tokens), type: "line", borderColor: "#6bd1ff", backgroundColor: "rgba(107,209,255,0.15)", borderWidth: 2, pointRadius: 3, tension: 0.3, yAxisID: "yTok" },
      ],
    },
    options: {
      responsive: true,
      plugins: { legend: { labels: { color: c.text } } },
      scales: {
        x: { ticks: { color: c.muted }, grid: { color: c.border } },
        yCost: { position: "left",  ticks: { color: "#c98bff", callback: (v) => "$" + v }, grid: { color: c.border } },
        yTok:  { position: "right", ticks: { color: "#6bd1ff", callback: (v) => fmtTok(v) }, grid: { display: false } },
      },
    },
  });
}

function renderModelChart(rows) {
  const c = chartColors();
  const ctx = $("#chart-model").getContext("2d");
  const palette = ["#c98bff", "#6bd1ff", "#5ee08a", "#ffb86b", "#ff7f9c", "#a0a3ff", "#ffd166"];
  if (modelChart) modelChart.destroy();
  modelChart = new Chart(ctx, {
    type: "doughnut",
    data: { labels: rows.map((r) => r.model), datasets: [{ data: rows.map((r) => r.cost_usd), backgroundColor: rows.map((_, i) => palette[i % palette.length]) }] },
    options: { responsive: true, plugins: { legend: { position: "right", labels: { color: c.text } } } },
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

const CTX_1M   = 1_048_576;
const CTX_256K = 262_144;
const CTX_200K = 200_000;

function ctxColor(tokens) {
  if (tokens >= CTX_1M)   return "var(--danger, #ff5572)";
  if (tokens >= CTX_256K) return "var(--warn)";
  if (tokens >= CTX_200K) return "#f0c040";
  return "var(--muted)";
}

function ctxLabel(tokens) {
  const t = fmtTok(tokens);
  if (tokens >= CTX_1M)   return `${t} >1M`;
  if (tokens >= CTX_256K) return `${t} >256K`;
  if (tokens >= CTX_200K) return `${t} >200K`;
  return t;
}

function renderSessions(rows) {
  const tbody = $("#tbl-sessions tbody");
  tbody.innerHTML = "";
  for (const r of rows) {
    const hitRate = (r.input_tokens + r.cache_read_tokens) > 0
      ? ((r.cache_read_tokens / (r.input_tokens + r.cache_read_tokens)) * 100).toFixed(0) + "%"
      : "—";
    const hitClass = parseFloat(hitRate) >= 80 ? "good" : parseFloat(hitRate) >= 40 ? "" : "warn";
    const ctx = r.peak_context_tokens || 0;
    const tools = [];
    if (r.web_search_requests > 0) tools.push(`🔍${r.web_search_requests}`);
    if (r.web_fetch_requests > 0) tools.push(`🌐${r.web_fetch_requests}`);
    const toolSuffix = tools.length ? ` <span style="color:var(--muted);font-size:10px">${tools.join(" ")}</span>` : "";
    const tr = document.createElement("tr");
    tr.innerHTML = `<td>${r.session_id.slice(0, 8)}…</td><td>${escapeHtml(shortenPath(r.project_path))}</td><td>${shortTs(r.started_at)}</td><td>${shortTs(r.ended_at)}</td><td class="right">${fmtUsd(r.cost_usd)}</td><td class="right">${fmtNum(r.calls)}${toolSuffix}</td><td class="right" style="color:var(--${hitClass || 'text'})">${hitRate}</td><td class="right" style="color:${ctxColor(ctx)}" title="${ctx.toLocaleString()} tokens">${ctxLabel(ctx)}</td>`;
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

refresh().then(async () => {
  try { const d = await jget("/api/last-timestamp"); lastAutoTs = d.last_timestamp; } catch (_) {}
}).catch((e) => {
  console.error(e);
  $("#footer-meta").textContent = "Erreur: " + e.message;
});

// ── Optimisation / Insights ───────────────────────────────────────────────────

async function refreshInsights() {
  const [wasteStats, allModels, allSummary, monthData] = await Promise.all([
    jget("/api/waste-stats"),
    jget("/api/by-model"),
    jget("/api/summary"),
    jget("/api/by-month?months=6"),
  ]);
  renderInsights(wasteStats, allModels, allSummary, monthData);
}

function renderInsights(waste, byModel, summary, months) {
  const list = $("#insights-list");
  const cards = [];
  let totalAvoidable = 0;

  // ── R1 : Cache hit rate ─────────────────────────────────────────────────────
  const rate = waste.alltime_hit_rate;
  const ratePct = (rate * 100).toFixed(1);
  if (rate >= 0.75) {
    cards.push({ sev: "good", icon: "✅", title: t('insights.cache.good.title'),
      desc: t('insights.cache.good.desc').replace("{rate}", ratePct), action: null, savings: null });
  } else {
    // Estimated savings: if we reach 80% hit rate, the delta in cost would be...
    // savings ≈ total_cost × (target - actual) × cache_discount (cache is ~10% of input)
    const savings = summary.total_cost_usd * (0.80 - rate) * 0.45;
    totalAvoidable += Math.max(0, savings);
    const sev = rate < 0.5 ? "high" : "medium";
    const key = rate < 0.5 ? 'insights.cache.low' : 'insights.cache.med';
    cards.push({ sev, icon: sev === "high" ? "🔴" : "🟡",
      title: t(key + '.title'),
      desc: t(key + '.desc').replace("{rate}", ratePct),
      action: t('insights.cache.action'),
      savings: Math.max(0, savings) });
  }

  // ── R2 : Sessions courtes gaspillées ────────────────────────────────────────
  if (waste.wasted_sessions > 0) {
    totalAvoidable += waste.wasted_cost_usd;
    cards.push({ sev: "medium", icon: "🟡",
      title: t('insights.waste.title'),
      desc: t('insights.waste.desc').replace("{n}", waste.wasted_sessions).replace("{cost}", fmtUsd(waste.wasted_cost_usd)),
      action: t('insights.waste.action'),
      savings: waste.wasted_cost_usd });
  }

  // ── R3 : Mix modèles Opus ───────────────────────────────────────────────────
  const opusCost = byModel.filter((m) => m.model.toLowerCase().includes("opus")).reduce((s, m) => s + m.cost_usd, 0);
  const opusFraction = summary.total_cost_usd > 0 ? opusCost / summary.total_cost_usd : 0;
  if (opusFraction > 0.25) {
    const savings = opusCost * 0.5 * (1 - 3 / 5); // 50% des appels Opus → Sonnet
    totalAvoidable += savings;
    cards.push({ sev: "medium", icon: "🟡",
      title: t('insights.model.title'),
      desc: t('insights.model.desc').replace("{pct}", (opusFraction * 100).toFixed(0)),
      action: t('insights.model.action'),
      savings });
  }

  // ── R4 : Contexte proche limite ─────────────────────────────────────────────
  if (waste.sessions_near_limit > 0) {
    cards.push({ sev: "info", icon: "ℹ️",
      title: t('insights.ctx.title'),
      desc: t('insights.ctx.desc').replace("{n}", waste.sessions_near_limit),
      action: t('insights.ctx.action'),
      savings: null });
  }

  // ── R5 : Plan abonnement ────────────────────────────────────────────────────
  const current = currentYearMonth();
  const complete = months.filter((m) => m.month !== current);
  if (complete.length >= 2) {
    const avgMonthly = complete.reduce((s, m) => s + m.cost_usd, 0) / complete.length;
    if (avgMonthly > 20) {
      const save = avgMonthly - 20;
      totalAvoidable += save;
      cards.push({ sev: "info", icon: "💡",
        title: t('insights.plan.title'),
        desc: t('insights.plan.desc').replace("{avg}", fmtUsd(avgMonthly)).replace("{save}", fmtUsd(save)),
        action: t('insights.plan.action'),
        savings: save });
    }
  }

  // ── Rendu ────────────────────────────────────────────────────────────────────
  const summaryEl = $("#insights-summary");
  if (totalAvoidable > 0.5) {
    summaryEl.textContent = `${t('insights.avoidable')} : ${fmtUsd(totalAvoidable)}/mois`;
    summaryEl.style.color = "var(--warn)";
  } else {
    summaryEl.textContent = "";
  }

  if (cards.length === 0) {
    list.innerHTML = `<p style="color:var(--muted);padding:12px 0">${t('insights.none')}</p>`;
    return;
  }

  list.innerHTML = cards.map((c) => `
    <div class="insight-card ${c.sev}">
      <div class="insight-icon">${c.icon}</div>
      <div class="insight-body">
        <div class="insight-title">${escapeHtml(c.title)}</div>
        <div class="insight-desc">${escapeHtml(c.desc)}</div>
        ${c.action ? `<div class="insight-action">→ ${escapeHtml(c.action)}</div>` : ""}
      </div>
      ${c.savings != null && c.savings > 0.1 ? `
        <div class="insight-savings">
          <span class="label">${t('insights.avoidable')}</span>
          <span class="amount">${fmtUsd(c.savings)}/mois</span>
        </div>` : ""}
    </div>`).join("");
}

// ── Sessions : contrôles filtre/limite ───────────────────────────────────────

async function refreshSessions() {
  const limit = parseInt($("#sessions-limit").value, 10) || 50;
  const project = $("#sessions-project").value || undefined;
  const rows = await jget("/api/by-session" + qs({ limit, project }));
  renderSessions(rows);
}

async function loadSessionProjects() {
  const projects = await jget("/api/by-project");
  const sel = $("#sessions-project");
  sel.innerHTML = '<option value="">Tous</option>';
  for (const p of projects) {
    const opt = document.createElement("option");
    opt.value = p.project_path;
    opt.textContent = shortenPath(p.project_path);
    sel.appendChild(opt);
  }
}

$("#sessions-limit").addEventListener("change", () => refreshSessions().catch(() => {}));
$("#sessions-project").addEventListener("change", () => refreshSessions().catch(() => {}));

loadSessionProjects().catch(() => {});

// ── Auto-refresh ──────────────────────────────────────────────────────────────

setInterval(async () => {
  try {
    const d = await jget("/api/last-timestamp");
    const ts = d.last_timestamp;
    if (ts && lastAutoTs !== null && ts !== lastAutoTs) {
      await refresh();
    }
    if (ts) lastAutoTs = ts;
  } catch (_) {}
}, 30_000);

// ── Navigation par onglets ────────────────────────────────────────────────────

// ── Sélecteurs langue / thème ─────────────────────────────────────────────────

document.getElementById("lang-select").value    = currentLang;
document.getElementById("theme-select").value   = currentTheme;
document.getElementById("estimate-toggle").checked = currentEstimates;

document.getElementById("lang-select").addEventListener("change", (e) => {
  applyLang(e.target.value);
  refreshInsights().catch(() => {});
  refresh().catch(() => {});
});

document.getElementById("theme-select").addEventListener("change", (e) => {
  applyTheme(e.target.value);
  refresh().catch(() => {});
});

document.getElementById("estimate-toggle").addEventListener("change", (e) => {
  currentEstimates = e.target.checked;
  localStorage.setItem('cc-estimates', currentEstimates);
  refresh().catch(() => {});
});

document.querySelectorAll(".tab-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".tab-btn").forEach((b) => b.classList.remove("active"));
    document.querySelectorAll(".tab-section").forEach((s) => s.classList.remove("active"));
    btn.classList.add("active");
    document.getElementById("tab-" + btn.dataset.tab).classList.add("active");
    // Chart.js se redimensionne incorrectement quand le canvas est caché à la création
    [dayChart, modelChart, planChart].forEach((c) => c && c.resize());
  });
});

$("#alert-banner-goto").addEventListener("click", () => {
  document.querySelector(".tab-btn[data-tab='alertes']").click();
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
    ? `<span class="badge">${t('badge.custom')}</span>`
    : (!r.is_known ? `<span class="badge badge-unknown">${t('badge.unknown')}</span>` : "");
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

// ── Comparaison de plans ─────────────────────────────────────────────────────

const PLANS = [
  { name: "Pro",      price: 20,  color: "#5ee08a" },
  { name: "Max 5×",  price: 100, color: "#ffb86b" },
  { name: "Max 20×", price: 200, color: "#ff5572" },
];

let planChart = null;

async function refreshPlans() {
  const months = parseInt($("#plans-months").value, 10) || 12;
  const rows = await jget(`/api/by-month?months=${months}`);
  renderPlansChart(rows, months);
  renderPlansTable(rows, months);
}

$("#plans-months").addEventListener("change", () => refreshPlans().catch(() => {}));

function currentYearMonth() {
  const n = new Date();
  return `${n.getUTCFullYear()}-${String(n.getUTCMonth() + 1).padStart(2, "0")}`;
}

function renderPlansChart(rows, months = 12) {
  const c = chartColors();
  const ctx = $("#chart-plans").getContext("2d");
  const current = currentYearMonth();
  // Tous les mois de la période + mois courant, avec $0 pour les mois sans données
  const allM = [...allCompletedMonths(months), current];
  const costByMonth = Object.fromEntries(rows.map((r) => [r.month, r.cost_usd]));
  const labels = allM;
  const costs  = allM.map((m) => costByMonth[m] || 0);
  // Flat datasets for plan thresholds
  const planDatasets = PLANS.map((p) => ({
    type: "line",
    label: `${p.name} ($${p.price})`,
    data: allM.map(() => p.price),
    borderColor: p.color,
    borderWidth: 1.5,
    borderDash: [5, 4],
    pointRadius: 0,
    fill: false,
  }));
  if (planChart) planChart.destroy();
  planChart = new Chart(ctx, {
    type: "bar",
    data: {
      labels,
      datasets: [
        {
          label: t('chart.cost_usd'),
          data: costs,
          backgroundColor: allM.map((m) =>
            m === current ? "rgba(201,139,255,0.45)" : "rgba(201,139,255,0.75)"
          ),
          borderColor: "#c98bff",
          borderWidth: 1,
        },
        ...planDatasets,
      ],
    },
    options: {
      responsive: true,
      plugins: {
        legend: { labels: { color: c.text, font: { size: 11 } } },
        tooltip: { callbacks: { label: (ctx) => ` ${ctx.dataset.label}: $${(ctx.parsed.y).toFixed(2)}` } },
      },
      scales: {
        x: { ticks: { color: c.muted, font: { size: 11 } }, grid: { color: c.border } },
        y: { ticks: { color: c.muted, callback: (v) => "$" + v }, grid: { color: c.border } },
      },
    },
  });
}

function allCompletedMonths(months) {
  // Génère la liste de tous les YYYY-MM des (months-1) derniers mois complets
  const current = currentYearMonth();
  const result = [];
  const now = new Date();
  for (let i = months - 1; i >= 1; i--) {
    const d = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth() - i, 1));
    const ym = `${d.getUTCFullYear()}-${String(d.getUTCMonth() + 1).padStart(2, "0")}`;
    result.push(ym);
  }
  return result;
}

function renderPlansTable(rows, months = 12) {
  const current = currentYearMonth();
  // Tous les mois complets de la période, même ceux sans données (= $0 d'API)
  const allMonths = allCompletedMonths(months);
  const n = allMonths.length;
  const costByMonth = Object.fromEntries(rows.map((r) => [r.month, r.cost_usd]));
  const totalApi = allMonths.reduce((s, m) => s + (costByMonth[m] || 0), 0);

  const tbody = $("#tbl-plans-body");
  tbody.innerHTML = "";

  for (const plan of PLANS) {
    const totalPlan = plan.price * n;
    const delta = totalApi - totalPlan;           // positif = sub moins chère
    const profitable = allMonths.filter((m) => (costByMonth[m] || 0) > plan.price).length;
    const regularity = n > 0 ? profitable / n : 0;
    const regClass = regularity >= 0.75 ? "" : regularity >= 0.4 ? "mid" : "low";

    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td style="color:${plan.color};font-weight:600">${plan.name}</td>
      <td class="right">$${plan.price}</td>
      <td class="right">$${totalPlan.toFixed(2)} <span style="color:var(--muted);font-size:11px">(${n}m)</span></td>
      <td class="right">$${totalApi.toFixed(2)}</td>
      <td class="right ${delta >= 0 ? "delta-pos" : "delta-neg"}">
        ${delta >= 0 ? "+" : ""}$${Math.abs(delta).toFixed(2)}
        <span style="color:var(--muted);font-size:10px">${delta >= 0 ? t('plans.saved') : t('plans.lost')}</span>
      </td>
      <td class="right">${profitable}/${n}</td>
      <td class="right">
        <span class="regularity-bar"><span class="regularity-fill ${regClass}" style="width:${(regularity*100).toFixed(0)}%"></span></span>
        ${(regularity * 100).toFixed(0)}%
      </td>
    `;
    tbody.appendChild(tr);
  }

  const currentRow = rows.find((r) => r.month === current);
  const noteEl = $("#plans-note");
  if (currentRow) {
    const projected = currentRow.cost_usd / (new Date().getUTCDate()) * 30;
    noteEl.textContent =
      `Mois en cours (${current}) : $${currentRow.cost_usd.toFixed(2)} dépensés → projection $${projected.toFixed(2)}/mois. ` +
      `Les mois rentables = mois où votre dépense API dépasse le prix de l'abonnement. ` +
      `Δ économie = différence sur ${n} mois complets.`;
  } else {
    noteEl.textContent = `Analyse sur ${n} mois complets. Δ économie = différence totale vs abonnement.`;
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
    `${unknowns.length} ${t('banner.models_a')} ${names}.`;
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
  const tabBadge = $("#alert-tab-badge");
  if (triggered.length === 0) {
    banner.classList.add("hidden");
    banner.classList.remove("critical");
    tabBadge.classList.add("hidden");
    return;
  }
  tabBadge.textContent = triggered.length;
  tabBadge.classList.remove("hidden");
  banner.classList.remove("hidden");
  const msgs = triggered.map((a) => {
    const period = a.period === "week" ? t('week') : t('month_period');
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
    const periodLabel = a.period === "week" ? t('plans.week_label') : t('plans.month_label');
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
