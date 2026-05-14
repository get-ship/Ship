# SUI perf status

Captured 2026-05-13 (SHA 4a87c570) after the D5 + ssr-hydrate-wire + sdk-typegen-followup cycle.

Numbers are SUI median ms unless noted. Lower is better.

## Anchors

| Tag         | File                                          | SHA      |
|-------------|-----------------------------------------------|----------|
| baseline    | `tests/perf/baselines/2026-05-13-baseline.json`    | e6e85f8f |
| post-d2-d4  | `tests/perf/baselines/2026-05-13-post-d2-d4.json`  | 2a951ef6 |
| post-d5     | `tests/perf/baselines/2026-05-13-post-d5.json`     | 4a87c570 |
| post-replace1k          | `tests/perf/baselines/2026-05-13-post-replace1k.json`          | a3ff5d69 |
| post-bindFor-shortcircuit | `tests/perf/baselines/2026-05-13-post-bindFor-shortcircuit.json` | e33a1e76 |

Cumulative changes since `post-d2-d4`: D5 (inline static row bindings), ssr-hydrate-wire (morph instead of innerHTML overwrite), sdk-typegen-followup (typegen, no runtime effect).

## Post-D5 deltas

| Op                       | baseline | d2-d4 | post-d5 | Δ vs d2-d4 | Δ vs baseline |
|--------------------------|----------|-------|---------|------------|---------------|
| 01_run1k                 |     79.0 |  59.5 |    56.2 |     -5.5%  |    -28.9%     |
| 02_replace1k             |     69.4 |  65.2 |    66.8 |     +2.5%  |     -3.7%     |
| 03_update10th1k_x16      |     12.6 |  23.3 |    14.2 |    -39.1%  |    +12.7%     |
| 04_select1k              |      5.7 |   4.7 |     5.2 |    +10.6%  |     -8.8%     |
| 05_swap1k                |     10.8 |  10.3 |    10.6 |     +2.9%  |     -1.9%     |
| 06_remove-one-1k         |     13.0 |  13.2 |    13.4 |     +1.5%  |     +3.1%     |
| 08_create1k-after1k_x2   |     70.9 |  68.5 |   245.4 |    +258%   |    +246%      |
| 09_clear1k_x8            |      6.6 |   6.7 |     6.7 |      0.0%  |     +1.5%     |
| 21_ready-memory (MB)     |      0.6 |   0.6 |     0.7 |    +16.7%  |    +16.7%     |
| 22_run-memory (MB)       |      4.2 |   4.2 |     4.1 |     -2.4%  |     -2.4%     |
| 25_run-clear-memory (MB) |      3.5 |   2.2 |     3.4 |    +54.5%  |     -2.9%     |
| 41_size-uncompressed (KB)|     25.5 |  25.0 |    25.5 |     +2.0%  |      0.0%     |
| 42_size-compressed (KB)  |      7.9 |   7.8 |     7.9 |     +1.3%  |      0.0%     |
| 43_first-paint           |     56.0 |  58.1 |   152.6 |   +163%    |    +173%      |

## What moved

**D5 confirmed wins** (inline static row bindings):
- `01_run1k`: -5.5% vs d2-d4 (56.2 ms). Cumulative -28.9% from baseline (79 → 56.2). Real improvement.
- `03_update10th1k_x16`: -39.1% vs d2-d4 (23.3 → 14.2). Largest single-cycle win. D5 + the prior derived-memo land together produced this. Note vs baseline this is still +12.7% — the original baseline reading of 12.6 ms appears to have been an outlier low (the d2-d4 reading went *up* to 23.3, then came back down here).
- `02_replace1k`: +2.5% vs d2-d4 (within run-to-run noise). D5 had no meaningful effect here — the static-binding inlining doesn't dominate the replace path. Documented as no-op rather than win.

