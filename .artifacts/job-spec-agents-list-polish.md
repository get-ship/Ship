# Job Spec: agents list polish — `/agents` to match `agents-list.html`

**Branch:** `job/agents-list-polish`
**Agent:** ship-designer
**Mode:** autonomous
**Design source:** `.artifacts/studio-design/agents-list.html`

## Goal

Small polish on the existing `/agents` list view to match the design's spacing, typography, and card aesthetic. Per the design's PATH.md: *"The existing source is good; the diff is small."*

Independent of the keystone job (running in parallel). The list aesthetic is its own pattern (1024–1100px wide, hue letters on cards, provider blobs, filter row) and doesn't depend on the detail-page shape.

## File scope

```
views/agents/page.sui                    # existing list view — restyle
views/agents/controller.ts               # likely unchanged; only touch if data shape needs adjusting
views/agents/style.css                   # most of the diff lives here
```

DO NOT touch:
- `views/agents/[id]/**` — separate keystone job is polishing the detail page.
- `views/agents/create/**`, `views/agents/[id]/{wire,settings}/**` — separate jobs.
- `apps/shell/**`, `views/studio/**`, `views/themes/**`, `views/components/**` — parallel jobs.
- `packages/components/primitives/**` — no primitive changes.

## What to change

### Card spacing + typography

Reference: `.artifacts/studio-design/agents-list.html` — compare against the current `views/agents/page.sui` + `style.css`.

Tightening that should land:

1. **Page width**: confirm `max-width: 1100px` centered.
2. **Filter row** ("All / agents / templates" + search + new): spacing, alignment, the count chip on the segmented options.
3. **Card grid**: gap, breakpoints (3-col at 1024, 2-col at 720, 1-col at narrow).
4. **Card shape**: hairline border (`var(--border-subtle)`), no shadow at rest, soft shadow on hover, gentle lift.
5. **Card content layout**:
   - Hue letter tile (top-left), ~40×40, same hue palette as agent-detail
   - Agent name (heading font, --text-md, semibold)
   - Provider/model meta line (mono, muted-foreground, smaller)
   - Description line (clamped to 2 lines)
   - Provider blobs at the bottom (small badge-like chips, one per provider)
6. **Header**: "Agents" h1 + subtitle ("Configured agents in this project. New agents pull from your library; templates live in Studio.")
7. **Empty state**: when no agents exist — use `<sui-empty-state>` with the icon, brief copy, and a "+ New agent" action button.

### What NOT to change

- Don't restructure the data flow.
- Don't add new sub-routes or actions.
- Don't introduce new primitives.
- This is **typographic and spacing polish**, not a feature change.

## Architectural context

- Charter: L3 view. No SDK or primitive changes.
- File cap: 300 lines.
- Coordinate with the parallel agent-detail-keystone job: both use the same hue palette tokens. If keystone adds `--hue-*` tokens to tokens.css, this view consumes them. Pull from `agent-detail.html`'s CSS for the hue tile recipe (inner highlight + soft shadow).

## Acceptance criteria

1. `/agents` renders matching the design's visual fidelity at `ship-light`.
2. Card grid responsive at the three breakpoints (1024 / 720 / narrow).
3. Hue tiles render correctly per agent (consistent with the detail page when keystone lands).
4. Empty state renders when no agents exist.
5. All 7 themes apply correctly.
6. `bun run build` passes. Bundler succeeds.
7. Tests: existing list-view tests still pass; add at least one smoke test for the empty state and hue-tile render.
8. Visual QA artifact at `.artifacts/web-qa/agents-list-polish/<timestamp>/`. Cover empty, populated, and at least three themes (ship-light, ship-dark, console). 1024x720 + 720x... viewports.

## Pin

This is the smallest of the parallel jobs. Don't over-engineer. The design is largely the existing shape; the diff is mostly:
- Typography pass
- Spacing rhythm tightening
- Card chrome reduction (less border weight, no resting shadow)
- Empty state polish
- Filter row alignment

Resist the urge to add new affordances or restructure.

## Out of scope

- Detail page (keystone job handles).
- Create flow (separate later job).
- Cross-resource links to skills/pipelines (later round when those exist).
- Studio integration / route migration (will land when Studio absorbs all domains).

---

> Execute this job yourself. Do NOT delegate to a subagent.
>
> Commit shape: one or two commits — restyle pass, then tests + QA.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
