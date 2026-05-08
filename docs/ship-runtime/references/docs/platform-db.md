---
group: Runtime
order: 3
title: Platform Database
description: platform.db schema, migrations, connection pool, WAL config, and the single-file architecture.
audience: internal
---

# Platform Database

Runtime state. Path: `~/.ship/platform.db`. Lives at `crates/core/runtime/src/db/`.

Ship has a second SQLite database — `~/.ship/data.db` — that holds user content collections (specs, ADRs, notes, feedback). `platform.db` and `data.db` are separate files with separate migrations and separate responsibilities. See [Collections](/runtime/collections/) for the data side.

## Two databases, clear split

| File | Holds | Written by |
|---|---|---|
| `platform.db` | Runtime state: workspaces, sessions, jobs, events, kernel routing, actor registry, KV | Runtime code via `persistence` + `db` modules |
| `data.db` | User content: collection items, schemas, attachments | Collections service only |

The rest of this page covers `platform.db`.

Historically Ship split `platform.db` further across multiple DBs (kernel events, per-actor events, workspace state). That's gone. Every non-collection row is now in `platform.db`. Benefits: one backup target for runtime state, one WAL, fewer file descriptors, joinable queries across domains. Cost: all runtime writers share a single write lock, so throughput depends on SQLite's WAL concurrency and busy_timeout behaviour.

User content was migrated out of `platform.db` into `data.db` via a one-time migration in `collections::migration`. A completion flag in `data.db` makes the migration idempotent.

Tests get automatic isolation via `runtime::project::get_global_dir()` — it detects test binaries and returns a per-thread temp directory. Any code that resolves paths through `db_path()` inherits that isolation.

## Connection PRAGMAs

Set on every call to `ensure_db_at_path`:

| PRAGMA | Value | Why |
|---|---|---|
| `journal_mode` | `WAL` | Concurrent readers with a single writer. Required for the pool. |
| `busy_timeout` | `5000` (ms) | Tolerate brief lock contention without instant failure. |
| `foreign_keys` | `ON` | FKs enforced; schema can rely on cascade deletes. |

`open_db_at(path)` (no migration) sets only `busy_timeout` — the DB is assumed already initialized. `ensure_db_at_path` is the one-stop "make sure this DB exists, is migrated, and has the right PRAGMAs."

## Constructors

`crates/core/runtime/src/db/mod.rs`:

| Function | Purpose |
|---|---|
| `db_path() -> Result<PathBuf>` | Resolves `~/.ship/platform.db` (or test-thread temp dir). |
| `ensure_db() -> Result<()>` | Runs migrations idempotently at `db_path()`. |
| `ensure_db_at(ship_dir: &Path) -> Result<()>` | Same, but at an explicit ship_dir. Use in tests and handlers that take `ApiState`. |
| `open_db() -> Result<SqliteConnection>` | Shortcut: `ensure_db()` then `connect()`. |
| `open_db_at(path: &Path) -> Result<SqliteConnection>` | Open without migrating. DB must already exist. |

Migration failure causes `ensure_db` to refuse startup. The error message surfaces the hint "this usually means a different Ship binary created this database" and points to `ship daemon reset-db`. **Migrations never delete user data on failure.**

## Connection pool

`crates/core/runtime/src/db/pool.rs`. Optional — used by the daemon for hot paths (event queries, KV reads, projection lookups). Not used by CLI invocations that open and close per-command.

- Size: `DEFAULT_POOL_SIZE = 5` (4 read + 1 write, per SQLite WAL semantics).
- Acquisition: `platform_pool() -> &'static SqlitePool`. Lazy; initialized on first call via `OnceLock`.
- Custom size: `create_pool(path, size)` for explicit construction in tests or specialized callers.

**Not suitable for per-test isolation** — the static `PLATFORM_POOL` persists across test runs and its path is fixed at first init. Tests must use `open_db_at()` or `create_pool(path, size)` against a temp dir.

## Migrations

Two files in `crates/core/runtime/migrations/`:

| File | Contents |
|---|---|
| `0001_schema.sql` | Every table below. 283 lines. |
| `0002_specs.sql` | The `spec` table (added later). 18 lines. |

Header comment in `0001_schema.sql`: *"Ship platform schema — single-file, pre-launch. When the schema changes, update this file and delete the DB."* The intent is to collapse everything back to a single migration before launch; adding `0002_specs.sql` was a reluctant exception.

Migrations run via `sqlx::migrate!("./migrations")`. Tracking table: `_sqlx_migrations`. `ensure_db_at_path` invokes migrations on every call — the tracking table makes it idempotent.

**Stale reference.** `crates/core/runtime/src/db/schema/mod.rs` says *"The canonical DDL lives in `migrations/0001_initial.sql`"* — the file is actually `0001_schema.sql`. Comment needs updating.

## Tables

Organized as two layers in the source (`db/schema/platform.rs` and `db/schema/workflow.rs`), but all in the same DB.

### Platform tables

Portable runtime primitives.

| Table | Purpose |
|---|---|
| `kv_state` | Generic namespaced KV. `(namespace, key)` primary key; JSON values. |
| `workspace` | Branch-keyed unit of work. One row per branch. |
| `workspace_session` | Time-bounded work interval. Indexed by workspace + start time. |
| `workspace_session_record` | Immutable end-of-session snapshot. One record per completed session. |
| `branch_config` | Per-branch compiled agent config (preset, plugins). |
| `branch_context` | Branch → linked entity pointers. |
| `event_log` | Older audit trail. `action` + `entity_type` + `entity_id` + correlation fields. |
| `events` | Actor-scoped event bus. Used by kernel routing and `SqliteEventStore`. |
| `actors` | Actor registration metadata. |
| `agent_artifact_registry` | Content-addressed artifact store. |
| `managed_mcp_state` | Ship-managed MCP server tracking (PID, port, status). |

