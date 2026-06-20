# Theme C/D surface-cleaning — ORCHESTRATOR (omp): seam-doc · D3 · D2 · D12

DISPATCH after batch-1 merges (one omp job at a time). Orchestrator: commit per item, build green,
NEVER push. Verify-then-act (locate by symbol — lines drift); skip+report if a finding doesn't hold.
Stay OUT of view/deck/sui/reactive crates (live worktrees).

## 1. Theme-C seam-documentation pass (NO CUTS — comments only)
For each ratified-but-unbuilt seam, add a clear `// DEFERRED SEAM — keep (<why>)` comment so future
clearing passes don't mistake it for dead code, and fix any stale/misleading comment. Do NOT remove
or change behavior.
- `durable-execution` `load_all_incomplete` (store.rs:53) — floki Stage-3 resume seam.
- `ship-substrate/crates/ship-pipelines` `PipelineStepDef::spec` (definition.rs:23) — P0.2 spec seam; note positional `step_specs` retires later.
- `ship-device` `SandboxLifetime::PerRun` (types.rs:54) — no reaper yet; fix the misleading `tests.rs:188` comment.
- `agent-def` harness const vs registry (harnesses.rs:92 — HARNESSES(2) vs REGISTRY(5), "Open Call #3") — add a note that the const is intentionally narrow pending a decision (do NOT expand/cut).
- `ship-js` single VM thread (script_executor.rs:14) — Phase-D pool seam.

## 2. D3 — remove dead L0 aggregate
`durable-execution/src/engine.rs:362` `AggregateDecision`/`aggregate` are re-exported but never
called; the canonical fanout aggregation is L1 `ship_pipelines::fanout`. Remove the dead L0 copy.
Fix the `parallel` docstring link first if it points at the removed item.

## 3. D2 — remove dead kernel bypass routes
`ship-kernel/src/router.rs:168` `send_to_actor`/`mailbox_sender` are bypass-the-route-pipeline
methods with zero callers. Remove them. LEAVE the inner `with_validator` layer alone (that's a
flagged triage, not this batch).

## 4. D12 — relocate agent-schema helpers out of the fs crate
`ship-files/src/jsonc.rs:18` holds agent-schema jsonc helpers in the wrong crate. Move them to
`ship-agents` (or the agent-def crate if that's the true owner — pick by who consumes them); update
imports. Pure relocation, no behavior change.

## Acceptance
One commit per item (seam-doc = one commit). `cargo check --workspace --exclude ship-host --features
unstable` green; affected tests pass. NO push.
