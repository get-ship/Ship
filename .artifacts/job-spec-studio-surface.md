# Job Spec: Studio surface scaffold — `/studio/*` route family + internal chrome

**Branch:** `job/studio-surface`
**Agent:** ship-designer
**Mode:** autonomous
**Design source:** `.artifacts/studio-design/studio-shell.html` (Studio's own chrome — rail + top + content)

## Goal

Build the Studio surface as a discrete view that paints its own chrome into the host shell's canvas. Studio is **one view among many** — its specialness is that it's where Ship's configuration lives. It's accessed via `⌘K → Open Studio` or the `⌘.` hotkey (wired by the shell-chrome job running in parallel). When open, Studio paints a 220px internal rail + 40px top strip + content area; closing returns to whatever was on the canvas.

The Studio surface's role is to host configuration views for: Project (overview, isolation, network, etc.), Configure (Agents, Skills, MCP, Pipelines, Components, Themes, Permissions), and Library (installed, marketplace, authored). For this job, **only the scaffold + routing**; the individual section interiors are stubs.

## File scope

```
views/studio/page.sui                    # NEW — the Studio surface (rail + top + content slot)
views/studio/controller.ts               # NEW — state for active section, breadcrumb, navigation
views/studio/style.css                   # NEW — the rail + top strip styles, lifted from studio-shell.html
views/studio/manifest.jsonc              # NEW — if your view convention requires it
views/studio/<section>/page.sui          # NEW — one stub per section: project, agents, skills, mcp, pipelines, components, themes, permissions, library
views/studio/<section>/controller.ts     # NEW — minimal stubs; just enough to render a placeholder
.ship/views.jsonc                        # if your router config needs the new routes wired
```

Use existing primitives: `<sui-rail>`, `<sui-rail-item>`, `<sui-icon>`, `<sui-segmented>`, `<sui-breadcrumb>` (verify which exist). Don't invent new primitives unless one is genuinely missing.

DO NOT touch:
- `apps/shell/**` — shell chrome is a parallel job; you only consume what's there.
- `views/agents/**` — separate parallel job is polishing the agent detail page.
- `crates/**`, `packages/components/**` — no compiler or primitive changes.

## What to change

### Routing

Add `/studio/:section?/:id?` to the router. Default section when none specified: `agents` (the most-used). The route resolves to `views/studio/page.sui` which then internally routes to the matching sub-section view via `controller.ts` state.

Wire the `⌘.` hotkey (the shell-chrome-refactor job emits `sui-open-studio` or similar — coordinate via the parent shell). When the shell receives `sui-open-studio`, it pushes the `/studio` route, which mounts this view.

Closing Studio: emit `sui-close-studio` on the host shell when the user clicks the `×` button or the "Close Studio" rail-foot link. The shell-chrome job is responsible for handling that event by popping back to the blank canvas.

### `views/studio/page.sui` — the Studio surface

Reference: `.artifacts/studio-design/studio-shell.html`.

Layout (CSS grid):
```
grid-template-columns: 220px 1fr;
grid-template-rows: 40px 1fr;
grid-template-areas:
  "rail top"
  "rail content";
```

**Rail (left, 220px, full height):**
- Header (10px 16px padding, border-bottom):
  - 20×20 mark with primary-tinted background, primary-colored Ship glyph
  - "Studio" label in heading font, 13px
  - 24×2 gradient underline (primary → transparent)
- Section group "PROJECT" with sub-items (Overview, Isolation, Network, Collaborators, Lifecycle) — STUB rows for now, each linking to `/studio/project/<sub>`. Only `Overview` needs an active state in the default load.
- Section group "CONFIGURE · 7" (the section header has a count chip):
  - Agents (icon: bot, count: 8) — active by default
  - Skills (icon: zap, count: 14)
  - MCP servers (icon: terminal, count: 5)
  - Pipelines (icon: workflow, count: 3)
  - Components (icon: layers, count: 94)
  - Themes (icon: palette, count: 7)
  - Permissions (icon: shield, count: —)
- Section group "LIBRARY":
  - Installed (icon: package, count: 22)
  - Marketplace (icon: store, count: —)
  - Authored (icon: pen, count: 4)
- Foot: "Close Studio" link with × icon, quiet styling

Active item: primary-tinted background, primary-colored icon, semibold label. Hover: muted background.

Use real counts from the SDK where available (`ship.agents.list()`, `ship.skills.list()`, etc.). For things that don't exist yet (MCP, Permissions, Authored), show `—` or a placeholder count.

**Top strip (40px, spans content column):**
- Breadcrumb on the left: `Studio / <Section> / <Subitem>` with "Studio" in primary color + semibold; current item in muted-foreground or foreground.
- Right side:
  - `⌘K` accelerator pill — "Search Studio…" with leading search icon and trailing `⌘K` kbd. Click emits `sui-open-studio-palette` (Studio's own ⌘K scope; the actual palette is a later job).
  - `+` icon button — "New in this section" (stub: emit `sui-studio-new` event; no implementation).
  - `×` icon button — close Studio (emits `sui-close-studio`).

**Content area:**
- Slot for the current section's view. Use a `<sui-view-slot>` or whatever the current view-routing primitive is. Default content: agents section's stub page.

### `views/studio/<section>/page.sui` stubs

For each of the 9 sections (project/agents/skills/mcp/pipelines/components/themes/permissions/library):

Render a placeholder:
- The section name as h1 (heading font, --text-3xl)
- A subtitle: brief one-liner describing what this section configures (e.g., "Agents you've configured for this project")
- A `<sui-empty-state variant="placeholder">` with appropriate icon and text saying "This section's interior is coming in a follow-up. Other tracks are landing the per-section content."

The stubs are intentionally bare — their job is to prove the routing + rail + breadcrumb + chrome plumbing works. The real content lands in subsequent jobs.

### `views/studio/controller.ts`

- Reactive state for: active section, active subitem (within Project), open/closed state.
- Subscriptions to SDK counts for the rail badges.
- Event listeners for `sui-close-studio`, etc.
- Theme reactivity is automatic via tokens; you don't need to wire it.

## Architectural context

- Charter rules apply (L2 has no privileged access, etc.).
- File cap: 300 lines per .sui / .ts. The Studio page may need to split into helper modules for the rail item list; that's fine.
- Per CLAUDE.md: prefer composition over re-implementation. Use existing primitives.
- The `views/studio/` convention follows the established `views/<name>/{page.sui, controller.ts, style.css}` pattern.
- Coordinate event names with the shell-chrome-refactor job (running in parallel). Use the names listed above: `sui-open-studio`, `sui-close-studio`, `sui-open-studio-palette`, `sui-studio-new`. If the parallel job picks different names, reconcile when both branches merge.

## Acceptance criteria

1. `/studio` route resolves to the Studio surface; default opens with Agents section active.
2. `/studio/<section>` resolves to that section's stub view (one of 9 sections).
3. Studio's internal rail renders with all sections grouped (PROJECT · CONFIGURE · LIBRARY), counts populated from SDK where data exists.
4. Clicking a rail item swaps the content area to that section's stub, updates the breadcrumb, updates the URL.
5. Top strip's `⌘K` pill, `+`, and `×` buttons render and emit the expected events when clicked.
6. Clicking `Close Studio` (rail foot) OR `×` (top strip) emits `sui-close-studio` event.
7. The Studio paints inside the host shell's canvas — no rail in the shell, the shell's top + bottom strips are visible outside the Studio chrome.
8. All 7 themes apply correctly: ship-light, ship-dark, paper, halo, glass, console, contrast. Visual transitions are clean.
9. `bun run build` in the relevant package passes. `bun packages/deck/bundler/index.ts --all` succeeds.
10. Tests:
    - `views/studio/page.test.ts` (new) — minimum: rail items render, click navigation works, breadcrumb updates, close events fire.
11. Visual QA artifact at `.artifacts/web-qa/studio-surface/<timestamp>/`. Cover at least Agents and Themes sections, three themes (ship-light, ship-dark, console), 1280x720.

## Pin

Match `studio-shell.html` visually at `ship-light`. Rail spacing, item styles, the gradient underline accent on the rail header, the active-item primary tint — everything that the mock decides should be preserved in the implementation. Use real SDK data for counts, but the visual structure stays.

## Out of scope

- Section interiors beyond the stub. Agents detail, Themes editor, etc. — separate jobs.
- The shell chrome (parallel job).
- Project metadata schema (e.g., what the Project / Overview section actually contains in its UI — that's pending design).
- Per-project theme switching (separate job).
- Marketplace integration (Library section stub is enough for now).

---

> Execute this job yourself. Do NOT delegate to a subagent.
>
> Commit per logical unit: scaffold + routing first, then rail, then top strip, then per-section stubs, then tests.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
