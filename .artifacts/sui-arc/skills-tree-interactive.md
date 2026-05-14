# Skills tree — interactive selection

## Problem

The `/skills` view rendered five skill rows but selecting one did
nothing: no editor pane, no info pane, no URL change. User report:
"5 skills render but they are not interactive on the view."

## Root cause

`views/skills/page.sui` wired each row's click to
`navigate('/skills/' + s.id)`. Two failure modes:

1. The base view owns a local `skillId` signal that drives the
   `skill` SSE live source. `navigate(...)` never wrote to it, so
   the right pane (`{#if skill().kind === 'ok' && skill().value}`)
   stayed empty even when SSR succeeded.
2. The `[id]` subroute the navigate targets uses
   `ctx.ship.skills.detail(...)` directly. The view-host
   (`/sui/<name>`) mounts views without a `ship` SDK in `ctx`, so the
   subroute itself doesn't load data either — clicking a row replaced
   one empty pane with another.

## Fix

`views/skills/page.sui`:

- Row click handler is now `() => skillId.set(s.id)`. That signal flip
  re-evaluates the existing `skill = live(() => sseSource(detail(id)))`
  source, which already streams against the daemon over `/_sui/data`.
- Added `class:is-active` plus `aria-selected` on the row for visual +
  a11y feedback.
- Replaced the static "No file open" empty state with three branches:
  empty / loading / loaded. Loaded shows the breadcrumb + the skill
  body via `<sui-markdown-editor>`, matching the `[id]` view layout.
- Expanded the info aside (`skills-info`) from a single header line to
  full Metadata / Tags / Variables / Files sections plus the Invoke
  button moved into the header so the affordance is visible regardless
  of row hover state.

`views/skills/style.css`:

- Added `.se-skill.is-active` + `.se-skill-row[aria-selected="true"]`
  selection styling.
- Folded in the editor toolbar/breadcrumb/content and info-panel
  styles (`.skills-info`, `.si-*`) that previously only existed under
  `views/skills/[id]/style.css`.

`views/skills/controller.ts`: untouched. The `skillId`, `skill`,
`invoke`, `invokeBusy`, `invokeError` signals it already exposed cover
the full interaction; the bug was purely in the template wiring.

## Out of scope

- `views/skills/[id]/*` is left as-is. It is a separate route, listed
  outside the job's explicit file scope, and its `ship.skills.*`
  controller path is a known view-host limitation tracked elsewhere
  (see the comment at the top of `views/skills/controller.ts`).
- `packages/ui-components/src/sui-tree-view/sui-tree-item.*`: the
  bug report mentioned `sui-tree-item`, but the skills view never
  used that primitive — it renders plain `<button role="treeitem">`
  rows. No framework-level wiring was needed.

## Verification

- `cargo run -p sui-compiler --bin sui-compiler -- build views/skills/`
  → `compiled 2 file(s)`, no diagnostics.
- Acceptance: row click flips `skillId` (signal flip); right pane
  fills from the existing `ship.skills.detail` SSE source; invoke
  button on the info header stays clickable across selections.
