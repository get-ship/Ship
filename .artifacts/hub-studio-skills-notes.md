# hub-studio-skills — assessment

**Branch:** job/hub-studio-skills · **Base:** optionB-routes-unify (`c23da1844`)
**Outcome:** studio routes wired; live data via Connect SDK; clean
hub build; 21 vitest cases green.

## Design adopted

Per charter §1-§2: the studio skills surface adopts the
hub's standard "Composite owns the data story + empty-state"
pattern (the precedent set by `@ship/home` for the spaces
BACKEND-GAP and `@ship/studio` for the components BACKEND-GAP).
The shell's editor detail page was deliberately not ported — the
hub is a read-only viewing surface per the spec; the shell's
write path stays where it lives.

Two composites:

- `<ship-skill-list>` — the hub `/studio/skills` index. Reads
  `ship.skills.list()` (Connect SDK LiveSource) and renders
  the registered skills grouped by SkillScopeTag
  (project/user/library, in stable display order).
- `<ship-skill-detail>` — the hub `/studio/skills/:id` viewer.
  Reads `ship.skills.detail({ id })` (Connect SDK LiveSource)
  and renders the frontmatter (key/value), body (via
  `<sui-markdown>`), and files panel.

Both compose existing `sui-*` primitives — no new primitives,
no forked layout. The chrome (top nav, breadcrumb) is the root
layout's `<ship-app-chrome>`.

## Connect methods wired

- `ship.skills.list()` →
  `<ship-skill-list>` controller
  (`packages/components/ship/skills/src/ship-skill-list/ship-skill-list.controller.ts`).
  Typed `LiveSource<{ items: SkillIndexEntry[] }>` per
  `packages/deck/sdk-client/src/ship-sdk.ts:211`; controller
  narrows `unknown` defensively (each entry coalesced against
  missing fields, hostile shapes render as empty-state).
- `ship.skills.detail({ id })` →
  `<ship-skill-detail>` controller
  (`packages/components/ship/skills/src/ship-skill-detail/ship-skill-detail.controller.ts`).
  Typed `LiveSource<unknown>` per
  `ship-sdk.ts:219`; controller narrows to the Rust
  `SkillDetail` shape (`{ id, scope, frontmatter, body, files }`
  from `ship_agents::services::skills::state::SkillDetail`) and
  guards against stale-id races (a payload whose id doesn't
  match the requested id renders the "no such skill"
  empty-state, not a wrong skill).

The list's grouping uses the daemon's `SkillScopeTag` (`project`
/ `user` / `library`) as the section key, with stable display
labels + a fixed display order so the section list is
byte-stable across reads. An unknown scope value (the daemon
adds one in the future) is still rendered, appended after the
known order.

## Data sources evaluated + why NOT a BACKEND-GAP

1. **Connect SDK ops (`ship.skills.list` /
   `ship.skills.detail`)** — both exist. The SDK's full surface
   (see `packages/deck/sdk-client/src/ship-sdk.ts`) includes
   `skills` as a namespace with all three ops the surface needs
   (the third — `ship.skills.summaries` — was deliberately not
   used; see "Why not summaries" below).
