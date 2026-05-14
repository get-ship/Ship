---
group: Runtime
order: 3
title: Platform Database
description: platform.db schema, migrations, connection pool, WAL config, and the single-file architecture.
audience: internal
---

# Platform Database

Runtime state. Path: `~/.ship/machine/db/platform.db`. Lives at `crates/core/ship-substrate/src/db/`.

Ship has two other machine-local SQLite databases — `data.db` (user content collections) and `kv.db` (namespaced key-value state) — all under `~/.ship/machine/db/`. Each file has separate migrations and separate responsibilities. See [Data DB](/runtime/data-db/) for the collections side.

## Database split

| File | Holds | Written by |
|---|---|---|
| `platform.db` | Runtime state: sessions, jobs, events, kernel routing, actor registry, auth | Runtime code via `persistence` + `db` modules |
| `data.db` | User content: collection items, schemas, attachments | Collections service only |
| `kv.db` | Namespaced key-value state | `db::kv` |

The rest of this page covers `platform.db`. Every runtime row lives in one file: one backup target, one WAL, fewer file descriptors, joinable queries across domains. Cost: all runtime writers share a single write lock, so throughput depends on SQLite's WAL concurrency and busy_timeout behaviour.

Tests get automatic isolation via `ship_substrate::project::get_global_dir()` — it detects test binaries and returns a per-thread temp directory. Any code that resolves paths through `db_path()` inherits that isolation.

## Connection PRAGMAs

Set on every call to `ensure_db_at_path`:

| PRAGMA | Value | Why |
|---|---|---|
| `journal_mode` | `WAL` | Concurrent readers with a single writer. Required for the pool. |
| `busy_timeout` | `5000` (ms) | Tolerate brief lock contention without instant failure. |
| `foreign_keys` | `ON` | FKs enforced; schema can rely on cascade deletes. |

`open_db_at(path)` (no migration) sets only `busy_timeout` — the DB is assumed already initialized. `ensure_db_at_path` is the one-stop "make sure this DB exists, is migrated, and has the right PRAGMAs."

## Constructors

`crates/core/ship-substrate/src/db/mod.rs`:

| Function | Purpose |
|---|---|
| `db_path() -> Result<PathBuf>` | Resolves `~/.ship/machine/db/platform.db` (or test-thread temp dir). |
| `ensure_db() -> Result<()>` | Runs migrations idempotently at `db_path()`. |
| `ensure_db_at(ship_dir: &Path) -> Result<()>` | Same, but at an explicit ship_dir. Use in tests and handlers that take `ApiState`. |
| `open_db() -> Result<SqliteConnection>` | Shortcut: `ensure_db()` then `connect()`. |
| `open_db_at(path: &Path) -> Result<SqliteConnection>` | Open without migrating. DB must already exist. |

Migration failure causes `ensure_db` to refuse startup. The error message surfaces the hint "this usually means a different Ship binary created this database" and points to `ship daemon reset-db`. **Migrations never delete user data on failure.**

## Connection pool

`crates/core/ship-substrate/src/db/pool.rs`. Optional — used by the daemon for hot paths (event queries, KV reads, projection lookups). Not used by CLI invocations that open and close per-command.

- Size: `DEFAULT_POOL_SIZE = 5` (4 read + 1 write, per SQLite WAL semantics).
- Acquisition: `platform_pool() -> &'static SqlitePool`. Lazy; initialized on first call via `OnceLock`.
- Custom size: `create_pool(path, size)` for explicit construction in tests or specialized callers.

**Not suitable for per-test isolation** — the static `PLATFORM_POOL` persists across test runs and its path is fixed at first init. Tests must use `open_db_at()` or `create_pool(path, size)` against a temp dir.

## Migrations

Three files in `crates/core/ship-substrate/migrations/`:

| File | Contents |
|---|---|
| `0001_schema.sql` | The core schema — all tables below. |
| `0002_drop_kv_state.sql` | Drops `kv_state` — KV moved to `kv.db`. |
| `202605121200_jobs_projection.sql` | The `jobs` projection table + its indexes. |

Header comment in `0001_schema.sql`: *"Ship platform schema — single-file, pre-launch."* The pre-launch convention is to fold schema changes into `0001` and wipe + recreate the DB rather than chain incremental migrations — Ship has no users, so there is nothing to preserve. New numbered/timestamped migrations land only when an incremental change is genuinely cheaper than a wipe.

