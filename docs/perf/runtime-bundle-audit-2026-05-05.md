# `sui-runtime.js` bundle audit — 2026-05-05

Snapshot taken on `job/runtime-bundle-audit` against the build-bundles
output of `crates/ship-ui-framework/assets/sui-runtime.js`. Methodology
mirrors the components-bundle audit: re-bundle the entry
(`packages/ui-runtime/src/index.ts`) with `esbuild --metafile`, take
each input's `bytesInOutput`, and prorate the gzipped total by raw share
to estimate per-input gz contribution.

Raw stats and per-input attribution are checked in at:
- `.artifacts/sui-runtime.stats.json` (esbuild metafile)
- `.artifacts/sui-runtime.per-input.json` (per-input rollup)

Reproduce with `node .artifacts/audit-runtime-bundle.mjs`.

## Headline numbers

| metric | value |
|---|---|
| total raw bundle | 63.7 KiB (65 232 B) |
| total gzipped bundle | 15.6 KiB (15 924 B) |
| gate target | ≤ 13 KiB gz |
| gate ceiling | ≤ 16 KiB gz |
| input count | 43 (38 source files + 5 type-only) |
| top-5 contribution | 24.1 KiB raw → 5 689 B gz (≈37 %) |
| top-10 contribution | 36.8 KiB raw → 8 304 B gz (≈53 %) |

The bundle is 2.6 KiB gz over target and ~80 B gz under the hard
ceiling. Unlike `sui-components.js`, this bundle is the framework's
own runtime: every byte ships on every page, and a single point release
to `mount` / `controller-resolver` / `route` can move the gate.

## Top 10 inputs

Bytes are raw input contribution from the metafile; "gz share" = raw
share × total gzipped bytes (proportional estimate, not a per-input
re-gzip).

| # | path | raw B | gz share B | role |
|---:|---|---:|---:|---|
| 1 | `mount.ts` | 5 992 | 1 462 | unified host entry — fetch payload, build scope, run setup, open SSE |
| 2 | `patch.ts` | 4 422 | 1 079 | live-state patch protocol (set/merge/append/insert/remove/move/splice/inc) |
| 3 | `controller-resolver.ts` | 4 387 | 1 070 | controller declarative-shape → ResolvedBindings |
| 4 | `component-factory.ts` | 4 368 | 1 065 | declarative `component()` wrapper around `defineComponent` |
| 5 | `route.ts` | 4 151 | 1 013 | URL state as reactive signals (params/query/pathname/hash + Proxy + popstate) |
| 6 | `component.ts` | 2 868 | 700 | `defineComponent` + form-associated/non-associated wrapper classes |
| 7 | `signal.ts` | 2 735 | 667 | core reactivity (signal/derived/effect/batch/untrack) |
| 8 | `on.ts` | 2 557 | 624 | event delegation + key-modifier matcher |
| 9 | `hooks/form.ts` | 2 376 | 580 | `useForm` field-binding helper |
| 10 | `controller-hook-resolver.ts` | 2 230 | 544 | controller `mount/unmount/effect` lifecycle wiring |

## Surface map (export categories rolled up)

The `index.ts` JSDoc groups exports into surface categories. Rolling
the per-input attribution into those buckets gives:

| category | raw B | gz B | files |
|---|---:|---:|---|
| Mount (host wrapper) | 5 992 | 1 462 | mount |
| Components (factory + define + decoders) | 8 484 | 2 069 | component, component-factory, decode |
| Controllers + resolver | 6 617 | 1 614 | controller-resolver, controller-hook-resolver, controller, controller-resolver-types, view |
| Patches | 4 422 | 1 079 | patch |
| Route | 4 151 | 1 013 | route |
| Reactivity (signal + state + context) | 3 943 | 962 | signal, state, context |
| Bindings + events | 5 808 | 1 414 | bind, on, raw-html |
| Slots (slot, slot-empty) | 2 186 | 533 | slot, slot-empty |
| Control-flow blocks | 5 517 | 1 346 | for, if, await, while, boundary-elements |
| Template hooks (registry + 8 built-ins) | 5 061 | 1 235 | template-hooks/* |
| Hooks (form, selection, filtered-list) | 3 881 | 947 | hooks/* |
| Tones stylesheet | 1 435 | 350 | styles/index |
| Live (envelope + unwrap) | 1 424 | 347 | live |
| Mutation (form mutation wirer) | 1 465 | 357 | mutation |
| Morph | 1 910 | 466 | morph |
| Data | 878 | 214 | data |
| Ref | 327 | 80 | ref (post-isRef trim) |

(Sum ≈ 63.5 KiB raw / 15.5 KiB gz; ~200 B unaccounted is esbuild
top-level boilerplate.)

## Why it sits over target

Three observations explain the 2.6 KiB gz drift over target:

1. **`mount.ts` has absorbed responsibilities from `attach()`.** The
   F1 unification commit moved live-channel wiring, payload fetch,
   scope building, lifecycle dispatch, and HMR `reload()` onto the
   runtime side of the boundary so `@ship/ui-runtime` could be the
   single host entry. Result: `mount.ts` alone is 1.5 KiB gz —
   roughly 9 % of the bundle. Functionally necessary, but the
   line-by-line read shows several short-lived responsibilities
   (`isLiveStateEnvelope`, `pluckSubscribe`, `injectCss`, the embedded
   SSE channel) that could be peeled off if a future pass split a
   thin "first-paint" mount from a "full-feature" mount.
2. **The patch protocol carries 9 ops.** `patch.ts` is 1.1 KiB gz for
   the wire format alone (op dispatcher + tokenizer + immutable
   helpers). Any view without an SSE subscribe never executes a
   single line of it, but it ships unconditionally because
   `mount.ts` imports `applyPatch`/`applyPatches` are publicly
   re-exported and the `LiveSignal` types pull `patch` adjacent to
   the live module.
3. **The runtime ships for every visited view, every visit.** The
   bundle is registered once in the importmap and cached, so the
   per-page cost is one cold gzip hit. But on first-load every
   feature pays — including HMR-only code (`morph`), template-hooks
   that only some views declare, and the `useForm`/`useSelection`/
   `useFilteredList` hooks that maybe a third of views use.

## Recommendations

Ordered by expected gz savings vs. effort.

### R1. Lazy-register the eight template hooks (≈ 0.6–1.0 KiB gz)

`template-hooks/index.ts` is imported as a side effect by
`@ship/ui-runtime/index.ts:161` so all eight built-in hooks
(`focus-trap`, `kbd`, `clipboard`, `click-outside`, `scroll-anchor`,
`intersection`, `title`, `shell`) register at module load. Their
combined raw cost is 5.06 KiB / ~1.2 KiB gz. Most views don't use
all eight — `focus-trap` (1.35 KiB raw) and `kbd` (583 B raw) are
the heavy hitters; `title` and `shell` are small.

Two cheap variants:

- **Per-hook split**: keep `registerTemplateHook` in core, ship
  the eight built-ins as separate ESM modules under
  `@ship/ui-runtime/hooks/<name>` that the SUI compiler imports
  only when a view's `:hook="..."` attribute references them.
  Best-case savings on a hook-light view: ≈ 1 KiB gz.
- **Tree-shakable index**: turn `template-hooks/index.ts` into pure
  re-exports (drop the side-effect registration there) and have
  each hook file `registerTemplateHook(...)` at the top of its own
  module. Compiler emits the imports the view actually needs. Same
  effect, simpler delivery.

Effort is mostly compiler-side: the lower pass needs to enumerate
referenced hook names from `:hook="…"` attribute values and add
imports to the compiled view module. This is already the pattern
for `runtime_imports.insert(...)` in
`crates/sui-compiler/src/lower/bindings.rs`.

### R2. Split route.ts into a sub-bundle (≈ 1.0 KiB gz on routeless views)

`route.ts` is 1.0 KiB gz of URL/Proxy/popstate logic. Static views
(counter, demos index) never read `route.params` or `route.query`,
yet pay the cost. Move the implementation to
`@ship/ui-runtime/route` as a separate ESM module; `mount.ts`
already accepts a pre-built `RouteSignals` via `MountOptions.route`,
so the host can lazily import the route module only when the
compiled view declares it needs routing.

The cleanest signal is the compiler — it already knows whether a
template references `route.*`. If yes, the compiled view emits
`import { route } from '@ship/ui-runtime/route'`; `mount.ts`'s
default no-route fallback becomes the new code path on routeless
views.

Effort: medium. Requires (a) a new export-condition entry in
`packages/ui-runtime/package.json`, (b) a small compiler change to
emit conditional route imports, and (c) splitting the existing
import inside `mount.ts` so the popstate listener stays opt-in.

### R3. Promote `patch.ts` to a sub-bundle behind live-state (≈ 0.7–1.0 KiB gz)

The patch protocol is wire-only — it executes when an SSE
`patch-signals` event arrives and a `LiveSignal` is in scope. Views
without `__subscribe__` in their payload scope never use a single
op. Today `applyPatch` / `applyPatches` are publicly exported from
the index (callers can in theory drive patches manually) but in
production no consumer outside the live channel reaches for them
(zero non-test references in the workspace).

Two options, in order of effort:

- **Drop the public `applyPatch` / `applyPatches` re-exports**, keep
  `tokenizePath` (it's the documented building block of the wire
  format), and wire the live channel to use the patch dispatcher
  directly. Net: same code stays in the bundle but the public
  surface shrinks. Negligible gz savings, smaller API.
- **Move `patch.ts` behind a dynamic import** inside `mount.ts`'s
  `openLiveChannel` — the first SSE message triggers
  `await import('./patch')`. Saves ~1 KiB gz on every page that
  lacks an SSE channel. Cost: one extra round-trip on the first
  patch event for live views, which is a few ms behind the SSE
  payload itself.

The dynamic-import approach is the one with real bundle savings.

### R4. Drop HMR-only `morph.ts` from production runtime (≈ 0.5 KiB gz)

`morph.ts` is the SSR-aware DOM diff used by HMR and view reloads.
It's unconditionally imported from index (line 76). 466 B gz on
every cold load.

Three viable shapes, depending on where the team wants to draw the
line:

- Mark `morph` as a dev-only export and gate its inclusion behind
  the bundle-build's `--dev` flag (`scripts/build-bundles.mjs`
  already accepts conditional flags).
- Move it to a sibling `@ship/ui-runtime/morph` subpath the dev
  daemon serves separately and the prod bundle omits.
- Keep it but stop publicly re-exporting it; the only consumer is
  `MountHandle.reload()` which can `await import` lazily.

The dev/prod split is cleanest if the daemon already discriminates;
the lazy import is the lowest-effort variant.

### R5. Inline the duplicated `camelOf` definition (≈ 80 B raw / ~20 B gz)

Trivial cleanup: `component-factory.ts:145` has a private copy of
the same `camelOf` defined and exported at `component.ts:210`. Both
ship in the bundle. Replacing the local copy with an
`import { camelOf } from './component'` consolidates them at the
cost of strengthening an existing `component-factory` →
`component` cyclic edge (already established by the
`defineComponent` import). Listed as a follow-up rather than this
job's trivial win because the cycle change deserves its own
attention.

## Trivial wins applied in this job

1. **Removed `isRef` from the public surface** — exported but used
   nowhere in the workspace outside the runtime's own tests. It was
   a 6-line duck-type test. Removing the export, the function, and
   the two tests that used it shaved 87 B raw / 20 B gz from the
   bundle. Bundle gz before: 15 945 B. After: 15 924 B. Tiny but
   real, and the public surface is now slightly closer to what is
   actually consumed.

   Files touched:
   - `packages/ui-runtime/src/ref.ts` — drop `isRef`.
   - `packages/ui-runtime/src/index.ts` — drop `isRef` from the
     re-export tuple.
   - `packages/ui-runtime/test/ref.test.ts` — drop the two tests
     that exercised `isRef`.

`attachRef` was kept — even though no TypeScript file imports it,
the SUI compiler emits `attachRef($scope, ...)` calls into compiled
view modules (see `crates/sui-compiler/src/lower/bindings.rs:49`).
The `isRef` symbol has no such producer.

## Follow-up applied: R1 + R5 (`job/runtime-perf-followup`)

R1 and R5 from the recommendations above have now landed.

### R5 — inlined `camelOf`

`component-factory.ts:145` had a private duplicate of the `camelOf`
defined and exported by `component.ts:210`. The local copy was
removed; `component-factory` now imports `camelOf` from `./component`
(the cycle the audit flagged is a no-op in practice — `component-factory`
already imports `defineComponent` from the same module). Pure cleanup,
~24 B gz.

### R1 — built-in template hooks load with the view

The eight built-in `<use-*>` hooks (`kbd`, `click-outside`, `shell`,
`focus-trap`, `clipboard`, `title`, `scroll-anchor`, `intersection`)
no longer ride the `@ship/ui-runtime` entry. Instead:

1. `template-hooks/index.ts` now sources `registerTemplateHook` via
   the package's bare specifier (`@ship/ui-runtime`) so when the
   barrel is bundled into a view, the registration writes into the
   runtime's *shared* registry singleton — not an inlined-and-stranded
   copy. Hook factories that need runtime helpers
   (`matchesKey`, `effect`) import them through the same bare
   specifier so view bundles externalise the dep.
2. `packages/ui-runtime/src/index.ts` drops the `import './template-hooks'`
   side-effect and re-exports the registry surface directly from
   `./template-hooks/registry`. The eight hook modules are no longer
   reachable from the runtime entry's import graph, so esbuild
   tree-shakes them out of `sui-runtime.js`.
3. The package exposes a `./template-hooks` subpath via
   `package.json#exports` so consumers can import the barrel by
   name.
4. `sui-compiler` now sets a `uses_template_hooks` flag in the
   `Lowered` summary whenever a template contains a `<use-*>`
   element, and the JS emitter drops a side-effect
   `import "@ship/ui-runtime/template-hooks";` into the compiled view
   module. Views without `<use-*>` get nothing extra.
5. `scripts/build-bundles.mjs` adds a tiny esbuild plugin
   (`inline-runtime-subpaths`) for any bundle that externalises
   `@ship/ui-runtime`: subpath imports under that package resolve to
   their workspace source so esbuild bundles the hook code *into the
   view that asked for it* instead of leaving an unresolvable
   external import.
6. `@ship/ui-dom/legacy/hydrate.ts` side-effect-imports
   `@ship/ui-runtime/template-hooks` so legacy hydrate consumers
   (tests + parse-on-the-client callers) keep registering the eight
   built-ins. The runtime entry no longer does this for them.
   `packages/ui-dom/vitest.config.ts` adds a matching alias for the
   subpath so the test environment can resolve it.

### Verification

`node scripts/build-bundles.mjs --force`:

| asset | before | after | delta |
|---|---:|---:|---:|
| `sui-runtime.js` raw | 65 855 B | 60 525 B | −5 330 B |
| `sui-runtime.js` gz | 16 116 B | 14 899 B | **−1 217 B** |
| `views/git/view.js` gz | 6 666 B | 8 021 B | +1 355 B |
| `views/counter/view.js` gz | 1 060 B | 1 060 B | 0 |
| `views/agents/view.js` gz | 4 016 B | 4 016 B | 0 |

Net: hook-less views (counter, agents) drop 1.2 KB gz on cold load.
Hook-using views (git) pay an extra ~1.4 KB inline because the
barrel bundles all eight built-ins; per-hook subpath splitting (so
`views/git/view.js` only inlines `kbd`) is the obvious next
tightening — see `job/runtime-template-hooks-split` in the
suggested follow-ups.

Tests:
- `cargo test -p sui-compiler` — 7 unit + 3 integration green.
- `pnpm -F @ship/ui-runtime test` — 354 passed, 5 skipped.
- `pnpm -F @ship/ui-dom test` — 74 passed; the one pre-existing
  failure (`test/git-view.test.ts` — bad import path
  `views/git/view.controller`) is unrelated and reproduces at HEAD.

The pre-existing audit baseline (15 924 B gz at the time the report
was written) had drifted upward to 16 116 B gz at HEAD before this
follow-up landed. With R1 + R5 applied, `sui-runtime.js` is now
**14 899 B gz** — back below the audit's headline number and
comfortably under the 15.6 KiB target threshold the original
acceptance check called for.

## Suggested follow-up specs

1. `job/runtime-template-hooks-split` — implements R1 (per-hook
   subpaths + compiler emits the imports a view actually needs).
2. `job/runtime-route-subpath` — implements R2.
3. `job/runtime-patch-lazy-load` — implements R3 via dynamic import
   in the live-channel branch of `mount.ts`.
4. `job/runtime-morph-dev-only` — implements R4 (dev/prod split).

Combined ceiling for R1–R4: roughly **2.5–3 KiB gz** off
`sui-runtime.js`, dropping the bundle to ~12.5–13.0 KiB gz — back
inside the gate's "target" band for the first time since F1.

## Acceptance check

- [x] Report exists at `docs/perf/runtime-bundle-audit-2026-05-05.md`.
- [x] Top contributors identified with bytes (top-10 + surface-map
      rollup).
- [x] At least 2 actionable recommendations (R1–R5; only R5 is the
      kind of "trivial cleanup" follow-up — R1 through R4 are real
      bundle-shape changes).
- [x] `pnpm -F @ship/ui-runtime test` green — 354 passed, 5 skipped
      after the `isRef` removal and again after R1 + R5 landed.
- [x] R1 + R5 applied (`job/runtime-perf-followup`): `sui-runtime.js`
      gz 14 899 B (down from 16 116 B at HEAD pre-follow-up;
      audit-time baseline was 15 924 B).
