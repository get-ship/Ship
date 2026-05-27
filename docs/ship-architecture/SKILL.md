---
name: ship-architecture
stable-id: ship-architecture
description: Use when understanding Ship's architecture — crate structure, app binaries, key design decisions. How the compiler, runtime, kernel, and daemon fit together.
tags: [ship, architecture, concepts]
authors: [ship]
---

# Ship Architecture

> **Target architecture**: see [Architecture Charter](references/docs/charter.md) — the four layers, contract boundaries, and non-negotiables. The sections below describe the codebase as it currently stands; the charter is canonical for intent.

Ship is an agent configuration compiler and runtime. It compiles declarative agent configuration (`.ship/`) into harness-native files (Claude, Gemini, Codex, Cursor), and provides a daemon for multi-agent coordination, pipeline execution, and event-driven state management.

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

- `config/` — `.ship/` config parsing, catalog, security, harnesses
- `agents/` — agent profiles, export, skill paths, skill vars, permissions, rules
- `collections/` — collection types and persistence over `data.db`
- `artifacts/` — artifact metadata, backends, session artifact persistence
- `registry/` — package install, fetch, resolve, verify, tracking
- `hooks/` — trigger engine, loader, filter, rendering
- `services/` — orchestrator (pipeline execution), harness adapters, git service, files, sync
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

Compiles `.ship/` configuration into harness-specific output files. Pure transformation — no side effects, no database, no network. Takes agent config as input, emits files like `CLAUDE.md`, `AGENTS.md`, `.cursor/rules/*.mdc`, `GEMINI.md`, and MCP server configs.

### mcp-framework

MCP (Model Context Protocol) transport abstraction. Defines the protocol layer for stdio and HTTP MCP communication.

### cli-framework

CLI abstraction layer. Shared CLI utilities used by the `ship-cli` binary.

## App Binaries

All app binaries live under `apps/`:

### shipd (daemon)

The Ship daemon — an Axum HTTP server that hosts the `ShipRuntime`. Responsibilities:

- **ShipRuntime** — central manager owning the kernel, mesh registry, and lazily-initialized managers (`JobManager`, `MeshManager`, `SessionScanner`)
- **Op surface** — the L1 ABI: per-domain ConnectRPC services under `ship.ops.v1` (rules, git, agents, …), each **authored from proto** (the SSOT for op shape *and* policy), with reads, commands (gated at the dispatch chokepoint), and server-streams (events, PTY bytes). The same proto source generates the sdk-skills and contract fixtures. The legacy REST + OpenAPI surface is being migrated onto it and then deleted. See [The op surface](references/docs/op-surface.md).
- **Pipeline orchestrator** — advances pipelines step-by-step as agent sessions complete
- **Session registry** — scoped sessions; grants minted at dispatch, read by the gate
- **PTY bridge** — terminal I/O over the Connect `SessionBytes` stream (and an SSE adapter during migration)

### mcp (external adapter)

A standalone MCP adapter (`apps/mcp`) over the daemon's op surface, for MCP-speaking tools. The daemon-hosted internal MCP server was retired — agents now use **skills + the Connect op surface** (`ship ops`) as their interface, not an in-daemon MCP tool gate.

### ship-cli

Command-line interface. Thin transport — delegates to the compiler for `compile`/`use`, to the daemon for `run`/`workspace`/`daemon`, and handles package management locally.

## Key Design Decisions

### EventSink trait

The kernel (`ship-kernel`) defines an `EventSink` trait but has no database dependency. The runtime injects `SqliteEventStore` as the concrete implementation. This keeps the kernel testable with mock sinks and free of SQLite/sqlx dependencies.

### ShipRuntime manager pattern

`ShipRuntime` in `shipd` uses `tokio::sync::OnceCell` for lazy initialization of managers. `JobManager`, `MeshManager`, and `SessionScanner` are only created on first access. This avoids startup ordering issues and keeps initialization fast.

### Four runtime primitives

The runtime has four concepts — no `workspace` primitive. **Space** (formerly *project*; the rename is complete at the service/proto surface — `SpaceService` — with some internal symbols still named `project` pending churn) is a directory with `.ship/` inside it: the unit of isolation/coordination/config (see [Spaces](references/docs/spaces.md)), not a context-lock. **Actor** is a running thing — it owns isolation (`cwd`, `space_id`, permissions, transport, lifecycle, MCP tool gate) and is supervisor-managed. **Session** is an analytics rollup over actor activity, with a `kind` discriminator (`pipeline_step | chat_thread | direct_attach`); telemetry is inline, there is no live session row. **Adapter** is an in-process module that talks to the outside world (git, tmux, harnesses) and emits events into the runtime log. Git worktrees are a git feature, resolved on demand — not a runtime primitive.

### Simplified pipelines

Pipelines use a simplified JSONC format. The short form (`agents: [...]`) auto-chains agents into sequential steps. The long form (`steps: [...]`) supports mixed step types: agent strings, `run:` commands, `human:` pauses, and `parallel:` fan-out groups. Template variables (`{{ var }}`) are interpolated at dispatch time.

### Capability gate

The daemon enforces per-agent capability at **one gate** on the dispatch chokepoint (`services::gate`, invoked by `services::command::call`), decided on **caller × blast-radius × grant**: C0 reads / C1 workspace writes are open to any in-scope agent; restricted domains (`ship-orchestration`, `ship-authoring`) require a grant minted at dispatch; C2 (credential/escalation) is operator-only; C3 (critical — e.g. revoking a window token) is operator-only and never cleared by a space shell. See [Daemon authentication model](/reference/architecture/auth-model/). The legacy `Scope` enum now serves only REST route-auth + artifact perms and is slated for deletion with the REST surface.

### State ownership & sync

The runtime is event-sourced — the log is truth for **operations** on a space. Other state legitimately lives elsewhere, classed by who owns the authoritative copy: **event log** (sessions, actors, grants, pipeline-runs, per-space settings — the event *is* the change), **`.ship/` files** (authored source; the `fsnotify` watcher only *observes* and emits a stamped `*.changed` signal; distributed via the package format — **git is opt-in/supplementary, never load-bearing or assumed**), **collections** (`data.db` — per-item **CRDT/Automerge**, `data.db` is a rebuilt projection, NOT a blob), and **artifacts** (the real blobs — content-addressed bytes, object-store diff sync). Settings split: global daemon settings are machine-local KV (not synced); space settings are event-sourced + synced, merged over global defaults at read time in `SpaceService.Info`. One content hash (`compute_tree_hash`) across cache key / registry `contentHash` / bundle attestation. Cross-machine sync (HLC ordering, sync-policy/retention tiers, `observe_external`, macaroon auth) is mapped but not built. See [State ownership & sync](references/docs/state-and-sync.md).

### Spaces

A **space** is the unit of isolation/coordination/config — its own window, one `.ship/` directory, containing views/agents/skills/pipelines. **One daemon serves many spaces** (not forked per space): the emit path resolves `space_id` per-connection, so a space window's events partition into its own space. A space webview holds a **pid-bound C2 token** (minted over UDS from the kernel-attested peer pid); it never touches C3. The daemon stays authoritative over windows it can't itself spawn — it mediates (`WindowIntent` focus-vs-spawn, `WindowList`) and holds the kill switch (`CloseWindow` revokes the pid-bound token, C3). Per-space config is global defaults overlaid by event-sourced per-space overrides (`space.setting.changed` → `space_settings`), surfaced merged by `SpaceService.Info`. See [Spaces](references/docs/spaces.md).
