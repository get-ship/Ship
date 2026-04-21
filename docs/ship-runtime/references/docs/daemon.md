---
group: Runtime
order: 14
title: Daemon (shipd)
description: Startup sequence, ShipRuntime, HTTP router, service actor lifecycle, pidfile/portfile management, shutdown.
audience: internal
---

# Daemon

`apps/shipd/`. The Axum HTTP server hosting the runtime. Single process, binds `127.0.0.1:9315` by default, stays up across CLI invocations. Every transport (CLI, MCP, shell, Studio) talks to it over HTTP.

Entry point: `shipd::run_network(host, port, dev, with_ui, workspace_dir)` in `apps/shipd/src/lib.rs`.

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

5. **Spawn mesh service**: `connections::spawn_mesh_service(kernel, health_registry)` returns a `SharedMeshRegistry`. `runtime.set_mesh_registry(registry)`.
6. **Subscribe workspace events**: `supervisor::subscribe_workspace_events(kernel, global_dir, health_registry)` — workspace projection sync actor.
7. **Init `JobManager`**: `managers::JobManager::init(...)` runs `subscribe_job_events` which spawns the orchestrator. Returns a `DispatchHandle`; runtime holds it via `set_dispatch_handle`.
8. **Start session scanner**: `runtime.session_scanner()` — periodic task that detects abandoned sessions.
9. **Spawn human gateway**: `human_gateway::spawn_human_gateway(kernel, health_registry)` (only if Telegram env vars are set).
10. **Token management**:
    - Read existing token from `{global_dir}/daemon.token` if present.
    - Otherwise generate new token via `auth::generate_token`, write to file.
11. **Write pidfile + portfile**: `daemon::write_port_file`, `daemon::write_pid_file`.
12. **Spawn shutdown listener**: tokio task waiting for SIGINT/SIGTERM. Cancels a `CancellationToken`.
13. **Hydrate paired sessions** (not in dev): `auth::hydrate_sessions(&session_registry)` restores sessions from prior daemon lifetimes.
14. **Build router**: `router::build_router(runtime, token, session_registry, pairing_registry, attestation_verifier, child_token, with_ui)`.
15. **Bind TCP**: `TcpListener::bind("{host}:{port}")`. On failure, emit `daemon.port_bind_failed` with `already_bound_by_pid` if another daemon holds the pidfile, then return error.
16. **Emit `daemon.startup_completed`**: includes pid, port, version, commit_sha, event count at startup, latest event id, init duration.
17. **Serve** until the cancellation token fires.
18. On shutdown: emit `daemon.shutdown_completed`, clean up pidfile/portfile.

## ShipRuntime

`apps/shipd/src/ship_runtime.rs`. Central handle passed to every HTTP handler via `ApiState`.

Key fields:

- `kernel: SharedKernel` — `Arc<Mutex<KernelRouter>>`. Single write path.
- `mesh_registry` — `SharedMeshRegistry` for REST read access.
- `agent_mailboxes` — map of mailboxes exposed to MCP bridge.
- `pty_connections` — per-workspace PTY connection counter for rate limiting.
- `ship_dir: PathBuf` — global Ship dir.
- `session_registry` — paired session tokens.
- `health_registry` — supervised task health for all spawned actors.
- `data_service` — lazy `DataService` for `data.db` access.
- `view_sessions` — view rendering session registry.
- `dispatch_handle` — `DispatchHandle` from supervisor's `subscribe_job_events`.
- Several `OnceCell` / `OnceLock` fields for lazy init (mesh, jobs, session_scanner, dispatch_handle).

`persist_and_route(&envelope)` is the single write path handlers use to put an event through the kernel. Delegates to `kernel.lock().await.emit(envelope, ctx)`. The daemon doesn't expose raw sink access.

## HTTP router

`apps/shipd/src/router.rs` composes all route modules under `/api/*`:

- `runtime_api.rs` — workspaces, sessions, jobs, events CRUD.
- `actor_api.rs` — actor lifecycle.
- `artifacts_api.rs` — artifact registry.
- `collection_api.rs` — collections (`data.db`).
- `specs_api.rs`, `settings_api.rs`, `project_api.rs` — domain APIs.
- `annotations_api.rs`, `feedback_api.rs` — canvas interaction.
- `pty_handler.rs` — WebSocket PTY bridge.
- `view_sessions_api.rs`, `views/*` — view rendering and manifest APIs.
- `human_webhook.rs` — inbound replies from human gateway adapters.
- `auth`, `pairing`, `attest` — session registration and capability grant.
- `mcp_apps_bridge.rs` — MCP-over-HTTP endpoint.
- `shell_assets` — serves bundled shell/Studio assets when `with_ui`.

Every handler runs through auth middleware that populates `AuthContext`. Scope checks are per-handler.

## Connections module

`apps/shipd/src/connections.rs`. Spawns long-lived connection-backed services:

- `spawn_mesh_service(kernel, health_registry)` — launches `MeshService` as a service actor. Returns the `SharedMeshRegistry`.

PTY connection counts and agent mailbox bookkeeping live on `ShipRuntime` directly.

## Managers

`apps/shipd/src/managers/`. Thin wrappers that own the lifecycle of heavier runtime pieces:

- `JobManager::init(kernel, ship_dir, health_registry)` calls `supervisor::job_dispatch::subscribe_job_events` and holds the returned `DispatchHandle`. Used for graceful shutdown + activity signal forwarding.

## Daemon events

`runtime::events::daemon`. The daemon emits its own lifecycle events:

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

`{global_dir}/shipd.pid` and `{global_dir}/shipd.port`. Written at startup, cleaned at shutdown. `daemon::cleanup_stale_daemon_files` detects crashed prior daemons by checking if the pid in the file is still alive; if not, reclaims both files.

`pidfile_pid_for_port(global_dir, port)` is the function used in the bind-failure path — it reads the pidfile to name whoever currently holds the port, so the error message is actionable.

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

- [Scopes](/reference/runtime/scopes/) — auth middleware and token types.
- [Supervisor](/reference/runtime/supervisor/) — everything the orchestrator needs from the host.
- [Orchestrator](/reference/runtime/orchestrator/) — started by `JobManager::init` → `subscribe_job_events`.
- [Mesh](/reference/runtime/mesh/) — spawned by `spawn_mesh_service`.
- [PTY Bridge](/reference/runtime/pty-bridge/) — `/api/runtime/workspaces/:id/pty` handler.
