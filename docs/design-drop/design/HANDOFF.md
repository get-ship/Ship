# Ship — Design Handoff

**One source of truth.** Two buckets only: **RATIFIED** (build this, it's the agreed design)
and **MOCK / REFERENCE** (illustrates composition or explores — do not build from it directly).
If something isn't in the Ratified table, it is not ratified.

Status legend: ✅ ratified · 🟡 authored, needs a verify/catalog pass before ratifying · ⬜ not started.

---

## A. RATIFIED — the build list

Canonical `.sui` under `design/components/`. One version each (no before/after, no variants).
Each pack has a catalog `*.html` that compiles + renders every component vs fixtures.

### @ship/chrome — `components/ship/chrome/` · catalog `chrome.html` · ✅
The shell chrome. **Build first** — every surface composes it.
| tag | props |
|---|---|
| `ship-command-bar` ★ | `open` · `mode(find\|search\|create\|ai)` |
| `ship-activity-bar` | `active(agents\|views\|components\|pipelines\|themes\|settings)` |
| `ship-file-tree` / `ship-tree-item` | tree `label·scope`; item `name·kind·filetype·open·selected·depth·status` |
| `ship-status-bar` | `daemon(ready\|busy\|offline)·sessions·approvals·sync(synced\|syncing\|offline)·space·context·theme` — Ship-native ambient state, **not** git/problems/Ln-Col |
| `ship-dock` / `ship-dock-app` | app `label·active·hue` |

### @ship/agents — `components/ship/agents/` · catalog `agents-pack.html` · spec `handoff.md` · ✅
`ship-session-list · ship-session-metrics · ship-op-card · ship-agent-controls · ship-usage-chip ·
ship-agent-card · ship-agent-chat · ship-agent-canvas · ship-agent-slot` (9)

### @ship/authority — `components/ship/authority/` · catalog `authority.html` · ✅
`ship-approval-prompt · ship-approvals-queue · ship-capability-bar · ship-policy-matrix ·
ship-principal-list · ship-gate-log · ship-blast-tag · ship-gate-badge` (8)

### sui — `components/sui/` · ✅ (ratified subset)
`sui-brand-mark` · `chat-elements/message`. Prebuilt primitives consumed from
`sui/sui-components.js`: `sui-button · sui-icon · sui-status-dot` (+ card/select/table).

---

## B. RATIFIED — the surfaces (reference compositions)

`design/scaffolds/`. These ARE ratified as the **intended product designs** (the UX/layout the
team builds toward) — but they are **compositions, not components**: production builds them by
assembling the packs above, not by copying mock markup. 1340×850 frames; production is responsive.

| Surface | File |
|---|---|
| Shell desktop | `shell/desktop.html` |
| Marketplace (Ship Shop) | `shell/marketplace.html` |
| Library | `shell/library.html` |
| Shell settings | `shell/shell-settings.html` |
| Studio landing | `shell/studio.html` |
| Studio IDE | `shell/studio-ide.html` |
| Agent editor | `shell/agent-editor.html` |
| Space scaffold | `spaces/space-scaffold.html` |
| Space settings | `spaces/space-settings.html` |
| Agent Canvas (chat-canvas) | `spaces/interactive-chat.html` |
| Config-seam map | `scaffolds/config-seam.html` |

---

## C. MOCK / REFERENCE — do NOT build from these

- `design/components/sui/**` **before/after pairs**, `new/existing` chart variants, `media/`,
  `canvas/`, `proposed/` atoms — explorations. The `.after.sui` / `.new.sui` is the lean, but
  **none are ratified** until collapsed to one verified version in a catalog.
- `design/components/_repo-src/**` — read-only mirror of the real `sui` standard library.
  Reference for primitive props/behavior; not our design.
- `reference/**` — repo source pulled in for grounding (chrome mirror, agent views, configs).
- `design/_archive/**` — superseded explorations (old agent-canvas concepts, roster/wire,
  the JSX spaces legacy). Kept for history; never build from.
- `shell/nav-frame.html`, `shell/mock.html`, `shell/studio/shell.html` — prior-art IA references.

**Rule going forward:** no competing versions in the ratified tree. An exploration lives in
`proposed/` or `_archive/` until it wins; when it wins it becomes the single `.sui` (no suffix)
with a catalog entry, and the alternatives are archived.

---

## D. Completeness — what's left to ratify

**Net-new** ⬜ — `ship-library-picker` (Studio↔Library interop) · Agent-editor **Wire** lens.

**Authored, needs verify/catalog pass** 🟡 → ✅ —
`@ship/code` (code-editor · terminal · git-diff) · `@ship/pipelines` (pipeline-graph · run-history) ·
`@ship/spaces` (registry · switcher · add-dialog · active-badge) · `@ship/markdown` · `@ship/pdf` ·
`@ship/viewer` · `@ship/arcade`; and the `sui` charts / media / canvas / remaining chat-elements.

**Primitives floor** 🟡 — confirm the `_repo-src` primitives the packs depend on
(field · input · select · dialog · drawer · tree · avatar · chip · tabs · empty-state ·
spinner · skeleton) compile under the current SUI compiler; re-author + catalog the ones used.

**Consolidation** — retrofit `desktop` · `studio` · `studio-ide` inline chrome onto `@ship/chrome`.

**Composition fidelity (the pattern, proven ✅ — retrofit complete)** — editor surfaces compose the **prebuilt** `sui-*`
primitives, not hand-rolled HTML. Retrofitted: `studio-collections` (real `sui-table` +
`sui-checkbox` gutter + `sui-badge` cells), `studio-artifacts` · `studio-skills` (`sui-input` ·
`sui-button` · `sui-badge`), `studio-themes` (`sui-button`/`sui-badge` in the live preview),
`studio-pipelines` (`ship-blast-tag` on the gate node), `agent-editor` (`sui-button` Save +
`sui-badge` tags). `space-scaffold` was already composed (`ship-command-bar` ai-mode · `ship-op-card`
· `sui-message`). Two real constraints found in the bundle:
  1. **`view()` components need the daemon** (e.g. `sui-tab`, rich `sui-select`) — won't render
     standalone. Compose the `component()` primitives (table · input · button · badge · field ·
     checkbox) which need no daemon; reserve `view()` ones for the live runtime. (Native `<select>`
     stays native in the editors for this reason.)
  2. **`sui-table` variants:** `data` (set `.rows`/`.columns` in JS, built-in sort, plain-text cells)
     · `native` (slot a *complete* `<table>` — raw `<tr>` children get foster-parented out by the
     parser; this variant + light-DOM CSS keeps badges-in-cells) · `slot` (shell only). Use `native`
     when cells need composed children. Add fidelity (frosted sticky header, zebra, sort caret,
     tabular-nums, gutter) via page CSS over the native shell.
**Bespoke-by-design (correctly NOT primitives):** the capability tier-picker (agent-editor), the DAG
canvas + jsonc view (pipelines), the kanban board (space-scaffold), file trees' domain rows. These
are domain UI with no primitive equivalent — keep hand-rolled.

**Per-component ratification ✅ — specs written:** formal API contract per pack —
`components/ship/chrome/SPEC.md` (7) · `agents/SPEC.md` (9) · `authority/SPEC.md` (6+2 atoms).
Each: full prop table (type · default · enum · reflect) · slots · `::part`s · intended events ·
bind contract · the state matrix to ratify. Remaining before code: render the **state grids** in
each catalog (every variant side-by-side), then **a11y** (focus · keyboard · ARIA) and **all-6-theme**
sweeps. Events are intended (presentation `.sui` is controller-free); the `.controller.ts` emits them.

---

## E. How surfaces render real components (the chrome stack)

Each head loads, in order: `ship.css` → a fetch-patch for the icon registry
(`/_sui/lucide-icons.json` → local) → import-map (`@ship/sui-runtime`, `@ship/sui-dom`) →
`sui/sui-components.js` (prebuilt primitives) → per-component
`defineComponent({name, sui})` from `tools/catalog/sui-workbench.js`.

- `sui-icon` uses a **curated 36-icon registry** (`sui/lucide-icons.json`) — names outside it
  render blank; map to the available set.
- Themes: `themeBar(mount)` from `sui-workbench.js` wires the 6 ship themes
  (`ship-light · ship-dark · paper · halo · console · contrast`) via `<html data-theme>` + `.dark`.
- Fonts from `ship.css`: Syne (display) · DM Sans (body) · JetBrains Mono. **Tokens only — never hardcode.**

## F. Model & conventions (locked)

Shell (global: account · billing · space roster · marketplace · library · Studio host) ⊃
Space (app+storage; own chrome; holds views) ⊃ View (applet; multi-file `.sui`+ctrl+css+manifest).
The **config seam**: thin settings in the space; catalog + authoring in **Studio**; only **Author**
and **Install** cross it; the Library never enters a space. Studio is scoped to one space; the
activity-bar items **are the contexts** (no "Explorer"); the file-tree shows only for file-backed
contexts. **Permissions is not a domain** (per-domain access + the agent capability matrix).
`ship-*` = runtime-bound; `sui-*` = framework-general atoms. Blast tiers **C0–C3** = Read · Write ·
Elevated · Critical (word in config, terse C# on scan surfaces). **No JSX — author SUI.**
