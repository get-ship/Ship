---
group: Runtime
order: 9
title: Orchestrator
description: Pipeline advancement, step dispatch, gate retries, replay on startup. The long-running service actor that turns job events into agent work.
audience: internal
---

# Orchestrator

`crates/core/ship-platform/src/services/orchestrator/`. The service actor that consumes `job.*` and `step.*` events, advances pipelines step by step, and dispatches agent processes. Transport-agnostic — it calls back to the host (shipd) through injected trait objects.

```
orchestrator/
  mod.rs              — OrchestratorContext, StepActivity, module tree
  router.rs           — start_orchestrator + handle_job_event (event dispatch)
  dispatch.rs         — handle_job_created, dispatch_job_internal
  pipeline.rs         — advance_pipeline, step selection
  pipeline_loader/    — ensure_pipeline (parse JSONC into the payload)
  pipeline_specs.rs   — materialize .artifacts/pipeline/* inside the worktree
  prompt_composer.rs  — compose dispatch prompts for agents
  step_executor.rs    — drive a single step through the executor
  step_io.rs          — step input/output plumbing
  step_monitor/       — watch process exit, emit step_completed/failed
  gate_rules.rs       — gate step logic (pass/fail/retry)
  review_retry.rs     — re-dispatch failed gated steps
  run_step.rs         — shell-command step type
  fanout.rs           — fan-out step expansion
  fanout_aggregate.rs — aggregate fan-out branch results
  fanout_cancel.rs    — cancel outstanding fan-out branches
  fanout_recover.rs   — recover fan-out state on restart
  isolation.rs        — isolation-strategy helpers
  exit_handler.rs     — process-exit handling for non-pipeline jobs
  replay.rs           — reconcile + replay on daemon startup
  pending.rs          — dependency-blocked job queue
  sweep.rs            — periodic coherence checks (30s)
  dedup.rs            — event-id dedup to prevent double-advancement
  emit.rs             — helper wrappers around kernel emission
  handlers.rs         — misc event handlers (cancel, update, etc.)
  git_helpers.rs      — worktree-relative git commands
```

## OrchestratorContext

Bundle of injected strategies. Host (shipd) constructs it; orchestrator reads it.

```rust
pub struct OrchestratorContext {
    pub isolation: Arc<dyn IsolationStrategy>,     // GitWorktreeIsolation
    pub executor: Arc<dyn JobExecutor>,            // RoutingExecutor / TmuxWindowExecutor
    pub platform: Arc<dyn AgentPlatform>,          // ShipdPlatform
    pub registry: Arc<dyn ProcessRegistryPort>,    // RegistryAdapter
    pub probe: Arc<dyn ProcessTreeProbe>,          // SysinfoProcessTreeProbe
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    pub activity_txs: Arc<Mutex<HashMap<String, UnboundedSender<StepActivity>>>>,
    pub job_locks: Arc<Mutex<HashMap<String, ()>>>,
    pub event_dedup: Arc<dedup::EventDedup>,
}
```

Everything the orchestrator needs from the outside world is behind a trait. The core logic doesn't know what a tmux window is or how git worktrees work — it tells `isolation.prepare(ctx)` and gets back a path.

## Ports

Defined in `ship_substrate::services::dispatch_ports`:

| Trait | Concrete implementation | Role |
|---|---|---|
| `IsolationStrategy` | `GitWorktreeIsolation` | `prepare(JobContext) -> PathBuf`, creates a git worktree for the job. |
| `JobExecutor` | `RoutingExecutor`, `TmuxWindowExecutor`, `ProcessExecutor` | `spawn(JobContext) -> ExecutorHandle`. Launches the agent process. |
| `AgentPlatform` | `ShipdPlatform` | Host-side capabilities: worktrees dir, liveness sweep, orphan cleanup, kernel access. |
| `ProcessRegistryPort` | `RegistryAdapter` | Register/unregister running processes for query and cleanup. |
| `ProcessTreeProbe` | `SysinfoProcessTreeProbe` | Sample a process's entire tree for liveness + orphan detection. |

## start_orchestrator

```rust
pub async fn start_orchestrator(
    kernel: Arc<Mutex<KernelRouter>>,
    octx: Arc<OrchestratorContext>,
)
```

