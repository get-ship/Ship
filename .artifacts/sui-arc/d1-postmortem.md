# D1 — Template-clone emit — abandoned post-4th-pass

## What D1 was supposed to do

Per `.artifacts/sui-arc/perf-profile.md` D1: hoist `{#for}` body's static
skeleton into a module-init `<template>` constant; per-row body uses
`cloneNode(true)` + index-path hole resolution. **Projected gain:**
−6.5 ms on `01_run1k` (−12%), −5 ms on `02_replace1k` (−8%).

## What actually happened

Across 4 repair passes the agent (`rust-lane`, opus) produced
commits at `a60a86cf` → `14bfb4a7` → `ae75e53b` → `33db0cfb`,
each fixing a previous regression but introducing new ones. Final
bench numbers vs baseline (`2026-05-13-baseline.json`):

| Op | Baseline | D1 final | Delta |
|---|---|---|---|
| 01_run1k | 55.7 ms | 74.2 ms | **+33.2%** ✗ |
| 02_replace1k | 61.8 ms | 102.7 ms | **+66.2%** ✗ |
| 03_update10th1k_x16 | 11.0 ms | 11.4 ms | +3.6% |
| 04_select1k | 7.4 ms | 8.5 ms | +14.9% |
| 05_swap1k | 11.7 ms | 10.0 ms | **−14.5%** ✓ |
| 06_remove-one-1k | 14.7 ms | 12.8 ms | **−12.9%** ✓ |
| 08_create1k-after1k_x2 | 99.8 ms | 67.2 ms | **−32.7%** ✓ |
| 09_clear1k_x8 | 8.6 ms | 6.7 ms | **−22.1%** ✓ |
| 25_run-clear-memory | 0.9 MB | 2.1 MB | **+133.3%** ✗ |

Net assessment: the projected −6.5 ms / −5 ms wins on the primary
benchmarks reversed into +18.5 ms / +40.9 ms losses. Wins on
"second-create" + "swap" + "clear" workloads are real but those aren't
the headline framework-comparison ops.

## What went wrong (best inference, no proof)

1. **Template hoisting overhead dominates on first-create.** Solid
   pre-builds its template-clone code at compile-time as a tight inline
   function. D1's approach allocates a template-node graph + an
   index-path resolution array per `{#for}` AT MODULE INIT, but the
   hole-resolution path during cloneNode appears to be slower than
   the original `createElement/setAttribute/appendChild` for the
   `01_run1k` workload. Confirmed empirically (08_create1k-after1k_x2
   shows the wins clearly because that workload amortizes the init cost
   over two creates).

2. **Memory leak fix introduced runtime cost.** The cleanup-closure now
   nulls `$scope.var/idx/__v` per row. Three property writes × 1000 rows
   × N renders during `02_replace1k` adds measurable time. The agent's
   diff:
   ```rust
   cleanup: () => { for (const c of {cl}) try { c(); } catch (e) {}
     {cl}.length = 0;
     $scope.{var} = null;     // new
     $scope.{idx_name} = null; // new
     $scope.__v = null;        // new
   }
   ```

3. **Pass-by-pass repair without holistic redesign.** Each rejection
   triggered targeted fixes, but the underlying architecture was never
   reconsidered. By pass 4 the implementation had accumulated patches
   that fought each other.

## What's salvageable

- `08_create1k-after1k_x2 -32.7%` is a substantial win on a real
  workload (creating a list, clearing, creating again). Worth chasing.
- The 5/6/9 wins are also real.
- The compiler-side scaffolding (clone_emit.rs, the hole-resolution
  data structures in template_emit) is reusable if redesigned.

## Recommended D1' redesign

A future D1' should:
1. **Profile cloneNode + hole-resolution vs createElement path FIRST**
   to confirm the hoist actually beats per-row construction. The
   current data suggests it doesn't.
2. **Only hoist when the row body has zero per-element bindings.**
   Static markdown-ish content benefits; reactive content doesn't.
3. **Don't null `$scope` refs in cleanup** — use a `WeakMap` for
   row-scope references so GC handles it without per-render writes.
4. **Treat `02_replace1k` as the gate.** This op is the second-most
   referenced framework comparison after `01_run1k`. Any regression
   here is unacceptable.

## D1 work artifacts retained

- Branch `job/perf-d1-template-clone` deleted; final sha `33db0cfb`.
- Worktree `/home/madvib/dev/ship-worktrees/perf-d1-template-clone`
  removed.
- This postmortem at `.artifacts/sui-arc/d1-postmortem.md` (gitignored).
- The compiler-side template_emit/clone_emit.rs code is in the deleted
  branch; recoverable via `git reflog` if a future D1' wants to
  reference it.

## D2 + D4 stand

D2 (bindFor scratch reuse) and D4 (bind\* short-circuit) are in
`feat/ship-set` and provided real net-positive gains. Re-bench against
fresh baseline before launching D1'.

