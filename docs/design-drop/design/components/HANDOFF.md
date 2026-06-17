# HANDOFF — sui/ship design workspace (READ THIS FIRST)

> **Status (2026-06-03):** This workspace was built BEFORE the real repo source was
> pulled. Much of the `before/after` apparatus is unreliable. This doc is the single
> source of truth for the next session. Trust this + `_repo-src/` + `repo-reconciliation.md`.
> Distrust the old per-pack `handoff.md` / `INDEX.md` (deleted — they claimed false completeness).

---

## 1. Where the real source lives

| | Location |
|---|---|
| **Upstream** | `madvib/Ship`, branch **`job/sui-framework`**, path `packages/components/{sui,ship}` (commit `00e52626`) |
| **In this project** | **`design/components/_repo-src/`** — imported real `sui/primitives` source (`.sui` + `.controller.ts` + `.compiled.js` + tests). This is GROUND TRUTH. |
| **Runtime/tokens** | `/sui/ship.css` (tokens, per-theme), `/sui/sui-runtime.js`, `/sui/sui-components.js` (prebuilt), `/sui/sui_compiler.js` (+ wasm) — seeded into the project, look genuine. **Caveat:** the bundled `ship.css` may be STALE — real components consume `--focus-ring` / `--focus-ring-offset`, which the bundled `ship.css` does not define. Re-pull `ship.css` from the branch before trusting token work. |

**Repo layout per component:** one canonical `src/sui-<name>/sui-<name>.sui` (+ `.controller.ts`
where behavioral, + `.compiled.js`, + `test/`). There is **no `before/after`** in the repo —
that was a review convention invented here. There is **no `core/forms/nav/...` subfoldering** —
`primitives/src/` is flat; our subgroups are a design-review convenience, not repo structure.

## 2. How we render sui (verified working)

Two paths, both via `tools/catalog/sui-workbench.js`:

- **Prebuilt primitives** — `import '/sui/sui-components.js'` registers real `<sui-*>` tags
  (button/badge/card/select/table confirmed). No daemon needed for `component()` primitives.
- **Compile-at-runtime** — for a `.sui` not in the prebuilt bundle:
  ```js
  import init, { compile } from '/sui/sui_compiler.js'; await init();
  const out = compile(src, JSON.stringify({ mode:'component', componentName:'sui-x',
    shadow:true, setupUrl:null, styles:null }));
  await import(URL.createObjectURL(new Blob([out.js], {type:'text/javascript'})));
  ```
  `mode:'component'` for primitives (default/`view` emits a daemon-bound `view()`). The
  workbench's `defineComponent({name, sui})` wraps this. Compare pages live 4 dirs deep →
  `../../../../sui/ship.css` etc. (charts/canvas/media are 4 ups; primitives/<group> is 5).
- **Never** add JSX/React. `.sui` (+ `.controller.ts`) only. Tokens only — `var(--token)`, no hex.

## 3. The "before" problem — what needs replacing

**EVERY `*.before.sui` in this workspace is a hand-written GUESS** authored before source was
pulled. None were copied from the repo. The real components are richer than these guesses
(e.g. real `sui-button` = a `variant`×`tone` matrix — `solid|soft|outline|ghost|default` ×
8 tones — via `--tone`/`--tone-fg`/`--tone-contrast`, with `xs|sm|md|lg`, `loading`,
`reflect-empty` icon-only, `{#if}` blocks; consumes `--focus-ring`).

**Next-session task:** for every REAL component (bucket A below), replace the `.before.sui`
with the actual `_repo-src/sui-<name>/sui-<name>.sui`, then re-judge whether the `.after`
proposal still makes sense against the real API. Many "after" diffs may be moot or wrong
once the real baseline is in.

## 4. Authoritative inventory (from the real `index.ts` registries)

