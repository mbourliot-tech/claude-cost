// ── TOKEN SNAKE — Easter egg ──────────────────────────────────────────────────
// Déclenché par : code Konami (↑↑↓↓←→←→BA) ou 5 clics rapides sur le titre
(function () {

// ── Triggers ──────────────────────────────────────────────────────────────────
const KONAMI = ['ArrowUp','ArrowUp','ArrowDown','ArrowDown',
                'ArrowLeft','ArrowRight','ArrowLeft','ArrowRight','b','a'];
let ki = 0;
document.addEventListener('keydown', (e) => {
  if (gameOpen) return;
  ki = e.key === KONAMI[ki] ? ki + 1 : (e.key === KONAMI[0] ? 1 : 0);
  if (ki === KONAMI.length) { ki = 0; openGame(); }
});

let titleClicks = 0, titleTimer = null;
function setupTitleTrigger() {
  const h1 = document.querySelector('header h1');
  if (!h1) return;
  h1.style.cursor = 'default';
  h1.addEventListener('click', () => {
    titleClicks++;
    clearTimeout(titleTimer);
    if (titleClicks >= 5) { titleClicks = 0; openGame(); return; }
    titleTimer = setTimeout(() => { titleClicks = 0; }, 2000);
  });
}

// ── Web Audio ─────────────────────────────────────────────────────────────────
let actx = null;
function audio() {
  if (!actx) actx = new (window.AudioContext || window.webkitAudioContext)();
  return actx;
}
function tone(freq, dur, type = 'square', vol = 0.22, delay = 0) {
  try {
    const a = audio(), t = a.currentTime + delay;
    const osc = a.createOscillator(), g = a.createGain();
    osc.connect(g); g.connect(a.destination);
    osc.type = type; osc.frequency.setValueAtTime(freq, t);
    g.gain.setValueAtTime(vol, t);
    g.gain.exponentialRampToValueAtTime(0.001, t + dur);
    osc.start(t); osc.stop(t + dur);
  } catch (_) {}
}
const sfx = {
  start:  () => [262,330,392,523].forEach((f,i) => tone(f, 0.08, 'square', 0.2, i*0.07)),
  eat:    () => { tone(523, 0.06); tone(784, 0.09, 'square', 0.22, 0.06); },
  level:  () => [392,494,587,784].forEach((f,i) => tone(f, 0.07, 'square', 0.25, i*0.055)),
  pause:  () => tone(440, 0.06, 'sine', 0.15),
  die:    () => [330,247,196,147].forEach((f,i) => tone(f, 0.14, 'sawtooth', 0.28, i*0.1)),
};

// ── Constants ─────────────────────────────────────────────────────────────────
const GRID = 20, CELL = 20, SZ = GRID * CELL;
const SPEED_START = 145, SPEED_MIN = 55;
const FOODS = [
  { sym: '$',  col: '#ffd700' },
  { sym: '◆',  col: '#c98bff' },
  { sym: '💰', col: null },
];
const HI_KEY = 'cc-snake-hi';

// ── State ────────────────────────────────────────────────────────────────────
let canvas, cx, loop, gameOpen = false;
let snake, dir, nextDir, food, score, hi, level, paused, dead, started;

function rnd(n) { return Math.floor(Math.random() * n); }

function spawnFood() {
  const occ = new Set(snake.map(s => `${s.x},${s.y}`));
  let p;
  do { p = { x: rnd(GRID), y: rnd(GRID) }; } while (occ.has(`${p.x},${p.y}`));
  food = { ...p, ...FOODS[rnd(FOODS.length)] };
}

function init() {
  hi = parseInt(localStorage.getItem(HI_KEY) || '0');
  const m = GRID >> 1;
  snake = [{ x:m, y:m }, { x:m-1, y:m }, { x:m-2, y:m }];
  dir = nextDir = { x:1, y:0 };
  score = 0; level = 1; paused = false; dead = false; started = false;
  spawnFood();
  hud(); draw();
  msg('TOKEN SNAKE', '↑↓←→ pour démarrer', true);
}

function tick() {
  if (paused || dead || !started) return;
  dir = nextDir;
  const h = { x: snake[0].x + dir.x, y: snake[0].y + dir.y };
  if (h.x < 0 || h.x >= GRID || h.y < 0 || h.y >= GRID ||
      snake.some(s => s.x === h.x && s.y === h.y)) { die(); return; }
  snake.unshift(h);
  if (h.x === food.x && h.y === food.y) {
    const newScore = score + 10 * level;
    const newLevel = Math.min(Math.floor(newScore / 50) + 1, 10);
    if (newLevel > level) sfx.level(); else sfx.eat();
    level = newLevel; score = newScore;
    if (score > hi) { hi = score; localStorage.setItem(HI_KEY, hi); }
    spawnFood(); restartLoop();
  } else {
    snake.pop();
  }
  hud(); draw();
}

function die() {
  dead = true; sfx.die();
  if (loop) clearInterval(loop);
  draw();
  setTimeout(() => msg('GAME OVER', `Score : ${score} — Espace pour rejouer`), 600);
}

function restartLoop() {
  if (loop) clearInterval(loop);
  loop = setInterval(tick, Math.max(SPEED_MIN, SPEED_START - (level - 1) * 9));
}

function hud() {
  el('gs-score').textContent = score;
  el('gs-hi').textContent    = hi;
  el('gs-level').textContent = level;
}

// ── Rendering ────────────────────────────────────────────────────────────────
function css(v) { return getComputedStyle(document.documentElement).getPropertyValue(v).trim(); }

function hexRgb(h) {
  h = h.replace('#','');
  if (h.length === 3) h = h.split('').map(c=>c+c).join('');
  return [parseInt(h.slice(0,2),16), parseInt(h.slice(2,4),16), parseInt(h.slice(4,6),16)];
}
function lerpHex(a, b, t) {
  try {
    const [r1,g1,b1] = hexRgb(a), [r2,g2,b2] = hexRgb(b);
    return '#'+[r1+t*(r2-r1),g1+t*(g2-g1),b1+t*(b2-b1)]
      .map(v=>Math.round(v).toString(16).padStart(2,'0')).join('');
  } catch(_) { return a; }
}
function rr(x,y,w,h,r) {
  cx.beginPath();
  if (cx.roundRect) cx.roundRect(x,y,w,h,r); else cx.rect(x,y,w,h);
  cx.fill();
}

function draw() {
  const accent  = css('--accent')   || '#c98bff';
  const accent2 = css('--accent-2') || '#6bd1ff';
  const bg      = css('--bg')       || '#0f1115';
  const border  = css('--border')   || '#2a2f3a';
  const surface = css('--surface')  || '#171a21';

  // Background
  cx.fillStyle = bg;
  cx.fillRect(0, 0, SZ, SZ);

  // Grid dots
  cx.fillStyle = border + '55';
  for (let x = 0; x < GRID; x++)
    for (let y = 0; y < GRID; y++)
      cx.fillRect(x * CELL + CELL/2 - 0.5, y * CELL + CELL/2 - 0.5, 1, 1);

  // Snake body (gradient accent → accent2 with fade)
  for (let i = snake.length - 1; i >= 1; i--) {
    const s = snake[i], t = i / snake.length;
    cx.fillStyle = lerpHex(accent, accent2, t) + Math.round((1 - t * 0.4) * 255).toString(16).padStart(2,'0');
    rr(s.x * CELL + 2, s.y * CELL + 2, CELL - 4, CELL - 4, 4);
  }

  // Head
  const h = snake[0];
  cx.fillStyle = accent;
  rr(h.x * CELL + 1, h.y * CELL + 1, CELL - 2, CELL - 2, 5);

  // Inner head highlight
  const grd = cx.createRadialGradient(
    h.x*CELL+CELL/2-dir.x*2, h.y*CELL+CELL/2-dir.y*2, 1,
    h.x*CELL+CELL/2, h.y*CELL+CELL/2, CELL/2
  );
  grd.addColorStop(0, 'rgba(255,255,255,0.3)');
  grd.addColorStop(1, 'rgba(255,255,255,0)');
  cx.fillStyle = grd;
  rr(h.x*CELL+1, h.y*CELL+1, CELL-2, CELL-2, 5);

  // Eyes
  cx.fillStyle = '#000d';
  if (dir.x !== 0) {
    const ex = h.x*CELL + (dir.x > 0 ? CELL-6 : 3);
    cx.fillRect(ex, h.y*CELL+4, 3, 3);
    cx.fillRect(ex, h.y*CELL+CELL-7, 3, 3);
  } else {
    const ey = h.y*CELL + (dir.y > 0 ? CELL-6 : 3);
    cx.fillRect(h.x*CELL+4, ey, 3, 3);
    cx.fillRect(h.x*CELL+CELL-7, ey, 3, 3);
  }

  // Food
  if (food.col) {
    cx.fillStyle = food.col;
    cx.font = `bold ${CELL-3}px monospace`;
    cx.textAlign = 'center';
    cx.textBaseline = 'middle';
    cx.fillText(food.sym, food.x*CELL+CELL/2, food.y*CELL+CELL/2+1);
  } else {
    cx.font = `${CELL-1}px sans-serif`;
    cx.textAlign = 'center';
    cx.textBaseline = 'middle';
    cx.fillText(food.sym, food.x*CELL+CELL/2, food.y*CELL+CELL/2+1);
  }

  // Food glow
  const glw = cx.createRadialGradient(
    food.x*CELL+CELL/2, food.y*CELL+CELL/2, 2,
    food.x*CELL+CELL/2, food.y*CELL+CELL/2, CELL
  );
  glw.addColorStop(0, (food.col||'#ffd700')+'44');
  glw.addColorStop(1, 'transparent');
  cx.fillStyle = glw;
  cx.fillRect(food.x*CELL-CELL/2, food.y*CELL-CELL/2, CELL*2, CELL*2);

  // Dead overlay
  if (dead) {
    cx.fillStyle = 'rgba(0,0,0,0.45)';
    cx.fillRect(0, 0, SZ, SZ);
  }
}

// ── Message overlay ───────────────────────────────────────────────────────────
function msg(title, sub, showKeys = false) {
  const d = el('game-msg');
  if (!title) { d.style.display = 'none'; return; }
  d.style.display = 'flex';
  d.innerHTML = `
    <div class="gmsg-title">${esc(title)}</div>
    ${sub ? `<div class="gmsg-sub">${esc(sub)}</div>` : ''}
    ${showKeys ? `<div class="gmsg-keys">↑ ↓ ← →</div>` : ''}
  `;
}
function esc(s) { return String(s).replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])); }

