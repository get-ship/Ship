# Job Spec: Agent detail keystone — polish `views/agents/[id]/page.sui` to canonical shape

**Branch:** `job/agent-detail-keystone`
**Agent:** ship-designer
**Mode:** autonomous
**Design source:** `.artifacts/studio-design/agent-detail.html` (1192 lines, the canonical detail-page shape)

## Goal

Refactor the existing `/agents/[id]` view to match the canonical detail-page shape from the design bundle: single 880px reading column, hue tile + title + gradient underline, hairline-separated sections, prose description, no collapsibles. This is the **keystone** that every other `<domain>/[id]` page will mirror in subsequent jobs (skills detail, theme detail, component detail). Get this right and the pattern unlocks the rest of the design system.

The current view (`views/agents/[id]/page.sui`) wraps content in `sui-collapsible` blocks. The new shape is hairline-separated flat sections — same content, less chrome, more typographic hierarchy.

## File scope

```
views/agents/[id]/page.sui           # the detail view template
views/agents/[id]/controller.ts      # any derived state needed for the new layout
views/agents/[id]/style.css          # most CSS lifted from the mock; some folds into tokens.css
```

OPTIONAL (only if the file needs a new section that doesn't exist):
- `crates/sui/compiler/tokens.json` — add tokens if the design uses any not yet registered
- `packages/components/ui-components/src/tokens/tokens.css` — register matching CSS values

DO NOT touch:
- `views/agents/page.sui` (the list view — separate concern; `.artifacts/studio-design/agents-list.html` is a separate target for a later round)
- `views/agents/[id]/{wire,settings}/` (sub-routes — separate jobs; current sub-route behavior preserved)
- `apps/shell/**` (parallel job)
- `views/studio/**` (parallel job)
- `packages/components/primitives/**` (primitive layer is settled; only add NEW primitives if `agent-detail.html` uses one that doesn't exist — flag in commit body if so)

## What to change

### Layout (`page.sui`)

Reference: `.artifacts/studio-design/agent-detail.html` — the entire file is the design.

Top-level structure:
```html
<div class="detail">
  <a class="detail-back">←  Agents</a>
  <header class="detail-head">
    <div class="detail-tile" style="--tile-color: var(--hue);">{INITIALS}</div>
    <div class="detail-head-body">
      <h1 class="detail-name">{name}</h1>
      <div class="detail-meta">
        <code>{id}</code>
        <span class="detail-meta-sep"></span>
        <span>{provider} · {model}</span>
        <!-- … -->
      </div>
    </div>
    <div class="detail-head-actions"><!-- segmented or buttons --></div>
  </header>
  <div class="detail-rule"></div>          <!-- gradient underline -->
  <p class="detail-prose">{description}</p>

  <section class="detail-section">
    <h2 class="detail-section-title">Skills</h2>
    <ul class="detail-list">…</ul>
  </section>
  <hr class="detail-hairline" />

  <section class="detail-section">
    <h2 class="detail-section-title">MCP servers</h2>
    …
  </section>
  <hr class="detail-hairline" />

  <!-- … Rules · Permissions · Providers · Source … -->
</div>
```

### Specific design decisions to preserve

1. **`detail` container**: `max-width: 880px; margin: 0 auto; padding: 56px var(--space-6) 96px;` Single column.
2. **`detail-tile`** (80×80): `var(--tile-color)` (per-agent hue), `border-radius: var(--radius-lg)`, inner highlight `inset 0 1px 0 rgba(white, 22%)` + inner shadow `inset 0 -1px 0 rgba(black, 8%)`, drop shadow tinted by tile color. Initials in `--font-mono`, 32px, white, `letter-spacing: -0.02em`. Pseudo-element `::before` for the soft diagonal sheen.
3. **`detail-name`** (heading): heading font, `--text-3xl`, `--heading-weight`, `--heading-tracking`, `line-height: 1.05`.
4. **`detail-meta`** (under the name): flex row, `font-size: 13px`, muted-foreground; `<code>` chips use `--muted` background, mono font, `--text-sm`. Separators are 3px dots (`width: 3px; height: 3px; border-radius: 50%;`).
5. **`detail-rule`** (under the head): `width: 96px; height: 4px; border-radius: 2px;` `background: linear-gradient(90deg, var(--primary), transparent);` `margin: var(--space-6) 0;` The single gradient accent line — the visual anchor of the design.
6. **`detail-prose`**: `font-size: var(--text-md); line-height: var(--leading-relaxed);` `max-width: 62ch;` `margin-bottom: var(--space-8);` Prose description, not in a card or container.
7. **`detail-section`**: padding-top `var(--space-6)`, padding-bottom `var(--space-6)`. Each section has a small-caps-style title (`detail-section-title` — `font-size: var(--text-sm)`, `text-transform: uppercase`, `letter-spacing: var(--tracking-wide)`, `font-weight: var(--font-weight-semibold)`, `color: var(--muted-foreground)`).
8. **`detail-hairline`**: `border: 0; border-top: 1px solid var(--border-subtle); margin: 0;` (no extra vertical spacing — sections handle their own padding).
9. **Sections to include**: Description (prose, no h2), Skills, MCP servers, Rules, Permissions, Providers, Source.

### Sub-route navigation

The detail page currently has `<sui-segmented>` for Overview/Wire/Settings switching. Preserve this — it lives in the header right-actions area or as a sub-bar above the prose. Match the design's placement.

### Hue + initials

Each agent has a `hue` (one of: red, orange, amber, yellow, lime, green, teal, cyan, sky, blue, indigo, violet, purple, fuchsia, pink, rose) and computed initials (first 1–2 chars of agent id, uppercased). If the controller doesn't already derive these, add them. The CSS reads `--hue-<name>` tokens — verify these exist in `tokens.css`. If not, add a small palette (16 hues, oklch values) to the token registry.

### Drop sui-collapsible

The current implementation uses `<sui-collapsible>` for each section. Drop it. Each section renders inline, separated by `<hr class="detail-hairline">`. This is **intentional** — the new design uses typographic hierarchy + spacing instead of disclosure.

## Architectural context

- Charter: L2 / L3 separation. This is L3 view code, consumes the SDK.
- File cap: 300 lines (.sui templates can have an exception if heavily structured).
- Composition: this view will eventually live at `/studio/agents/[id]` when the studio surface job lands. For now, it stays at `/agents/[id]` and the route will be aliased later.
- The view consumes existing primitives only. If a needed primitive is missing, flag it in the commit body — don't invent.

## Acceptance criteria

1. `/agents/[id]` renders the new layout matching `.artifacts/studio-design/agent-detail.html` visually at `ship-light`.
2. Single 880px centered column. No collapsibles. Hue tile + gradient underline are visible. Sections are hairline-separated.
3. All existing data (skills, MCP, rules, permissions, providers, source) renders correctly. No data regression.
4. Sub-route navigation to Wire and Settings still works (segmented control or equivalent).
5. All 7 themes apply correctly. The hue tile + gradient underline survive theme transitions.
6. `bun run build` passes. `bun packages/deck/bundler/index.ts --all` succeeds with the agent view bundle building correctly.
7. Tests:
    - Existing tests in `views/agents/[id]/page.test.ts` (if any) pass after the migration.
    - New test: at minimum, smoke test that hue tile, name, prose, and at least one section render with expected DOM structure.
8. Visual QA artifact at `.artifacts/web-qa/agent-detail-keystone/<timestamp>/`. Cover an agent in `ship-light`, `ship-dark`, `console` themes at `1280x720`. The artifact must include before/after screenshots vs the design mock.

## Pin

This is the **keystone** — every future detail page mirrors this. The visual quality and typographic discipline matter more than feature parity. If you find a section in the current implementation that doesn't fit this shape (e.g., heavy data table), pause and flag it rather than forcing it in. The design has hairline-separated prose-and-list sections; a 1000-row data table doesn't belong here — it lives in a sub-route.

## Out of scope

- Studio routing (`/studio/agents/[id]` — separate job migrates).
- Skills/Themes/Components detail pages (later round, mirroring this shape).
- The agents list (`/agents`, the index view — separate job).
- Wire and Settings sub-routes (existing, untouched by this job beyond preserving the nav).

---

> Execute this job yourself. Do NOT delegate to a subagent.
>
> Commit incrementally: structure first (header + rule), then sections one at a time, then style polish, then tests + QA.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
