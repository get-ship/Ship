# Hub buildout flags

Operator-reviewed signals from hub-surface jobs. Appended by the
agent, surfaced for the operator's review (per the hub-buildout-
charter §Escalation). Read this, not micro-decisions in the diff.

## job/hub-home (2026-06-20)

### `BACKEND-GAP` — `ship.spaces.list()` (list-all-spaces)

- **Surface:** the hub `/` home dashboard's spaces grid
  (`<ship-spaces-hero>`, in `packages/components/ship/home/`).
- **Symptom:** the Connect SDK exposes only
  `ship.spaces.config({ id })` (per-space runtime config — see
  `packages/deck/sdk-client/src/spaces.ts` and `ship-sdk.ts:246`).
  There is no `ship.spaces.list()` op; the daemon does not broadcast
  the spaces registry. `/api/v1/projects` is cut (charter §Connect-
  only). Calling it would resurrect a dead endpoint.
- **Surface today:** the home controller (`apps/hub/routes/
  controller.ts`) passes `spacesJson = '[]'`. The component renders
  the empty-state with a "Register a space" CTA; clicking it emits
  `ship-space-add` (host-owned wiring).
- **What the backend team needs to ship:** a Connect op that returns
  the registered spaces (id + name + path). The component's prop
  surface (`spaces={JSON}`) is the contract; the controller switches
  from `derived(() => '[]')` to `live(() => ship.spaces.list())` and
  the empty-state disappears the moment the op lands.
- **Not blocking:** home renders cleanly today; the gap is a
  degradation, not a missing feature path.

No `DESIGN-FORK` or `FRAMEWORK-GAP` from this job.

## job/deck-root-layout (2026-06-21)

### `FRAMEWORK-GAP` — hub root layout `routes/_layout.sui` → prefix `/`

- **Surface:** every hub route — `/` and `/studio/components/:name`
  both render the shared `<ship-nav-top>` chrome.
- **Symptom (closed):** `crates/deck/deck-bundler/src/routes.rs::
  discover_layouts` walked only the subdirectories of `routes/` —
  `routes/<subdir>/_layout.sui` → prefix `/<subdir>`. A root
  `routes/_layout.sui` (the file that would cover `/` and every
  nested route via the runtime's longest-prefix match on
  prefix `/`) is a file, not a subdir, so it was never emitted. The
  hub composed `<ship-nav-top>` per-page (a deferred fallback
  documented in `routes/page.sui` and `routes/studio/components/
  [name]/page.sui` as `note(deferred): hub root layout — layout
  convention gap`).
- **Fix (deck-bundler, `crates/deck/deck-bundler/src/routes.rs`):**
  - `discover_layouts` now also picks up `routes/_layout.sui`
    (sibling of `routes/page.sui`) and emits a layout entry with
    prefix `/`. The subdir loop is unchanged — both flavors coexist
    and `sort_by(path_prefix)` keeps the emitted table
    byte-stable.
  - `render_routes_gen` now also emits a static import for each
    layout's `.sui` (`import './routes/_layout.sui'`) so the
    chrome wrapper it composes registers at first paint. The
    runtime's `mountRouter` builds the chain via
    `document.createElement(layout.tag)`; an unregistered tag is
    a no-op and the named-slot projection (`<slot name="route">`)
    is empty. The shell's `apps/shell/src/components/index.ts` was
    the host-side fix for the same gap; the framework now does
    it for every per-app host.
  - New helper `layout_module_path` is the inverse of the layout
    discovery — `/` → `./routes/_layout.sui`, `/studio` →
    `./routes/studio/_layout.sui` — and a test asserts the
    contract.
- **New chrome wrapper, `@ship/chrome`:** `<ship-app-chrome>` is
  a per-app root layout chrome that composes `<ship-nav-top>`
  plus a `<main><slot name="route"></slot></main>` outlet. Lives
  at `packages/components/ship/chrome/src/ship-app-chrome/`
  (sibling to the other chrome components, registered in the pack
  index). The wrapper is the chrome the LAYOUTS `tag` field names
  for the root layout; `_layout.sui` is just the assembly view of
  it. No business logic — `<ship-nav-top>` owns the bar, the
  runtime owns the route-content projection.
