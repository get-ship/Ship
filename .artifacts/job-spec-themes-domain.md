# Job Spec: themes domain — `/studio/themes` + `/studio/themes/[id]` (folds `/theme-studio`)

**Branch:** `job/themes-domain`
**Agent:** ship-designer
**Mode:** autonomous
**Depends on:** `studio-surface` must merge first — it establishes how shell-internal studio sections are built (shell vite bundle, not daemon view-bundler) and mounted (local intercept in main.ts, not `/_sui/views`). Follow its established pattern; do not invent a parallel build path.
**Design source:** `.artifacts/studio-design/themes-list.html` (list) + `theme-detail.html` (detail, absorbs the old `/theme-studio` token editor + live preview)

## Goal

Replace `/theme-studio` with the unified `/studio/themes` + `/studio/themes/[id]` route family. The detail page IS the token editor — list view is swatch-led and brief; detail does the work: a categorized token editor on the left, a live preview frame on the right.

This is independent of the keystone agent-detail polish (running in parallel). Themes has its own visual shape (swatches + preview) — it does not mirror the 880px single-column detail page.

**Studio is shell-internal.** This is NOT a generic view authored against the SDK — it's a first-class shell surface that ships with the shell. The themes section lives in `apps/shell/src/studio/themes/` and is reachable only at the protected `/studio/themes` route namespace. There is NO top-level `/themes` route.

## File scope

```
apps/shell/src/studio/themes/page.sui              # NEW — themes list (swatch-led cards)
apps/shell/src/studio/themes/controller.ts         # NEW — list state, signal subscriptions
apps/shell/src/studio/themes/style.css             # NEW — list view styles
apps/shell/src/studio/themes/[id]/page.sui         # NEW — theme detail (editor + preview)
apps/shell/src/studio/themes/[id]/controller.ts    # NEW — editor state, override tracking
apps/shell/src/studio/themes/[id]/style.css        # NEW — editor + preview layout
```

OPTIONAL (if needed):
- `crates/sui/compiler/tokens.json` — register any new tokens the design adds
- `packages/components/ui-components/src/tokens/tokens.css` — add token values

PORT, don't fork: the existing `/theme-studio` token-editor logic lives in `views/theme-studio/` today. Carry that logic into the new shell-internal location. The old `views/theme-studio/` is retired by the separate `old-routes-cleanup` job after this lands. Do NOT leave a parallel implementation.

DO NOT touch:
- `apps/shell/src/studio/page.sui` + the Studio rail/chrome — that's the `studio-surface` job's territory. You add the `themes` section content; studio-surface wires it into the rail + router.
- `apps/shell/src/chrome/**`, `apps/shell/src/components/**` — shell-chrome job's territory.
- `views/agents/**`, other studio sections — separate jobs.
- `packages/components/primitives/**` — no primitive changes.

## Coordination

The `studio-surface` job builds the Studio shell (rail + top + content slot + router) at `apps/shell/src/studio/`. THIS job fills in the `themes` section content. Coordinate: studio-surface owns the router + rail entry for `/studio/themes`; this job owns the section's view files. On merge, studio-surface's rail "Themes" item routes to this job's `apps/shell/src/studio/themes/page.sui`.

## What to change

### List view (`apps/shell/src/studio/themes/page.sui`)

Reference: `.artifacts/studio-design/themes-list.html`.

Shape:
- Header at the top: "Themes" h1 + a subtitle ("Token packs you can apply per project. Switch the active theme via Studio · Project.")
- Grid of theme cards. Each card:
  - **88px square 2×2 swatch grid** showing four key tokens per pack: `--background`, `--foreground`, `--primary`, `--accent`
  - **Kind chip** (light / dark / special) — tone-tinted: light = muted, dark = inverse, special = primary
  - Name + slug in mono
  - **Override count** (e.g. "12 tokens overridden vs ship-light") — derived per theme
  - Hairline border, hover lift
- Grid: 3 columns at 1100px, 2 at 720px, 1 at narrow

Data source: enumerate the 7 token packs (ship-light, ship-dark, paper, halo, glass, console, contrast). Override counts can be computed from `crates/sui/compiler/tokens.json` + the theme's defining file (each theme's CSS overrides are countable). If this is expensive, ship a hardcoded count map for now and TODO the derivation.

### Detail view (`apps/shell/src/studio/themes/[id]/page.sui`)

Reference: `.artifacts/studio-design/theme-detail.html`. This is the **token editor + live preview** — absorbs the existing `/theme-studio` functionality.

Layout (CSS grid):
```
grid-template-columns: 1fr 1fr;
grid-template-rows: 56px 1fr 56px;
grid-template-areas:
  "head head"
  "editor preview"
  "bar bar";
```