Entry point. Called once by the daemon at startup inside `supervised::spawn_monitored`.

1. Registers `service.job-dispatch` actor with the kernel:

   ```rust
   ActorConfig {
       namespace: "service.job-dispatch",
       write_namespaces: vec!["job.", "step."],
       read_namespaces: vec![],
       subscribe_namespaces: vec!["job.", "step."],
   }
   ```

2. Starts `platform.start_liveness_sweep(kernel)`.

3. Spawns startup tasks concurrently:
   - `replay::reconcile_dispatched_jobs` — detect jobs stuck in Dispatched across a restart.
   - `replay::replay_unprocessed_completions` — re-advance pipelines whose completion event landed but wasn't acted on.
   - `pending::dispatch_ready_pending_jobs` — jobs whose dependencies completed while the daemon was down.
   - `platform.cleanup_orphaned_worktrees` — remove worktrees for jobs that no longer exist.

4. Spawns the periodic coherence sweep (`sweep::start_coherence_sweep`, 30s interval).

5. Enters the main loop: `while let Some(envelope) = mailbox.recv().await { handle_job_event(...) }`.

Returns when the mailbox closes (daemon shutdown).

## handle_job_event

`router.rs`. Two guards before dispatch:

- **Dedup**: `octx.event_dedup.record(&envelope.id)` drops duplicates. Causes: mailbox redelivery, startup replay racing live delivery, tests replaying.
- **Terminal-state guard**: if `load_job_record(entity_id).status` is `Failed | Merged | Cancelled`, drop non-operator events. Late-arriving step signals can't resurrect a dead job. Operator events (`job.created`, `job.update`, `job.cancelled`) are exempt — they're signals *from* the operator, not pipeline progression.

Then dispatch by event type:

| Event | Handler | Effect |
|---|---|---|
| `job.created` | `dispatch::handle_job_created` | Parse payload, check deps, dispatch first step. |
| `job.update` | `handlers::handle_job_update` | Metadata update, no pipeline advance. |
| `job.cancelled` | `handle_job_cancelled` | Cleanup + emit cancellation chain. |
| `job.completed` | `advance_pipeline` + maybe `auto_merge` + `dispatch_ready_pending_jobs` | One step (or all) done — advance, maybe merge, wake deps. |
| `job.step_completed` | `advance_pipeline`; if no next step, `finalize_pipeline_if_done` | Within-pipeline step done. |
| `job.step_failed` | log + unregister | Pipeline halts at the failed step. |
| `job.merged` | `handle_job_cleanup` + unlock + dispatch pending | Branch merged, release job lock. |
| `job.failed` | cleanup registries (registry, done_flags, activity_txs) | Terminal. |

## handle_job_created

`dispatch.rs`. Parses `JobCreatedPayload`. If `depends_on` is non-empty, defers — the job sits pending until a dep completion wakes it via `dispatch_ready_pending_jobs`.

Otherwise calls `ensure_pipeline(payload)` (idempotent — already called by the HTTP handler before persist) then `dispatch_job_internal`.

## dispatch_job_internal

Branches on step type. First-step `Run` → `dispatch_first_run_step` (shell command, no agent). First-step agent →:

1. Resolve agent (first step's `agent` field or job-level `agent`).
2. Resolve harness/model (first step's or payload's).
3. Build `JobContext`:

   ```rust
   JobContext {
       job_id, slug, agent, branch, spec_path, spec_content,
       work_dir: octx.platform.worktrees_dir().join(slug),
       env: [("SHIP_MESH_ID", slug)].into(),
       launch: LaunchProfile { harness, model, persist_session: true },
       step: Some(StepInfo { index: 0, id }),
       headless,
       exit_tx: None,  // filled after isolation
   }
   ```

4. `octx.isolation.prepare(&ctx).await` — creates the git worktree on the job's branch, returns the path.
5. Materialize `.artifacts/pipeline/*` in the worktree:
   - `pipeline_specs::write_pipeline_specs` — overall spec + per-step bodies.
   - `pipeline_specs::write_dispatch_prompt` — prompt at `spec_path` pointing the first agent at its step file.
6. Create an `mpsc::unbounded_channel` for exit signals; attach `exit_tx` to the context.
7. `octx.executor.spawn(&ctx).await` — spawns the process.
8. Start a step monitor watching `exit_rx` — when the process exits, the monitor emits `job.step_completed` or `job.step_failed`.

Failures at any step emit `job.spawn_failed` and exit without partial cleanup — the next restart's replay handles it.

## advance_pipeline

`pipeline.rs`. Given a completion event, find the next unblocked step. DAG traversal over `payload.steps` — `Pipeline` and `StepType` come from `ship_substrate::events::job` (there is no separate `ship-jobs` crate):

1. Update completed-steps set with the event's step id.
2. Find steps whose `depends_on` is fully in completed-steps, not yet running, not yet completed.
3. If any such steps, pick the first (deterministic order), dispatch it. Return `true`.
4. If none, return `false` — caller checks auto_merge or finalizes.

Dispatch of a subsequent step reuses `step_executor::execute_step` rather than the first-step path, because the worktree already exists.

## Gates + retries

`gate_rules.rs`, `review_retry.rs`. A gate step observes another step's output and emits `gate.passed` or `gate.failed`. A failed gate triggers `review_retry::retry_gated_step` which increments `step_retries[step_id]` in the projection and re-dispatches the gated step up to `max_retries`. After max retries, the gate failure propagates to the job as `job.step_failed`.

## Replay on startup

Three phases in `replay.rs`:

- `reconcile_dispatched_jobs` — jobs with status Dispatched but no live process. Either the daemon died mid-dispatch, or the agent crashed without emitting step_failed. Probe the process tree; if dead, emit `job.step_failed`.
- `replay_unprocessed_completions` — `job.step_completed` events that landed in the log but whose advance never fired (daemon crash between persist and projection update). Re-apply advance.
- `dispatch_ready_pending_jobs` — jobs whose `depends_on` is now satisfied but that were never dispatched because the completion happened while the daemon was down.

## Coherence sweep

`sweep.rs`. Every 30s: walk live jobs, check their processes are alive, compare projection state against event log. Drift detection fires `supervisor.drift_detected`; repair fires `supervisor.coherence_restored`. Sweeps can also kill orphaned worktrees left by crashed jobs.

## Step monitor

`step_monitor/`. Per-step task watching `exit_rx`. Receives `ExitSignal { slug, job_id, exit_code, cause }`:

- `ExitCause::AgentExited` + `exit_code == Some(0)` → emit `job.step_completed`.
- `AgentExited` + non-zero exit → emit `job.step_failed`.
- `SessionDestroyed` → emit `job.step_failed` with a distinguishable reason. The substrate went away without the agent's own exit observed.

Also consumes `StepActivity` from `octx.activity_txs` — Stop hooks from harnesses send `TurnEnded` so the monitor can distinguish interactive mode (many turns) from headless (exit = done).

## Template variables

`pipeline_vars.rs`. Interpolates `{{ var }}` expressions in step commands and prompts. Variables come from three sources merged in order:

1. Pipeline defaults (`pipeline.vars` in the pipeline file).
2. CLI flags at dispatch (`ship run --var k=v`).
3. Job-level vars baked into the `JobCreatedPayload`.

Same templating engine as skill vars (MiniJinja), same limitations (no includes, no file loader, bounded scope).

## Dedup

`dedup.rs`. In-memory set of seen event ids with bounded capacity. Same event id → `Seen::Duplicate`, skipped. Prevents pipeline double-advancement from mailbox redelivery, startup-replay racing live-delivery, and test harness replays.

## Emit helpers

`emit.rs` wraps common kernel emission patterns with the right `EmitContext` and event type:

- `emit_job_failed(kernel, job_id, slug, reason)`
- `emit_spawn_failed(kernel, job_id, slug, phase, error)`
- `emit_step_completed(kernel, envelope, step_id)`
- `emit_step_failed(kernel, envelope, step_id, reason)`

Callers don't hand-build envelopes for these common cases.

## Related

- [Kernel](/architecture/kernel/) — where orchestrator events flow.
- [Events](/runtime/events/) — event types the orchestrator emits and consumes.
- [Supervisor](/runtime/supervisor/) — process dispatch and executor plumbing shipd hosts.
- [Mesh](/runtime/mesh/) — how the orchestrator's peer actors route inter-agent messages.