2. **The shell's loader (`apps/shell/routes/studio/skills/
   controller.ts`)** — used the same `ship.skills.summaries()`
   op via the `live()` runtime; same Connect surface, just
   different shape.
3. **No HTTP/legacy endpoint** — there is no historical REST
   `/api/v1/skills` to resurrect; the shell moved off REST in
   the Connect cutover (charter §Connect-only).

### Why not summaries (the shell's approach)

The shell's controller reads `ship.skills.summaries({ q, kind,
tag })` — a server-side filtered/aggregated tree grouped by
source. The hub controller reads `ship.skills.list()` directly
because:

- The spec narrows `/studio/skills` to "list the live skill
  list" + `/studio/skills/[id]` for detail. There is no `q` or
  `kind` filter in this pass — adding them would be YAGNI for
  a surface that's just landing.
- The controller owns the grouping, not the daemon. This keeps
  the scope buckets / labels a stable client-side contract
  (the `SCOPE_LABELS` + `SCOPE_ORDER` exports are
  vitest-asserted, see below); if the daemon's `summaries`
  payload shape drifts, the hub surface is unaffected.
- `list()` returns `{ items: SkillIndexEntry[] }` typed; the
  daemon's `SkillIndexEntry` (Rust `IndexEntry`, specta-registered
  in `crates/xtask/src/types.rs`) is the row-level canonical
  shape — `summaries` is a derived projection.

## New pack structure + resolution

- `packages/components/ship/skills/` — `@ship/skills`
  workspace package. Self-declares
  `deck.bundles: [{ name: "ship-skills", entry: "src/index.ts",
  externals: [runtime, dom, sdk-client], lazy: true }]`
  exactly like the existing `@ship/home` / `@ship/studio` /
  `@ship/chrome` packs. No central registry, no copied file,
  no bootstrap script — the bundler's existing per-package
  self-declaration + the `@sui-tag/<tag>` resolver in
  `crates/deck/deck-bundler/src/sui_plugin.rs::resolve_id`
  cover it (the same path the home and studio packs use).
- `src/index.ts` is the umbrella that imports each
  `<tag>.compiled.js` (a virtual id the bundler's `sui_plugin`
  redirects to the sibling `.sui` and compiles in-process —
  see `crates/deck/deck-bundler/src/sui_plugin.rs::load`).
- Two composite tags:
  - `<ship-skill-list>` — pure-derived presenter over the
    `ship.skills.list()` live source. `state` derived
    discriminates `loading | error | empty | ready` from the
    LiveSignal envelope's `kind`; `groups` derives the
    SkillGroup[] the template iterates; `sub` derives
    "N skills" / "1 skill"; `total` derives the count.
  - `<ship-skill-detail>` — pure-derived presenter over the
    `ship.skills.detail({ id })` live source. `state` derived
    discriminates `loading | error | missing | ready`;
    `detail` is the narrowed SkillDetail; `scopeLabel`
    projects the scope; `frontmatterRows` projects the
    free-form frontmatter into stable display rows.
- Both compose existing `sui-*` primitives (`<sui-stack>`,
  `<sui-cluster>`, `<sui-card>`, `<sui-icon>`, `<sui-badge>`,
  `<sui-empty-state>`, `<sui-spinner>`, `<sui-markdown>`).
  No new primitives, no forked layout — per the placement
  hard-rule (charter §Placement).
- The pack's components are reachable in the hub via the
  workspace resolution: `apps/hub/package.json` adds
  `@ship/skills: workspace:*`, the bundler picks up the
  package via the workspace's `component_roots` map.

## Studio route wiring

- `apps/hub/routes/studio/skills/page.sui` — the list index
  route. Composes `<ship-skill-list>` (single tag; the
  composite owns the data story + empty/loading/error states).
- `apps/hub/routes/studio/skills/controller.ts` — empty
  `scope: () => ({})`; the page is a pure composition.
- `apps/hub/routes/studio/skills/[id]/page.sui` — the detail
  route. Composes `<ship-skill-detail prop:id={id()}>` — the
  `[id]` route param flows through `id` in the controller to
  the composite's `id` prop.
- `apps/hub/routes/studio/skills/[id]/controller.ts` —
  reads `route.params.id` and returns it on `Scope` (the
  template's `id()` signal). The full detail logic lives in
  the composite; the route is the prop-binding glue.

The root layout (`apps/hub/routes/_layout.sui` →
`<ship-app-chrome>`) owns the top nav for both routes (closed
in `job/deck-root-layout`).

## Build result

`cargo run -q -p deck-cli -- build --project apps/hub
--bundle-framework` — clean (`_deck  4 files (self-contained)`).
The generated `routes.gen.ts` now carries:

```ts
import * as route3 from './routes/studio/skills/page.sui';
import * as route4 from './routes/studio/skills/[id]/page.sui';
…
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
  { pattern: '/' },
  { pattern: '/studio/components' },
  { pattern: '/studio/components/:name' },
  { pattern: '/studio/skills' },
  { pattern: '/studio/skills/:id' },
]
export const LAYOUTS: LayoutShape[] = [
  { pathPrefix: '/', tag: 'ship-app-chrome' },
]
```

The entry chunk inlines both composites'
`customElements.define()` calls (production bake builds eager
and lazy identically — the `deck.bundles` lazy flag only
gates the optional `DECK_DEV_SKIP_LAZY=1` dev-loop skip,
per `crates/deck/deck-bundler/src/catalog/workspace.rs:47`);
the runtime registers the tags before `mountRouter` runs, so
the page mounts the composites on first paint.

## Tests

21 focused cases for the pack's pure-derived helpers
(`packages/components/ship/skills/test/ship-skills.test.ts` +
`test/helpers.ts`):

- **Grouping** (5 cases) — stable scope order, unknown-scope
  fallback, empty-bucket drop, scope labels, scope order.
- **List narrowing** (`readList`, 6 cases) — non-object
  payloads, non-array items, non-object entries, missing-name
  id-fallback, missing-scope default, missing-description
  default.
- **Detail narrowing** (`readDetail`, 8 cases) — id match,
  stale-id mismatch, non-object payloads, missing id, missing
  frontmatter/body/files defaults, missing-scope default,
  files-array non-string filtering.
- **Frontmatter row projection** (3 cases) — scalars,
  JSON-encode of objects/arrays, insertion order.

`bunx vitest run` in the pack — 21 passed (1 file).

## Authoring rules honored

- `.sui` templates are pure expressions: no `const`/`let`,
  no I/O, no non-deterministic calls. Every render path is
  `state() === 'X'` → section, no per-render mutation.
- Logic in controllers: every derived is a pure function of
  its declared signals. The only mutation is the SDK's
  LiveSignal envelope (loading → ok | error) — driven by the
  Connect SSE stream, not the controller. No `Date.now`,
  `Math.random`, or `fetch` in either controller.
- Tokens only (spacing literals, color literals, radius
  literals all use the design-system `var(--space-N)`,
  `var(--radius-N)`, etc.); the `.card-desc` line-clamp
  fallback uses raw `-webkit-line-clamp: 2` (a legacy CSS
  property name, not a literal value — engine-required
  fallback the primitive-based layout primitives can't
  express; the value is data, not a magic number).
- No literal tags / SDK op names in user-facing copy. The
  error empty-state says "Couldn't load skills" — no
  `ship.skills.list()` in the user-visible string. The
  detail's "no such skill" empty-state shows the requested id
  (which is the user's own input) — no internal path leak.

## Empty/error copy — user-facing

- List loading: `<sui-spinner>` (no text — spinner is its own
  affordance).
- List error: "Couldn't load skills · The skill list couldn't
  be read right now. Reload to try again." — no SDK op name,
  no internal path.
- List empty: "No skills registered · Skills are folders with
  a `SKILL.md` file. Create one in your project or user skills
  directory to see it here."
- Detail loading: `<sui-spinner>` (no text).
- Detail error: "Couldn't load skill · The skill detail
  couldn't be read right now. Reload to try again." — no SDK
  op name.
- Detail missing: "No such skill · No skill with id `'<id>'`
  was found." — only the user's own route param is echoed
  back (the requested id is the user's input, not an internal
  identifier).
- Detail no body: "No body · This skill has no SKILL.md
  content yet."

## Out-of-scope (pre-existing, not introduced by this job)

- `cargo run -p deck-cli -- build-assets` fails on
  `sui-marketing` (TsconfigNotFound on
  `apps/site/src/marketing/marketing-bundle.ts` — `astro`
  package not installed in this worktree). Identical to
  baseline per `.artifacts/bundler-hygiene-notes.md` ("identical
  to baseline, unrelated to this job"). The hub build
  (`cargo run -p deck-cli -- build --project apps/hub`) does
  not require the marketing bundle, so it succeeds.
- `cargo run -p deck-cli -- build-assets` stage that copies
  `.deck/framework/*.js` → `crates/deck/deck-core/assets/`
  failed on `sui-marketing`, leaving the staging dir empty.
  This job copies the four framework bundles manually (one
  `cp` per file) so `deck build --project apps/hub` can run;
  this is a one-time bootstrap mirroring `dispatch.sh` in the
  deck-core build.rs docstring. The next `build-assets` run on
  a clean tree will overwrite these with fresh bundles.
- `cargo test -p deck-bundler --lib` not run from this branch
  — the spec's acceptance is the hub build + the vitest run on
  the new pack. The deck-bundler tests are covered by the
  `job/bundler-hygiene` baseline.
- The shell's `/studio/skills` and `/studio/skills/[id]`
  routes are unchanged — the shell continues to own the editor
  surface for skills (write path). The hub composes only the
  read-only viewing surface.

| Step | Result |
|---|---|
| `cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework` | clean, `_deck 4 files (self-contained)` |
| `bunx vitest run` (in `packages/components/ship/skills/`) | 21 passed (1 file) |
| `routes.gen.ts` registered patterns | `/`, `/studio/components`, `/studio/components/:name`, `/studio/skills`, `/studio/skills/:id` |
| `LAYOUTS` registered | `[{ pathPrefix: '/', tag: 'ship-app-chrome' }]` |
| `<ship-skill-list>` + `<ship-skill-detail>` custom-element registrations | present in the entry chunk |
| Authoring rules | pure expressions, no I/O, no non-determinism, tokens only |
| SDK ops wired | `ship.skills.list()` (list) + `ship.skills.detail({id})` (detail) |
| Backend-gaps | none |
| Design-forks | none |
| Framework-gaps | none |
