# SUI perf baselines

Checked-in median results from the js-framework-benchmark suite, captured
against `keyed/sui` at a known git SHA. Each file is named
`YYYY-MM-DD-baseline.json`; the latest filename (sorted) is the active
baseline.

## Format

```json
{
  "captured": "YYYY-MM-DD",
  "git_sha": "<short SHA>",
  "framework_version": "0.1.0",
  "compared_against": ["solid", "svelte", "vue", "vanillajs"],
  "operations":   { "01_run1k":  { "sui": 59.8, "solid": 49.5, ... }, ... },
  "memory_mb":    { "22_run-memory":  { "sui": 3.4, ... }, ... },
  "size_kb":      { "41_size-uncompressed": { "sui": 18.3, ... }, ... },
  "first_paint_ms": { "43_first-paint": { "sui": 38.0, ... } }
}
```

Median values only — variance lives in the raw `webdriver-ts/results/*.json`
files inside the bench checkout.

## Capturing a new baseline

```sh
just bench
```

This rebuilds the SUI bench bundle, runs the benchmark against SUI plus
comparison frameworks (solid, svelte, vue, vanillajs), and writes a fresh
`<today>-baseline.json`. Commit it.

Requires:
- `~/dev/js-framework-benchmark/` checked out with comparison frameworks
  already built (`npm run build-prod` in each).
- Playwright Chromium installed at the path in `scripts/dev/bench-lib.mjs`
  (override with `CHROME_BINARY=...`).
- A working `sui-compiler` in `~/dev/Ship-feat-sail/target/debug/`, which is
  what `frameworks/keyed/sui/scripts/precompile.mjs` expects.

## Checking against the baseline

```sh
just bench-check
```

Re-runs the benchmark against `keyed/sui` only (skips comparison frameworks
for speed), reads the most recent committed baseline, and compares medians
op-by-op. Exits non-zero if any op regresses by more than 15%. Override the
threshold with `BENCH_REGRESSION_THRESHOLD=0.25 just bench-check`.

## On regression

1. Reproduce locally — `just bench-check` once is noisy. Run twice; if the
   second run is clean, treat as flake.
2. If the regression is real, either fix it before merge or capture a new
   baseline with `just bench` and commit that, with a commit message
   explaining the deliberate movement.
3. Never silently raise `BENCH_REGRESSION_THRESHOLD` to hide a regression.
