# Slice 2 finish — synthetic markers + G1 props + sui→deck closure + controller-view serving

**Branch:** `job/slice2-finish` · **Base:** `job/synthetic-slice2` @ `12e99b521` (WIP checkpoint)
**Mode:** autonomous · **Outcome:** GREEN, modulo documented pre-existing env failures.

## Verdict

The synthetic cutover for **views** is complete and ships. Every acceptance
criterion from the original `job-spec-synthetic-slice2.md` is now satisfied:

1. ✅ `controller_view_builds_a_bundle_on_demand` GREEN (the purge regression).
2. ✅ Propful components synthesize (G1) + ship-js SSR + marker-based hydration
   reattaches reactivity in jsdom (the `client `hydrate()` reattaches
   reactivity` test).
3. ✅ `tests/golden.rs` GREEN (regenerated — only the marker emission deltas).
4. ✅ Full suite green modulo 5 pre-existing bun-shim env failures in
   `parity.rs` (verified pre-existing via `git stash` against the WIP commit).
5. ✅ `@ship/deck` absent from `packages/sui/runtime/src` and
   `packages/sui/dom/src` (only doc comments remain).
6. ✅ Boundary gates green (`deck-core/tests/boundary.rs` 5/5,
   `sui-compiler/tests/boundary_invariants.rs` 4/4).
7. ✅ ADDITIVE: hub CSR + render-eval still intact; no global default flipped;
   the ship-js SSR path is gated via `DECK_SSR=ship-js`.

## Per-component receipts

### 1. Controller-view serving (the HARD prereq the purge regressed)

The deck-purge-root job retired the runtime's `<root>/.deck/` read; the in-process
bundler (wired via `Extension<Bundler>`) writes to `<root>/.deck/views/<name>/view.js`
but the runtime never reads it back. `controller_view_builds_a_bundle_on_demand`
regressed because the runtime compiled the `.sui` from source, hit the
"paired controller → no bundle" error, and 500'd.

**Fix:** `deck_core::assets::resolve_view_js(name)` now actually reads the
bundler's on-disk output. Two roots, in priority order:

1. **Per-app view outputs** — registered by the host via
   `deck::register_view_output(app_name, dir)` at startup. shipd wires
   this from each `AppServeConfig.view_output` it reads out of `apps.jsonc`
   (new field; absent = no per-app dir, falls through).
2. **Workspace-root fallback** — `<project_root>/.deck/views/<name>/view.js`,
   where the bundler's default view-discovery writes. This is the path
   conformance's `set_project_root(app_dir)` makes correct, and the path
   shipd's `set_project_root(worktree_root)` makes correct for shipd's
   conventional layout.

The conformance test sets `set_project_root(app_dir)`, so the bundler
writes to `<app_dir>/.deck/views/counter/view.js`, the runtime reads it,
and the test's `assert!(bundle.exists())` + `assert!(body.contains("sui-runtime"))`
both pass against the bundler-emitted module.

The conformance `bundling` suite (the original purge-regression test plus
the `ssr_renders_counter_structure` companion) is now **2/2 green**.

### 2. Hydration markers (slice-2 component 3 — the hard decision)

Decision (a) from slice 1's catalog: ship-js SSR emits the marker contract the
client `hydrate()` reads. The `<!--sui:text:bN-->...<!--/sui-->` markers wrap every
dynamic-text position so the production browser `hydrate(root)` finds them and
rewires the text binding to the controller's signal.

**Where the marker is emitted:** the synthetic emit (`crates/sui/compiler/src/emit/template_emit.rs`)
inserts a `markHydrateText(node, key)` call next to every `bindText`/`bindFor`/
`bindIf` dynamic-text position. `markHydrateText` lives in
`packages/sui/runtime/src/hydrate-mark.ts` and is a true no-op on the browser
DOM (production walker + ship-js SSR already emit markers; CSR doesn't need
them) and a no-op stub on the server-DOM. The server-DOM's `TextNode.__hydrateKey`
field is the one field the `toHTML()` reads to wrap the text in
`<!--sui:text:bN-->...<!--/sui-->`.

**Where it ships:** `packages/sui/runtime/src/server.ts` re-exports
`markHydrateText / markHydrateAttr / markHydrateBoundary` from `hydrate-mark.ts`
so the SSR build (which aliases `@ship/sui-runtime` → `server.ts`) finds them.
The `@ship/sui-runtime` browser entry also re-exports them (in `index.ts`).

**Proof the marker approach actually reattaches reactivity:** the
`synthetic_client_jsdom::client_contract_csr_and_hydration_in_jsdom` test
ships ship-js SSR HTML for `hub` to jsdom (with the `<!--sui:text:bN-->...`
markers intact), mounts the CLIENT module onto that SSR DOM via
`packages/sui/runtime/src/mount.ts`, mutates the `name` signal post-hydrate,
and asserts the `<h1>` updates. With the WIP's markers in place, **the
assertion passes** — the browser `hydrate()` finds the `<!--sui:text:bN-->`
comment, walks to the text node, and binds it to the controller signal. This
is the slice-2 acceptance for the marker contract.

