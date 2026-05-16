---
group: Runtime
order: 14
title: Daemon (shipd)
description: Startup sequence, ShipRuntime, HTTP router, service actor lifecycle, pidfile/portfile management, shutdown.
audience: internal
---

# Daemon

`apps/shipd/`. The Axum HTTP server hosting the runtime. Single process, binds `127.0.0.1:9315` by default, stays up across CLI invocations. Every transport (CLI, MCP, shell, Studio) talks to it over HTTP.

Entry point: `shipd::run_network(host, port, dev, workspace_dir, daemon_mode)` in `apps/shipd/src/lib.rs`. `daemon_mode` controls cache headers, HMR route availability, and SSR caching — there is no `with_ui` flag.

## Startup sequence

1. **Dev-mode warning** (if `dev = true`): loud banner, force localhost bind, skip persistent session hydration.
2. **Cleanup stale daemon files** — `daemon::cleanup_stale_daemon_files(&global_dir)`. Removes pidfile/portfile from a prior crashed daemon.
3. **Log mode**: project-scoped (workspace_dir given) vs global-only.
4. **Construct `ShipRuntime`**:

   ```rust
   let meta = AppMetadata { global_dir, workspace_dir };
   let config = RuntimeConfig { host, port, ..Default::default() };
   let mut runtime = ShipRuntime::new(meta, config).await?;
   ```

   `ShipRuntime::new` initializes `platform.db`, sets up `KernelRouter` (wrapping `ship_kernel::KernelRouter` with `SqliteEventStore` as the `EventSink`), sets up the health registry and session registry.

5. **Spawn mesh service**: `transport::connections::spawn_mesh_service(kernel, health_registry)` returns a `SharedMeshRegistry`. `runtime.set_mesh_registry(registry)`.
6. **Init `JobManager`**: `managers::JobManager::init(...)` runs `subscribe_job_events` which spawns the orchestrator. Returns a `DispatchHandle`; runtime holds it via `set_dispatch_handle`.
7. **Start session scanner**: periodic task that detects abandoned sessions.
8. **Spawn human gateway**: `daemon::human_gateway::spawn_human_gateway(kernel, health_registry)` (only if Telegram env vars are set).
9. **Token management**: read existing token from `~/.ship/machine/daemon/` if present, otherwise generate a new one and write it.
10. **Write pidfile + portfile**: under `~/.ship/machine/daemon/`.
11. **Spawn shutdown listener**: tokio task waiting for SIGINT/SIGTERM. Cancels a `CancellationToken`.
12. **Hydrate paired sessions** (not in dev): `auth::hydrate_sessions(&session_registry)` restores sessions from prior daemon lifetimes.
13. **Build router**: `router::build_router(...)` with `daemon_mode`.
14. **Bind TCP**: `TcpListener::bind("{host}:{port}")`. On failure, emit `daemon.port_bind_failed` with `already_bound_by_pid` if another daemon holds the pidfile, then return error.
15. **Emit `daemon.startup_completed`**: includes pid, port, version, commit_sha, event count at startup, latest event id, init duration.
16. **Serve** until the cancellation token fires.
17. On shutdown: emit `daemon.shutdown_completed`, clean up pidfile/portfile.

The `workspace` runtime primitive was retired — there is no workspace-event subscription step.

## ShipRuntime

`apps/shipd/src/ship_runtime.rs`. Central handle passed to every HTTP handler via `ApiState`.

Key fields:

- `kernel: SharedKernel` — `Arc<Mutex<KernelRouter>>`. Single write path.
- `mesh_registry` — `SharedMeshRegistry` for REST read access.
- `agent_mailboxes` — map of mailboxes exposed to MCP bridge.
- `pty_connections` — per-branch PTY connection counter for rate limiting.
- `ship_dir: PathBuf` — global Ship dir.
- `session_registry` — paired session tokens.
- `health_registry` — supervised task health for all spawned actors.
- `data_service` — lazy `DataService` for `data.db` access.
- `view_sessions` — view rendering session registry.
- `dispatch_handle` — `DispatchHandle` from supervisor's `subscribe_job_events`.
- Several `OnceCell` / `OnceLock` fields for lazy init (mesh, jobs, session_scanner, dispatch_handle).

`persist_and_route(&envelope)` is the single write path handlers use to put an event through the kernel. Delegates to `kernel.lock().await.emit(envelope, ctx)`. The daemon doesn't expose raw sink access.

## HTTP router

`apps/shipd/src/router.rs` composes all route modules from `apps/shipd/src/routes/*` under `/api/v1/*`:

- `runtime_api.rs` — sessions, jobs, events.
- `rest_api.rs` — mesh register/send/broadcast/discover/status.
- `actor_api.rs`, `actors_api.rs` — actor lifecycle and listing.
- `artifacts_api.rs` — artifact registry.
- `collection_api.rs` — collections (`data.db`).
- `specs_api.rs`, `settings_api.rs`, `projects_api.rs` — domain APIs.
- `annotations_api.rs`, `feedback_api.rs` — canvas interaction.
- `view_sessions_api.rs`, `view_errors_api.rs` — view rendering APIs.
- `human_webhook.rs` — inbound replies from human gateway adapters.
- `mcp_api.rs` — MCP-over-HTTP endpoint.
- `shell_assets.rs` — serves bundled shell/Studio assets.

