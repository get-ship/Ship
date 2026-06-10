# Job S15: resilience + event-log emit (Phase 6)

## Context
Work in `spikes/ship-agent/`. README is SSOT. This is the first **Phase 6
(harden + graduate)** slice. It adds two cohesive, self-contained primitives —
no invasive loop rewrite, so the existing behavioral suite stays green.

Today:
- `events.rs` owns `AgentEvent`, the `EventSink` trait, `CollectingSink`
  (test/inspection), `TerminalSink` (stdout), and the pure `translate`.
- `spine.rs::run(label, execute, verify)` is the post-exec verify: `Execute`
  failures and `Unverified` refusals both return `SpineError`; the loop folds a
  denial in as data, an error/refusal as a refusal.

What's missing for "harden + graduate": (1) a **durable, replayable** record of
a run (event-sourcing — the daemon side will be `ship_subscriber`'s event log;
S14's design already names this as where the parent awaits child events), and
(2) **resilience** — a transient execution failure should not kill a run; a
real denial or refusal still must not be retried.

## Task — two primitives + their seam

### 1. `EventLog` (in `events.rs`) — durable, append-only emit
- `LoggedEvent { seq: u64, event: AgentEvent }`.
- `EventLog`: an `EventSink` whose `emit` assigns a **monotonic sequence
  number** (deterministic — NO wall-clock, so tests assert order not timestamps)
  and appends `LoggedEvent`. `replay() -> Vec<LoggedEvent>` returns the ordered
  snapshot. This is the event-sourced record; in the daemon it becomes a
  persistent log, here it's in-memory behind the same `EventSink` seam.

### 2. `resilience::run_resilient` (new `src/resilience.rs`) — retry the transient, never the terminal
- `RetryPolicy { max_attempts: u32 }`.
- Classify `SpineError`: `Execute` is **transient** (retryable — a flaky
  dispatch); `Denied` and `Unverified` are **terminal** (NEVER retried — a
  denial is data, an unverified result is a real refusal; retrying either is
  wrong and would mask a real signal).
- `async fn run_resilient(sink, label, policy, make_execute, verify)`:
  re-invokes `spine::run` up to `max_attempts` on transient failure, emitting a
  `Status` event per retry into the sink; returns immediately on `Ok` or on a
  terminal error; emits `Error` and returns the last error when retries are
  exhausted. The resilience narrative is thus recorded in the event log.

## Invariants (must hold)
- Resilience NEVER weakens the gate or the verify: `Denied`/`Unverified` are
  terminal. Retry only ever re-runs a *transient execute*, never re-decides the
  gate and never re-accepts a refused result.
- Determinism: the event log orders by an internal sequence counter, not time,
  so behavioral tests assert order/semantics, never timestamps or counts of
  retries-as-magic-numbers.
- `EventLog` is just an `EventSink` — every existing emit path works unchanged;
  this is additive.

## Acceptance (behavioral tests — semantics, not brittle)
- `EventLog`: events are replayed in emission order with strictly increasing
  seq; an event emitted later has a higher seq than one emitted earlier.
- `run_resilient`: a transient `Execute` that succeeds on a later attempt
  returns `Ok`, and the log shows a retry `Status` before the success.
- `run_resilient`: a `Denied` returns immediately and is NOT retried (no retry
  `Status` in the log — exactly one attempt).
- `run_resilient`: an `Unverified` is likewise terminal (not retried).
- `run_resilient`: a persistently-transient `Execute` exhausts retries, returns
  `Err(Execute)`, and the log ends with an `Error` event.

`cargo test` in `spikes/ship-agent`: all existing (124) + new pass.

## Constraints
- Do NOT change `gate.rs::decide`, the one-door dispatch, `auth.rs`,
  `runtime.rs`, `module_loader.rs`. `spine.rs` may gain a small classify helper
  but its `run` semantics are unchanged. Primary new code: `src/resilience.rs`
  + the `EventLog` addition to `events.rs`.
- Behavioral tests only. Update README (module map / Seams / arc).

## Decision log
- **Retry only `Execute`** — a transient dispatch/IO failure is the only thing
  safe to retry. Re-running a `Denied` could surface a different answer (it
  won't — the gate is deterministic — but retrying *signals* the wrong thing);
  re-running an `Unverified` would risk accepting a result the verifier refused.
  Both are terminal by construction.
- **Sequence, not timestamp** — keeps the log deterministic and the daemon swap
  (persistent log with real clocks) a transport detail, not a semantic one.
- **Additive, behind the `EventSink` seam** — no loop rewrite; wiring
  `run_resilient` into `run_step`'s tool dispatch is the follow-on (and is where
  S16 graduation will point it at the real `ship_subscriber` log).

## Done = on the branch, green
Commit when green; this slice lands directly on the integration branch.