- **Hub surface change:** the per-page `<ship-nav-top>` in
  `routes/page.sui` and `routes/studio/components/[name]/page.sui`
  is gone; the nav is declared ONCE in
  `routes/_layout.sui` → `<ship-app-chrome>`. The
  `note(deferred): hub root layout — layout convention gap`
  comments are dropped (gap closed).
- **Generated `apps/hub/routes.gen.ts` now carries:**
  ```ts
  export const LAYOUTS: LayoutShape[] = [
    { pathPrefix: '/', tag: 'ship-app-chrome' },
  ]
  ```
  and `import './routes/_layout.sui';` so the chrome registers
  before `mountRouter` runs.
- **Tests:** 4 new tests in `routes::tests` —
  `layout_discovery_root_layout_has_slash_prefix`,
  `layout_discovery_root_layout_only_when_no_subdir_layouts`,
  `layout_module_path_inverts_to_layout_file`,
  `render_emits_layout_imports_for_chrome_registration`. Existing
  layout/subdir-discovery tests unchanged. `cargo test -p
  deck-bundler --lib` is 61 passed (4 new) + 3 pre-existing
  failures (TsconfigNotFound on `apps/site/src/marketing/
  marketing-bundle.ts`, pixi inline-runtime / pdf-engine sizes —
  identical to baseline, unrelated to this job). `cargo test -p
  deck-cli` is 7 passed.
- **Runtime untouched:** `packages/deck/route/src/layouts.ts`
  already handled prefix `/` (`matchPrefix` line 50). Per the
  spec, no runtime changes were required; the gap was purely in
  the bundler's `discover_layouts`. The host-side
  chrome-registration fix is also in the bundler, not the
  runtime.

No `BACKEND-GAP` or `DESIGN-FORK` from this job.


## job/hub-studio-components (2026-06-21)

## job/hub-studio-components (2026-06-21)

### `BACKEND-GAP` — `ship.ui.components` (component catalog + component detail)

- **Surface:** the hub `/studio/components` index
  (`<ship-component-catalog>`, in `packages/components/ship/studio/`) and
  the hub `/studio/components/:name` viewer
  (`<ship-component-viewer>`, in `packages/components/ship/studio/`).