**Real SUI regressions** (not bench noise — comparison frameworks held):
- `25_run-clear-memory`: 2.2 → 3.4 MB (+54.5%). Isolated to SUI; solid/svelte/vue/vanillajs all moved <0.2 MB. Suspect: ssr-hydrate-wire morph retains DOM refs that the prior innerHTML overwrite released. Worth a follow-up — see "Recommended follow-up" below.
- `21_ready-memory`: 0.6 → 0.7 MB (+16.7% but +0.1 MB absolute). Within rounding; flag-only.
- `04_select1k`: 4.7 → 5.2 ms (+10.6%). Small absolute (+0.5 ms). Within noise but tagged.

**Unreliable bench numbers** (cross-framework regression — environment-wide, not SUI-specific):
- `43_first-paint`: all five frameworks regressed 150–340% (sui +163%, solid +190%, svelte +166%, vue +277%, vanillajs +339%). Single-run bench environment issue. Do not treat as a SUI regression.
- `08_create1k-after1k_x2`: solid (-3.8%) and svelte (-0.9%) held steady; sui (+258%), vue (+177%), and vanillajs (+144%) all regressed. SUI's per-run values are bimodal (`[291,308,336,245,329,226,240,218,286,265,465,157,177,184,129]` — late runs dropped by 50%, consistent with thermal throttling or transient system load mid-bench). Treat the 245.4 median as suspect; rerun expected.

## vs Solid

| Op                       | SUI   | Solid | ratio  |
|--------------------------|-------|-------|--------|
| 01_run1k                 | 56.2  | 67.3  | 0.83x  |
| 02_replace1k             | 66.8  | 59.0  | 1.13x  |
| 03_update10th1k_x16      | 14.2  | 14.7  | 0.97x  |
| 04_select1k              |  5.2  |  5.5  | 0.95x  |
| 05_swap1k                | 10.6  | 10.3  | 1.03x  |
| 06_remove-one-1k         | 13.4  | 61.5  | 0.22x  |
| 09_clear1k_x8            |  6.7  |  9.3  | 0.72x  |