**primitives** (flat): accordion(+item) · alert · alert-dialog · autocomplete · avatar · badge ·
button · calendar · card · checkbox · chip · cluster · code · collapsible · color-picker ·
combobox · context-menu · date-picker · dialog · diff · drawer · dropdown-menu · empty-state ·
error-boundary · error-state · field · file-upload · hover-card · icon · input · json-view ·
label · link · markdown · navigation-menu(+item) · pagination · popover · portal ·
radio(+radio-group) · scroll-area · select · separator · sheet · skeleton · slider · spinner ·
stack · status-dot · switch · tab-panel · table · tabs(+tab) · textarea · timeline(+item) ·
toast(+container) · toast-host · toggle(+group) · tooltip · tree · tree-grid · tree-view(+item)
**charts:** area-chart · bar-chart · line-chart · sparkline · donut · gauge · heatmap
**canvas:** canvas · board · draw
**chat-elements:** message · message-list · composer · thinking-dots · tool-call · citation
**media:** media-image · media-csv · media-av (+ engine) — NOT one generic `media`
**ship packs (8):** agents · arcade · authority · code · markdown · pdf · pipelines · viewer
(canonical `Tone` = default|primary|info|success|warning|destructive|accent|muted · `Size` = sm|md|lg)

## 5. Build classification (full detail in `repo-reconciliation.md`)

- **A — real, my "before" is a guess to replace:** button, card, link, icon, collapsible,
  accordion, scroll-area, separator, markdown, alert-dialog, file-upload, radio-group,
  autocomplete, date-picker, color-picker, combobox, input, textarea, select, checkbox,
  switch, slider, toggle-group, field, label, toast, error-state, alert, empty-state,
  skeleton, spinner, navigation-menu, tabs, pagination, table, tree-view, json-view,
  timeline, badge, chip, avatar, status-dot, sheet, tree-grid, code, + charts (all),
  board, canvas, chat-elements (all).
- **B — wrong name/shape:** `divider`→`separator`; `segmented`→`toggle-group`; generic
  `media`→three renderers; `component-viewer`→`@ship/viewer`.
- **C — INVENTED (do not exist) → moved to `sui/proposed/`:** copy-button, mark, timestamp,
  kbd, list-tile, toolbar, metachip, theme-toggle, app-icon, loading, step-segments,
  divider, segmented, breadcrumb, stepper, stat, progress, annotation-marker, annotation-list.
- **Never designed (real, untouched):** cluster, stack, context-menu, hover-card, popover,
  portal, drawer, dialog, diff, dropdown-menu, error-boundary, tooltip, tree, tab-panel,
  calendar; media renderers; ship packs agents/authority/pipelines.

## 6. What was purged this session (and why)

Deleted because they asserted false completeness / referenced invented components / were built
on guessed befores:
- `sui/INDEX.md`, `ship/INDEX.md`, `sui/review.md`, all per-pack `handoff.md`, `overview.*`,
  `theme-tokens.proposal.md` (its `halo`/`--divider` finding is real but must be re-derived
  against the branch's `ship.css`, not the possibly-stale bundled copy).
- Compare pages that showcased invented components: core-leaf/text/rows/content/icons/progress,
  forms.compare, nav.compare, data-display.compare, feedback.compare, canvas.compare, media.compare.
- KEPT: compare pages showing only real components (befores still guessed — see §3): core,
  core-containers, core-dialog-upload, tables, trees, engines, forms-input, forms-pickers,
  feedback-more, nav-menu, charts, charts-more, board, chat-elements. Treat their "before" as unverified.

## 7. `sui/proposed/` — the invented components

Moved there, NOT deleted, because the user notes several were **recently scrubbed from
primitives** (they were mostly "kinds" of card/button/etc.) and some may represent real GAPS
worth proposing back. See `sui/proposed/README.md` for the per-component call (kind-of-X vs
genuine gap). They are gone from the live dirs because they are not in source.

## 8. Suggested next-session order

1. Re-pull `ship.css` from the branch; diff against bundled; confirm `--focus-ring` et al.
2. Pick 3–4 flagships (button, field, sheet, tree-grid), replace `.before` with `_repo-src`
   truth, re-baseline the `.after` against the real API, rebuild their compare pages.
3. Triage `proposed/` with the user — which are real gaps to pitch, which to drop.
4. Then breadth: remaining bucket-A befores, untouched reals, the 3 missing ship packs.