The PTY WebSocket bridge is `apps/shipd/src/transport/pty.rs`; auth (`auth/api/pairing.rs`, `auth/api/attest.rs`, `auth/api/sessions.rs`) handles session registration and capability grant. Every handler runs through auth middleware that populates `AuthContext`. Scope checks are per-handler.

## Connections module

`apps/shipd/src/transport/connections.rs`. Spawns long-lived connection-backed services:

- `spawn_mesh_service(kernel, health_registry)` — launches `MeshService` as a service actor. Returns the `SharedMeshRegistry`.

PTY connection counts and agent mailbox bookkeeping live on `ShipRuntime` directly.

## Managers

`apps/shipd/src/managers/`. Thin wrappers that own the lifecycle of heavier runtime pieces:

- `JobManager::init(kernel, ship_dir, health_registry)` calls `supervisor::job_dispatch::subscribe_job_events` and holds the returned `DispatchHandle`. Used for graceful shutdown + activity signal forwarding.

## Daemon events

`ship_substrate::events::daemon`. The daemon emits its own lifecycle events:

| Event | Meaning |
|---|---|
| `daemon.startup_completed` | Startup succeeded. Payload: pid, port, version, commit, event count, latest event id, init duration. |
| `daemon.shutdown_completed` | Clean shutdown. |
| `daemon.crashed` | Emitted by best-effort panic hook when available. |
| `daemon.port_bind_failed` | Bind failed. Payload includes the pid of whoever's holding the port if known. |
| `daemon.pidfile_acquired` | Wrote a fresh pidfile. |
| `daemon.pidfile_stale_taken` | Reclaimed a pidfile left by a crashed prior daemon. |

These events go through the kernel like any others — they're in the `events` table, visible to projections and sync.

## Pidfile + portfile

Written under `~/.ship/machine/daemon/` (`pid` and `port`) at startup, cleaned at shutdown. `daemon::cleanup_stale_daemon_files` detects crashed prior daemons by checking if the pid in the file is still alive; if not, reclaims both files.

`pidfile_pid_for_port(global_dir, port)` is the function used in the bind-failure path — it reads the pidfile to name whoever currently holds the port, so the error message is actionable.

## Per-worktree Vite isolation

Each worktree runs its own Vite dev server on a distinct port (`3200 + (daemon_port - 9315)`). The launcher script `scripts/dev/shell-start.sh` (invoked by `just shell-dev`) records the Vite process PID at `{daemon_dir}/vite.pid` alongside the daemon's own `pid` and `port` files.

This enables coordinated cleanup:

- `just dev-stop` — kills both the daemon and the tracked Vite process for the current worktree.
- `just dev-restart` — stops daemon + Vite, reaps any orphan Vite processes running from deleted worktree paths, then rebuilds and restarts.
- `just dev-stop-all` — kills every daemon and Vite process tracked across all worktrees.
- `just worktree-remove <slug>` — reaps daemon + Vite for the named worktree, then calls `git worktree remove`. Use `-- --force` to force-remove a worktree with uncommitted changes.

The orphan-reaping pass in `dev-restart` walks all `~/.ship/machine/daemons/*/vite.pid` files and kills any Vite process whose `/proc/<pid>/cwd` resolves to a path that no longer exists. This prevents stale Vite processes (serving bytes from a deleted worktree) from persisting and conflicting with subsequent dev sessions.

## Shutdown

Graceful shutdown fires on SIGINT or SIGTERM. Flow:

1. Cancellation token fires → Axum drains in-flight requests.
2. `graceful_shutdown(registry, executor)` — iterate registered processes, `executor.stop(handle)` each, await all.
3. Emit `daemon.shutdown_completed`.
4. Clean up pidfile + portfile.

## Dev mode

`dev: bool` relaxes auth for local development:

- Forces localhost bind (override-proof — agents can't flip this).
- Auto-pairs browser clients (Studio connects without interactive pairing flow).
- Skips session hydration from prior daemon lifetimes.

Banner printed on startup. Never set `dev = true` in production.

## Network status / stop

From the daemon module, re-exported on the crate root:

- `network_status(global_dir)` — reports pid/port from pidfile, alive-check.
- `network_stop(global_dir)` — reads pidfile, sends SIGTERM.

Used by CLI subcommands (`ship daemon status`, `ship daemon stop`).

## Related

- [Scopes](/runtime/scopes/) — auth middleware and token types.
- [Supervisor](/runtime/supervisor/) — everything the orchestrator needs from the host.
- [Orchestrator](/runtime/orchestrator/) — started by `JobManager::init` → `subscribe_job_events`.
- [Mesh](/runtime/mesh/) — spawned by `spawn_mesh_service`.
- [PTY Bridge](/runtime/pty-bridge/) — `/api/v1/projects/:id/pty` handler.