### Workflow tables

Opinionated planning layer.

| Table | Purpose |
|---|---|
| `target` | Named goals — milestones and surfaces. |
| `capability` | Concrete requirements under a target. |
| `jobs` | Agent work queue. |
| `file_claim` | Batch-atomic file claims with workspace tracking. Per-file exclusive locks across parallel agents. |
| `note` | Human-facing scratchpad. |
| `adr` | Architecture decision records. |
| `spec` | Feature specs (added in `0002_specs.sql`). |

## Two event-ish tables

`event_log` and `events` both exist. They are not the same thing.

**`event_log`** — older audit trail. Columns: `action`, `entity_type`, `entity_id`, `workspace_id`, `session_id`, `job_id`, `correlation_id`, `causation_id`, `synced_at`. Populated by `log.rs` / projection helpers. Oriented around "who did what to which entity."

**`events`** — the actor-scoped event bus backing the kernel. Columns: `event_type`, `entity_id`, `actor`, `actor_id`, `parent_actor_id`, `payload_json`, `causation_id`, `target_actor_id`, `elevated`, `job_id`, `agent_id`, `agent_version`. Populated by `SqliteEventStore::append` and `ActorStore::append`. Oriented around "actor X emitted this typed event at this version."

Indexes on `events`: `(entity_id, event_type)`, `(workspace_id, created_at)`. On `event_log`: `workspace_id`, `session_id`, `job_id`, `(entity_type, entity_id)`.

New code writes to `events`. Paths that still write to `event_log` are legacy; they may continue to exist for audit queries but shouldn't gain new writers.

## Blocking helpers

`db/mod.rs` exposes three `block_on` variants because sqlx is async-only and much of Ship's call surface is sync:

| Helper | Signature | Use |
|---|---|---|
| `block_on` | `Future<Output=Result<T, sqlx::Error>>` | Standard sqlx calls. Current-thread runtime when no handle is present. |
| `block_on_anyhow` | `Future<Output=anyhow::Result<T>>` | Futures that internally call `block_in_place` (e.g. nested sqlx calls). Multi-thread runtime required. |
| `block_on_migrate` | `Future<Output=Result<(), MigrateError>>` | sqlx migration errors are a different type. |

All three prefer `Handle::try_current()` + `block_in_place` when a Tokio runtime exists, and build a fresh runtime only as fallback. This keeps CLI invocations fast (no runtime creation on the hot path if one's already running) and avoids nested-runtime panics.

## KV store

`crates/core/runtime/src/db/kv.rs`. Namespaced key-value pairs with JSON values. Two API shapes:

- **Raw**: `set(namespace, key, value)`, `get`, `list`, `delete`. Caller controls the namespace string.
- **Scoped**: `scoped_set(workspace_id, namespace, key, value)` — prefixes namespace with `{workspace_id}::`. Workspace isolation becomes structural; a caller without the workspace_id cannot read another's data.

Used throughout runtime: skill variable storage (`skill_vars:{id}`), managed state blobs, anywhere "write a JSON value against a key" fits.

## Rebuild

`crates/core/runtime/src/db/rebuild.rs` — projection rebuild from the event log. Drop projection tables (workspaces, sessions, jobs, etc.), replay `events` in insertion order, reconstruct them. Used on daemon startup when event-log integrity suggests projection drift. See [Projections](/runtime/projections/) for details.

## Event log summary on startup

`event_log_summary(ship_dir)` returns `(event_count, latest_event_id)` — a cheap COUNT + MAX used in startup logging. Never fails: any error returns `count=-1` and `None`, so startup proceeds. Defensive: a freshly-initialized or corrupted events table shouldn't block the daemon from coming up.

## Testing

The module ships `#[cfg(test) mod tests]` with three small tests (creates fresh, idempotent, returns connection). Deeper integration tests live in `tests_envelope.rs`, `tests_integrity.rs`, `tests_pool.rs`, `tests_rebuild.rs`, `tests_session_drain.rs` — one test module per concern, all covered by `cargo test -p runtime`.

Test setup pattern:

```rust
let tmp = tempdir().unwrap();
let _ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
ensure_db().unwrap();
```

`init_project` creates the `.ship/` dir; `get_global_dir()`'s test detection returns the thread-local temp path; `ensure_db()` runs migrations against that path. Full isolation, no shared state between tests.

## Invariants

- Every SQL path sets WAL + busy_timeout + foreign_keys.
- Migrations are idempotent, tracked in `_sqlx_migrations`. Failures refuse startup rather than corrupt state.
- `~/.ship/platform.db` is never inside a project directory — it's machine-global.
- User content (collection items, schemas) lives in `data.db`, not here. If you find yourself reaching for collection data in `platform.db`, you're in the wrong file — see the collections service.
- New event writers target `events`, not `event_log`.
- The pool is static; test isolation requires bypassing it.

## Related

- [Persistence](/runtime/persistence/) — `EventStore`, `SqliteEventStore`, `ActorStore`.
- [Projections](/runtime/projections/) — read-model rebuild from the event log.
- [Kernel](/architecture/kernel/) — routing contract, `EventSink` integration.