- **Symptom:** the Connect SDK (`packages/deck/sdk-client/src/
  ship-sdk.ts`) exposes only `account`, `agents`, `artifacts`, `canvas`,
  `collections`, `events`, `git`, `mesh`, `pipelines`, `sessions`,
  `settings`, `skills`, `spaces` — no `ui` / `components` namespace.
  Two historical data sources the shell used are also gone:
  1. The runtime HTTP endpoint `GET /_deck/catalog/workspace` was
     purged with `packs::serve_catalog_workspace`
     (`crates/deck/deck-axum/src/lib.rs:144`); the shell's loader
     (`apps/shell/routes/studio/components/catalog.ts:165`) calls the
     dead URL and gets a non-OK response — the shell's own studio
     components surface is also degraded.
  2. The framework's `refresh_catalog` (`crates/deck/deck-bundler/
     src/lib.rs:358`) is a no-op — the
     `.deck/catalogs/workspace.json` file (when it exists) is a stale
     snapshot from a previous build, not a live index of the current
     source tree. Using it as if it were live would be a resurrexit
     of a dead path the framework intentionally retired (charter
     §Connect-only: "anything calling a non-Connect / dead /
     outdated endpoint is cut or re-grounded on the current SDK —
     never resurrected").
- **Surface today:** the studio pack renders the documented
  empty-state (`<sui-empty-state>` with the gap caption + the
  operator-visible flag pointer). The catalog index exposes an
  "open the sui-card viewer" deep-link so the URL is still
  navigable end-to-end; the viewer renders the requested tag, its
  category (a stable public fact from a static map, not invented
  data), and the same gap caption. No fabricated data, no
  resurrected fetch, no shell endpoint.
- **What the backend team needs to ship:** a Connect op that
  returns the registered components (id + tag + pkg + description +
  props + slots + sourcePath). The natural shapes are
  `ship.ui.components.list()` (for the index) and
  `ship.ui.components.detail({ tag })` (for the viewer). The
  controllers' `entries` / `entry` deriveds are the stable boundary —
  flipping them to `live(() => ship.ui.components.list())` /
  `live(() => ship.ui.components.detail({ tag: tag() }))` is the
  only change required; the templates' `state() === 'ready'` /
  `state() === 'found'` branches then render the props / slots /
  source / snippet sections without a template change. The
  component prop surface (`tag` on the viewer, `filter` + `pkg` on
  the catalog) is the contract.
- **Not blocking:** the studio routes build clean and render the
  documented empty-state today; the gap is a degradation, not a
  missing feature path. The shell's own studio components surface
  has the same gap and is also degraded.

No `DESIGN-FORK` or `FRAMEWORK-GAP` from this job.

## job/port-studio (2026-06-21)

### `BACKEND-GAP` — `ship.events.list` for the agents list per-card status badge

- **Surface:** the hub `/studio/agents` index
  (`apps/hub/routes/studio/agents/controller.ts`, the
  `agentEvents` live signal at the top of `scope`).
- **Symptom:** the per-card "running / completed / errored" status
  chip reads from `ship.events.list({ event_type: 'agent.',
  limit: 50 })`. The hub's `createShipConnectClient` does not yet
  expose an `events.list` op on its Connect surface — the events
  namespace is a live SSE channel on `@ship/sdk-client`, not a
  Connect op. The page renders the full UI; the per-card status
  chip simply doesn't show. Same code path the shell's
  `apps/shell/routes/studio/agents/controller.ts` (verbatim
  copy) used.
- **Surface today:** the agents list renders the title, sub
  counts, project/library segmented filters, the search box, and
  the card grid with hue tiles, name, description, skills/mcp/
  rules counts, source badge, and tags. The per-card live status
  chip is the one element that doesn't paint.
- **What the backend team needs to ship:** a Connect op
  `ship.events.list({ event_type, limit })` that returns the
  same `{ groups: [{ rows: EventRow[] }] }` envelope the SSE
  channel emits. The controller's `agentEvents` is the stable
  boundary — switching from the SSE channel to a Connect op is
  a one-line change.
- **Not blocking:** the agents list builds clean and renders the
  full UI; the chip is a degradation, not a missing feature path.

### `BACKEND-GAP` — `routes.studioAgentsIdWire` / `studioAgentsIdSettings` (cut sub-routes)

- **Surface:** the hub `/studio/agents/:id` detail page's tab
  strip (`<sui-toggle value="overview|wire|settings">`, the
  shell's `agents/[id]/page.sui` lines 61-65) and the
  controller's `goWire` / `goSettings` / `goTab` handlers
  (`apps/hub/routes/studio/agents/[id]/controller.ts` lines
  192-200).
- **Symptom:** the sub-routes `apps/hub/routes/studio/agents/
  {create,settings,wire}/` were CUT per the port-studio spec
  ("CUT the YAGNI sub-routes: `create/`, `[id]/settings/`,
  `[id]/wire/`"). The generated `apps/hub/routes.gen.ts`
  therefore does not include `studioAgentsIdWire` /
  `studioAgentsIdSettings` / `studioAgentsCreate` keys. The
  controller's calls
  `navigate(routes.studioAgentsIdWire({ id: s.agentId }))` and
  similar throw "undefined is not a function" at runtime; the
  page's `banner` signal surfaces the error.
- **Surface today:** the detail page renders the full UI — hue
  tile, name, meta, action buttons (activate, edit, tabs),
  description, tags, skills section, MCP servers section, rules
  section, permissions section, providers section, source path.
  The Overview / Wire / Settings tab strip and the segment
  control work; clicking Wire or Settings throws a runtime
  error and the page's banner surfaces it.
- **What the backend team needs to ship:** the cut sub-routes
  themselves. Each is a separate view tree in the shell
  (`apps/shell/routes/studio/agents/{create,settings,wire}/`).
  Porting them (one job per sub-route, or a batch) is the
  same shape as this port: verbatim shell markup, the spec's
  two import seams, and the same BACKEND-GAP surface rules.
Resolves when those routes land.

No `DESIGN-FORK` or `FRAMEWORK-GAP` from this job.

## job/hub-data-layer (2026-06-22)

Wired the studio data surfaces to the daemon's live-data contract
(`apps/shipd/src/ship_runtime/live_resolvers.rs`): `ship` = live SSE client
(ops WITH a resolver), `ops` = unary Connect (everything else). Committed:
skills (unary + `{json}` decode), agents (live `agents.summaries`), client split.

### `BACKEND-GAP` — `events.list` has no live mux resolver
- **Surface:** the agents list's per-card run-status badge (`running` /
  `completed` / `errored`), `apps/hub/routes/studio/agents/controller.ts`
  (`agentEvents` → `agentStatus`).
- **Symptom:** `ship.events.list(...)` opens an SSE subscribe that 404s
  (`live_resolvers.rs` registers only git.overview / agents.summaries /
  collections.list / account.state). The mux-client marks the 4xx permanent
  (no retry storm) and the status map degrades to `{}`.
- **Surface today:** the 7 agent cards render (name, description, source, tags,
  active chip); only the live run-status pill is absent (it's `{#if}`-guarded).
- **Closes when:** a `events.list` live resolver lands (or events move to a
  Connect unary op the controller can poll).

### `BACKEND-GAP` — `skills.summaries` / `skills.detail` are unary-only
- **Surface:** `/studio/skills` (list) + `/studio/skills/:id` (detail).
- **Detail:** skills has NO live resolver, only the unary Connect ops, which
  wrap their payload in a generic `{ json }` envelope. The shell reads skills
  over its live client; the hub reads via `ops` + decodes the envelope. Works,
  but skills do not live-update. Not blocking — flagged for parity if skills
  ever need live (they rarely change).

### `BACKEND-GAP` / `FRAMEWORK-GAP` — components catalog (`/_deck/catalog`)
- **Surface:** `/studio/components`. Fetches `GET /_deck/catalog/workspace`,
  which 404s (no such daemon endpoint). 0 cells render.
- **Direction (operator):** load the component catalog from the compiler's
  custom-elements-manifest (CEM), NOT a deck REST endpoint. Needs CEM emission
  from the sui compiler/bundler + a hub client path that consumes it. Tracked
  as task #4.

### `FRAMEWORK-GAP` (observation) — generated entry passes `clients: {}`
- `apps/hub/routes.gen.ts` injects no named clients into the view mount ctx, so
  controllers that `scope: ({ ship, agents }) => …` (ported from the shell's
  ctx-injection model) get `undefined`. The hub convention is module-import
  clients (`src/lib/client.ts`); the agents controllers were rewired to that.
  Note for any further shell ports: use the module client, not ctx.

### QA-TOOLING note — headless render crash on large card grids
- `/studio/skills` (15 cards) crashes the headless chromium renderer in WSL2
  (debug build), even with `--disable-gpu`; `/studio/agents` (7 cards) is fine.
  The data wiring is verified correct (cap-3 render + CDP DOM). This is a
  headless artifact — 15 DOM/CSS cards are trivial for webkit2gtk. Real-app
  (Tauri) render sanity of large lists is owed; if it reproduces there, it's a
  sui-runtime keyed-`{#for}` scaling issue, not a data bug.
