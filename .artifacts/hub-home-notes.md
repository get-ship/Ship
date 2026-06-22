# hub-home — assessment

## Design adopted
Per charter §1-§2: the home dashboard composite takes its
structure from the design bundle (`.artifacts/design-drop/
design/scaffolds/shell/home/home.html` — the "Mission Control"
greeting + widget grid) and the home-domain mocks. The job spec
narrows the surface to greeting + spaces grid + recent runs
(other widgets — events, agents, pipelines, quick-invoke, pins —
are out of scope for this pass; the charter caps us at the
flag-types list and the spec is explicit).

## Connect methods wired
- `ship.pipelines.summaries()` → `<ship-activity-runs>` controller
  (`packages/components/ship/home/src/ship-activity-runs/
  ship-activity-runs.controller.ts`). The component reads
  `active_runs` defensively (`Partial<Summaries>` + `Array.isArray`
  guard, the same pattern as the shell's studio-pipelines
  controller).
- `ship.spaces.config({ id })` is exposed but unused on this
  surface (per-space only). See the `BACKEND-GAP` entry in
  `.artifacts/hub-buildout-flags.md`.
- The greeting + clock are pure-derived (no SDK). Workspace
  label is a constant string sourced from the hub's
  controller (`derived(() => 'ship')`); the spec defers the
  project-switcher wire to a later surface.

## Spaces-list finding: BACKEND-GAP
Verified: `createClient().spaces` exposes only
`config(args)` (`packages/deck/sdk-client/src/ship-sdk.ts:246`).
There is **no** list-all-spaces method, and `/api/v1/projects` is
cut (charter §Connect-only). The home page passes `spacesJson =
'[]'`, the component renders the empty-state with a "Register a
space" affordance. Flagged in
`.artifacts/hub-buildout-flags.md` (the operator hands the gap to
the backend team; the surface degrades gracefully until the
list-all op ships).

## New pack structure + resolution
- `packages/components/ship/home/` — `@ship/home` workspace
  package. Self-declares `deck.bundles: [{ name: "ship-home",
  entry: "src/index.ts", externals: [...runtime, ...dom, ...sdk-
  client], lazy: true }]` exactly like the existing
  `ship-code-editor` / `sui-arcade` / `sui-pdf` packs.
- `src/index.ts` is the umbrella that imports each
  `<tag>.compiled.js` (a virtual id the bundler's `sui_plugin`
  redirects to the sibling `.sui` and compiles in-process).
- Three composite tags:
  - `<ship-home-greeting>` — pure-derived presenter; ticker
    signal at 30s.
  - `<ship-spaces-hero>` — the spaces grid; presentational
    with degraded empty-state. `spaces` attribute is a JSON
    list; `limit` is the bound.
  - `<ship-activity-runs>` — reads
    `ship.pipelines.summaries()` and renders the
    `active_runs` slice; tick signal at 30s.
- All three compose existing `sui-*` primitives
  (`<sui-card>`, `<sui-stack>`, `<sui-cluster>`, `<sui-empty-state>`,
  `<sui-status-dot>`, `<sui-badge>`, `<sui-icon>`, `<sui-button>`).
  No new primitives, no forked layout — per the placement
  hard-rule (charter §Placement).
- The pack's components are reachable in the hub via the
  `app.manifest.jsonc`-style npm-workspace resolution: `apps/hub/
  package.json` adds `@ship/home: workspace:*`, the bundler
  picks up the package via the workspace's `component_roots` map
  (no central registry, no copied file, no bootstrap script —
  the bundler's existing per-package self-declaration + the
  `@sui-tag/<tag>` resolver in `crates/deck/deck-bundler/src/
  sui_plugin.rs` cover it).

## Build result
`cargo run -q -p deck-cli -- build --project apps/hub --bundle-
framework` — clean (`_deck  4 files (self-contained)`). The
`deck.bundles` self-declaration emits a separate `ship-home.js`
chunk (24,251 bytes) under `.deck/`. The hub entry chunk inlines
the components for the synchronous import path (the
`deck.bundles` lazy flag only gates the optional
`DECK_DEV_SKIP_LAZY=1` dev-loop skip — see `crates/deck/deck-
bundler/src/catalog/workspace.rs:47` — production bake builds
eager and lazy identically).

Synthetic emit (`SUI_CLIENT_EMIT=synthetic`) is **not** green for
this surface — the pre-existing `Expected 'from' but found '{'`
Oxc parse error documented in `cutover-clarity.md` §3 (item 3:
"Hub-on-synthetic-emit not yet validated") hits every hub route
including the unchanged `studio/components/[name]` page. The
production build is the acceptance bar in the spec; the synthetic
gap is on the framework side and is the gate for the operator's
Phase 1 cut, not a hub-pack regression.

## Authoring rules honored
- `.sui` templates are pure expressions: no `const`/`let`,
  no I/O, no non-deterministic calls (`Date.now()` /
  `Math.random` / `fetch` all live in the controller or in
  `mount`).
- Logic in controllers: every derived is a pure function of its
  declared signals; the only mutations are the tick signals
  pushed by `mount` + the `setInterval` lifecycle in
  `mount` (torn down on unmount).
- The lint flagged two non-blocking design choices
  (hand-rolled `display:flex` rows in the greeting header
  for `margin-left: auto` on the trailing chip) — each
  documented in the template.
- Tokens only (spacing literals, color literals, radius
  literals all use the design-system `var(--space-N)`,
  `var(--radius-N)`, etc.).
