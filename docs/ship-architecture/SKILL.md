---
name: ship-architecture
stable-id: ship-architecture
description: Use when understanding Ship's architecture — crate structure, app binaries, key design decisions. How the compiler, runtime, kernel, and daemon fit together.
tags: [ship, architecture, concepts]
authors: [ship]
---

# Ship Architecture

> **Target architecture**: see [Architecture Charter](references/docs/charter.md) — the four layers, contract boundaries, and non-negotiables. The sections below describe the codebase as it currently stands; the charter is canonical for intent.

Ship is an agent configuration compiler and runtime. It compiles declarative agent configuration (`.ship/`) into provider-native files (Claude, Gemini, Codex, Cursor), and provides a daemon for multi-agent coordination, pipeline execution, and event-driven state management.

## Crate Structure

All core crates live under `crates/core/`:

### ship-types

Shared types used across all crates. Zero internal dependencies. Defines `EventEnvelope`, `JobSpec`, `PipelineStep`, `StepType`, and other domain primitives.

### ship-jobs

Job and pipeline types. Defines `Pipeline`, `PipelineMeta`, pipeline JSONC normalization (`normalize_pipeline`), and step type parsing. Handles both short-form (`agents: [...]`) and long-form (`steps: [...]`) pipeline JSONC.

### ship-kernel

Pure event routing with no persistence implementation. Defines `KernelRouter`, `EventSink` trait, `Mailbox`, and `EventValidator`. Actors register with namespace-scoped read/write/subscribe permissions. The kernel routes events to subscribers and persists via the injected `EventSink`. Has zero filesystem or database dependencies — persistence is injected by the runtime.

Key types: `KernelRouter`, `EventSink`, `ActorConfig`, `Mailbox`, `KernelError`.

### ship-substrate (L0)

Substrate layer — persistence, the event log, and path resolution. Owns:

- `events/` — `KernelRouter` wrapper over `ship-kernel`, validators
- `persistence/` — `EventStore` trait, `SqliteEventStore`, actor store, cursors
- `db/` — SQLite schema, connection pool, sessions, events, actor events, cursors
- `projections/` — low-level read models (actor, process, session, spawn, registry)
- `services/` — mesh, human gateway, dispatch ports
- `home` / `env` — `~/.ship/` path resolution, `SHIP_*` env var access
- `data` — the data DB (`data.db`) backing collections
- `project` — project init, directory resolution, worktree-aware path lookup
- `supervisor/` — actor supervisor, restart policies
- `git` — git adapter (subprocess, returns pure data)

### ship-platform (L1)

Platform primitives — the domain services on top of substrate state. Owns:

- `config/` — `.ship/` config parsing, catalog, security, providers
- `agents/` — agent profiles, export, skill paths, skill vars, permissions, rules
- `collections/` — collection types and persistence over `data.db`
- `artifacts/` — artifact metadata, backends, session artifact persistence
- `registry/` — package install, fetch, resolve, verify, tracking
- `hooks/` — trigger engine, loader, filter, rendering
- `services/` — orchestrator (pipeline execution), provider adapters, git service, files, sync
- `sessions/` — session lifecycle, trash, artifact events, reconciliation
- `projections/` — high-level read models (auth, jobs, compile state, projects)
- `auth` / `kv` — auth primitives, key-value store
- `sync/` — cloud replication client and cursor

### ship-application (L2)

Application layer — built-in apps consuming the same SDK as L3. Owns `bundler/` (view bundler worker supervisor), `toolchain/` (Node toolchain management), and `project/`.

### ship-pipelines

Pipeline DAG types and step resolution. The runtime walks the declared step graph.

### build-info

Build-time version and git metadata baked into the binary.

### lint

Shared lint rules — module-DAG and file-cap checks wired into CI.

### compiler

Compiles `.ship/` configuration into provider-specific output files. Pure transformation — no side effects, no database, no network. Takes agent config as input, emits files like `CLAUDE.md`, `AGENTS.md`, `.cursor/rules/*.mdc`, `GEMINI.md`, and MCP server configs.

### mcp-framework

MCP (Model Context Protocol) transport abstraction. Defines the protocol layer for stdio and HTTP MCP communication.

### cli-framework

CLI abstraction layer. Shared CLI utilities used by the `ship-cli` binary.

## App Binaries

All app binaries live under `apps/`:

### shipd (daemon)

The Ship daemon — an Axum HTTP server that hosts the `ShipRuntime`. Responsibilities:

- **ShipRuntime** — central manager owning the kernel, mesh registry, and lazily-initialized managers (`JobManager`, `MeshManager`, `SessionScanner`)
- **HTTP API** — REST endpoints for workspaces, sessions, jobs, events, mesh operations
- **Job dispatch** — process executor that spawns agent processes in worktrees
- **Pipeline orchestrator** — advances pipelines step-by-step as jobs complete
- **Session registry** — scoped sessions with per-agent capability enforcement
- **PTY bridge** — WebSocket-to-tmux terminal bridge

### mcp (MCP server)

Stdio sidecar for AI agents. Exposes Ship's capabilities as MCP tools (`start_session`, `log_progress`, `end_session`, `step_done`, etc.). Connects to the daemon over HTTP.

### ship-cli

Command-line interface. Thin transport — delegates to the compiler for `compile`/`use`, to the daemon for `run`/`workspace`/`daemon`, and handles package management locally.

## Key Design Decisions

### EventSink trait

The kernel (`ship-kernel`) defines an `EventSink` trait but has no database dependency. The runtime injects `SqliteEventStore` as the concrete implementation. This keeps the kernel testable with mock sinks and free of SQLite/sqlx dependencies.

### ShipRuntime manager pattern

`ShipRuntime` in `shipd` uses `tokio::sync::OnceCell` for lazy initialization of managers. `JobManager`, `MeshManager`, and `SessionScanner` are only created on first access. This avoids startup ordering issues and keeps initialization fast.

### Four runtime primitives

The runtime has four concepts — no `workspace` primitive. **Project** is a directory with `.ship/` inside it; a tagging/filter primitive, not a context-lock. **Actor** is a running thing — it owns isolation (`cwd`, `project_id`, permissions, transport, lifecycle, MCP tool gate) and is supervisor-managed. **Session** is an analytics rollup over actor activity, with a `kind` discriminator (`pipeline_step | chat_thread | direct_attach`); telemetry is inline, there is no live session row. **Adapter** is an in-process module that talks to the outside world (git, tmux, providers) and emits events into the runtime log. Git worktrees are a git feature, resolved on demand — not a runtime primitive.

### Simplified pipelines

Pipelines use a simplified JSONC format. The short form (`agents: [...]`) auto-chains agents into sequential steps. The long form (`steps: [...]`) supports mixed step types: agent strings, `run:` commands, `human:` pauses, and `parallel:` fan-out groups. Template variables (`{{ var }}`) are interpolated at dispatch time.

### Session scopes

The daemon enforces per-agent capabilities via scoped sessions. When an MCP server registers, it receives a session token with role-based scopes (`ReadWorkspace`, `WriteWorkspace`, `CreateJobs`, `ManageJobs`, `MeshSend`, `StepDone`, etc.). Roles include `commander`, `lead`, `contributor`, and `worker`, each with different permission sets.
