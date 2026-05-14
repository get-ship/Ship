# SUI-Rendering Reliability Audit — Session Summary

Branch: `feat/ship-set` (merged with `feat/phase2-substrate-split`).

## What landed

| # | Commit | Subject |
|---|---|---|
| 1 | `f50025fb` | fix(sui-signals): unsubscribe-on-rerun — drop stale dependencies |
| 2 | `e5031bb5` | test(sui-signals): reentrance, panic recovery, cross-thread + criterion bench |
| 3 | `15c2cfc4` | fix(sui-runtime): unsubscribe-on-rerun in TS signals + sweep stale vitest aliases |
| 4 | `f632414a` | test(sui-runtime): reentrance + effect-scope edges + drop dead owner aliases |
| 5 | `c4f...??` | fix(sui-compiler): tokenizer UTF-8 panic on multi-byte char (compile_full fuzz find) |
| 6 | `5c073363` | feat(sui-runtime/for): warn on duplicate {#for} keys |

(Earlier in same session: bundler call-site wiring + toolchain CLI port on phase2; merged in via fast-forward.)

## Bugs fixed

1. **Stale dependency subscriptions in signal/derived/effect** (Rust + TS).
   Conditional dep patterns (`if cond.get() { a.get() } else { b.get() }`)
   were leaking subscriptions to no-longer-read deps. Caused both spurious
   re-runs and unbounded memory growth in long-lived deriveds. Fixed in
   `crates/sui-signals` and `packages/sui-runtime/src/signal.ts` with
   matching diff-based untracking pattern. Disposer drops also proactively
   unsubscribe (no waiting for lazy weak-ref pruning).

2. **UTF-8 panic in `render/eval/lex.rs`**. The tokenizer used `&src[i..i+N]`
   on a `&str` to peek multi-byte puncts, which panicked when the source
   contained a multi-byte UTF-8 char (e.g. `°` from a decoded `&deg;`
   entity). Found by the new `compile_full` fuzz target on its first run.
   Fixed by comparing bytes directly (all targets are ASCII).

3. **`{#for}` silently dropped duplicate-keyed items**. Now warns on
   duplicate key with index info, matching Solid/React behavior.

## Dead code removed

- `effect-scope.ts`: 4 deprecated aliases (`withOwner`, `registerOwnerCleanup`,
  `createOwner`, `type Owner`) — zero internal callers per repo grep.

## Vitest config repair

The earlier rename pass missed relative aliases in vitest configs (`../ui-runtime/`,
`../ui-dom/`, etc.). Swept across `sui-dom`, `sui-eval`, `sui-lsp`, `ship-sdk-client`,
`ui-components`. After fix, `sui-dom` test suite runs (79/79 passing).

## Test coverage added

| Where | New tests | Notes |
|---|---:|---|
| `crates/sui-signals/tests/dep_cleanup.rs` | 6 | Conditional deps, derived chains, dedup, dispose cycle |
| `crates/sui-signals/tests/reentrance.rs` | 8 | set-in-subscriber, self-loop, batch-in-effect, nested batch, panic recovery (effect + derived), cross-thread, dispose-mid-notify |
| `crates/sui-signals/tests/fuzz_regression.rs` | 1 | UTF-8 lex panic regression |
| `crates/sui-signals/benches/notify.rs` | criterion | fanout, many_deps, batch coalescing |
| `packages/sui-runtime/test/dep-cleanup.test.ts` | 8 | TS mirror of dep_cleanup |
| `packages/sui-runtime/test/reentrance.test.ts` | 7 | TS mirror of reentrance |
| `packages/sui-runtime/test/effect-scope.test.ts` | 5 added | Cleanup-during-cleanup, reentrant dispose, multi-throw siblings, grandchild cascade, post-scope onCleanup |
| `packages/sui-runtime/test/for.test.ts` | 3 added | Duplicate-key warning behavior |
| `crates/sui-compiler/fuzz/fuzz_targets/compile_full.rs` | fuzz target | Full pipeline View + Component |
| `crates/sui-compiler/fuzz/fuzz_targets/bindings_build.rs` | fuzz target | parse → bindings |

**Totals:**
- Rust: +14 tests, +1 regression test, +1 bench, +2 fuzz targets
- TS: +23 tests
- Bugs found: 3 fixed, 6 follow-ups logged

## Per-unit audit docs

- `audits/sui-signals.md` (Rust)
- `audits/sui-runtime-signals.md` (TS signal + effect-scope)
- Skipped per-unit docs for sui-compiler (15.5k LOC; focused contribution
  was the fuzz expansion + UTF-8 fix), sui-dom (test suite restored via
  alias fix), and sui-runtime rest (touched only `for.ts`).

## What is NOT done (deliberately bounded or blocked)

| Area | Status |
|---|---|
| In-browser playwright verification | **Blocked.** SUI routes (`/_sui/*`, `/view-host`) are not mounted on shipd's router on the merged branch — see follow-up `F-P4`. The handlers and server crate exist; the route table doesn't reference them. Resolving is mechanical but is host-integration work, not rendering. |
| Platform DB migration drift | Pre-existing follow-up `F-P6`. Daemon won't start against the user's main `.ship`. Worked around for local audit by `SHIP_HOME=/tmp/...`. |
| sui-dom deep audit | Restored to passing (79/79). Full source walk deferred. |
| sui-runtime mount.ts / control flow / template-hooks deep audit | Touched only `for.ts` (duplicate key warning). `mount.ts` patch-elements stub flagged in earlier audit, untouched. |
| sui-compiler deep audit | Found + fixed one UTF-8 bug; added two fuzz targets. Did not walk all 79 source files. Further fuzz time would likely surface more bugs. |
| sui-lsp / sui-eval audit | Pre-existing test failures logged as F-P2/F-P3; not investigated. |

## Follow-ups (see `.artifacts/sui-arc/follow-ups.md`)

- F-S1: sui-signals concurrent dispatch can lose untracks (rare, daemon use)
- F-P1: ui-components 57 suites missing generated lucide registry
- F-P2: sui-lsp server.test.ts timeout
- F-P3: sui-eval cli.test.ts + score.test.ts failures
- F-P4: SUI HTTP routes not mounted on shipd router (blocks browser verification)
- F-P5: SUI bundler retries forever when `~/.ship/runtime/` isn't populated
- F-P6: Platform DB migration drift after substrate merge
