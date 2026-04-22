---
title: "Supervisor"
description: "Process management for shipd — executors, isolation, registry, liveness, orphan cleanup, terminal launch. The concrete host-side half the orchestrator injects."
sidebar:
  label: "Supervisor"
  order: 11
---
`apps/shipd/src/supervisor/`. Platform-specific process management. The orchestrator (in the runtime crate) is transport-agnostic and calls through trait objects; the supervisor provides the concrete implementations for shipd — tmux, git worktrees, process spawning, liveness probing, orphan cleanup.

Split is explicit at the top of `supervisor/mod.rs`:

> After the orchestrator extraction, this module contains ONLY platform-specific process management: executors, liveness monitoring, orphan cleanup, process registry, terminal launching, and the platform trait implementations. Job orchestration lives in `runtime::services::orchestrator`.

```
supervisor/
  mod.rs                — module tree, re-exports
  platform.rs           — ShipdPlatform (AgentPlatform impl)
  adapters.rs           — GitWorktreeIsolation, RegistryAdapter
  executors/
    mod.rs              — executor surface
    routing.rs          — RoutingExecutor (per-job executor selection)
    tmux.rs             — TmuxExecutor (legacy single-session)
    tmux_window.rs      — TmuxWindowExecutor (interactive default)
    process.rs          — ProcessExecutor (headless)
    server.rs           — ServerExecutor (long-lived session mode)
  registry.rs           — ProcessRegistry (slug → ExecutorHandle)
  liveness.rs           — start_exit_handler, start_liveness_sweep
  orphan_cleanup.rs     — cleanup_orphaned_worktrees
  agent_lifecycle.rs    — agent process lifecycle events
  process_events.rs     — process.* event emission helpers
  terminal_launcher.rs  — open a terminal into a workspace
  helpers.rs            — worktrees_dir, path helpers
  job_dispatch.rs       — subscribe_job_events (public entry; spawns orchestrator)
  workspace_api.rs      — start_workspace HTTP-adjacent API
  workspace_sync.rs     — workspace event → projection sync
```

## ShipdPlatform

`platform.rs`. Implements `AgentPlatform` — the trait the orchestrator uses to ask the host "where do worktrees go, start a liveness sweep, clean up orphaned worktrees, give me the kernel."

```rust
pub struct ShipdPlatform {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn JobExecutor>,
    pub health_registry: HealthRegistry,
    pub kernel: Arc<Mutex<KernelRouter>>,
}
```

Implementations:

- `worktrees_dir() -> PathBuf` — resolves `{global_dir}/worktrees/`.
- `start_liveness_sweep(kernel)` — tokio task kicking `liveness::start_liveness_sweep` every 30s.
- `cleanup_orphaned_worktrees()` — delegates to `orphan_cleanup::cleanup_orphaned_worktrees`.

## Isolation — GitWorktreeIsolation

`adapters.rs`. Implements `IsolationStrategy`. `prepare(ctx)`:

1. Ensure worktrees directory exists (`worktrees_dir()`).
2. Compute target path `{worktrees_dir}/{slug}`.
3. Run `git worktree add -b {branch} {path}` against the main checkout.
4. Return the worktree path.

`teardown(slug)` runs `git worktree remove --force {path}` and deletes the branch locally. Called by cleanup handlers on `job.merged` / `job.failed`.

`RegistryAdapter` wraps `ProcessRegistry` to implement `ProcessRegistryPort`.

## Executors

One trait, multiple implementations. Routing picks per-job.

### Trait

From `runtime::services::dispatch_ports::JobExecutor`:

```rust
pub trait JobExecutor: Send + Sync {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle>;
    async fn is_alive(&self, handle: &ExecutorHandle) -> bool;
    async fn stop(&self, handle: &ExecutorHandle) -> Result<()>;
    async fn send(&self, handle: &ExecutorHandle, input: &str) -> Result<()>;
}
```

`ExecutorHandle` is opaque; each executor downcasts to its own state struct.

### RoutingExecutor

Default. Per-job routing precedence (highest first):

1. **ServerExecutor** — when the provider adapter declares `caps().server_mode == true` AND the session registry has a bound session for that provider AND `ctx.headless == false`. Server-mode providers hold a persistent session (e.g. codex server, opencode server) and route new jobs to existing sessions rather than spawning fresh processes.
2. **ProcessExecutor** — when `ctx.headless == true`, or when TmuxWindow is unavailable (no tmux binary, Windows).
3. **TmuxWindowExecutor** — default for interactive jobs. Opens a new window inside a shared tmux session so an operator can attach.

Lifecycle calls (`is_alive`, `stop`, `send`) dispatch by inspecting the handle's state type; the router doesn't track which executor spawned what.

### TmuxWindowExecutor

Interactive. Creates a tmux session `ship-{slug}` if missing, opens a window inside it, spawns the agent process in that window with a PTY. Operator attaches with `tmux attach -t ship-{slug}`.

`tmux_window.rs::session_name(slug)` sanitizes `.` and `:` into `_`. The CLI mirrors this function when it prints the attach command so operator instructions match what the executor uses.

### ProcessExecutor

Headless. Plain `tokio::process::Command::spawn`. stdout/stderr piped to the kernel via `process_events`. No PTY. No tmux. Used when `--headless` or when tmux isn't available.

