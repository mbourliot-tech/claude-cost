# claude·cost

> Dashboard local qui suit la consommation et les coûts de Claude Code en parsant les transcripts JSONL.

[🇬🇧 English version](README.md)

---

## Présentation

**claude-cost** est une application web Rust auto-hébergée qui lit les journaux de conversation JSONL écrits par Claude Code (`~/.claude/projects/**/*.jsonl`), calcule le coût de chaque appel API, stocke les résultats dans une base SQLite locale, et sert un dashboard sur `http://localhost:3737`.

Aucune donnée ne quitte votre machine. Aucune clé API requise.

---

## Fonctionnalités

| Onglet | Contenu |
|--------|---------|
| **Vue d'ensemble** | KPIs de coût avec tendance vs période précédente · Stats de cache · Graphe temporel · Donut par modèle · Top projets · Patterns par jour de semaine et par heure |
| **Sessions** | Toutes les sessions avec fenêtre de contexte max, cache hit %, outils web · Filtres projet et limite · Export CSV |
| **Plans** | Comparaison API à la demande vs Pro / Max 5× / Max 20× sur N mois · Score de régularité |
| **Optimisation** | Recommandations actionnables de réduction des coûts avec économie mensuelle estimée |
| **Alertes** | Seuils budgétaires configurables (hebdo / mensuel, global ou par projet) |

**Fonctionnalités supplémentaires**
- 🌙 / ☀️ / 🌌 Trois thèmes : Sombre, Clair, Minuit
- 🇫🇷 / 🇬🇧 Deux langues : Français, Anglais
- ⚡ Scan incrémental — les fichiers JSONL inchangés sont ignorés
- 🔔 Auto-actualisation toutes les 30 s (polling `/api/last-timestamp`)
- 📺 Vue temps réel sur `/live` (polling 5 s, tableau animé)
- 🖥️ CLI complète : sous-commandes `summary`, `by-model`, `today`, `live`
- 🕹️ Easter egg caché (essayez le code Konami sur le dashboard…)

---

## Prérequis

- **Rust 1.75+** (stable)
- Windows, macOS ou Linux

---

## Installation

### Depuis les sources

```bash
git clone <dépôt>
cd claude-cost
cargo install --path .
```

Le binaire est installé dans `~/.cargo/bin/claude-cost`.

### Mise à jour

```bash
cd claude-cost
cargo install --path .
```

---

## Utilisation

### Dashboard web (mode par défaut)

```bash
claude-cost
# ou
claude-cost serve
```

Ouvre automatiquement `http://localhost:3737` dans le navigateur.

**Options**

```
--port <PORT>               Port d'écoute [défaut : 3737]
--projects-dir <DIR>        Remplace ~/.claude/projects
--rescan-interval <SECS>    Intervalle de rescan en arrière-plan [défaut : 60]
--no-open                   Ne pas ouvrir le navigateur automatiquement
```

### Commandes CLI

```bash
claude-cost summary                    # Résumé global coloré
claude-cost summary --since 2026-05-01 # Filtré par date
claude-cost by-model                   # Répartition par modèle
claude-cost today                      # Détail par heure pour aujourd'hui
claude-cost live                       # Mode surveillance temps réel
claude-cost live --interval 3 --limit 20
```

---

## Description des onglets

### Vue d'ensemble
Quatre KPIs (coût total, appels API, sessions, tokens) avec une flèche de tendance comparée à la période équivalente précédente. En dessous : barre de statistiques de cache, graphe temporel (horaire pour aujourd'hui, journalier sinon), donut des modèles, tableau des top projets, et deux graphes de patterns (coût par jour de semaine et par heure de la journée).

### Sessions
Tableau complet des sessions avec : fenêtre de contexte maximale (code couleur par rapport aux limites 200 K / 256 K / 1 M), taux de cache hit, compteurs de recherches et fetch web. Filtres par projet et limite (20 / 50 / 100 / 500). Bouton d'export CSV direct.

### Plans
Graphe mensuel en barres avec lignes de seuil à 20 $ / 100 $ / 200 $ (plans Pro / Max 5× / Max 20×). Tableau avec économie ou surcoût total par plan, nombre de mois rentables et barre de régularité. Sélecteur de période : 3, 6 ou 12 mois.

### Optimisation
Analyse automatique de cinq leviers de réduction des coûts :
1. **Taux de cache** — suggère un `CLAUDE.md` si le taux est faible
2. **Sessions courtes gaspillées** — sessions qui ont écrit dans le cache sans jamais le relire
3. **Mix modèles** — signale une utilisation élevée d'Opus et estime les économies si passage sur Sonnet
4. **Contexte proche limite** — sessions approchant 200 K tokens, recommande `/compact`
5. **Abonnement** — recommande Pro si la dépense API mensuelle dépasse régulièrement 20 $

Chaque recommandation affiche une économie mensuelle estimée en $.

### Alertes
Créez des alertes budgétaires avec un nom, une période (semaine / mois), un périmètre optionnel (projet) et un seuil en $. Le dashboard affiche une bannière d'avertissement et un badge sur l'onglet dès qu'un seuil est dépassé.

---

## Prix des modèles

Les prix intégrés suivent la grille tarifaire officielle d'Anthropic (mai 2026). Les modèles non-Anthropic routés via Claude Code Router (MiMo, DeepSeek) sont également inclus.

Des prix personnalisés peuvent être définis par modèle via le modal **Prix des modèles** (bouton dans le header). Chaque modification déclenche un rescan immédiat qui recalcule les coûts historiques.

---

## Architecture

```
src/
  main.rs       — CLI (clap), démarrage du serveur, boucle de rescan auto
  lib.rs        — ré-exports des modules publics
  api.rs        — routeur Axum + tous les handlers HTTP
  scanner.rs    — parcours incrémental des JSONL (basé sur mtime)
  parser.rs     — désérialisation JSONL ligne par ligne
  pricing.rs    — ModelPrice, price_for(), effective_cost()
  storage.rs    — SQLite via rusqlite (mode WAL)
  types.rs      — RawLine, RawUsage, UsageRecord
  assets.rs     — fichiers statiques embarqués (rust-embed)
assets/
  index.html    — dashboard principal
  app.js        — logique du dashboard (JS vanilla)
  game.js       — Easter egg
  live.html / live.js — vue temps réel
  style.css     — thèmes sombre / clair / minuit
tests/
  integration.rs — 41 tests (storage, scanner, API HTTP)
```

---

## Développement

```bash
cargo build                # build debug
cargo test                 # lancer les 41 tests
cargo install --path .     # installer / mettre à jour le binaire
```

Après modification des assets, toucher `src/assets.rs` pour forcer rust-embed à les ré-embarquer :

```bash
touch src/assets.rs && cargo install --path .
```

---

## Licence

MIT