Migrations run via `sqlx::migrate!("./migrations")`. Tracking table: `_sqlx_migrations`. `ensure_db_at_path` invokes migrations on every call — the tracking table makes it idempotent.

## Tables

`0001_schema.sql` creates:

| Table | Purpose |
|---|---|
| `kv_state` | Created here, then **dropped** by `0002_drop_kv_state.sql`. KV lives in `kv.db` now. |
| `sessions` | Session analytics records. Indexed by branch + start time. |
| `events` | Actor-scoped event bus. Used by kernel routing and `SqliteEventStore`. |
| `cursors` | Replay/delivery position tracking. |
| `daemon_settings` | Daemon-level settings. |
| `auth_sessions` | Paired daemon session tokens (the auth-session projection). |
| `daemon_owner` | Daemon-owner binding (which user paired this daemon). |
| `projects` | Project registry — one row per known project path. |
| `project_compile_state` | Last-compile state per project. |
| `actors` | Actor registration metadata. |

`202605121200_jobs_projection.sql` adds:

| Table | Purpose |
|---|---|
| `jobs` | Agent work queue / job projection. Indexed by status and slug. |

There is no `event_log` table. The kernel event bus is the single `events` table — every typed event goes there.

## The events table

`events` is the actor-scoped event bus backing the kernel. Populated by `SqliteEventStore::append` and `ActorStore::append`. Oriented around "actor X emitted this typed event at this version." Indexes: `(entity_id, event_type)`, `(workspace_id, created_at)`.

## Blocking helpers

`db/mod.rs` exposes three `block_on` variants because sqlx is async-only and much of Ship's call surface is sync:

| Helper | Signature | Use |
|---|---|---|
| `block_on` | `Future<Output=Result<T, sqlx::Error>>` | Standard sqlx calls. Current-thread runtime when no handle is present. |
| `block_on_anyhow` | `Future<Output=anyhow::Result<T>>` | Futures that internally call `block_in_place` (e.g. nested sqlx calls). Multi-thread runtime required. |
| `block_on_migrate` | `Future<Output=Result<(), MigrateError>>` | sqlx migration errors are a different type. |

All three prefer `Handle::try_current()` + `block_in_place` when a Tokio runtime exists, and build a fresh runtime only as fallback. This keeps CLI invocations fast (no runtime creation on the hot path if one's already running) and avoids nested-runtime panics.

## KV store

KV does not live in `platform.db` — it has its own file, `kv.db`, under `~/.ship/machine/db/`. The API is `crates/core/ship-platform/src/kv.rs`: `set(ns, key, value)`, `get`, `delete`, `list_keys`, `list_all`, all keyed by a typed `KvNamespace` enum. Used for skill variable storage, managed state blobs, anywhere "write a JSON value against a key" fits. `migrate_from_platform_db` is a one-time startup move that drained the old `kv_state` table into `kv.db` before `0002_drop_kv_state.sql` dropped it.

## Rebuild

`crates/core/ship-substrate/src/db/rebuild.rs` — projection rebuild from the event log. Drop projection tables (sessions, actors), replay `events` in insertion order, reconstruct them. Used on daemon startup when event-log integrity suggests projection drift. See [Projections](/runtime/projections/) for details.

## Event log summary on startup

`event_log_summary(ship_dir)` returns `(event_count, latest_event_id)` — a cheap COUNT + MAX used in startup logging. Never fails: any error returns `count=-1` and `None`, so startup proceeds. Defensive: a freshly-initialized or corrupted events table shouldn't block the daemon from coming up.

## Testing

The module ships `#[cfg(test) mod tests]` with three small tests (creates fresh, idempotent, returns connection). Deeper integration tests live in `tests_envelope.rs`, `tests_integrity.rs`, `tests_pool.rs`, `tests_rebuild.rs`, `tests_session_drain.rs` — one test module per concern, all covered by `cargo test -p ship-substrate`.

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
- `~/.ship/machine/db/platform.db` is never inside a project directory — it's machine-global.
- User content (collection items, schemas) lives in `data.db`, not here. If you find yourself reaching for collection data in `platform.db`, you're in the wrong file — see the collections service.
- All typed events go to the single `events` table.
- The pool is static; test isolation requires bypassing it.

## Related

- [Persistence](/runtime/persistence/) — `EventStore`, `SqliteEventStore`, `ActorStore`.
- [Projections](/runtime/projections/) — read-model rebuild from the event log.
- [Kernel](/architecture/kernel/) — routing contract, `EventSink` integration.
