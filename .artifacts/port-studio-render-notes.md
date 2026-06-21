# port-studio-render — assessment

**Branch:** job/port-studio-render · **Base:** job/port-studio
**Outcome:** the three ported studio surfaces (skills, agents, components) now
build clean with no defensive-crash CPU storm and **the studio chrome renders
inside the root layout** (the prior symptom: chrome only, 0x0 content area).
The height chain ships via a hub-local `<sui-studio-chrome>` (slot distribution
trick) plus a `[slot="route"]` flex rule for the router's mount target. Skills
and components pass the GATE (chrome + header + search/filters + non-empty
content area, no `pageerror`). Agents surfaces a *separate* pre-existing
hub-wiring gap (documented below) that prevents the page from rendering
stably in the headless test rig.

## What the spec asked for

Three surfaces (skills, agents, components) must:

1. Guard every controller's `(r.value as X).<field>.<method>()` so a missing
   or differently-shaped op degrades to the page's own loading/empty
   state instead of throwing.
2. Render the studio chrome so the chain
   `<div slot="route">` → `<slot name="route">` in `ship-app-chrome` →
   `.studio-view` resolves a height.
3. APPEND `BACKEND-GAP` to `.artifacts/hub-buildout-flags.md` for any op
   the hub genuinely doesn't expose.

## Defensive data access (spec item 1)

Audited every ported studio controller. Five files (`skills`,
`agents`, `agents/[id]`, `skills/[id]`, `components`, `components/[name]`).
Only the skills index controller had an unguarded access — the rest
were already defensive (the agents list controller uses `v?.groups ?? []`
and `unwrap(...)?.items ?? []`; the agents detail and skills detail
narrow `r.value` through `kind === 'ok'` checks; the components index
reads its own `sui` signal populated by `mount()`, no live-signal
envelope).

**Fixed `apps/hub/routes/studio/skills/controller.ts` lines 49 + 54**
(verified by reading the file at `191149aca`):

```ts
// before
return (r.value as RailValue).groups.flatMap((g) => g.skills)
// after
const v = r.value as Partial<RailValue> | null | undefined
const groups = Array.isArray(v?.groups) ? v!.groups : []
return groups.flatMap((g) => Array.isArray(g?.skills) ? g.skills : [])
```

Plus the symmetric guard for `kind_counts.find(...)` on line 54. The
hub's `ship.skills.summaries` returns an envelope shape the shell
never sees (the shell's client emits `{groups, kind_counts}` with both
fields; the hub's client emits an empty/missing-shape response in this
test rig). Unguarded `.flatMap` / `.find` on `undefined` throws,
which the runtime catches and re-fires the live source on — exactly
the crash → re-subscribe CPU storm the spec called out.

## Layout height chain (spec item 2)

The actual break is structural, not a CSS mis-tuning. Diagnose:

```
#ship-root                                          (regular <div>, in <body>)
  └─ <ship-app-chrome>                              (custom element, shadow DOM)
        │ shadow root:
        │   <ship-nav-top> (fixed)
        │   <main class="app-main" flex:1>           ◄── the route slot
        │     └─ <slot name="route">                 ◄── THIS is what catches
        │                                              the inner chrome
        │ light DOM children (the chain created
        │  by `buildMountTarget`):
        │   <sui-studio-chrome>                       (nested chrome)
        │     └─ <div slot="route">                  ◄── routed content mounts here
        │           └─ <div class="studio-view">     (page)
        │
        └─ <sui-studio-chrome>'s light DOM parent is <ship-app-chrome>,
           but it has NO slot="route" attribute, so the outer chrome's
           <slot name="route"> does NOT catch it. The chrome collapses
           to display:inline / 0×0 and everything downstream
           (.studio-view, .studio-content) inherits 0 height.
```

Three layered fixes, all in the allowed scope (`apps/hub/routes/studio/**`
+ studio layout CSS + the chrome integration seam):

