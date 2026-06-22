# shipjs-bound — bound the ship-js SSR isolate

**Job:** `job/shipjs-bound` · **Mode:** autonomous · **Branch:** `job/shipjs-bound`

## What changed

`ship-js` (the deno_core/V8 SSR engine) used to run the controller bundle in an
unbounded isolate. A runaway controller — infinite loop, stuck host op,
unbounded allocation — could wedge or OOM the daemon. This change bounds every
run with a wall-clock deadline AND a V8 heap cap; the bound fires → the isolate
is terminated → `run_module` / `run_script` returns `Err(RunError::Terminated)`
→ the SSR gate (`deck_core::synthetic_ssr`) falls back to the archived
render-eval.

### Files touched

- `crates/core/ship-substrate/crates/ship-js/src/deno/runtime.rs` — bounded
  entry points, bound types, helper for heap-cap callback + deadline thread,
  tests.
- `crates/core/ship-substrate/crates/ship-js/src/deno/mod.rs` — re-exports
  (`RunLimits`, `RunError`, `TerminateReason`, the `_with_limits` variants).
- `crates/core/ship-substrate/crates/ship-js/Cargo.toml` — `tokio` gains the
  `time` feature (for the wall-clock thread); existing deps unchanged.
- `crates/deck/deck-ssr/src/run.rs` — `render()` is now bounded via
  `RunLimits::default()` (with env overrides); new `render_with_limits()`
  exposes the knob; back-compat `render(&bundle, dispatch)` keeps its
  signature so `tests/{ssr,reliability,serve_wired}.rs` need no changes.
- `crates/deck/deck-ssr/tests/runaway.rs` (NEW) — the bound's end-to-end
  proof: runaway controller terminated, runaway heap allocation terminated,
  and the bounded `run_module_with_limits` API itself terminates.

### Files NOT touched

- `crates/deck/deck-core/src/synthetic_ssr.rs` — the SSR gate. Its contract
  (any `Err` from `first_paint` → render-eval fallback) already absorbs the
  bounded `Err` correctly; no change needed.
- The `ScriptExecutor` bridge (`script_executor.rs`) — it calls
  `runtime::run_module`, which is now bounded automatically.
- Apps / SDK / data-mux — spec scope said no.

## Signature + the deno-core mechanism

```rust
// crates/core/ship-substrate/crates/ship-js/src/deno/runtime.rs

pub struct RunLimits {
    pub deadline: Duration,         // wall-clock; ZERO disables
    pub max_heap_bytes: usize,      // V8 cap; ZERO disables
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            deadline: Duration::from_secs(5),
            max_heap_bytes: 256 * 1024 * 1024,
        }
    }
}

pub enum RunError {
    V8(anyhow::Error),
    Terminated {
        reason: TerminateReason,    // WallClockDeadline | NearHeapLimit
        elapsed: Duration,
        source: Option<anyhow::Error>,
    },
}
```

Back-compat: `run_module(...)` / `run_script(...)` keep their original
signatures; they are now thin wrappers around `run_module_with_limits(..,
RunLimits::default())`. Every existing call site (`deck-ssr::run::render`,
`ScriptExecutor`, the `floki::agent_loop` code-mode tool) gets the bounded
behavior automatically.

### How the bound is wired

**Wall-clock deadline** — a DEDICATED OS thread sleeps for `deadline`, then
calls `v8::IsolateHandle::terminate_execution()`. A tokio task is the wrong
shape here: V8 runs synchronously inside the tokio poll, so a
`tokio::time::sleep` on the same runtime is starved and never fires. An OS
thread is the only reliable way to break into a tight JS loop. After the run
returns, the thread is signalled via a `mpsc::Sender<()>` so a clean run
doesn't pay the full `deadline` waiting for the thread.

The `terminate_execution` flag is observed by V8 at the next JS statement
boundary (per V8 docs); on `mod_evaluate` it surfaces as
`Err(CoreError)` ("execution terminated"), which `run_with_deadline` maps to
`RunError::Terminated { reason: WallClockDeadline, elapsed, source: Some(e) }`.

**Heap cap** — `v8::CreateParams::heap_limits(0, max_heap_bytes)` sets the
total heap ceiling at construction time, AND
`set_max_old_generation_size_in_bytes(max_heap_bytes)` caps the OLD generation
separately. Both are required: a long-lived runaway (e.g. a string concat
that grows forever) lives in the old generation, and without constraining it
the script can push past the old-gen default (~2 GB) and trigger V8's hard
OOM abort even though the total heap cap was respected.

The `add_near_heap_limit_callback` captures an `Arc<AtomicBool>` to record
that the heap fired; the callback also calls `terminate_execution()` so V8
settles at the next safe point. The callback returns `current * 2` to honor
V8's contract that the callback RAISE the limit (otherwise V8 hard-aborts);
the `terminate_execution` + the relatively small live-set of a SSR run mean
we don't double forever in practice.

**Reason attribution** — after `work.await`, if it errored AND
`heap_fired.load()` is `true`, the kill is attributed to
`TerminateReason::NearHeapLimit`; otherwise `WallClockDeadline`. This is what
lets tests and observability distinguish a runaway loop from a runaway
allocation.

### Configuration

Per-call via `render_with_limits(bundle, dispatch, RunLimits { ... })`. Host-wide
via env (read at every call by `RunLimits::from_env_or`):

