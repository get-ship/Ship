# Post-bindFor-shortcircuit re-bench

**Captured:** 2026-05-13 (SHA `e33a1e76`)
**Baseline file:** `tests/perf/baselines/2026-05-13-post-bindFor-shortcircuit.json`
**Trigger:** runtime-perf-followups merge (#145 → 96b65e6f) added a same-array
short-circuit to `bindFor` plus a row identity cache. Goal of this run: confirm
the post-d5 + post-replace1k picture still holds and capture the new anchor.

## Plausibility caveat (read first)

The bench harness flagged plausibility failures on these SUI ops:

| Op                       | Reason                                |
|--------------------------|---------------------------------------|
| `01_run1k`               | "at most one mousedown event"         |
| `03_update10th1k_x16`    | "at most one mousedown event"         |
| `07_create10k`           | (already excluded from anchors)       |
| `09_clear1k_x8`          | "at most one mousedown event"         |

When the per-op JSON is unusable, the aggregator falls back to the most recent
result file in `webdriver-ts/results/` for that op + framework, which is the
prior run's data. That means the SUI values for `01_run1k` (59.4), `03_update10th1k_x16`
(11.4), and `09_clear1k_x8` (9.0) in the new baseline are **duplicates of
`2026-05-13-post-replace1k.json`** — not fresh measurements from this run.

Same pattern as the 2026-05-13-confirm rerun; cause documented in
`bench-rerun-confirm.md`. Fix is the same: stop the aggregator from silently
falling through to stale result files.

Cross-framework plausibility also failed for svelte (01/02/03/04/05/07/08/09)
and vue (01/02/07/08) and vanillajs (07/08/09), so several comparison values are
likewise stale. The SUI numbers are what this writeup focuses on.

## Fresh SUI ops this run (real measurements)

| Op                       | post-d5 | post-replace1k | this run | Δ vs post-d5 | Δ vs post-replace1k |
|--------------------------|---------|----------------|----------|--------------|---------------------|
| 02_replace1k             |   66.8  |    66.8        |  120.2   | **+79.9%**   | **+79.9%**          |
| 04_select1k              |    5.2  |     5.8        |    5.2   |     0.0%     |    -10.3%           |
| 05_swap1k                |   10.6  |    13.9        |   12.2   |   +15.1%     |    -12.2%           |
| 06_remove-one-1k         |   13.4  |    13.8        |   13.3   |    -0.7%     |     -3.6%           |
| 08_create1k-after1k_x2   |  245.4  |    73.7        |   67.2   |   **-72.6%** |     -8.8%           |
| 21_ready-memory (MB)     |    0.7  |     0.6        |    0.6   |   -14.3%     |     0.0%            |
| 22_run-memory (MB)       |    4.1  |     2.9        |    2.8   |   -31.7%     |     -3.4%           |
| 25_run-clear-memory (MB) |    3.4  |     2.7        |    2.7   |   -20.6%     |     0.0%            |
| 41_size-uncompressed (KB)|   25.5  |    25.5        |   25.6   |    +0.4%     |    +0.4%            |
| 42_size-compressed (KB)  |    7.9  |     7.9        |    7.9   |     0.0%     |     0.0%            |
| 43_first-paint           |  152.6  |    77.1        |  216.1   |   +41.6%     |   +180%             |

## Stale ops (skip — duplicate values, not fresh)

| Op                       | Value (stale) | Source     |
|--------------------------|---------------|------------|
| 01_run1k                 | 59.4          | post-replace1k |
| 03_update10th1k_x16      | 11.4          | post-replace1k |
| 09_clear1k_x8            | 9.0           | post-replace1k |

These cannot inform the bindFor-shortcircuit verdict; an op-level rerun is
needed before we can say what `runtime-perf-followups` did to `03_update10th1k_x16`,
the op most expected to move from a same-array short-circuit.

## What this run actually confirms

1. **`08_create1k-after1k_x2` is healthy again** (245.4 → 67.2 ms).
   Confirms the post-d5 spike was bench-environment noise, exactly as the
   post-d5 writeup hypothesised (`perf-status.md`, "Unreliable bench numbers").
   The cumulative regression-vs-baseline (70.9 → 67.2) is now within noise.

2. **Memory readings settled.** `22_run-memory` and `25_run-clear-memory` both
   match the post-replace1k reading. The 4.1 / 3.4 spikes in post-d5 were
   per-run variance, not regressions — already resolved in
   `bench-rerun-confirm.md`, and this run is the second confirmation.

3. **`41_size-uncompressed` +0.1 KB.** Plausibly the new short-circuit code in
   `bindFor`. Within rounding; not actionable.

## Real regressions (>5%) — recommendations

### `02_replace1k`: 66.8 → 120.2 ms (+79.9%)

Largest fresh-data delta. Context to weigh against calling this a SUI regression:

- **Cross-framework regression on the same op.** Comparison frameworks where
  the value is *fresh* this run all also regressed:
  - solid 59.0 → 110.8 (+87.8%)
  - vanillajs 53.8 → 534.6 (+893%)
  - (svelte 309.8 and vue 71.9 are stale duplicates of post-replace1k — exclude)
- Svelte and vue couldn't even complete this op (plausibility failure).
- The pattern — multiple frameworks all regressing on the same op while at least
  two fail outright — is the same environmental noise signature as the post-d5
  `08_create1k-after1k_x2` spike and the post-d5 / post-replace1k `43_first-paint`
  cross-framework drift.

**Recommendation:** rerun `02_replace1k` (op-targeted) on a quieter host before
attributing to the bindFor short-circuit. The same-array short-circuit path
should not affect replace-all (replace-all swaps the array reference, so the
short-circuit branch is not taken).

### `43_first-paint`: 152.6 → 216.1 ms (+41.6% vs post-d5, +180% vs post-replace1k)

Cross-framework: solid 155.9 (was 61.9), vue 380.8 (was 194.2), svelte 241.2
(was 124.8), vanillajs 200.3 (was 78.6). All five frameworks regressed in lockstep
again — same environment signature seen in post-d5. Not a SUI regression.

**Recommendation:** none new — `43_first-paint` is already a known noisy
single-run metric on this host. Per-run variance dwarfs any code-level signal.

### `05_swap1k`: 10.6 → 12.2 ms (+15.1% vs post-d5)

Small absolute (+1.6 ms). Goes the other way vs post-replace1k (13.9 → 12.2,
-12.2%). The d2-d4 / post-d5 / post-replace1k / this-run sequence is
10.3 / 10.6 / 13.9 / 12.2 — variance band ~3 ms. Not a regression; flag-only.

## Net for the bindFor short-circuit (#145)

Inconclusive. The two ops most likely to benefit — `03_update10th1k_x16`
(repeated same-array updates) and `01_run1k` (steady-state row construction) —
are both stale this run, so we cannot read the effect. `04_select1k` (also
same-array) was flat vs post-d5 (5.2 ms both times) and -10.3% vs post-replace1k,
which is the direction the change predicts but is small enough to be within
noise.

Recommended follow-up:

1. **Rerun the three stale SUI ops** (`01_run1k`, `03_update10th1k_x16`,
   `09_clear1k_x8`) on a quiet host before drawing a verdict on #145.
2. **Rerun `02_replace1k`** to confirm the +79.9% is environment, not code.
3. **Aggregator fix carried over from `bench-rerun-confirm.md`:** the aggregator
   should error or mark the op `null` when a plausibility failure leaves no
   fresh result JSON, not silently re-use the prior run's value. The repeated
   pattern of stale-value contamination across recent baselines is now the
   dominant source of confusion in this perf cycle.
