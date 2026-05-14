# Audit — `crates/sui-signals` (Rust)

## Inventory

| File | LOC | Responsibility |
|---|---|---|
| `src/lib.rs` | 43 | Public re-exports + doctest |
| `src/runtime.rs` | 79 | Thread-local observer + batch state |
| `src/signal.rs` | 129 | `Signal<T>` — Send+Sync reactive source |
| `src/effect.rs` | 66 | `effect()` + `Disposer` |
| `src/derived.rs` | 64 | `derived()` — memoized computed |
| `src/batch.rs` | 35 | `batch()` — defer notify |
| `src/untrack.rs` | 35 | `untrack()` — read without subscribing |
| `tests/signal_basics.rs` | 234 | Algorithm parity with TS |
| `tests/concurrency.rs` | 137 | Send/Sync, multi-thread, tokio |

## Bugs found

### B1. Stale subscriptions on conditional dep changes (correctness + memory leak)

**Symptom.** Derived/effect closures keep firing when signals they no longer read are written. Each `Signal` retains weak refs to the effect's `run_ref` (a `OnceLock<SubFn>` set once at construction and reused for every re-run). When the closure conditionally reads `a` then later only reads `b`, `a.subs` still has the weak ref to `run_ref`, so writing `a` invalidates an effect that has no dep on `a` anymore.

Same shape as the TS bug at `packages/sui-runtime/src/signal.ts:131-142` (`newTracked` never populated). Both fail to do unsubscribe-on-rerun.

**Root cause.** `effect.rs:34-35` — `run_ref: OnceLock<SubFn>` is set once. No per-run dep tracking, no diff, no removal of old subscribers.

**Impact.**
- Correctness: stale deps trigger unnecessary recomputes (if `f()` has side effects, those fire).
- Performance: O(stale-deps) extra invalidations per write.
- Memory: weak refs accumulate in `Signal.subs` for the lifetime of each signal until the signal is next written and `drain_live`/`subscribe_if_new` prune dead refs. With short-lived effects against long-lived signals, weak refs grow unbounded.

**Fix.** Diff-based unsubscribe-on-rerun. Each tracked run records an "untrack thunk" per new subscription; at the start of the next run (and on `Disposer::drop`), thunks fire to clear prior subs.

### B2. Disposer drop is lazy on cleanup (memory)

**Symptom.** `Disposer::drop` only flips an atomic flag. Weak refs in `Signal.subs` remain until the signal is next written. If a signal is never written after effect disposal, the dead ref sits forever.

**Root cause.** `effect.rs:20-24` — Drop only sets `active = false`. No proactive removal of subs.

**Impact.** Bounded by the number of disposed-but-not-pruned subscriptions × signals never written after disposal. Not catastrophic, but eliminated for free by B1's fix (untracks run on drop too).

## Edge cases not tested

| # | Case | Why it matters |
|---|---|---|
| E1 | Derived with conditional deps (`if cond { a.get() } else { b.get() }`) — B1 manifests here | Direct test of B1 |
| E2 | Derived → derived → effect chain — does the middle derived clean up properly? | Composability |
| E3 | Same signal read multiple times in one effect run — should record one untrack, not N | Memory hygiene |
| E4 | `Disposer::drop` while signal is mid-notify (race) | Safety |
| E5 | `set` inside a subscriber (notification reentrance) — does it deadlock or behave consistently? | Reentrance |
| E6 | `batch` inside an effect that fires during another `batch`'s flush | Batch nesting |
| E7 | Effect that writes a signal it reads (self-loop) | Loop guard / termination |
| E8 | Cross-thread effect re-run (effect created on thread A, signal written on thread B) — currently no test for the actual re-run path on a different thread | Correctness across threads |
| E9 | Throwing closure in `effect` / `derived` — what does the panic do to tracking state? | Resilience |
| E10 | Late `Signal::set` after `Disposer::drop` — should not fire | Disposal semantics |

## Performance

No benchmarks exist. Hot paths:
- `Signal::set` (per write): lock inner, equality check, `drain_live` (O(subs), upgrades each weak), unlock, dispatch (fire each).
- `Signal::get` inside effect: lock, dedupe-or-push to subs (O(subs) scan), unlock. With B1 fix, also pushes one untrack thunk per subscription event.
- `effect` run: invocation overhead + observer swap + (after fix) drain old untracks O(old-deps) + collect new untracks O(new-deps).

To add: criterion benchmark covering single-write fan-out (N effects each subscribing to one signal), and N-deps single-effect (one effect subscribing to N signals).

## Decisions surfaced

| # | Decision | Recommendation |
|---|---|---|
| D1 | Should sui-signals expose `onCleanup`-like callbacks (effect-scope analog)? | Defer. Server use-case is simple compute → notify. Add if a real consumer needs it. |
| D2 | Should `Disposer` be `Send + Sync`? Currently it's `Send` (Arc inside) but not explicitly Sync. | Confirm test pass with `assert_send_sync::<Disposer>()`. Add if missing. |
| D3 | Should `derived` accept `FnMut`? Currently `Fn`. | Keep `Fn`. Mutable state in deriveds is an anti-pattern; force users to thread state via signals. |

## Plan

1. Add failing test for B1 (derived with conditional dep) → confirms baseline.
2. Implement diff-based untracking: `runtime::{enter,exit}_tracking` + `record_untrack`; `signal::get` records untracks; `effect` drains old + collects new; `Disposer::drop` drains.
3. Add tests for E1–E10.
4. Add criterion bench (single-write fan-out, N-deps single-effect).
5. Doctest update: add a stale-dep doctest in `derived.rs` that proves the fix.
