---
title: Porting from React
audience: agent-only
---

# Porting from React — Concrete Exercise Notes

Captured from porting the Agents design (`Agents.html`, React + Babel + hand-rolled JSX) to Ship's `.ship/views/agents/`. Each entry identifies a specific friction point and either the framework improvement that would remove it or the pattern a view author should reach for today.

## What ported trivially

| React | Ship equivalent | Notes |
|---|---|---|
| `useState + localStorage` for view mode | `data-signals:view="'list'"` + `persist` plugin | ~1:1 |
| `React.useEffect` applying `data-theme` | `<sui-view-chrome>` contribution | Shell owns chrome |
| `onClick={() => setView("create")}` | `data-on:click="$view = 'create'"` | Shorter |
| `<ShipButton variant="primary">` | `<sui-button tone="accent" variant="solid">` | Same shape |
| Conditional rendering (`view === "list" && …`) | `data-show="$view == 'list'"` | Same shape |
| Card grid | Existing `.agents-grid` + `sui-list-tile variant="card"` | Ship's card handles hover-lift + navigable |

## Concrete gaps found

### 1. Per-identity avatar colors

The design's `Avatar` renders one of eight conic-gradient palettes driven by `agent.color` (`conic-orange`, `conic-red`, `conic-violet`, …). Ship's `<sui-avatar>` has `tone="primary|success|…"` and a single default gradient — no per-identity hue. Today all agents render with the same `tone="primary"` square, which flattens the grid visually.

**Gap:** `<sui-avatar>` needs an `identity-hue` attr (or equivalent) that accepts a string and hashes it to a stable palette entry. The design-system roadmap should call this out as a Tier-1 primitive extension — every "list of things with a face" view wants it.

**Workaround today:** write the gradient inline per card, bypassing the primitive. Don't. Wait for the primitive.

### 2. Provider blobs

Tiny colored dots representing which AI providers (claude / codex / cursor / gemini / opencode) an agent compiles to. Pattern appears in at least three upcoming views (agents, skills, pipelines that dispatch agents).

**Gap:** `<sui-provider-blob provider="claude">` primitive in `packages/ship-set/src/web-components/domain/` — a domain component since "AI provider" is a Ship-platform concept.

**Workaround today:** inline `<span class="prov-blob prov-<provider>">` with CSS tokens. Applied in `.ship/views/agents/index.html.j2` as a temporary measure; CSS mix is theme-aware.

### 3. Rich-card composition without view-local CSS

The design's agent card is a complete composition (avatar / name / desc / stats / providers / badges). We express this via `<sui-list-tile>` with slots + view-local CSS for the stats row, provider row, and badge row. That's ~70 lines of CSS for what React does in ~25 lines of JSX.

**Root cause:** `<sui-list-tile>`'s slot model is row-oriented. For card mode, the primary slot becomes a vertical stack but there's no idiomatic way to structure "title / description / metadata-rows / chips" without reaching for view-local CSS.

**Gap:** a `<sui-card>` domain primitive (or a `sui-list-tile variant="card" density="rich"` mode) that owns the vertical stack layout, accepts named slots for title/desc/meta/chips, and wraps chips with `flex-wrap`. Halves the CSS per card-grid view.

**Workaround today:** the view-local CSS pattern in `.ship/views/agents/index.html.j2` is the template to copy. Document as a recipe until the primitive lands.

### 4. Canvas / detail view routing

The design has four views per page (`canvas`, `list`, `detail`, `create`, `settings`) switched via a single `view` signal. React's `App` component is one pattern-match on `values.view`.

Ship's equivalent works but forces every sub-view's JSON context to be available up-front, because `data-show="$view == 'foo'"` renders them all and hides the non-active ones. For heavy views this costs render + payload weight.

**Gap:** a `{% lazy %}` macro or `data-load-on-show` attribute that defers rendering of a subtree until its condition becomes true the first time, then caches. Datastar can SSE-patch into an empty shell on demand. Not critical — the composition works today — but significant for data-heavy multi-view pages.

**Workaround today:** split into separate routes per view (`/agents/list`, `/agents/create`, `/agents/<id>`) — what our existing agents view does. Lose the single-page feel; gain obvious lazy loading.

### 5. Tweaks panel / design-canvas scaffolding

The React design ships a "Tweaks" panel that lets the designer swap theme / layout / data-state live. Pure design-tool output; not needed in production.

**Observation:** we already absorbed this pattern at the shell level (theme picker in topbar). The per-view tweaks panel is a design-time affordance that doesn't port — and shouldn't.

### 6. Data-shape parity

React design uses a hand-authored `AGENTS` array with 14 columns per row (id, name, v, color, letter, desc, scope, skills, mcp, rules, providers, active, tags). Ship renders from `AgentSummary` served by the daemon, which has ~8 columns — missing `color`, `letter`, `active`, `tags`, `providers`.

**Gap:** `AgentSummary` in `apps/shipd/src/views_agents.rs` needs extending: `providers: Vec<String>`, `active: bool`, `tags: Vec<String>`, `color_seed: String` (hash input for identity-hue). `letter` can derive client-side from `name`.

Once extended, the generated `sdk-view-contexts.json` will surface the new fields automatically and AST-lint catches any template that references them incorrectly. The pipeline is already there; the data model just needs the columns.

### 7. Mental model — one concept Ship should lean into

React's `Tweaks` panel mutates live props; the design canvas lets the designer A/B layouts. We don't ship an equivalent and we don't need to — but the **dry-run render endpoint** (T7) is an adjacent primitive. An agent-authoring UI that lets a user POST mock contexts to `?__dry-run=1&__context=<json>` and see the view render instantly *is* the "tweaks panel" equivalent for views. Worth wiring into the shell's command palette as "preview with mock data."

## What React gave us that Ship doesn't (and honestly shouldn't)

- Per-component state hooks. Ship's signals are one flat namespace per view. For the kind of "this accordion is open" state Ship views need, that's fine. If the app truly needs per-item hierarchical state, wrap in a web component with its own state.
- JSX type inference. The SDK manifest + AST lint cover most of the slippage; the rest is where TypeScript-end-to-end would have helped and won't.
- CSS Modules scoping. Our view-local styles are genuinely view-global (they live in the document head). A `<style data-ship-view-scope>` attribute that auto-prefixes selectors with the view's container class would close this. Not critical today.

## What Ship gave us that React doesn't

- Server rendering is the default — HTML arrives complete, no hydration lag.
- Design-system first — tokens + themes + primitives are a first-class contract; no ecosystem shopping.
- Zero build in dev — edit the `.j2`, refresh, see. (Caveat: `_ship.html.j2` is embedded via `include_str!` — daemon rebuild required for SDK changes. Tracked in roadmap as dev-ergonomics.)
- Agent retrieval via `GET /api/sdk/query` — no React-side equivalent unless you ship CEM + a search index manually.
- AST-lint catches undefined-context-field at authoring time, not runtime — better than TypeScript's component-render errors.

## Top three framework investments this exercise motivated

1. **`<sui-card>` domain primitive** — closes the "~70 lines of view-local CSS per card grid" gap. Visible on every list view we build.
2. **`<sui-avatar identity-hue>` extension** — ten-minute primitive change; every faces-grid view benefits.
3. **`AgentSummary` schema expansion + sdk-view-contexts regen** — surface the fields the design uses (`providers`, `active`, `tags`). Touches one Rust struct + a schemars derive; every agents-shaped view picks it up automatically.

Each is a small, scoped change. In aggregate they'd take the agent-page port from "70 lines of view CSS" to "15 lines, three of them layout."
