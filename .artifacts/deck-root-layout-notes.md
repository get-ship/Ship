# deck-root-layout — assessment

**Branch:** job/deck-root-layout · **Base:** optionB-routes-unify (`78a5c60df`)
**Outcome:** FRAMEWORK-GAP closed; nav declared ONCE in a hub root
layout; existing subdir-layout discovery unbroken; hub build clean.

## What the gap was

`crates/deck/deck-bundler/src/routes.rs::discover_layouts` walked only
the **subdirectories** of `routes/` — `routes/<subdir>/_layout.sui` →
prefix `/<subdir>`. A root `routes/_layout.sui` (the file that would
cover `/` and every nested route via the runtime's longest-prefix
match on prefix `/`) is a file, not a subdir, so it was never
emitted. The runtime (`packages/deck/route/src/layouts.ts::matchPrefix`
line 50) already treats prefix `/` as "every pathname" — the gap was
purely in the bundler.

Result: the hub composed `<ship-nav-top>` per-page (a deferred
fallback), documented in `routes/page.sui` and `routes/studio/
components/[name]/page.sui` as
`note(deferred): hub root layout — layout convention gap`.

## Fix (deck-bundler, `crates/deck/deck-bundler/src/routes.rs`)

1. **`discover_layouts`** — now picks up `routes/_layout.sui` (a
   sibling of `routes/page.sui`) and emits a layout entry with
   prefix `/`. The subdir loop is unchanged, so both flavors coexist
   and `sort_by(path_prefix)` keeps the emitted table byte-stable.
   Root layout is checked first (so a root layout's path_prefix `/`
   sorts before any `/<subdir>`).

2. **`layout_module_path`** — new helper, the inverse of layout
   discovery: `/` → `./routes/_layout.sui`, `/studio` →
   `./routes/studio/_layout.sui`. Mirrors `module_import_path` for
   routes.

3. **`render_routes_gen`** — now also emits a static import for each
   layout's `.sui` (`import './routes/_layout.sui'`), so the chrome
   wrapper it composes registers at first paint. The runtime's
   `mountRouter` builds the chrome chain via
   `document.createElement(layout.tag)`; an unregistered tag is a
   no-op and the named-slot projection (`<slot name="route">`) is
   empty. The shell's `apps/shell/src/components/index.ts` was the
   host-side fix for the same gap (the shell's `index.ts` line 26
   imports `@sui-tag/sui-studio-chrome`); the framework now does it
   for every per-app host, so a project doesn't have to remember to
   seed the chrome.

## New chrome wrapper, `@ship/chrome`

`<ship-app-chrome>` is a per-app root layout chrome that composes
`<ship-nav-top>` plus a `<main><slot name="route"></slot></main>`
outlet. Lives at `packages/components/ship/chrome/src/ship-app-chrome/`
(sibling to the other chrome components, registered in
`src/index.ts` alongside `<ship-nav-top>`). The wrapper is the
chrome the LAYOUTS `tag` field names for the root layout; the
`_layout.sui` is just the assembly view of it.

- `ship-app-chrome.sui` — `<ship-nav-top>` + a `<main part="main">`
  with a single `<slot name="route">`. Column flex, min-height 100vh.
- `ship-app-chrome.controller.ts` — empty `scope: () => ({})` (the
  compiler requires a paired controller to emit the custom element
  registration; template-only components without a controller do
  not register).

The wrapper exposes only the `route` slot. View-injected
pass-through slots (per-view top-bar action buttons, command-bar
host) belong on `<ship-nav-top>` (it already exposes `actions` and
`command-bar`); duplicating them on the wrapper would muddy which
chrome layer the content lives in.

## Hub surface change

The per-page `<ship-nav-top></ship-nav-top>` line is gone from:

- `apps/hub/routes/page.sui`
- `apps/hub/routes/studio/components/[name]/page.sui`

The `note(deferred): hub root layout — layout convention gap`
comments are dropped (gap closed). The new
`apps/hub/routes/_layout.sui` is the single source of truth for the
nav:

```
<ship-app-chrome>
  <slot name="route" />
</ship-app-chrome>
```

The runtime's `buildMountTarget`
(`packages/deck/route/src/mount-router.ts:170-182`) inserts a
`<div slot="route">` as the innermost element of the chrome chain;
the chrome's `<slot name="route">` projects it.