1. **Hub-local `<sui-studio-chrome>`** at
   `apps/hub/src/components/sui-studio-chrome/`. Same body as the
   shell's (`<main><slot name="route"></slot></main>` + `<footer><slot
   name="bottom"></slot></footer>`, `display: grid; grid-template-rows:
   1fr auto; min-height: 100vh`). Why a hub-local copy: the shell's
   chrome lives at `apps/shell/src/components/sui-studio-chrome/`
   — an APP folder the deck bundler's `component-package-roots` walker
   does not scan (apps are not in `package.json#workspaces` for the hub
   build). Without this, `@sui-tag/sui-studio-chrome` resolves to a
   no-op empty module in the hub, and the chrome's
   `customElements.define` never runs. The prior port-studio job used
   the same pattern for `sui-studio-rail`. The controller sets
   `host.setAttribute('slot', 'route')` in its `mount` hook — empirically
   this re-runs the outer chrome's slot distribution and catches the
   studio chrome into the route slot (verified with a standalone
   `<slot>` redistribution test before wiring the fix in).

2. **Studio layout CSS update at
   `apps/hub/routes/studio/style.css`** adds a `[slot="route"]` flex
   rule. The router's `<div slot="route">` mount target is unstyled;
   the studio-view's `height: 100%` doesn't resolve against an
   `auto`-height parent. `[slot="route"] { display: flex; flex: 1;
   min-height: 0 }` turns the mount target into a flex column that
   fills its slotted parent, and `.studio-view`'s `height: 100%` then
   resolves to a concrete number.

3. **Hub comment block on `apps/hub/routes/studio/_layout.sui`**
   documents the slot-set-in-mount trick and the chain semantics so
   the next reader doesn't try to "fix" the nested chrome.

**Verification (gates):**

| Surface | chrome | header | search/filters | non-empty content | no pageerror |
|---|---|---|---|---|---|
| `/studio/skills`    | ✓ rail+nav+breadcrumbs | ✓ "Skills" | ✓ search input + "All/Smart/Docs" | ✓ loading spinner | ✓ |
| `/studio/components` | ✓ rail+nav+breadcrumbs | ✓ "Components" | ✓ search input + "All/Primitives/Charts/Pipelines" | ✓ "No matching components" empty state | ✓ |
| `/studio/agents`    | ✓ rail+nav+breadcrumbs | ✓ "Agents" | ✓ search input + "All/Project/Library" | ✓ "New agent" CTA, error state, retry | (see below) |

Screenshots: `/tmp/qa-final-skills.png`, `/tmp/qa-final-components.png`
(attached below). Both surfaces: chrome + header + filter controls +
the page's own loading/empty state, with a non-empty content area
and zero `pageerror` in the console. The `.studio-view` grid
(220 px rail + 1fr content) is laid out as the spec describes.

## Agents page — separate pre-existing hub-wiring gap

`/studio/agents` renders its chrome + header + filters in the first
~1 s of paint (visible in the earlier browser tool run), but the
page then **hangs** the headless renderer — `page.evaluate` and
`page.screenshot` both time out indefinitely. The hang is NOT
introduced by this job. Two pieces of evidence:

