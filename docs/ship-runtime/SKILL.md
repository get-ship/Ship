---
name: ship-runtime
stable-id: ship-runtime
description: Use when working on Ship's DAEMON runtime internals — shipd, persistence, workspace management, pipeline orchestration, event system, database layer.
tags: [ship, runtime, internals, daemon]
authors: [ship]
audience: internal
---

# Ship Runtime (Daemon Internals)

> **Boundary:** This is the **daemon** runtime — shipd, the DB, kernel, event store, workspace state, pipelines. For the **SUI** runtime (signals, bindings, transitions, mount, attach), see `docs/ship-sui/SKILL.md` and `packages/sui-runtime/src/`. Different package, different concerns.

Ship's runtime layer owns all persistent state, pipeline execution, and inter-agent communication. The core library lives in `crates/core/runtime/`; the daemon binary lives in `apps/shipd/`.

## Module Layout

**`crates/core/runtime/src/`**:

### persistence/

Event sourcing infrastructure. `EventStore` trait with `SqliteEventStore` implementation. Also contains `actor_store` (actor lifecycle persistence), `snapshot` (state snapshots), and `cursor` (replay position tracking).

### workspace/

Workspace and session management:

- `lifecycle.rs` — workspace creation and archival
- `session.rs`, `session_lifecycle.rs` — session start, progress, end
- `crud.rs` — workspace CRUD operations
- `trash.rs` — soft-delete and restore for workspace files (24h retention, auto-purge)
- `artifact_events.rs` — session artifact persistence
- `compile.rs` — triggers compiler for active agent config
- `reconcile.rs` — workspace state reconciliation
- `types.rs`, `types_session.rs` — `Workspace`, `WorkspaceSession`, `WorkspaceSessionRecord`, `TrashEntry`

### projections/

Read models rebuilt from events. Projection handlers for jobs (`JobRecord`, `load_jobs`), sessions, workspaces, actors, and the package registry. Includes `async_projection` for background rebuilds and `spawn` for projection actor startup.

### services/

Service actors and coordination:

- `orchestrator/` — pipeline execution engine:
  - `pipeline.rs` — `advance_pipeline` (finds unblocked steps, dispatches next)
  - `pipeline_loader.rs` — `ensure_pipeline` (loads and normalizes YAML)
  - `pipeline_vars.rs` — template variable interpolation
  - `gate_retry.rs` — re-dispatches failed gate steps up to `max_retries`
  - `step_monitor.rs` — watches agent process exit, sets done flag
  - `run_step.rs` — executes `StepType::Run` shell commands
  - `dispatch.rs`, `handlers.rs`, `emit.rs` — job dispatch and event emission
  - `pending.rs` — concurrency-limited pending job queue
  - `replay.rs` — daemon startup replay of incomplete pipelines
- `mesh.rs` — inter-agent message routing via `SharedMeshRegistry`
- `human_gateway.rs` — human-in-the-loop step handling
- `dispatch_ports.rs` — trait abstraction for job dispatch
- `provider_adapter.rs`, `providers/` — provider-specific adapters

### config/

Configuration loading from `.ship/`:

- `catalog.rs` — `CatalogEntry`, `CatalogKind` for skill/agent discovery
- `hooks.rs` — `RuntimeHooks` trait, `DefaultRuntimeHooks` for lifecycle events
- `security.rs` — security policy enforcement

### agents/

Agent profile management:

- `config.rs` — `WorkspaceAgentSettings`, `ProviderSettings`
- `export/` — agent export for publishing
- `permissions.rs` — `Permissions`, `AgentLimits`
- `rule.rs`, `skill.rs` — `Rule`, `Skill` types
- `skill_paths.rs` — skill file resolution
- `skill_vars.rs` — skill variable substitution

### registry/

Package registry client:

- `install.rs` — dependency installation
- `fetch.rs` — package download
- `resolver.rs` — version constraint resolution
- `verify.rs`, `hash.rs` — integrity verification
- `tracking.rs` — install tracking
- `cache.rs` — local package cache

### db/

SQLite database layer:

- `schema/` — migration files
- `pool.rs` — connection pool management
- `kv.rs` — key-value store
- `session.rs`, `session_drain.rs` — session persistence and cleanup
- `workspace_db.rs` — per-workspace database
- `workspace_state.rs` — workspace state queries
- `artifact_registry.rs` — artifact metadata storage
- `envelope.rs`, `events.rs` — event persistence
- `rebuild.rs` — projection rebuild from event log

### sync/

Cloud replication:

- `client.rs` — sync client
- `cursor.rs` — replication cursor tracking
- `types.rs` — sync protocol types

### actor/

Actor model primitives:

- `mod.rs` — actor trait and lifecycle
- `supervisor.rs` — actor supervision and restart policies

### events/

Thin re-export facade combining types from `ship-kernel` and `ship-types`. Includes `KernelRouter` initialization (`init_kernel_router`), security extensions (`kernel_security`), event filtering, and payload schema validation.

## Daemon (shipd)

The daemon binary (`apps/shipd/`) hosts the `ShipRuntime`:

- `ship_runtime.rs` — `ShipRuntime` struct with lazy `OnceCell` managers
- `rest_api.rs` — REST endpoints for mesh and workspace operations
- `runtime_api.rs` — read-only API for Studio (workspaces, sessions, agents, SSE)
- `supervisor/` — job dispatch, pipeline engine, workspace setup, terminal launcher
- `pty_handler.rs` — WebSocket-to-tmux PTY bridge
- `connections.rs` — per-connection lifecycle and cleanup
- `scopes.rs` — `DaemonSession`, `Scope` enum, role-based capability enforcement
- `auth.rs` — session registry and token management

## Key Subsystems

**Orchestrator**: Advances pipelines step-by-step. When a job completes, `advance_pipeline` finds newly unblocked steps and dispatches them. Gate steps can retry failed reviews up to `max_retries`. The `StepMonitor` watches process exit and signals completion.

**Persistence**: All state changes produce append-only events. `SqliteEventStore` persists to `platform.db`. Projections rebuild read models from the event log. The `rebuild` module can reconstruct all projection tables from scratch.

**Workspace**: Simplified model — no status machine. Workspaces are created (with optional git worktree), hold sessions, and get archived. Trash provides soft-delete with 24-hour retention and auto-purge on daemon startup.

**Session telemetry**: Sessions track agent work within a workspace. Each session records start time, progress notes, end summary, model used, files changed, and gate results. Session artifacts are persisted to disk.
