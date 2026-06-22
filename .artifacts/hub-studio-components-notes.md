# hub-studio-components — assessment

**Branch:** job/hub-studio-components · **Base:** optionB-routes-unify (`c23da1844`)
**Outcome:** studio routes wired; BACKEND-GAP flagged; documented
empty-state; clean hub build.

## Design adopted

The studio pack renders the documented empty-state for both the catalog
and the viewer (per the spec: "if a needed read is absent → BACKEND-GAP
flag + degrade (empty/placeholder state)"). No design drop exists in
this worktree (`.artifacts/design-drop/` is absent), and the charter's
§Connect-only hard rule + the spec's literal BACKEND-GAP instruction
make the empty-state the only honest surface today. The pack's
templates + controllers are written so a future
`ship.ui.components.list()` / `ship.ui.components.detail({ tag })`
op flips the live branch on with a one-line change (the
`entries()` / `entry()` derived becomes a `live(...)`).

## Connect methods wired

None on this surface (BACKEND-GAP). The pack reads no SDK data — both
composites are pure-derived presenters over a static (today empty) data
signal. The controllers' `entries` / `entry` deriveds are the stable
boundary; the contract is documented in the BACKEND-GAP flag entry in
`.artifacts/hub-buildout-flags.md`.

The shell's `ship.ui.read` was retired with the catalog endpoint; the
SDK has no `ui` / `components` namespace to source from. The home
pack's `ship-spaces-hero` (also a documented empty-state) is the
precedent for this degradation pattern; the studio pack follows it
identically.

## Data sources evaluated + why BACKEND-GAP

1. **Connect SDK op (`ship.ui.components.list` / `ship.ui.components
   .detail`)** — does not exist. The SDK's full surface is in
   `packages/deck/sdk-client/src/ship-sdk.ts`; no `ui` or
   `components` namespace. The charter forbids inventing a
   non-existent op.
2. **`GET /_deck/catalog/workspace`** — was the shell's historical
   source. Purged with `packs::serve_catalog_workspace`
   (`crates/deck/deck-axum/src/lib.rs:144`); the runtime no longer
   serves the catalog. The shell's loader calls the dead URL and
   gets a non-OK response — the shell's own studio components
   surface is also degraded.
3. **`.deck/catalogs/workspace.json`** — exists in this worktree
   (217 entries) but the framework's `refresh_catalog` is a no-op
   (`crates/deck/deck-bundler/src/lib.rs:358`). The file is
   gitignored and is a stale snapshot from a previous build, not a
   live index. Using it as if it were live would be a resurrexit of
   a dead path the framework intentionally retired (charter
   §Connect-only).

Per the spec, the catalog IS a build-time index, but the framework
no longer maintains the index — "use what exists" is moot when the
index is no longer being kept in sync. The right call is to flag
the gap and let the backend team ship a real Connect op.

## New pack structure + resolution

- `packages/components/ship/studio/` — `@ship/studio` workspace
  package. Self-declares
  `deck.bundles: [{ name: "ship-studio", entry: "src/index.ts",
  externals: [runtime, dom, sdk-client], lazy: true }]` exactly like
  the existing `@ship/home` / `@ship/chrome` / `@ship/code-editor`
  packs. No central registry, no copied file, no bootstrap script —
  the bundler's existing per-package self-declaration + the
  `@sui-tag/<tag>` resolver in `crates/deck/deck-bundler/src/
  sui_plugin.rs::resolve_id` cover it (the same path the home pack
  uses; documented in `.artifacts/hub-home-notes.md`).
- `src/index.ts` is the umbrella that imports each
  `<tag>.compiled.js` (a virtual id the bundler's `sui_plugin`
  redirects to the sibling `.sui` and compiles in-process).
- Two composite tags:
  - `<ship-component-catalog>` — pure-derived presenter. Wired
    `entries` / `state` / `sub` / `groups` deriveds; the catalog
    filter (`filter` + `pkg` props) is applied against the static
    map. Today's `entries` is `signal<CatalogEntry[]>([])` — the
    BACKEND-GAP boundary.
  - `<ship-component-viewer>` — pure-derived presenter. The
    `category` derived reads a small static tag → category map
    (the same map the shell's `categoryFor` exports, mirrored
    inline so the viewer doesn't import shell internals);
    `entry` is `derived<ViewerEntry | null>(() => null)` — the
    BACKEND-GAP boundary. The meta line says "BACKEND-GAP · no
    components op" today; when the op lands, it becomes
    "N props · M slots" automatically.
- Both compose existing `sui-*` primitives
  (`<sui-empty-state>`, `<sui-stack>`, `<sui-icon>`). No new
  primitives, no forked layout — per the placement hard-rule
  (charter §Placement).
- The pack's components are reachable in the hub via the
  `app.manifest.jsonc`-style npm-workspace resolution:
  `apps/hub/package.json` adds `@ship/studio: workspace:*`, the
  bundler picks up the package via the workspace's `component_roots`
  map.

## Studio route wiring

- `apps/hub/routes/studio/components/page.sui` — the catalog index
  route. Composes `<ship-component-catalog>` (single tag; the
  composite owns the data story + empty-state).
- `apps/hub/routes/studio/components/controller.ts` — empty
  `scope: () => ({})`; the page is a pure composition.
- `apps/hub/routes/studio/components/[name]/page.sui` — the
  viewer route. Composes `<ship-component-viewer prop:tag={name()}>`
  — the `[name]` route param flows through `name` in the
  controller to the viewer's `tag` prop.
- `apps/hub/routes/studio/components/[name]/controller.ts` —
  reads `route.params.name` and returns it on `Scope` (the
  template's `name()` signal). The full viewer logic lives in the
  composite; the route is the prop-binding glue.

The existing placeholder `apps/hub/routes/studio/components/[name]/
page.sui` (which was just `{name()}`) is replaced. The full
inspector (source panel, prop knobs, slot matrix) lands with the
studio surface build-out — this is the real viewer surface today
(degraded). When the SDK ships the detail op, the composite's
`state() === 'found'` branch flips on without a template change.

The existing `<ship-nav-top>` per-page composition is gone
(closed in `job/deck-root-layout` — the root layout owns the
nav). The new `routes/_layout.sui` → `<ship-app-chrome>` is the
single source of truth for the top bar.

## Build result

`cargo run -q -p deck-cli -- build --project apps/hub
--bundle-framework` — clean (`_deck  4 files (self-contained)`).
The generated `routes.gen.ts` carries:

```ts
import * as route0 from './routes/page.sui';
import * as route1 from './routes/studio/components/page.sui';
import * as route2 from './routes/studio/components/[name]/page.sui';
import './routes/_layout.sui';
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
  { pattern: '/' },
  { pattern: '/studio/components' },
  { pattern: '/studio/components/:name' },
]
export const LAYOUTS: LayoutShape[] = [
  { pathPrefix: '/', tag: 'ship-app-chrome' },
]
```

The entry chunk (237KB unminified) carries both composites'
`customElements.define()` calls; the runtime registers the tags
before `mountRouter` runs, so the page mounts the composites on
first paint.

## Tests

Four focused tests for the viewer's `categoryFor` classifier
(`packages/components/ship/studio/test/ship-component-viewer
.test.ts`):

- `classifies the core sui-* primitives` — 15 assertions, one
  per primitive.
- `classifies the ship-* chrome pack tags` — 6 assertions.
- `classifies the ship-* domain composites the hub actually
  mounts` — 5 assertions, covering the home + studio composites
  the hub routes depend on (these silently falling through to
  "Other" would mis-label host pages).
- `falls through to "Other" for unknown tags without throwing` —
  3 assertions, including a hostile-input check (`sui-<script>`)
  to confirm the map is data-only and safe to call with
  untrusted route params.

`bunx vitest run` — 4 passed.

## Authoring rules honored

- `.sui` templates are pure expressions: no `const`/`let`, no
  I/O, no non-deterministic calls. The `<sui-stack>` /
  `<sui-empty-state>` / `<sui-icon>` compositions are pure
  declarative — no per-render mutation.
- Logic in controllers: every derived is a pure function of
  its declared signals. No SDK call, no I/O, no `Date.now` /
  `Math.random` / `fetch`. The empty-state degrades cleanly on
  every load — no race conditions, no async mount.
- The lint flagged raw pixel literals (4px, 6px, 12px) on the
  initial pass; all fixed to design-system tokens
  (`var(--space-1)` / `var(--space-2)` / `var(--space-3)`) and
  to primitive-based layouts (`<sui-stack gap="1">` instead of
  hand-rolled `display:flex; flex-direction:column; gap`).
- Tokens only (spacing literals, color literals, radius
  literals all use the design-system `var(--space-N)`,
  `var(--radius-N)`, `var(--color-N)`).

## Out-of-scope (pre-existing, not introduced by this job)

- `cargo run -p deck-cli -- build-assets` fails on
  `sui-marketing` (TsconfigNotFound on
  `apps/site/src/marketing/marketing-bundle.ts` — `astro`
  package not installed in this worktree). Identical to
  baseline per `.artifacts/bundler-hygiene-notes.md` ("identical
  to baseline, unrelated to this job"). The hub build
  (`cargo run -p deck-cli -- build --project apps/hub`) does
  not require the marketing bundle, so it succeeds.
- The shell's own studio components surface is degraded
  (purged endpoint) — pre-existing, not introduced by this
  job. The hub follows the same degradation pattern as the
  home pack.
- `cargo run -p deck-cli -- build-assets` stage that copies
  `.deck/framework/*.js` → `crates/deck/deck-core/assets/`
  failed on `sui-marketing`, leaving the staging dir empty.
  This job copies the four framework bundles manually (one
  `cp` per file) so `deck build --project apps/hub` can run;
  this is a one-time bootstrap mirroring `dispatch.sh` in
  the deck-core build.rs docstring. The next `build-assets`
  run on a clean tree will overwrite these with fresh
  bundles.
- `cargo test -p deck-bundler --lib` not run from this
  branch — the spec's acceptance is the hub build + the
  vitest run on the new pack. The deck-bundler tests are
  covered by the `job/bundler-hygiene` baseline
  (61 passed + 3 pre-existing failures on this worktree).

| Step | Result |
|---|---|
| `cargo run -p deck-cli -- build --project apps/hub --bundle-framework` | clean, `_deck 4 files (self-contained)` |
| `bunx vitest run` (in `packages/components/ship/studio/`) | 4 passed (1 file) |
| `routes.gen.ts` registered patterns | `/`, `/studio/components`, `/studio/components/:name` |
| `LAYOUTS` registered | `[{ pathPrefix: '/', tag: 'ship-app-chrome' }]` |
| `<ship-component-catalog>` + `<ship-component-viewer>` custom-element registrations | present in the entry chunk |
| Authoring rules | pure expressions, no I/O, no non-determinism, tokens only |
