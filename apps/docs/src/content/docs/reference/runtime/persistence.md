---
title: "Persistence"
description: "EventStore trait, SqliteEventStore, ActorStore, and the append path. How events land in platform.db and how actors see their own slice."
sidebar:
  label: "Persistence"
  order: 2
---
Location: `crates/core/runtime/src/persistence/`. Four files (plus `cursor.rs` behind the `unstable` feature).

```
persistence/
  mod.rs            — re-exports
  event_store.rs    — EventStore trait, SqliteEventStore
  actor_store.rs    — ActorStore, raw_append helper
  snapshot.rs       — ActorSnapshot, event_stats helper
  cursor.rs         — ActorCursor (unstable, replay tracking)
```

Public re-exports from `mod.rs`:

```rust
pub use actor_store::ActorStore;
pub use event_store::{EventStore, SqliteEventStore, append_event_with_conn};
pub use snapshot::ActorSnapshot;
```

## EventStore trait

The persistence surface is a four-method trait:

```rust
pub trait EventStore: Send + Sync {
    fn append(&self, event: &EventEnvelope) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<EventEnvelope>>;
    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>>;
    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>>;
}
```

`query_aggregate` is `query` with only `entity_id` set — a convenience for loading the full event stream for one aggregate.

## SqliteEventStore

Concrete implementation. Three constructors, three use cases:

| Constructor | Use case |
|---|---|
| `new()` | Default global Ship dir resolved via `db_path()`. Production CLI path. |
| `new_at(ship_dir: &Path)` | Explicit directory. Required in tests to avoid thread-local / env-var isolation issues. |
| `new_pooled(ship_dir: &Path)` | Connection pool (size 5). Daemon uses this for high-throughput events. Requires a running Tokio runtime. |

All three call `ensure_db()` or `ensure_db_at()` — migrations run idempotently before the store is usable.

Struct:

```rust
pub struct SqliteEventStore {
    db_path: PathBuf,
    pool: Option<sqlx::SqlitePool>,
}
```

`pool` is `Some` only when constructed via `new_pooled`. `append`, `get`, and `query` check the field and either use the pool or open a per-call connection via `open_db_at`. There's no fallback path from pool to per-call — if the pool is exhausted, the operation fails.

## EventSink implementation

`SqliteEventStore` also implements `ship_kernel::EventSink`:

```rust
impl EventSink for SqliteEventStore {
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        self.append(event)
    }
}
```

This is what lets `ship_kernel::KernelRouter` use `SqliteEventStore` as its persistence back-end. The kernel's routing contract calls `sink.persist(&event)` synchronously before mailbox delivery. See [Kernel](/reference/architecture/kernel/) for the full contract.

## The events table

One table, shared by every actor:

```sql
CREATE TABLE events (
  id              TEXT PRIMARY KEY,
  event_type      TEXT NOT NULL,
  entity_id       TEXT NOT NULL,
  actor           TEXT,              -- human-readable actor name
  payload_json    TEXT,
  version         INTEGER NOT NULL,
  causation_id    TEXT,
  workspace_id    TEXT,
  session_id      TEXT,
  actor_id        TEXT,              -- logical isolation key
  parent_actor_id TEXT,
  elevated        INTEGER,
  created_at      TEXT NOT NULL,     -- rfc3339
  target_actor_id TEXT               -- direct-message target
);
```

Queries order by `id ASC` — IDs are monotonic so this preserves insertion order. `EventFilter` supports filtering by `entity_id`, `event_type`, `workspace_id`, `session_id`, `actor_id`, `parent_actor_id`, `elevated_only`, and `since` (timestamp), with optional `limit`.

## ActorStore

The actor-scoped handle. Lives in `actor_store.rs`. Enforces two boundaries:

**Write namespace enforcement.** `append(event)` checks `event.event_type` against `write_namespaces` (prefix match). Out-of-scope writes return an error before hitting SQL.

**Actor ID stamping.** Every event written through `ActorStore::append` has its `actor_id` overwritten with the store's owning actor. Any pre-existing `actor_id` on the envelope is discarded — a store cannot launder events for another actor.

**Read scoping.** `query(filter)` overrides `filter.actor_id` with the store's own ID. If `filter.event_type` is set, it must pass a prefix match against `read_namespaces`.

Structure:

```rust
pub struct ActorStore {
    actor_id: String,
    db_path: PathBuf,
    write_namespaces: Vec<String>,
    read_namespaces: Vec<String>,
}
```

