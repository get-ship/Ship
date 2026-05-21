# Job Spec: components domain — `/studio/components` + `/studio/components/[id]` (folds `/ship-ui-studio`)

**Branch:** `job/components-domain`
**Agent:** ship-designer
**Mode:** autonomous
**Depends on:** `studio-surface` must merge first — it establishes how shell-internal studio sections are built (shell vite bundle, not daemon view-bundler) and mounted (local intercept in main.ts, not `/_sui/views`). Follow its established pattern; do not invent a parallel build path.
**Design source:** `.artifacts/studio-design/components-list.html` (grouped index) + `component-detail.html` (variant×tone matrix + playground, absorbs `/ship-ui-studio`)

## Goal

Replace `/ship-ui-studio` with `/studio/components` + `/studio/components/[id]`. List is a grouped index (Actions / Inputs / Layout / Feedback / Indicators / Overlays / Data). Detail is the storybook: variant×tone matrix preview, knobs playground with segmented controls per prop and live render, props table, slots, used-by surfaces, source path.

Independent of keystone — components detail has its own shape (matrix + playground, not the 880px single-column form). Compatible with all currently-running parallel jobs.

**Studio is shell-internal.** Not a generic SDK-authored view — a first-class shell surface that ships with the shell. The components section lives in `apps/shell/src/studio/components/` and is reachable only at the protected `/studio/components` route namespace. There is NO top-level `/components` route.

## File scope

```
apps/shell/src/studio/components/page.sui                # NEW — grouped index
apps/shell/src/studio/components/controller.ts           # NEW — list state, category grouping
apps/shell/src/studio/components/style.css               # NEW — grouped index styles
apps/shell/src/studio/components/[id]/page.sui           # NEW — component detail (matrix + playground)
apps/shell/src/studio/components/[id]/controller.ts      # NEW — knob state, code-snippet emission
apps/shell/src/studio/components/[id]/style.css          # NEW — matrix + playground styles
```

PORT, don't fork: the existing `/ship-ui-studio` catalog logic lives in `views/ship-ui-studio/` today. Carry it into the new shell-internal location. The old `views/ship-ui-studio/` is retired by the separate `old-routes-cleanup` job after this lands. Do NOT leave a parallel implementation.

## Coordination

`studio-surface` builds the Studio shell (rail + router) at `apps/shell/src/studio/`. THIS job fills in the `components` section content. studio-surface owns the rail entry + router for `/studio/components`; this job owns the section's view files.

DO NOT touch:
- `apps/shell/src/studio/page.sui` + Studio chrome/rail — studio-surface job.
- `apps/shell/src/chrome/**`, `apps/shell/src/components/**` — shell-chrome job.
- `views/agents/**`, other studio sections — separate jobs.
- `packages/components/primitives/**` — no primitive changes.

## What to change

### List view (`apps/shell/src/studio/components/page.sui`)

Reference: `.artifacts/studio-design/components-list.html`. Grouped index style — section headers + rows, scannable.

Categories (from the design):
- **Actions**: button, button-group, chip, toggle, switch, theme-toggle
- **Inputs**: input, textarea, select, combobox, autocomplete, checkbox, radio, slider, date-picker, color-picker, file-upload, tag-input
- **Layout**: card, stack, cluster, grid, panel-dock, rail, toolbar, divider, scroll-area
- **Feedback**: alert, alert-dialog, toast, dialog, drawer, popover, tooltip, hover-card, empty-state, error-state, skeleton, spinner, progress
- **Indicators**: badge, status-dot, avatar, kbd, label, metachip, stat, timestamp
- **Overlays**: dropdown-menu, context-menu, navigation-menu, accordion, collapsible, tabs, tab-panel
- **Data**: table, list-tile, tree-view, history-row, breadcrumb, code, diff, json-view, markdown, mark

Use the catalog at `packages/components/primitives/dist/custom-elements.json` (or the SDK catalog endpoint) as the live source of truth — don't hardcode the component list. Categories may need to be inferred from tag name pattern or live in a small metadata file.