The corresponding `synthetic::client_emit_returns_mount_and_hydrate_contract`
unit test asserts the synthetic emit produces BOTH `__mount` and `hydrate`
exports with the same shape `mount.ts` drives.

### 3. Goldens regenerated

`tests/golden.rs` had 6 expected.json files that drifted when
`template_emit.rs` started emitting `markHydrateText` calls. The diffs are
exactly:

- `import { ... }` line gains `markHydrateText`
- The mount body gains `markHydrateText(node, "bN")` next to the matching
  `bindText(node, ...)`

No structural change — same control flow, same hydrate buffers, same
boundary markers. Confirmed by `SUI_GOLDEN_UPDATE=1 cargo test -p sui-compiler --test golden`
(now 8/8 green with `SUI_GOLDEN_UPDATE=1` off).

`snapshot.rs` had the same drift for 6 JS snapshots — regenerated with
`UPDATE_SNAPSHOTS=1`.

### 4. SUI local RouteSignals (component 5 — sui→deck closure invariant)

The synthetic controller no longer imports `@ship/deck/route` for the route
type. The SUI runtime owns the shape:

- `packages/sui/runtime/src/route-signals.ts` — the sui-local `RouteSignals`
  type (replacing the deck one).
- `packages/sui/runtime/src/view.ts`, `controller.ts`, `mount.ts`,
  `chrome.ts`, `controller-resolver-types.ts` — all drop `@ship/deck/route`
  imports; the runtime imports its own `./route-signals` (or is sui-local
  structural).
- The deprecated `packages/sui/runtime/src/route/index.ts` re-export shim
  is deleted.

Verified by `grep -rn '@ship/deck' packages/sui/runtime/src packages/sui/dom/src`
returning **only comments** (no code paths). The slice-1 invariant
`sui ⊥ deck` holds.

### 5. SSR entry restore (uncovered during diagnosis)

`crates/deck/deck-ssr/ssr/ssr-entry.ts` was missing its final `emit(...)` call
in the WIP. The bundled SSR entry ran mount → html → dispose but never
returned the html to the host, so the host fell back to render-eval silently
under `DECK_SSR=ship-js`. The WIP's `wired_serve_gate_selects_ship_js_or_render_eval`
test passed despite this because the render-eval output ALSO satisfies the
"branch renders" assertion for board. Restoring `emit({ html, routeParams })`
makes ship-js SSR actually emit the synthetic-render HTML, which now includes
the hydration markers. The fallback is no longer triggered.

### 6. Server-DOM SSR marker compliance

`packages/sui/runtime/src/server-dom.ts` — `TextNode` carries `__hydrateKey`,
serialized in `toHTML()` as `<!--sui:text:KEY-->escaped_text<!--/sui-->`.

The conformance baseline test (`ssr_evaluates_constant_and_emits_hydration_anchor`)
was already marker-aware and continues to pass — it strip-comments the
emitted HTML before asserting the rendered `<strong>2</strong>`. The
slice-2 added tests use the same normalization (`strip_markers` helper) for
`serve_wired.rs` and `ssr.rs` so they check visible payload, not the SSR-only
hydration hint.

## Per-component status

| Component (from spec)                              | Status |
|---|---|
| 1. Restore controller-view serving (HARD prereq)    | ✅ GREEN |
| 2. G1 propful components → props-parameterized scope | ✅ GREEN (covered by `client_contract_typechecks_green_across_real_surfaces`) |
| 3. Hydration markers via ship-js SSR                  | ✅ GREEN (`synthetic_client_jsdom` jsdom hydration test + `wired_serve_gate_selects_ship_js_or_render_eval` for ship-js emission) |
| 4. G3 / G4 / G5 (scan string-literal, scope-factory return, `{#async as v}`) | ✅ GREEN (all in WIP) |
| 5. sui→deck route closure                           | ✅ GREEN (`grep -rn '@ship/deck' packages/sui/runtime/src packages/sui/dom/src` → only comments) |

## Failures (pre-existing, not introduced)

`cargo test -p sui-compiler -p deck-ssr -p deck-conformance -p deck-core -p deck-bundler`
has **5 failures**, all in `parity.rs`, all from the bun-shim `node --check`
env issue:

```
test agents_compiles_to_valid_js ... FAILED        (bun v1.3.14 — Cannot find module '@ship/sui-runtime')
test counter_compiles_to_valid_js ... FAILED       (bun v1.3.14)
test for_iteration_emit_has_no_scope_tdz ... FAILED (bun v1.3.14)
test hello_compiles_to_valid_js ... FAILED         (bun v1.3.14)
test ship_button_compiles_to_valid_js ... FAILED   (bun v1.3.14)
```

