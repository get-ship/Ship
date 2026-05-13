# SUI perf baselines

Checked-in median results from the js-framework-benchmark suite, captured
against `keyed/sui` at a known git SHA. Standard filename is
`YYYY-MM-DD-baseline.json`; the latest such filename (sorted) is the
active anchor that `just bench-check` reads.

## Current anchor

- `2026-05-13-post-d2-d4.json` — post D2 (`bindFor` scratch reuse) + D4
  (bind\* short-circuit) merges. Captured at `2a951ef6` on 2026-05-13.
  See `.artifacts/sui-arc/perf-status.md` for the delta write-up.
- `2026-05-13-baseline.json` — preserved pre-perf-cycle reference
  (captured at `56bea690`). Do not overwrite.

Files tagged with a `-<label>.json` suffix (e.g. `-post-d2-d4`) record
a deliberate inflection point — a merged optimization, an abandoned
attempt, a methodology change. They sit beside the standard
`-baseline.json` files so history is legible. Naming convention for
future deliberate captures: `YYYY-MM-DD-<short-label>.json`.

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

## Profiling methodology

`just bench` answers "how fast"; profiling answers "why this fast / what's
the dominant cost". Use `scripts/dev/perf-profile.mjs` to capture a V8
CPU profile of any bench op against SUI plus one comparison framework.

```sh
# Default: SUI / 01_run1k / 10 iterations
node scripts/dev/perf-profile.mjs

# Sweep the create-path ops against both frameworks
node scripts/dev/perf-profile.mjs --all --iter 10

# Targeted capture
node scripts/dev/perf-profile.mjs --framework solid --op 02_replace1k --iter 20

# Watch the run in a visible browser (useful when the page can't load)
node scripts/dev/perf-profile.mjs --headed
```

The harness:

1. Starts the bench's http-server on :8080 if it isn't already up
   (override the path with `BENCH_HOME=…`).
2. Launches Playwright Chromium (already a devDep — no new install).
3. Attaches a CDP session and enables `Profiler` at 100µs sampling
   (10 kHz — fine enough to resolve sub-ms native calls).
4. Warms the page with one unprofiled click, clears, then runs the
   op N times under the profiler with `waitForFunction` predicates
   between iterations.
5. Writes `tests/perf/profiles/YYYY-MM-DD-<framework>-<op>.cpuprofile`
   (open it in Chrome devtools → Performance → Load profile) and
   prints a self-time top-10 to stdout.

Currently mapped ops: `01_run1k`, `02_replace1k`, `05_swap1k`. Add new
ops to the `OPS` table in `scripts/dev/perf-profile.mjs` (each entry
declares the setup clicks, measured click(s), and a `waitForFunction`
predicate so the profiled window covers the right work).

When analysing a profile, group self-time by:

- **native** (`createElement`, `setAttribute`, `appendChild`, etc.)
  — the DOM construction model. Compare against the comparison
  framework's `cloneNode` count.
- **framework code** (URLs ending `.compiled.js`, `dist/main.js`,
  or `sui-runtime/*`) — reconciler + binding overhead. The `bindFor`
  effect body's per-render allocations show up here.
- **GC** — proxy for allocation pressure. Bigger gap than the bench
  number suggests = signal/object churn in the row body.

The first audit using this harness is `.artifacts/sui-arc/perf-profile.md`
(gitignored — transient report).
