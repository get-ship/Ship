---
group: Architecture
order: 2
title: Runtime
description: What the runtime owns -- sessions, events, jobs, actors, targets, capabilities, skill vars, skill paths, and the platform database.
audience: internal
---

# Runtime

The runtime owns all persistent state, and is split across three layered crates. **`crates/core/ship-substrate/`** (L0) owns persistence itself — the kernel wrapper, the event log, the platform database, home/env path resolution, and the data DB. **`crates/core/ship-platform/`** (L1) owns the domain services that sit on top of that state — config, collections, artifacts, registry, hooks, agents, the orchestrator, and sessions. **`crates/core/ship-application/`** (L2) owns the application layer — bundler, toolchain, project. The platform database lives at `~/.ship/machine/db/platform.db`. There are **no per-actor SQLite files** and **no separate kernel event store** — every actor shares the single `events` table in `platform.db`, with logical isolation by `actor_id`. No other layer touches the database directly — all access goes through the KernelRouter and ActorStore.

## Module Map

Top-level modules across the three crates:

| Crate | Module | Owns |
|-------|--------|------|
| `ship-substrate` | `events` | `KernelRouter` wrapper over `ship-kernel`, `EmitContext`-aware validators |
| `ship-substrate` | `db` | Platform database schema, migrations, connections, table modules |
| `ship-substrate` | `persistence` | `EventStore` trait, `SqliteEventStore`, actor store, replay cursors |
| `ship-substrate` | `projections` | Low-level read models (actor, process, session, spawn, registry) |
| `ship-substrate` | `services` | Mesh, human gateway, dispatch ports |
| `ship-substrate` | `home` / `env` | `~/.ship/` path resolution, `SHIP_*` env var access |
| `ship-substrate` | `data` | The data DB (`data.db`) backing collections |
| `ship-substrate` | `project` | Project directory resolution, worktree-aware path lookup |
| `ship-substrate` | `git` | Git subprocess adapter — returns pure-data results |
| `ship-substrate` | `supervisor` | Actor supervision and restart policies |
| `ship-platform` | `sessions` | Session lifecycle — start, progress, end, reconciliation |
| `ship-platform` | `services` | Orchestrator, harness adapters, GitService, files, sync |
| `ship-platform` | `config` | Project config (`ship.jsonc`), agents, MCP servers, harnesses, gitignore |
| `ship-platform` | `agents` | Agent config, export/import, permissions, rules, skills, skill vars/paths |
| `ship-platform` | `collections` | Collection logic over the data DB |
| `ship-platform` | `artifacts` | Artifact storage and backends |
| `ship-platform` | `registry` | Registry operations, dependency resolution |
| `ship-platform` | `hooks` | Trigger engine, loader, filter, rendering |
| `ship-application` | `bundler` / `toolchain` | View bundler worker supervisor, Node toolchain management |

## Database Layer

The `db` module (`crates/core/ship-substrate/src/db/`) contains all platform.db access. One file per concern:

| DB Module | Concern |
|-----------|---------|
| `db::pool` | Connection pool, `ensure_db`, migration runner |
| `db::events` | Event-stream persistence — the `events` table |
| `db::envelope` | `EventEnvelope` row mapping |
| `db::actor_events` | Actor lifecycle event persistence |
| `db::session` | Session records — the `sessions` table |
| `db::session_events` | Session event projection |
| `db::session_drain` | Session drain operations |
| `db::cursors` | Sync / rebuild cursor bookmarks |
| `db::daemon_settings` | Machine-wide daemon settings |
| `db::rebuild` | Projection rebuild from the event log |
| `db::repair` / `db::integrity` | Schema integrity checks and repair |
| `db::project_id_reconcile` | Project-id reconciliation |
| `db::types` | Shared DB type definitions |

The platform database path is `~/.ship/machine/db/platform.db`. The `ensure_db()` function runs sqlx migrations idempotently, sets WAL journal mode and foreign keys, then opens a connection. There are no per-actor event stores and no separate kernel store — all actors share `platform.db`'s `events` table (see Actor Model below). The key-value store moved out of platform.db; KV now lives in its own `kv.db`, accessed via `ship_platform::kv` (a single `kv.rs` with a namespaced API).

{% aside type="tip" %}
Tests get automatic isolation. The `get_global_dir()` function detects test binaries and returns a per-thread temp directory instead of `~/.ship/`.
{% /aside %}

## Sessions

A session is an analytics rollup over actor activity. Lifecycle: start, work (log progress), end with summary. It has a `kind` discriminator (`pipeline_step | chat_thread | direct_attach`) and context dimensions (`project_id`, `pipeline_run_id?`, `chat_thread_id?`, `branch?`).

Session records carry start/end timestamps, model used, progress notes, summary, files-changed count, gate result, and exit reason — telemetry is inline on the row, set at `session.ended`. There is no live session row: "what's running" is an actor question; "what happened" is a session query.

Session lifecycle code lives in `ship_platform::sessions` — `start_session`, `end_session`, `record_progress`, `compile_session_context`.

## Actor Model

The runtime uses an actor model with **logical** per-actor isolation. Every entity on the runtime — agents, apps, services — is an actor with a mailbox and a scoped `ActorStore` handle. All actors share the single `events` table in `platform.db`; isolation is by `actor_id`, not by file. Actors communicate through kernel-routed messages, never shared storage.

