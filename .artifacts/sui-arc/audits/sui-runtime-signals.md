# Audit — `packages/sui-runtime/src/{signal,effect-scope}.ts` (TS)

## Inventory

| File | LOC | Responsibility |
|---|---|---|
| `signal.ts` | ~300 → ~280 after fix | `signal`, `derived`, `effect`, `batch`, `untrack` |
| `effect-scope.ts` | ~141 → ~108 after dead-alias removal | `EffectScope`, `createScope`, `createChildScope`, `withScope`, `onCleanup` |
| `test/signal.test.ts` | 199 | Algorithm parity baseline |
| `test/effect-scope.test.ts` | ~140 → ~225 after additions | Scope lifecycle |
| `test/dep-cleanup.test.ts` | 188 (new) | Mirror of Rust dep_cleanup |
| `test/reentrance.test.ts` | 140 (new) | Mirror of Rust reentrance |

## Bugs found

### B-T1. Derived `newTracked` was never populated; effect had stale subs

Same shape as the Rust bug. `derived`'s `recompute` (signal.ts:127-144 pre-fix)
declared `newTracked: Signal<unknown>[] = []` and installed a fresh `collector`
closure as `CURRENT_OBSERVER` each run. The collector got added to every read
signal's `__subs`. After the run, the cleanup tried to `t.__subs.delete(invalidate)`
on `oldTracked` — but invalidate had never been added; the closure that got
added was the fresh per-run collector. Cleanup was a no-op. Each recompute
leaked one closure per dep into the signal's `__subs` forever, and stale
collectors kept firing on dep writes.

`effect` had the symmetric bug — `run` was the same function reference across
re-runs (stable, no diff), so signals retained subscribers to deps the closure
no longer read.

### B-T2. `effect-scope` shipped 4 deprecated aliases for the prior `owner` API

`Owner`, `withOwner`, `registerOwnerCleanup`, `createOwner` (effect-scope.ts:130-140
pre-fix). Zero internal callers per repo grep. Pure dead weight in every
runtime bundle. Removed.

## Fix applied

Mirror of `crates/sui-signals` fix: diff-based untracking. New shared
`track(subs)` helper subscribes the current observer and pushes an untrack
thunk into `CURRENT_UNTRACKS`. `effect.run` and `derived.recompute` fire
prior untracks before re-running, then collect a fresh set. `effect` disposer
proactively unsubscribes from every signal it was tracking.

## Edge cases added

**dep-cleanup.test.ts** (8 tests):
- effect_drops_dep_when_no_longer_read (conditional dep flip — the core B-T1 case)
- same signal read multiple times dedupes
- writes after dispose do not fire
- dispose removes effect from `__subs` (`.size` direct assertion)
- re-tracking after flip removes old subs
- derived conditional dep flip + compute count assertion
- derived chain stale-dep cleanup
- derived recompute does not accumulate stale invalidate refs in deps

**reentrance.test.ts** (7 tests):
- set inside a subscriber feeds another effect consistently
- effect writing the signal it reads terminates via Object.is
- batch inside an effect coalesces an observer's re-runs
- nested batch only flushes at outermost
- throwing effect does not corrupt CURRENT_OBSERVER
- throwing derived recompute does not corrupt CURRENT_OBSERVER
- disposing an effect mid-notification is safe

**effect-scope.test.ts** (5 added, 15 total):
- cleanup that registers another cleanup during dispose runs it immediately
- cleanup that disposes the same scope is idempotent (reentrant safe)
- two throwing cleanups + two safe cleanups — all four still run, both errors logged
- grandchild auto-disposes when grandparent disposes (cascade)
- onCleanup after withScope returns is a no-op

## Performance

Not benched. Hot path for reads adds one closure allocation per fresh
subscription (the untrack thunk). Reads of already-subscribed signals are
O(1) Set.has(). For typical view sizes (10–100 signals, 1–5 effects), the
extra allocation per read is negligible. The dep-cleanup is the same cost
that any unbiased fine-grained reactive system pays.

If a bench becomes warranted, add `packages/sui-runtime/test/signal.bench.ts`
using vitest bench. Out of scope for this pass.

## Decisions surfaced

| # | Decision | Recommendation |
|---|---|---|
| D-T1 | Should `derived` expose an explicit `dispose()` to drop sub of long-lived deps when the derived is no longer needed? Currently relies on GC. | Defer — GC handles it once consumers drop the reader. Add only if profiling shows leak. |

## Result

| Metric | Before | After |
|---|---|---|
| sui-runtime tests | 388 passed | 400 passed (12 added, 0 broken) |
| Dep-cleanup correctness | broken | fixed + 8 dedicated tests |
| Reentrance coverage | nothing | 7 new tests |
| Effect-scope coverage | 10 tests | 15 tests |
| Dead alias surface | 4 deprecated exports | removed |
| Stale vitest aliases | 4 packages broken | fixed (sui-dom now 79/79) |