**Head (56px)**: theme name + kind chip + override count + close (back to list)

**Editor (left)**: categorized token list. Categories:
- Foundation (background, foreground, primary, muted, accent, border)
- Tone ramp (info, success, warning, destructive)
- Typography (text-* sizes, leading, tracking, font stacks)
- Radii (radius, radius-sm, radius-md, radius-lg, radius-xl)
- Motion (duration-*, ease-*)
- Effects (shadow-*, focus-ring, etc.)

Each token row:
- Token name in mono (left)
- Current value (right) — for color tokens, a swatch preview + oklch value; for sizes, numeric input; for shadows, the rendered shadow preview
- **Override indicator**: small primary-tinted dot when this token has been customized away from Ship defaults
- Click to edit (inline; no modal)

Source of truth for "Ship defaults": the base tokens.css values. Override = anything different.

**Preview (right)**: live-rendered Ship UI using this theme's tokens. The preview frame should render at least:
- A `<sui-button>` row (variants: solid/soft/outline/ghost/default; tones: default/primary/info/success/warning/destructive)
- A `<sui-card>` with header + content + footer
- A `<sui-input>` and `<sui-select>`
- A `<sui-nav-top>` mock or simple breadcrumb row
- Stat tiles (`<sui-stat>`)
- A toast / alert (`<sui-alert>`)

The preview re-renders whenever the editor changes a token. Easiest implementation: scope the preview in its own DOM subtree with a `<style scoped>` that overrides the tokens, OR use a child custom element with shadow DOM.

**Bottom bar (56px)**: Import / Export buttons (Import CSS or JSON; Export the current theme as a token-only CSS file or JSON). Plus a "Reset to defaults" affordance.

### Controller wiring

- The detail view subscribes to a theme registry (build the registry as a small data structure for now — 7 packs, their token overrides each)
- Editor edits write to a project-scoped overlay (the **project-themes-override**). For this job, scope to localStorage or a session signal; the persistence path through `.ship/project.jsonc` lands when project metadata schema is finalized.
- "Apply to project" button at the top right of the detail head — wires to whatever theme-activation signal Studio's Project section will eventually consume.

## Architectural context

- Charter: L3 view code. Consumes SDK. No reaching into compiler internals.
- File cap: 300 lines per `.sui`/`.ts`. The detail page editor will be long — split into helper modules: `apps/shell/src/studio/themes/[id]/editor.ts` (state) + `apps/shell/src/studio/themes/[id]/preview.ts` (preview rendering) is fine.
- The theme editor today (`/theme-studio`) is the source for behavioral parity. Read `views/theme-studio/controller.ts` and `views/theme-studio/page.sui` to inherit the editing logic, then re-style to match the new mockup.
- This is a **port + restyle**, not a from-scratch build. Carry over logic; restyle UI.

## Acceptance criteria

1. `/studio/themes` renders a 7-card grid matching `themes-list.html` visually.
2. `/studio/themes/[id]` resolves for each of the 7 token packs and renders the editor + live preview matching `theme-detail.html`.
3. Editing a token in the editor live-updates the preview frame.
4. Override indicators show correctly (a dot when the token deviates from base, none when matching).
5. Import / Export work (Export emits CSS or JSON; Import parses and applies).
6. All 7 themes apply themselves correctly when viewing their own detail page.
7. `/theme-studio` (the old route) still works — no regression. A follow-up will redirect and delete.
8. `bun run build` passes. `bun packages/deck/bundler/index.ts --all` succeeds.
9. Tests:
    - `apps/shell/src/studio/themes/page.test.ts` (new) — list view smoke test
    - `apps/shell/src/studio/themes/[id]/page.test.ts` (new) — editor + preview test
10. Visual QA artifact at `.artifacts/web-qa/themes-domain/<timestamp>/`. Cover the list view + at least 2 detail views (ship-light, ship-dark) at 1280x720.

## Pin

Match the mockups for visual fidelity. The 88×88 swatch grid, the kind chips, the override-count meta line on cards — all preserve. The detail page editor's left-right split is critical; the preview iframe sells the experience.

## Out of scope

- Deleting/migrating `/theme-studio` (separate cleanup job).
- Marketplace themes (separate job; this scope is the 7 built-in packs only).
- Per-project theme activation persistence (waits for project metadata schema).
- Theme authoring from scratch (creating a new theme pack — separate job, lower priority).

---

> Execute this job yourself. Do NOT delegate to a subagent.
>
> Commit incrementally: list view first, then detail scaffold, then editor, then preview, then tests + QA.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