### TmuxExecutor (legacy)

Old single-session interactive mode. Kept behind `SHIP_EXECUTOR=tmux` for users pairing with an agent in a workspace session. One session, all jobs share it. Superseded by TmuxWindowExecutor (window-per-job).

### ServerExecutor

Long-lived session mode. Maintains a connection to a running provider server; new jobs route as requests rather than new processes. Currently scaffolding — registry is empty for every provider; real bindings land in `provider-codex-02` / `provider-opencode-02` per the routing.rs docstring.

## ProcessRegistry

`registry.rs`. `HashMap<slug, ExecutorHandle>`. Used for:

- Graceful shutdown (`graceful_shutdown(registry, executor)`): iterate handles, call `executor.stop(handle)` for each, await all.
- Startup reconciliation: check `executor.is_alive(handle)` for every registered slug; dead handles get their jobs marked Failed via `liveness::drain_dead`.
- Sending input to a running job (`executor.send(handle, input)`).

Mutations (`register`, `unregister`, `drain_dead`) happen under `Arc<Mutex<_>>`.

## Liveness

Two mechanisms, both independent of the agent's own lifecycle events.

### Exit signal handler

`liveness::start_exit_handler(kernel, registry, exit_rx)`. Each dispatched job gets an `UnboundedReceiver<ExitSignal>`. The executor's monitor thread sends on the paired sender when the process/session exits. The handler emits:

- `job.completed` — exit code 0 or unknown.
- `job.failed` — non-zero exit.

This is for non-pipeline jobs (single agent, no steps). Multi-step pipelines route through `orchestrator::step_monitor` instead — same exit signal, different consumer.

### Liveness sweep

`start_liveness_sweep(kernel, registry, probe)`. Background task, 30s interval:

1. `registry.drain_dead` — iterate handles, call `is_alive`; remove dead ones and emit `job.failed` for jobs that never sent an exit signal.
2. Scan the job projection: any job in Dispatched with no activity in >5 minutes is flagged. Can emit `supervisor.drift_detected`.

Liveness doesn't kill anything — it observes and emits. Killing is the job of shutdown or cleanup paths.

## Orphan cleanup

`orphan_cleanup::cleanup_orphaned_worktrees(kernel)`. Startup-only:

1. `load_jobs()` reads the job projection.
2. For each job in `JobStatus::Merged` or `JobStatus::Failed`, check if the worktree still exists.
3. If yes, invoke the standard cleanup handler — delete worktree, remove branch, emit `executor.session_destroyed` with `DestructionCause::Startup`.

Idempotent. Catches worktrees left behind by crashes mid-cleanup.

## Process events

`process_events.rs`. Emit helpers for the `process.*` namespace:

- `emit_process_spawned(kernel, slug, pid)` — emits `process.spawned`.
- `emit_process_exited(kernel, slug, pid, exit_code)` — emits `process.exited`.
- `emit_process_signal_sent(...)` — when the executor sends a signal.
- `emit_process_orphaned(...)` — when liveness detects a process whose parent is gone.
- `emit_process_killed(...)` — when graceful-shutdown terminates a process.

These feed the Process projection (computed, no table) — `load_processes(slug)` folds them.

## Terminal launcher

`terminal_launcher.rs`. `launch_terminal(workspace, provider)` opens a terminal attached to the workspace's tmux session for interactive use. Called by `ship` CLI commands that need to drop the operator into an agent session.

## Workspace API + sync

- `workspace_api.rs` — `start_workspace(...)` HTTP-adjacent helper used by handlers that start a workspace's first session.
- `workspace_sync.rs` — `subscribe_workspace_events(...)` spawns a workspace-sync actor subscribed to `workspace.*` that forwards events into the workspace projection.

## subscribe_job_events (public entry)

`job_dispatch.rs`. The daemon calls `subscribe_job_events(kernel, ship_dir, health_registry)` at startup. Builds an executor (`RoutingExecutor` by default, or `TmuxExecutor` if `SHIP_EXECUTOR=tmux`), constructs an `OrchestratorContext` with shipd's concrete traits, and spawns the orchestrator:

```rust
let octx = Arc::new(OrchestratorContext {
    isolation: Arc::new(GitWorktreeIsolation),
    executor: executor.clone(),
    platform,
    registry: Arc::new(RegistryAdapter(registry.clone())),
    probe: Arc::new(SysinfoProcessTreeProbe::new()),
    done_flags: Arc::new(Mutex::new(HashMap::new())),
    activity_txs: activity_txs.clone(),
    job_locks: Arc::new(Mutex::new(HashMap::new())),
    event_dedup: Arc::new(EventDedup::new()),
});
spawn_monitored(health_registry, "job-orchestrator", async move {
    orchestrator::router::start_orchestrator(kr, octx).await;
});
```

Returns a `DispatchHandle { registry, executor, activity_txs }` which the daemon uses for graceful-shutdown and for the API-to-step-monitor signal channel.

## Related

- [Orchestrator](/reference/runtime/orchestrator/) — the transport-agnostic half that uses these traits.
- [Events](/reference/runtime/events/) — `supervisor.`, `process.`, `executor.` namespaces.
- [Projections](/reference/runtime/projections/) — Process records fold from `process.*`.