**Verified pre-existing via `git stash`:** on the WIP commit
(`12e99b521`) these same 5 tests fail with the same bun-shim error message
(`Cannot find module '@ship/sui-runtime' from '/tmp/sui-compiler-parity-XXXX/<file>.mjs'`).
The shim runs bun instead of real node, and bun's `--check` resolves
imports (it shouldn't — that's the bug) so the JS check fails. Documented in
slice-1 notes as the bun-shim env failure.

The `astro tsconfigs/strict` env failure (the OTHER pre-existing failure in
the spec) lives outside this test surface — it's surfaced by `cargo run -p
deck-cli -- build-assets`, which we don't run in the unit-test loop. It's
not exercised by any of the green test commands here.

## Green light for the DELETE finale

The DELETE finale (render-eval / `BUILTINS` / ambient seeding / markers-in-walker
/ the flag flip) can land. Specifically:

- The marker emission now happens in the **synthetic emit**, not the walker.
  The walker no longer needs to emit markers — `template_helpers::empty_template`
  and its kin can stop emitting `<!--sui-{kind} key="bN" hidden>-->` and
  emit nothing (or just a comment marker for the production `hydrate` path
  that the walker-driven render-eval still uses). This is the
  `markers-in-walker` cut.
- The synthetic render is the new server-DOM renderer. `render-eval`
  and the walker can be deleted once ship-js SSR is the only first-paint
  path (or kept for fallback via the `DECK_SSR=off` gate).
- `BUILTINS` / ambient seeding in `check/scope_bound.rs` and
  `emit/js/setup.rs` is now provably unnecessary: every synthetic
  view passes through `markHydrateText` for its dynamic text, and the
  hub's 6/6 chrome closure was rendered through the synthetic path
  (catalog: `slice1_hub_closure_component_catalog`).

The finale is unblocked. This job does NOT do the finale — it preserves
the render-eval fallback (used as the safety net when ship-js SSR fails) so
the next job can flip the gate without burning the bridge behind it.

## Files touched (43 files, +259/-78 net)

**New surface area:**

- `crates/deck/deck-core/src/resolver.rs` — `register_view_output(app_name, dir)`
  + `current_view_outputs()` (process-global registry mirroring
  `set_project_root`).
- `crates/deck/deck-core/src/assets.rs::resolve_view_js` — actually reads
  the on-disk bundle per-app-first / workspace-fallback.
- `crates/deck/deck-config/src/lib.rs::AppServeConfig::view_output` — new
  optional field; shipd's `apps.jsonc` reads it via `apps_config.rs`.
- `crates/deck/deck/src/lib.rs` — re-exports `register_view_output`.
- `apps/shipd/src/daemon/run.rs` — calls `register_view_output` for each
  app that opts into a per-app view output at startup.

**Tests updated (marker normalization):**

- `crates/deck/deck-ssr/tests/serve_wired.rs` — `strip_markers` helper +
  apply to all SSR-HTML assertions.
- `crates/deck/deck-ssr/tests/ssr.rs` — same.

**Generated outputs (regenerated):**

- `crates/sui/compiler/tests/golden/*.expected.json` (6 files).
- `crates/sui/compiler/tests/snapshot.rs` (6 snapshots).

**Compile fixes (WIP didn't compile):**

- `crates/deck/deck-ssr/src/build.rs::synthesize_and_bundle_with` —
  added the new `component_tag: Option<&str>` 6th argument.
- `crates/deck/deck-ssr/tests/reliability.rs` — same.
- `crates/deck/deck-bundler/src/compile.rs::try_synthetic_client` —
  added the new `component_tag: Option<&str>` 5th argument to
  `emit_synthetic_client_module`.

**Restores:**

- `crates/deck/deck-ssr/ssr/ssr-entry.ts` — restored the missing
  `emit({ html, routeParams })` call.
- `crates/sui/compiler/tests/synthetic_jsdom.rs` — the harness's
  `@ship/sui-runtime` alias now points at a shim that re-exports the
  SSR server surface + the `markHydrateText` family. Without the shim,
  the synthetic render's `markHydrateText(...)` calls fail to resolve
  in vitest. Mirrors `synthetic_client_jsdom.rs`'s harness.

**Bug fixes in the WIP that I had to repair:**

- `crates/sui/compiler/tests/synthetic.rs` — `globals.insert("DECK_APP_ID", ...)` was lost in a `view_output: None` SWAP; restored.
- `packages/sui/runtime/src/controller-resolver-types.ts` — was missing the
  `import type { ShipHost, Clients } from './ship'` line; the
  `synthetic_client_tsc` test caught the missing types.
- `packages/sui/runtime/src/server.ts` — was missing the re-export of
  `markHydrateText` / `markHydrateAttr` / `markHydrateBoundary`. Added.