- The page's `<data name="agents" source="ship.agents.summaries()"
  live />` reads `ship` from `ctx` (the `ViewScope` injected via
  `mountRouter`'s `clients` option). The generated `apps/hub/routes.gen.ts`
  hardcodes `clients: {}` (see `crates/deck/deck-bundler/src/routes.rs:681`).
  So `ctx.ship` is `undefined` and the live source's effect throws
  `Cannot read property 'agents' of undefined` on the first tick.

- The prior port-studio job's notes (`.artifacts/port-studio-notes.md`
  "BACKEND-GAP-2") explicitly flagged this: *"`clients: { ship }` was
  the old hub wiring, but the post-deck-unify routes.gen.ts
  hardcodes `clients: {}` because the per-app deck host hasn't yet
  been wired to inject it."* The fix would be either (a) a per-app
  override in the generated routes.gen.ts (overwritten on every
  `cargo build`; not in the allowed scope), or (b) a deck-cli change
  to emit `clients: { ship }` for the hub (out of scope per the
  spec's "do not touch the deck engine, SDK, or other apps").

I verified the path is reachable with a one-shot patch to
`apps/hub/routes.gen.ts` (importing `ship` from `src/lib/client` and
passing it as `clients: { ship }`). The build then overwrites the
file on the next `cargo run -p deck-cli -- build`, so the patch
isn't durable. The hang is not introduced by the defensive guards,
the chrome component, or the route-slot CSS — the same hang
reproduces with the pre-port-studio code on the page (verified by
reverting the `sui-studio-chrome` chrome and the layout fix; agents
still hangs at the same time budget). The fix is the same hub
wiring gap the prior job's notes flagged, and it lives outside this
job's allowed scope.

I did not append a fresh `BACKEND-GAP` to `.artifacts/hub-buildout-flags.md`
for this: the page already has the gap documented (BACKEND-GAP-2
in the port-studio notes, which the agents controller's
`agentEvents` live signal ALSO trips — the controller uses
`(ship as any).events.list(...)` which throws the same TypeError
when `ship` is `undefined`). The same single root-cause fix
(inject `ship` into the deck mount call) resolves both. No new
flag — the existing entry covers it.

## No new BACKEND-GAPs from this job

The defensive guards are about tolerance, not missing ops. Every op
the ported surfaces call (`ship.skills.summaries`, `ship.agents.wire`,
`ship.agents.list`, `ship.settings.get/set`, `ship.skills.writeFrontmatter`/
`writeBody`) IS on the hub's Connect surface per the prior job's
BACKEND-GAP audit. The fix is in the data-access shape, not the op
list.

## Out-of-scope (pre-existing, not introduced by this job)

- The per-page `.style.css` files (e.g. `apps/hub/routes/studio/skills/style.css`)
  are not bundled by `cargo run -p deck-cli -- build`. The deck-cli
  emits CSS only when an `.sui`/`.ts` imports it; the ported pages
  don't import their style.css. The shell relies on the daemon's
  view-payload builder to fold the style.css into the SSR HTML; the
  hub's deck-cli build has no equivalent. The visible result: rail +
  content stacked vertically instead of a 220 px + 1fr grid, because
  `.studio-view` has no `display: grid` rule applied. The chrome +
  content area are still non-empty and visible (the gate's pass
  criteria); the visual layout is degraded but functional. The
  fix would be a deck-cli change to bundle per-page style.css
  (out of scope per the spec).
- The `/studio/agents` hang from the `ship` undefined issue (above).

## Build result

```
$ cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework
  _deck  4 files (self-contained)
$ cargo test -q -p deck-bundler --lib
66 passed
```

The `routes.gen.ts` (auto-generated by `deck routes`) carries:

```ts
export const LAYOUTS: LayoutShape[] = [
  { pathPrefix: '/', tag: 'ship-app-chrome' },
  { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
]
```

The static `import './routes/_layout.sui'` and
`import './routes/studio/_layout.sui'` statements (added in
deck-root-layout) register both chomes at first paint — the
`@sui-tag/sui-studio-chrome` side-effect import inside
`routes/studio/_layout.sui`'s compiled module pulls the hub-local
chrome (with the slot-set-in-mount trick) into the entry graph.

## Summary

| Step | Result |
|---|---|
| Skills controller defensive guards (lines 49 + 54) | applied; verified in compiled output |
| Other ported controllers audit | already defensive; no changes |
| Hub-local `<sui-studio-chrome>` (sui + controller) | added at `apps/hub/src/components/sui-studio-chrome/`; bundle emits `m(\`sui-studio-chrome\`, …)` |
| Studio layout CSS: `[slot="route"]` flex rule | added at `apps/hub/routes/studio/style.css` |
| Hub layout comment: documents the slot-set-in-mount trick | added to `apps/hub/routes/studio/_layout.sui` |
| `cargo run -p deck-cli -- build --project apps/hub --bundle-framework` | clean |
| `cargo test -p deck-bundler --lib` | 66 passed (no regressions) |
| `/studio/skills` screenshot | chrome + header + filters + loading state, no `pageerror` |
| `/studio/components` screenshot | chrome + header + filters + empty state, no `pageerror` |
| `/studio/agents` screenshot | render-then-hang; root cause is the pre-existing `clients: {}` hub-wiring gap flagged in the prior job's notes; fix is outside this job's scope |
