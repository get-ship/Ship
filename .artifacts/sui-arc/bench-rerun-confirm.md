# Bench rerun: confirm `25_run-clear-memory`

Captured 2026-05-13 (SHA f36d8e1b) on the same host that produced `2026-05-13-post-d5.json` earlier the same day. Single-run capture via `just bench confirm`.

## Resolution

**`25_run-clear-memory`: confirmed bench-environment noise, not a real SUI regression.**

Sequence on this metric: 3.5 (baseline) → 2.2 (d2-d4) → 3.4 (post-d5) → 2.7 (confirm). The post-d5 +54% spike vs d2-d4 sits inside the run-to-run variance band the metric has shown without any memory-related code change. The earlier code audit (`perf-followups.md` recommendation #2) found no retention path in the ssr-hydrate-wire morph; this rerun lines up with that.

`22_run-memory` told the same story: 4.1 → 2.9 MB (-29%) on a build with no memory changes since post-d5, so the per-run variance on memory ops on this host is ±~0.6 MB.

No follow-up dispatch needed for memory.

## What is and isn't trustworthy in `2026-05-13-confirm.json`

The bench runner's plausibility checker rejected several sub-benchmarks this run. The runner does not delete result JSONs on failure, so the aggregator silently picked up the prior post-d5 result files for those ops.

Stale (read from 12:35–14:07 post-d5 result files, not this run):

| Op                     | confirm sui | matches post-d5? |
|------------------------|-------------|------------------|
| 01_run1k               | 56.2        | yes (identical)  |
| 02_replace1k           | 66.8        | yes (identical)  |
| 08_create1k-after1k_x2 | 245.4       | yes (identical)  |

Fresh (written 16:42–17:07 in this run): `03_update10th1k_x16`, `04_select1k`, `05_swap1k`, `06_remove-one-1k`, `09_clear1k_x8`, `21_ready-memory`, `22_run-memory`, `25_run-clear-memory`, `41_size-uncompressed`, `42_size-compressed`, `43_first-paint`.

Plausibility failure list (from runner stdout):
- `01_run1k`: solid, sui, svelte, vanillajs, vue — "at most one mousedown event is expected"
- `02_replace1k`: sui, vue
- `07_create10k`: all five (07 is a known-flaky op, excluded from baselines anyway)
- `08_create1k-after1k_x2`: sui
- `09_clear1k_x8`: vue (sui's 09 was fresh: 9.9 ms)

The `mousedown` plausibility failure is the same harness flake noted as the rerun trigger for follow-up #1 in `perf-status.md` — likely related to a recent Chromium/webdriver update affecting initial-paint event dispatch. It is bench-harness behavior, not SUI behavior.

## Recommended follow-up to the runner

When plausibility fails, the aggregator should mark that op as missing rather than reading the stale JSON. Two viable fixes:

1. `runBench` deletes `webdriver-ts/results/*.json` for the requested frameworks before invoking the runner, so stale data can't bleed in.
2. The runner parses its own stdout for the "failed:" list and the aggregator omits those (framework, op) pairs.

(1) is simpler and matches what a clean capture conceptually means. Filed here rather than enacted because this job's scope forbids touching `scripts/dev/bench-run.mjs`.