| env var               | unit      | default      |
|-----------------------|-----------|--------------|
| `SHIP_JS_DEADLINE_MS` | ms        | 5000         |
| `SHIP_JS_MAX_HEAP_MB` | MiB       | 256          |

Zero in either field disables that bound (V8 default size / no deadline).

## The reliability contract

`deck-ssr/src/synthetic_ssr.rs::first_paint` is the gate:

```rust
match r.first_paint(req) {
    Ok(html) => Some(html),
    Err(e) => {
        tracing::warn!(view = req.view_name,
            "ship-js SSR failed; falling back to render-eval: {e:#}");
        None
    }
}
```

Any `Err` — including the new `RunError::Terminated` from a bounded
controller — is treated as a fallback trigger to the archived `render-eval`.
This is the exact contract `tests/reliability.rs` already operates under:

> "every target is rendered; a failure records the precise gap (the input that
> scopes the full sweep), never a silent exclusion."

The `tests/reliability.rs` sweep (`slice1_reliability_sweep`) and the warm
latency probe (`slice1_warm_ssr_latency`) both call `render(&bundle, dispatch)`
which is now bounded. Both still pass — the bound's defaults (5 s + 256 MiB)
are loose enough that well-behaved runs complete well within them.

## Runaway test proof

`crates/deck/deck-ssr/tests/runaway.rs` (NEW) — three tests, all green:

1. **`runaway_controller_terminated_within_deadline_does_not_hang_ssr`** —
   the bound's whole reason for existing. Runs a hand-rolled runaway bundle
   (`while (true) { i++; ... }`) through the REAL `render_with_limits` path
   (the one production uses) with a 250 ms deadline. Asserts:
   - `render_with_limits` returns within 5 s (proves no hang).
   - Returns `Err` whose chain contains a `Terminated` (proves the bound fired,
     not a panic / 500 / OOM).

2. **`runaway_heap_allocation_terminated_by_heap_cap`** — runs a string-concat
   runaway with a 30 s deadline + 4 MiB heap cap. Asserts:
   - Returns within 10 s.
   - Returns `Err` with `TerminateReason::NearHeapLimit`.

3. **`engine_run_module_with_limits_terminates_runaway_script`** — the
   focused engine-level proof on `run_module_with_limits` directly. Asserts
   `Err(RunError::Terminated { .. })`.

Plus 5 new unit tests in `ship-js/src/deno/runtime.rs::tests` that lock in
the engine's contract:

- `run_script_wall_clock_deadline_terminates_infinite_loop` — terminates
  `while(true){}` in <200 ms wall-clock.
- `run_script_heap_cap_terminates_unbounded_allocation` — terminates
  `s += "x".repeat(1024)` (8 MiB cap).
- `run_script_well_behaved_completes_with_bound_active` — the bound doesn't
  poison good runs (regression guard for the wall-clock thread + heap
  callback).
- `run_module_wall_clock_deadline_terminates_infinite_main` — the SSR
  engine path.
- `run_module_heap_cap_terminates_unbounded_allocation` — the SSR heap path.
- `run_limits_from_env_or_reads_overrides` — `SHIP_JS_DEADLINE_MS` /
  `SHIP_JS_MAX_HEAP_MB` env contract.

## Test results

```
$ cargo test -p ship-js --features deno
running 11 tests
... all 11 pass in 0.26 s

$ cargo test -p deck-ssr -- --test-threads=1
running 12 tests across ssr / reliability / serve_wired / runaway
... all 12 pass in ~4 s
  reliability: slice1_reliability_sweep, slice1_warm_ssr_latency,
               slice1_hub_closure_component_catalog
  runaway:     runaway_controller_terminated_within_deadline_does_not_hang_ssr,
               runaway_heap_allocation_terminated_by_heap_cap,
               engine_run_module_with_limits_terminates_runaway_script
  ssr:         greet / board / glyph / feed / styled
  serve_wired: wired_serve_gate_selects_ship_js_or_render_eval

$ cargo build -p shipd
... green (1 unrelated dead-code warning in deck-bundler, not from this job).
```

## What this fixes (lineage)

- **`41c2339f6`** — `live()` leak. A controller that calls `live()` in a
  loop without ever disconnecting. Pre-fix: the wire accumulates pending
  ops, the SSR run hangs, the daemon's axum worker thread is starved.
  Post-fix: the wall-clock deadline fires → `run_module` returns `Err`
  → fallback to render-eval.

- **`cbfc22028`** — reconnect storm. A controller that immediately retries
  on `route.current` rejection. Pre-fix: the controller hammers the
  host-dispatch wire, each call allocates, V8 heap grows unboundedly,
  the daemon OOMs the host. Post-fix: the heap cap fires →
  `terminate_execution` → the isolate is killed before it can OOM.

## Things deliberately NOT done

- No app / SDK / data-mux changes (spec scope).
- No change to the SSR engine gate in `synthetic_ssr.rs` (ship-js stays
  default; only the run is bounded, not the gate).
- No change to `ScriptExecutor`'s public surface — the bound applies
  automatically because the underlying `run_module` is bounded.
- File cap of 400 lines respected — `runaway.rs` is ~165 lines, the ship-js
  bound additions are ~280 lines (split across runtime.rs, mod.rs, the
  existing `tests` module).