Each row in a category:
- Tag name in mono (`<sui-button>`)
- One-line description (from custom-elements.json description field)
- Compact, scannable layout — no card chrome around each row, just a hover state

Header: "Components" h1 + subtitle ("Built-in primitives. Click for variants, props, and source.")

### Detail view (`apps/shell/src/studio/components/[id]/page.sui`)

Reference: `.artifacts/studio-design/component-detail.html`. Multi-section layout:

**Head**: tag name (large, mono) + description + import path

**Variant × Tone Matrix** (the storybook):
- Grid of rendered component instances
- Rows = variants (solid/soft/outline/ghost/default)
- Columns = tones (default/primary/info/success/warning/destructive/accent/muted)
- Each cell renders the actual primitive with the variant + tone combination
- Skipped cells where the component doesn't have that variant/tone

**Playground** ("knobs"):
- For each prop, a control: `<sui-segmented>` for enums, `<sui-switch>` for booleans, `<sui-input>` for strings, etc.
- Live render of the component with current knob values
- Code snippet emitter at the bottom: copy-able `<sui-X prop="value">` ready to paste
- "Reset" affordance

**Props table**: name · type · default · description. Pulled from custom-elements.json.

**Slots list**: name + description. Pulled from custom-elements.json.

**Used by**: scan the codebase (views/ + apps/) for usages of this tag, link to the source files. Could be precomputed; for first pass, a TODO is fine.

**Source path**: link to `packages/components/primitives/src/<tag>/<tag>.sui` (with line counts, file size).

### Controller wiring

- List view: read catalog from custom-elements.json at module init; group by tag-name prefix or a small static map.
- Detail view: parse the component's prop schema, render appropriate knob control per prop type, maintain reactive knob state.
- Code snippet emitter: serialize the current knob state to a `<sui-X prop1="val1" prop2="val2">…</sui-X>` string, render in a `<sui-code>` block.

## Architectural context

- Charter: L3 view. Consumes the SDK's catalog data.
- The catalog (`packages/components/primitives/dist/custom-elements.json`) is the source of truth for prop schemas, slots, descriptions. Verify it's complete and reflects the current state of primitives; if there are gaps, flag in commit body — that's a primitives crate concern, not this view's.
- This is a **port + restyle** of `/ship-ui-studio`. Behavior carries over, design changes.

## Acceptance criteria

1. `/studio/components` renders the grouped index matching `components-list.html`.
2. `/components/<tag>` resolves for every registered primitive and renders the storybook.
3. The variant × tone matrix renders correctly for at least sui-button (the canonical case).
4. The playground knobs work — changing a knob updates the live render AND the code snippet.
5. Props table populated from custom-elements.json.
6. All 7 themes apply correctly to both views.
7. `/ship-ui-studio` (old route) still works — no regression.
8. `bun run build` passes. Bundler succeeds.
9. Tests:
    - `apps/shell/src/studio/components/page.test.ts` — list smoke + category presence
    - `apps/shell/src/studio/components/[id]/page.test.ts` — detail smoke + knob change updates render
10. Visual QA artifact at `.artifacts/web-qa/components-domain/<timestamp>/`. Cover list + at least sui-button detail + sui-input detail across ship-light, ship-dark.

## Pin

The grouped index aesthetic is "manual page" — quiet category headers, scannable rows, minimal chrome. Resist the urge to add card backgrounds or borders per row. The detail page's variant × tone matrix is the storybook payoff — make sure it renders with REAL primitives (live, themed, interactive), not screenshots.

## Out of scope

- Component authoring flow (creating new components from this UI).
- Marketplace components (the "installed via marketplace" tab — separate job).
- Cleanup of `/ship-ui-studio` (separate cleanup job).
- `used-by` codebase scan beyond TODO (separate enhancement).

---

> Execute this job yourself. Do NOT delegate to a subagent.
>
> Commit incrementally: list first, then detail scaffold, then matrix, then playground, then tests + QA.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