Constructed by the kernel via `KernelRouter::spawn_actor`. Actors never build their own. `open(actor_id, base_dir, writes, reads)` exists for callers that need to reconstruct a store against an existing platform.db — typically test harnesses and targeted queries.

## Logical vs physical isolation

Historical Ship gave each actor its own SQLite file. That's gone. Today, one `events` table in `~/.ship/machine/db/platform.db` holds every actor's events; isolation is enforced by:

- `actor_id` column on every row — populated at the `ActorStore::append` boundary.
- Read queries scoped by `actor_id` — unfiltered reads return only the actor's own events.
- Write namespace check — rejected before SQL.

The guarantee is at the `ActorStore` API. Direct SQL bypasses it. Tests and migrations that write via `raw_append` must set `actor_id` correctly themselves.

## raw_append

`pub(crate) fn raw_append(db_path, event)` — the shared insert path without namespace enforcement. Used internally by `ActorStore::append` (after its checks) and by the kernel router for kernel-namespace events that aren't owned by any actor. Not part of the public API.

## append_event_with_conn

```rust
pub fn append_event_with_conn(event: &EventEnvelope, conn: &mut sqlx::SqliteConnection) -> Result<()>
```

Appends using a caller-supplied connection. Used by transactional callers that need to write an event and update a projection in the same transaction. Opens no new connection; commits on the caller's schedule.

## ActorSnapshot

`snapshot.rs` exports a serializable snapshot type:

```rust
pub struct ActorSnapshot {
    pub actor_id: String,
    pub namespace: String,
    pub config: ActorConfig,
    pub db_bytes: Vec<u8>,      // MessagePack via to_bytes/from_bytes
    pub created_at: DateTime<Utc>,
    pub event_count: u64,
    pub last_event_id: Option<String>,
}
```

`event_stats(db_path, actor_id)` counts events filtered by actor_id and returns the last event ID. Used by `KernelRouter::snapshot` to populate metadata.

**Caveat on current state.** The `db_bytes` field documents as "the full SQLite DB file after a WAL checkpoint — all actor state lives in this file," which described the per-actor-DB era. Under shared-log architecture, the meaning of "this actor's DB bytes" is ambiguous — there is no per-actor file to copy. The struct is still defined; its population path needs to either extract an actor-filtered dump or the field needs to be retired. Verify before relying on snapshot/restore round-tripping.

## ActorCursor (unstable)

```rust
#[cfg(feature = "unstable")]
pub mod cursor;
```

`cursor.rs` defines `ActorCursor` — per-actor delivery position tracking:

```rust
pub struct ActorCursor {
    pub actor_label: String,
    pub last_event_id: Option<String>,
}
```

`init_cursor_table(db_path)` creates the `actor_cursors` table (primary key `actor_label`, value `last_event_id`). Feature-gated; not in the default build. Used by experimental replay logic to resume an actor's event stream from where it left off.

In shared-log Ship there's no separate kernel DB — callers pass `platform.db`.

## Testing

`new_at(ship_dir)` is the test-friendly constructor. Tests create a `tempdir()`, pass it in, and get full isolation without polluting `~/.ship`. The `#[cfg(test)] mod event_sink_tests` at the bottom of `event_store.rs` is the template — two tests verifying the `EventSink` trait impl writes to the events table and can be wrapped in `Arc<dyn EventSink>`.

Global runtime test isolation goes further: `runtime::project::get_global_dir()` detects test binaries and returns a per-thread temp directory instead of `~/.ship`. Any code that resolves a path via `db_path()` (rather than `db_path_at(ship_dir)`) inherits that isolation.

## Invariants

- Events are append-only. No update, no delete in production paths. Migrations excepted.
- `EventStore::append` is synchronous from the caller's view; SQL commit happens before return.
- `ActorStore::append` always stamps `actor_id` — the envelope passed in is treated as not-yet-scoped.
- `EventStore::query` orders by `id ASC`. Callers depending on insertion order get it.
- Pool-backed stores never fall back to per-call connections. Pool exhaustion = operation failure.

## Related

- [Kernel](/reference/architecture/kernel/) — routing contract, how `EventSink` integrates.
- [Platform DB](/reference/runtime/platform-db/) — full schema, migrations, WAL config.
- [Projections](/reference/runtime/projections/) — how read models are rebuilt from the event log.