### Actor types

| Type | Has UI | Has MCP | Examples |
|------|--------|---------|----------|
| Agent | no | yes (client) | Code agent, design agent |
| App | yes | yes (server) | Ship Studio |
| Service | no | no | Sync, auth, docs API |

### KernelRouter

The substrate `KernelRouter` (`ship_substrate::events::kernel_router`) wraps the pure `ship_kernel::KernelRouter` — it delegates registration, mailboxes, and subscriptions to the inner router and adds `EmitContext`-aware validation, `ActorStore` construction, and write-namespace enforcement. Public methods:

- `spawn_actor(id, config)` — delegates registration to the inner router; returns `(ActorStore, Mailbox)`.
- `emit(event, ctx)` — validates, enforces write namespaces, persists, delivers.
- `deliver(&event)` — pushes an already-persisted event to subscribed mailboxes (no validation, no persist).
- `stop_actor(id)` — delegates to the inner router, then flushes WAL.
- `with_validator(v)` — registers an `EmitContext`-aware validator (builder).
- `has_actor`, `actor_count`, `list_actors`, `msg_rate` — introspection.
- `kernel_store_path()` — path to `platform.db`.

There is no `route`, `snapshot`, `suspend`, or `restore` method.

### ActorStore

A scoped event handle bound to one actor, backed by the shared `platform.db`. Enforces namespace boundaries.

- Writes reject event types outside the actor's `write_namespaces`.
- Reads filter by the `actor_id` column — an actor's queries only return its own events.
- There is no per-actor file; isolation is the `ActorStore` API plus the `actor_id` column.

Actors never construct their own store. The kernel creates it via `spawn_actor`.

### Mailbox

Per-actor mpsc channel. The kernel delivers messages based on namespace subscriptions. An actor subscribing to `["studio.", "agent."]` receives all events with those prefixes.

### Namespace enforcement

Every event type is namespaced (e.g. `studio.message.visual`, `agent.task.completed`). `RESERVED_NAMESPACES` in `validator.rs` is the single source of truth. Agents cannot emit reserved prefixes through MCP tools.

### Storage layout

```
~/.ship/machine/db/
  platform.db           # Single event log + projections — all actors
  data.db               # Collections
```

There are no `~/.ship/actors/{id}/events.db` files and no `~/.ship/kernel/events.db`. Every actor's events land in the one `events` table in `platform.db`, distinguished by the `actor_id` column.

### Event flow

```
Actor emits event → KernelRouter.emit() → validate
                  → persist to platform.db (events table)
                  → deliver to subscribed mailboxes
                  → subscriber's EventRelay → SSE notification to UI
```

Persist-before-deliver: nothing reacts to an event that isn't durable first. Reads through `ActorStore` are filtered by `actor_id`; unfiltered cross-actor queries are an admin/debug concern, not an actor-level capability.

## Events

Events are immutable, append-only records. Each event has: id (ULID), event_type (namespaced), entity_id, actor, payload_json, version, causation_id, project_id, session_id, actor_id, parent_actor_id, job_id, agent_id, target_actor_id, elevated flag, and created_at timestamp. The `events` table has a `BEFORE UPDATE` trigger that hard-fails any mutation — append-only is enforced at the SQL layer.

Skills declare artifact types in their SKILL.md frontmatter (`artifacts: [html, pdf, adr]`), not events. The platform infers applicable event types from artifact types. There is no `events.json` in skills.

Agents do not have read access to the event store. They receive events through their mailbox via SSE notifications. The `list_events` MCP tool has been removed.

## Jobs

Jobs coordinate work across agents. Status lifecycle: `pending` -> `running` -> `complete` or `failed`. Jobs can be blocked by other jobs via `blocked_by`.

Each job carries: kind, description, assignment, branch, file scope, touched files, acceptance criteria, capability link, priority, and a log.

## Targets and Capabilities

Targets are named goals. Milestones are time-bounded (v0.1). Surfaces are evergreen domains (Compiler, Studio). Targets carry `body_markdown` for strategy and context.

Capabilities are verifiable slices of a target. Status: `aspirational` -> `in_progress` -> `actual`. Evidence required to mark actual. Each has acceptance criteria, phase, file scope, priority, and assignment.

## Skill Vars

Runtime functions for skill variable state management:

- `get_skill_vars(ship_dir, skill_id)` -- merged variable state (defaults + user + project)
- `set_skill_var(ship_dir, skill_id, key, value)` -- set a variable value
- `list_skill_vars(ship_dir)` -- all skills with configurable variables
- `reset_skill_vars(ship_dir, skill_id)` -- clear all overrides for a skill

Variable values are stored in the KV store (`kv.db`, via `ship_platform::kv`) with keys like `skill_vars:{id}`, `skill_vars.local:{ctx}:{id}`, and `skill_vars.project:{ctx}:{id}`.

## Skill Paths

`read_skill_paths(ship_dir)` resolves all skill file paths for the active project, walking the skills directory tree.

## How Transport Uses the Runtime

The CLI maps clap flags to runtime function calls and formats results for terminal output. The MCP server wraps runtime calls as tool handlers, adding project directory resolution and resource notification. Both layers are thin -- validation, state management, and business logic live in the runtime.
