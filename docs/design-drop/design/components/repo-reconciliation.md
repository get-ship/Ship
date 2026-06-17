# Repo reconciliation — `job/sui-framework` vs this design project

> Pulled real source from **`madvib/Ship@job/sui-framework`**,
> `packages/components/{sui,ship}` (commit `00e52626`). This records the drift
> between that ground truth and what's in this design project, so nothing here is
> mistaken for a faithful mirror.
>
> **✅ ACTIONED 2026-06-03:** real `primitives` source imported to `_repo-src/`;
> invented components (bucket C) moved to `sui/proposed/`; false-completeness docs
> + compare pages showing invented components purged. See `HANDOFF.md` for the
> resulting state. Buckets A (re-baseline befores) and the gap-fill remain TODO.

## TL;DR

The design project is **NOT a checkout of the repo.** The `sui/` runtime + `ship.css`
that were dropped in look genuine, but every `.before.sui` / `.after.sui` here is a
**hand reconstruction / proposal** I authored from the conventions — not copied from
source. Some reconstructions are close; several diverge materially (see below).

## Structural drift

| | Repo (`job/sui-framework`) | This project |
|---|---|---|
| **Layout** | one `.sui` per component in `src/sui-<name>/` (+ `.controller.ts` where behavioral), inside packages with `package.json`/`test/`/`vitest.config.ts` | flat `<name>.before.sui` + `<name>.after.sui` pairs + `*.compare.html` |
| **before/after** | does not exist — there's one canonical `.sui` | my review convention, invented here |
| **sui packages** | `canvas · charts · chat-elements · media · primitives · **viewer**` | I folded `viewer` into `media/`; repo keeps it **separate** |
| **primitives** | **64 entries** in `src/` — incl. `cluster · context-menu · hover-card · portal · error-boundary · drawer · diff · dialog · popover …` I never touched | I covered a large subset |
| **ship packages** | **8**: `agents · arcade · authority · code · markdown · pdf · pipelines · viewer` | I built **5** (code · markdown · pdf · viewer · arcade) — **missing `agents · authority · pipelines`** |

## Substantive drift (example: `sui-button`)

The real `sui-button.sui` (7.5 KB) is richer than my reconstruction:
- **variant × tone matrix** — `variant` = `solid|soft|outline|ghost|default` is **orthogonal**
  to `tone` = `default|primary|info|success|warning|destructive|accent|muted`, driven by a
  local **`--tone` / `--tone-fg` / `--tone-contrast`** channel convention.
- size scale **`xs|sm|md|lg`** using `min-height`; `loading` spinner state; `pressed`
  (`true|false|mixed`) + `expanded` aria; **`reflect-empty="data-icon-only"`** slot
  directive for icon-only collapse; `{#if}` template blocks.

My versions tended to assume a thinner API and hand-roll things the system already has.

## Token drift (important)

- Real components consume **`--focus-ring`** + **`--focus-ring-offset`** tokens
  (`outline: var(--focus-ring)`). The `ship.css` snapshot in THIS project **does not
  define `--focus-ring`** — so either the bundled `ship.css` is **stale** relative to the
  branch, or the ring tokens live in another stylesheet. Meanwhile **every focus state I
  authored hand-rolled a stacked `box-shadow` ring** instead of consuming the token. → drift
  on both sides.
- Real buttons use a per-tone **`--tone-contrast`** (solid-fill text color); I didn't.
- ✅ Confirmed REAL and correctly used by me: `--tone-<x>-bg/-fg/-border`, `--selected-bg`,
  `--hover-bg`, `--border-strong`, `--border-subtle`, `--divider`, `--shadow-*`.

## What this means for the theme-token proposal

`theme-tokens.proposal.md` still stands (the `halo`/`--divider` hairline finding is real and
token-level), but it should be **re-derived against the branch's `ship.css`**, not the bundled
snapshot — confirm whether the bundled copy is stale first (the missing `--focus-ring` says it
might be).

## Component-by-component reality check (the real damage)

Cross-referenced my builds against the real registries (`primitives/src/index.ts`,
`chat-elements`, `media`). Three buckets:

### A — Built, and the component IS real (fixable: re-baseline "before" to real `.sui` + correct API/tokens)
`button · card · link · icon · collapsible · accordion · scroll-area · separator ·
markdown · alert-dialog · file-upload · radio-group · autocomplete · date-picker ·
color-picker · combobox · input · textarea · select · checkbox · switch · slider ·
toggle-group · field · label · toast · error-state · alert · empty-state · skeleton ·
spinner · navigation-menu · tabs · pagination · table · tree-view · json-view ·
timeline · badge · chip · avatar · status-dot · sheet · tree-grid · code`
→ these exist; my "before" is still a guess and the API/tokens drift (focus-ring,
variant×tone matrix, etc.), but the component is real.

### B — Built under the WRONG NAME / WRONG SHAPE for a real component
- **`divider`** → repo calls it **`separator`** (I built BOTH as if distinct — `divider` is fictional)
- **`segmented`** → repo has **`toggle-group`** (no separate `segmented`)
- **one generic `media`** with image/audio/csv modes → repo splits into **three** renderers: **`sui-media-image` · `sui-media-csv` · `sui-media-av`**
- **`component-viewer`** → lives in the **`@ship/viewer`** router pack, not a media primitive

### C — INVENTED: built before/after pairs for components that DO NOT EXIST in any pack I checked
`copy-button · mark · timestamp · kbd · list-tile · toolbar · metachip ·
theme-toggle · app-icon · loading · step-segments · progress · breadcrumb · stepper · stat`
→ not in `primitives`, `chat-elements`, or `media`. Pure fabrication. (The chat set
`sui-message/composer/thinking-dots/tool-call/citation` IS real chat-elements;
`board` is `sui/canvas` — but the names above have no repo counterpart I can find.)

### Real components I never designed at all (untouched)
`cluster · stack · context-menu · hover-card · popover · portal · drawer · dialog ·
diff · dropdown-menu · error-boundary · tooltip · tree · tab-panel · calendar` +
chat-elements + media renderers + the 3 missing ship packs (`agents · authority · pipelines`).

## Options to reconcile (pick one)

1. **Import the real `.sui` source** for the components I've designed, diff against my
   `.after` proposals, and rewrite each compare page's "before" to the genuine source — so
   reviewers compare *real current* vs *proposed*. (Highest fidelity; most work.)
2. **Import token/source for a few flagship components** (button, the field family, sheet,
   board) to re-baseline the API + token conventions, then fix my reconstructions to match.
3. **Leave as-is, relabel** — rename "Existing" → "Reference sketch" so the workspace is
   honestly framed as proposals, not a repo mirror.
4. **Fill the gaps** — build the 3 missing ship packs (`agents`, `authority`, `pipelines`)
   and the untouched primitives, after deciding 1–3.

No edits made to repo-derived files pending your call.