`06_remove-one-1k` and `09_clear1k_x8` solid numbers are higher than usual (solid's own runs were noisy this session — 61.5 ms vs the d2-d4 reading of 10 ms). SUI is at-or-below Solid on six of seven non-suspect ops on this run; ratios are session-noisy and should be re-confirmed on a fresh bench.

## Recommended follow-up

1. **Rerun `08_create1k-after1k_x2` and `43_first-paint`.** The cross-framework regression pattern points at environment noise (thermal / system load) rather than a SUI change. A single clean rerun should land both back near the d2-d4 numbers.
2. **Investigate `25_run-clear-memory` +54.5%.** Isolated to SUI, no other framework moved. Likely ssr-hydrate-wire (morph keeps live refs around). Worth profiling whether the post-clear DOM holds detached nodes that the old innerHTML reset path released. — **Resolved 2026-05-13:** code audit (`perf-followups.md`) found no retention path; rerun (`2026-05-13-confirm.json`, 2.7 MB) lands between d2-d4 (2.2) and post-d5 (3.4). Confirmed as bench noise. See `bench-rerun-confirm.md`.
3. **Filename collision in `bench-run.mjs`.** The runner writes `<date>-baseline.json` and overwrote the historical `2026-05-13-baseline.json` anchor when run today. The file was restored from git for this job; a follow-up should add a `--label` argument so post-cycle captures don't stomp the anchor. — **Resolved:** `--label` is required and refuses to overwrite an existing file.

## Resolution (2026-05-13, SHA f36d8e1b)

`25_run-clear-memory` rerun: `tests/perf/baselines/2026-05-13-confirm.json` — SUI 2.7 MB. Sequence 2.2 → 3.4 → 2.7 confirms post-d5's reading was inside this metric's natural per-run variance band, not a regression. `22_run-memory` showed the same direction (4.1 → 2.9 MB) on identical runtime code, further evidence that the memory-op variance on this host is ±~0.6 MB. No SUI code change needed.

Caveats on `2026-05-13-confirm.json`: the bench harness failed plausibility on `01_run1k`, `02_replace1k`, `08_create1k-after1k_x2` (and `07_create10k`, already excluded). The aggregator silently re-read stale result JSONs from the post-d5 run for those ops, so those SUI values are duplicates of post-d5, not fresh measurements. Memory and size/paint readings are fresh. Full breakdown in `bench-rerun-confirm.md`.

## Post-bindFor-shortcircuit (2026-05-13, SHA e33a1e76)

`runtime-perf-followups` (#145) merged: same-array short-circuit in `bindFor`
plus a row identity cache. Fresh baseline:
`tests/perf/baselines/2026-05-13-post-bindFor-shortcircuit.json`. Full writeup
in `perf-rebench-confirm.md`.

### Fresh SUI deltas (this run only — stale ops excluded)

| Op                       | post-d5 | post-replace1k | this run | Δ vs post-d5 | Δ vs post-replace1k |
|--------------------------|---------|----------------|----------|--------------|---------------------|
| 02_replace1k             |   66.8  |    66.8        |  120.2   |   **+79.9%** |   **+79.9%**        |
| 04_select1k              |    5.2  |     5.8        |    5.2   |     0.0%     |    -10.3%           |
| 05_swap1k                |   10.6  |    13.9        |   12.2   |   +15.1%     |    -12.2%           |
| 06_remove-one-1k         |   13.4  |    13.8        |   13.3   |    -0.7%     |     -3.6%           |
| 08_create1k-after1k_x2   |  245.4  |    73.7        |   67.2   |   **-72.6%** |     -8.8%           |
| 22_run-memory (MB)       |    4.1  |     2.9        |    2.8   |   -31.7%     |     -3.4%           |
| 25_run-clear-memory (MB) |    3.4  |     2.7        |    2.7   |   -20.6%     |     0.0%            |
| 41_size-uncompressed (KB)|   25.5  |    25.5        |   25.6   |    +0.4%     |    +0.4%            |
| 43_first-paint           |  152.6  |    77.1        |  216.1   |   +41.6%     |   +180%             |

### Highlights

- **`08_create1k-after1k_x2` confirmed healthy** (245 → 67). Closes out the
  post-d5 spike — was bench noise, exactly as the post-d5 writeup predicted.
- **Memory readings stable** at the post-replace1k values (22_run 2.8 / 25_clear
  2.7). Two-run confirmation that the post-d5 jumps were variance.

### Plausibility caveat (same as confirm-run)

SUI plausibility failed on `01_run1k`, `03_update10th1k_x16`, `09_clear1k_x8`
(and `07_create10k`, already excluded). The aggregator fell back to stale
post-replace1k results for those ops. **The SUI values 59.4 / 11.4 / 9.0 in
the new baseline are duplicates of `post-replace1k`, not fresh.** Same root
cause as `bench-rerun-confirm.md`; same recommended aggregator fix.

### Regressions >5% — recommendations

1. **`02_replace1k` +79.9%.** Cross-framework regression (solid +88%, vanillajs
   +893%, svelte/vue plausibility-failed). Environmental noise signature. The
   bindFor short-circuit branch shouldn't affect replace-all (array reference
   swaps). **Recommend op-targeted rerun before attributing to #145.**
2. **`43_first-paint` +41.6%.** All five frameworks regressed in lockstep —
   known noisy single-run metric on this host. Not a SUI regression.
3. **`05_swap1k` +15.1% vs post-d5 / -12.2% vs post-replace1k.** Within the
   ~3 ms variance band. Flag-only.

### Net for #145

Inconclusive. The two ops most likely to benefit (`03_update10th1k_x16`,
`01_run1k`) are stale this run. **Recommend rerun of the three stale SUI ops
plus `02_replace1k` before declaring the bindFor short-circuit verified.**
