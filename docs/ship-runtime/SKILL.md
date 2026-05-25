---
name: ship-runtime
stable-id: ship-runtime
description: Use when working on Ship's DAEMON runtime internals — shipd, persistence, workspace management, pipeline orchestration, event system, database layer.
tags: [ship, runtime, internals, daemon]
authors: [ship]
audience: internal
---

# Ship Runtime (Daemon Internals)

> **Boundary:** This is the **daemon** runtime — shipd, the DB, kernel, event store, workspace state, pipelines. For the **SUI** runtime (signals, bindings, transitions, mount, attach), see `docs/ship-sui/SKILL.md` and `packages/sui/runtime/src/`. Different package, different concerns.

Ship's runtime layer owns all persistent state, pipeline execution, and inter-agent communication. It is split across three layered crates — `crates/core/ship-substrate/` (L0: kernel, events, persistence, db, home, env, data, plus the `mesh` and `human_gateway` service actors and low-level projections), `crates/core/ship-platform/` (L1: config, collections, artifacts, registry, hooks, agents, the `orchestrator` service, sessions, and high-level projections), and `crates/core/ship-application/` (L2: bundler, toolchain, project). The daemon binary lives in `apps/shipd/`.

## Module Layout

### persistence/ — `crates/core/ship-substrate/src/persistence/`

Event sourcing infrastructure. `EventStore` trait with `SqliteEventStore` implementation. Also contains `actor_store` (actor lifecycle persistence), `snapshot` (state snapshots), and `cursor` (replay position tracking).

### sessions/ — `crates/core/ship-platform/src/sessions/`

Session lifecycle and analytics. The `workspace` runtime primitive was retired entirely — sessions are immutable analytics records over actor activity:

- `session.rs` — session queries, progress, artifact persistence
- `session_lifecycle.rs` — `start_session` (idempotent), `end_session`
- `compile.rs` — triggers compiler for active agent config at session start
- `helpers.rs` — branch-key validation and harness-ref normalization
- `types_session.rs` — `Session`, `SessionStatus`, `SessionLiveness`, `EndSessionRequest`

### projections/ — `crates/core/ship-substrate/src/projections/` (low-level) + `crates/core/ship-platform/src/projections/` (high-level)

Read models rebuilt from events. Projection handlers for jobs (`JobRecord`, `load_jobs`), sessions, actors, processes, and the package registry. Includes `async_projection` for background rebuilds and `spawn` for projection actor startup.

### services/ — `crates/core/ship-platform/src/services/` (orchestrator, harnesses) + `crates/core/ship-substrate/src/services/` (mesh, human_gateway, dispatch_ports)

Service actors and coordination:

- `orchestrator/` — pipeline execution engine:
  - `pipeline.rs` — `advance_pipeline` (finds unblocked steps, dispatches next)
  - `pipeline_loader.rs` — `ensure_pipeline` (loads and normalizes JSONC)
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
- `harness_adapter.rs`, `harnesses/` — harness-specific adapters

### config/ — `crates/core/ship-platform/src/config/`

Configuration loading from `.ship/`:

- `catalog.rs` — `CatalogEntry`, `CatalogKind` for skill/agent discovery
- `security.rs` — security policy enforcement

### agents/ — `crates/core/ship-platform/src/agents/`

Agent profile management:

- `config.rs` — `WorkspaceAgentSettings`, `ProviderSettings`
- `export/` — agent export for publishing
- `permissions.rs` — `Permissions`, `AgentLimits`
- `rule.rs`, `skill.rs` — `Rule`, `Skill` types
- `skill_paths.rs` — skill file resolution
- `skill_vars.rs` — skill variable substitution

### registry/ — `crates/core/ship-platform/src/registry/`

Package registry client:

- `install.rs` — dependency installation
- `fetch.rs` — package download
- `resolver.rs` — version constraint resolution
- `verify.rs`, `hash.rs` — integrity verification
- `tracking.rs` — install tracking
- `cache.rs` — local package cache

### db/ — `crates/core/ship-substrate/src/db/`

SQLite database layer:

- `pool.rs` — connection pool management
- `session.rs`, `session_drain.rs`, `session_events.rs` — session persistence, drain, and transactional event inserts
- `daemon_settings.rs` — daemon settings table access
- `envelope.rs`, `events.rs`, `actor_events.rs` — event persistence
- `cursors.rs` — replay cursor tracking
- `integrity.rs`, `repair.rs`, `project_id_reconcile.rs` — integrity checks and repair
- `rebuild.rs` — projection rebuild from event log

### sync/ — `crates/core/ship-platform/src/sync/`

Cloud replication:

- `client.rs` — sync client
- `cursor.rs` — replication cursor tracking
- `types.rs` — sync protocol types

### supervisor/ — `crates/core/ship-substrate/src/supervisor/`

Actor model primitives:

- `mod.rs` — actor trait and lifecycle
- `supervisor.rs` — actor supervision and restart policies

### events/ — `crates/core/ship-substrate/src/events/`

Thin re-export facade combining types from `ship-kernel` and `ship-types`. Includes `KernelRouter` initialization (`init_kernel_router`), security extensions (`kernel_security`), event filtering, and payload schema validation.

## Daemon (shipd)

The daemon binary (`apps/shipd/`) hosts the `ShipRuntime`:

- `ship_runtime.rs` — `ShipRuntime` struct with lazy `OnceCell` managers
- `routes/` — all HTTP route modules (`rest_api.rs` for mesh, `runtime_api.rs` read-only API for Studio, plus per-domain APIs)
- `supervisor/` — integration-test scaffolding only; production supervisor code lives in the `crates/core/ship-supervisor/` crate
- `transport/pty.rs` — WebSocket-to-tmux PTY bridge
- `transport/connections.rs` — per-connection lifecycle and the mesh service spawn
- `auth/scopes.rs` — re-exports `DaemonSession`, `Scope` from `ship_platform::auth::scopes`; role-based capability enforcement
- `auth.rs` / `auth/` — session registry and token management

## Key Subsystems

**Orchestrator**: Advances pipelines step-by-step. When a job completes, `advance_pipeline` finds newly unblocked steps and dispatches them. Gate steps can retry failed reviews up to `max_retries`. The `StepMonitor` watches process exit and signals completion.

**Persistence**: All state changes produce append-only events. `SqliteEventStore` persists to `platform.db`. Projections rebuild read models from the event log. The `rebuild` module can reconstruct all projection tables from scratch.

**Session telemetry**: Sessions are immutable analytics records over actor activity. Each session records start time, progress notes, end summary, model used, files changed, lines added/removed, tool calls, and gate results — end-of-session telemetry lands inline on the session row. Session artifacts are persisted to disk.