// ── Keyboard ──────────────────────────────────────────────────────────────────
const DIRS = { ArrowUp:{x:0,y:-1}, ArrowDown:{x:0,y:1}, ArrowLeft:{x:-1,y:0}, ArrowRight:{x:1,y:0} };
function onKey(e) {
  const d = DIRS[e.key];
  if (d) {
    e.preventDefault();
    if (!started) { started = true; msg('',''); sfx.start(); restartLoop(); return; }
    if (d.x !== -dir.x || d.y !== -dir.y) nextDir = d;
  }
  if (e.key === ' ') {
    e.preventDefault();
    if (dead) { init(); return; }
    paused = !paused; sfx.pause();
    paused ? msg('PAUSE', 'Espace pour continuer') : msg('','');
  }
  if (e.key === 'Escape') closeGame();
}

// ── Open / close ──────────────────────────────────────────────────────────────
function openGame() {
  gameOpen = true;
  el('game-overlay').classList.remove('hidden');
  canvas = el('game-canvas');
  cx = canvas.getContext('2d');
  document.addEventListener('keydown', onKey);
  el('game-close').onclick = closeGame;
  el('game-overlay').onclick = e => { if (e.target === el('game-overlay')) closeGame(); };
  if (loop) clearInterval(loop);
  init();
}

function closeGame() {
  gameOpen = false;
  el('game-overlay').classList.add('hidden');
  document.removeEventListener('keydown', onKey);
  if (loop) clearInterval(loop);
}

function el(id) { return document.getElementById(id); }

document.addEventListener('DOMContentLoaded', setupTitleTrigger);

})();