## Generated `apps/hub/routes.gen.ts`

```ts
import { mountRouter, type RouteShape, type LayoutShape } from '@ship/deck/route'
import * as route0 from './routes/page.sui';
import * as route1 from './routes/studio/components/[name]/page.sui';
import './routes/_layout.sui';
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
  { pattern: '/' },
  { pattern: '/studio/components/:name' },
]
export const LAYOUTS: LayoutShape[] = [
  { pathPrefix: '/', tag: 'ship-app-chrome' },
]
…
```

`LAYOUTS: [{ pathPrefix: '/', tag: 'ship-app-chrome' }]` — acceptance
criterion 1 met.

## Build result

`cargo run -q -p deck-cli -- build --project apps/hub
--bundle-framework` — clean: `_deck 4 files (self-contained)`. The
entry chunk (`apps/hub/.deck/assets/entry-*.js`, ~210 KB unminified)
inlines the compiled `ship-app-chrome` and `ship-nav-top`
components; the `component()` calls register the custom elements
on import, so by the time `mountRouter` runs, both tags are live.

## Tests

Four new tests in `routes::tests`:

- `layout_discovery_root_layout_has_slash_prefix` — root
  `routes/_layout.sui` produces a layout entry with prefix `/`,
  coexisting with a subdir layout (`/studio`).
- `layout_discovery_root_layout_only_when_no_subdir_layouts` — a
  sole root layout yields exactly one LAYOUTS entry with prefix `/`.
- `layout_module_path_inverts_to_layout_file` — pure-logic test for
  the new helper (`/` → `./routes/_layout.sui`, `/studio` →
  `./routes/studio/_layout.sui`, `/a/b` → `./routes/a/b/_layout.sui`).
- `render_emits_layout_imports_for_chrome_registration` — the
  generated `routes.gen.ts` includes a static `import './routes/
  _layout.sui';` (and a parallel one for a subdir layout) so the
  chrome wrapper registers at first paint.

Existing tests unbroken. Pre-existing failures identical to
baseline (3 in `rolldown_bundle::tests`, all unrelated to routes/
layouts: `sui-marketing` TsconfigNotFound, `pdf` / `arcade` pixi
inline-runtime sizes). Deck-cli tests (`cargo test -p deck-cli`) 7
passed.

| Step | Result |
|---|---|
| `cargo test -p deck-bundler --lib` (layout_ + render) | 7 passed |
| `cargo test -p deck-bundler --lib` (full, post-fix) | 61 passed (4 new), 3 pre-existing failures |
| `cargo test -p deck-bundler --lib` (stashed baseline) | 57 passed, 3 pre-existing failures (no regression) |
| `cargo test -p deck-cli` | 7 passed |
| `cargo run -p deck-cli -- build --project apps/hub --bundle-framework` | clean, `_deck 4 files (self-contained)` |
| `cargo clippy -p deck-bundler --no-deps` | OK |

## Runtime untouched

Per the spec ("Do NOT touch the runtime `resolveLayoutChain`
unless a real, proven gap — if so, flag it."), no runtime changes
were required. `packages/deck/route/src/layouts.ts::matchPrefix`
already treats prefix `/` as "every pathname" (line 50), and
`buildMountTarget` already builds the chain via
`document.createElement(layout.tag)`. The only thing the runtime
needed was a guarantee that the chrome wrapper registers before
`mountRouter` runs — which the framework now provides via the
generated `import './routes/_layout.sui'`.

## Out-of-scope (pre-existing, not introduced by this job)

- The 3 `rolldown_bundle` test failures (TsconfigNotFound, pixi
  inline-runtime, pdf engine size) are pre-existing on this
  worktree; verified identical to the stashed baseline.
- `apps/hub` `package.json` does not declare `@ship/chrome`. The
  per-view resolver (`crates/deck/deck-bundler/src/sui_plugin.rs`
  `resolve_id`) maps `@sui-tag/ship-app-chrome` to the chrome
  pack's source `.sui` via the global `component_index` (built by
  scanning every workspace pack's `src/`); a `package.json` dep is
  not required for chrome-pack tag resolution. The existing
  `<ship-nav-top>` (used per-page before this job) worked the same
  way — the dep was already implicit.
