---
group: Runtime
order: 4
title: Projections
description: Read models derived from the event log. EventBus, Projection trait, sync vs async variants, rebuild path.
audience: internal
---

# Projections

Projections are split across two crates. Low-level projections (actor, process, session, spawn, registry) live in `crates/core/ship-substrate/src/projections/`; higher-level domain projections (auth, job, compile_state, projects) live in `crates/core/ship-platform/src/projections/`. Read models maintained as SQL tables in `platform.db`, derived entirely from events. Drop any projection table and rebuild it from the log — projections are never the source of truth.

```
ship-substrate/src/projections/
  mod.rs                — re-exports
  registry.rs           — Projection trait, EventBus
  async_projection.rs   — AsyncProjection trait, broadcast-consuming spawner
  spawn.rs              — spawn_with_failure_counter (Mailbox-based)
  session.rs            — SessionProjection (materialized)
  actor.rs              — ActorProjection (materialized)
  process.rs            — ProcessRecord, ProcessStatus (computed)

ship-platform/src/projections/
  mod.rs                — re-exports
  auth.rs               — auth projection
  job.rs                — JobRecord, JobStatus, load_jobs, project (computed)
  job_transitions.rs    — state machine for job status transitions
  compile_state.rs      — compile-state projection
  projects.rs           — projects projection
```

Two patterns live here:

- **Materialized**: a projection writes to a SQL table on every applicable event. `Session`, `Actor`. Table is a cache; truncated and replayed on rebuild.
- **Computed**: no table. `load_jobs()` / `load_processes()` reads events, folds them in memory, returns state. `Job`, `Process`.

The two shapes coexist by design. Materialized projections earn their keep when the read load justifies a persistent index (list all workspaces, sort sessions by start time). Computed projections skip the table when the fold is cheap enough or the write volume is too noisy.

## Projection trait

```rust
pub trait Projection: Send + Sync {
    fn name(&self) -> &str;
    fn event_types(&self) -> &[&str];
    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()>;
    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()>;
}
```

- `name()` — for logging and selective rebuild.
- `event_types()` — exact event-type strings this projection handles. `EventBus` uses this for dispatch filtering. Not a prefix match — each string is an exact `event_type`.
- `apply(event, conn)` — mutate the read model for a single event. Same code path used live (on append) and during replay.
- `truncate(conn)` — drop all derived state. Called at the start of a rebuild before replay.

One code path for live and rebuild is the design — if your projection produces a different result on replay than it did live, you have a bug, and the rebuild path is the forcing function.

## EventBus

```rust
pub struct EventBus { projections: Vec<Box<dyn Projection>> }
```

- `register(projection)` — add a projection to the bus.
- `dispatch(event, conn)` — called synchronously after every event append in the live path. Each projection whose `event_types()` contains `event.event_type` gets `apply()` invoked. Handler errors are logged but do not fail the caller — the event is already persisted.
- `rebuild(events, conn)` — truncate all registered projections, then replay the full event list through `apply`. Used by `db::rebuild::rebuild_projections`.

The live-path call is synchronous so event append + projection update can share a single transaction. See the `ship_substrate::db::session_events` helpers (e.g. `insert_session_with_started_event`) for the transactional pattern.

## AsyncProjection

```rust
pub trait AsyncProjection: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn event_types(&self) -> &[&str];
    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()>;
    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()>;
}
```

Same contract, different execution model. Designed to run as a long-lived tokio task consuming from a broadcast receiver or a kernel `Mailbox`.

Two spawner functions:

- `async_projection::spawn_projection<P>(projection, broadcast::Receiver, db_opener)` — reads from a broadcast channel. Filters by `event_types()` before calling `apply`. Opens a fresh connection per event via `db_opener`. Logs `Lagged(n)` and continues (broadcast receivers can fall behind).
- `spawn::spawn_with_failure_counter<P>(projection, Mailbox, db_opener)` — reads from a kernel `Mailbox`. Increments an atomic `AtomicU64` counter on every apply or db-open error. Returns a `ProjectionHandle { name, handle: JoinHandle<()>, failure_count }`. The daemon uses this to watch projection health.

Concrete async implementations: `SessionProjection`, `ActorProjection` implement both `Projection` (sync, for rebuild) and `AsyncProjection` (for live daemon operation). The daemon spawns them as async tasks; rebuild runs them as sync projections through the `EventBus`.

## Materialized projections

### SessionProjection

Maintains the `sessions` table in `platform.db`. Handles:

```
session.started (INSERT), session.attached (no-op — row already exists),
session.ended (UPDATE), session.drain_started, session.drain_completed,
session.drain_aborted, session.tool_count_incremented, session.heartbeat
```

`session.progress` is deliberately not handled — progress events don't affect the session row. If you need progress-level queries, read the event log.

### ActorProjection

Maintains `actors` table. Handles actor lifecycle:

```
actor.created, actor.woke, actor.slept, actor.stopped, actor.crashed
```

## Computed projections

### Jobs (`job.rs`)

No table. `load_jobs()` queries `events` filtered by `job.*` types and folds them into a `Vec<JobRecord>` via `project()`.

```rust
pub struct JobRecord {
    job_id, slug, agent, branch, spec_path, source_spec_path,
    depends_on, status, worktree, blocker, error, model, harness,
    created_at, updated_at,
    harness_meta,
    pipeline, steps, current_phase,       // pipeline state (legacy + DAG)
    completed_steps, current_step, vars, step_retries,
    ...
}

pub enum JobStatus {
    Pending, Dispatched, Completed,
    GatePending, Blocked, Merged,
    Failed, Cancelled,
}
```

`JobStatus::Cancelled` is deliberately distinct from `Failed` — operator-initiated cancellation vs execution failure. Post-mortems care about the difference.

`job_transitions.rs` holds the state machine — which status → status transitions are valid, who fires them, what event type corresponds to each.

### Processes (`process.rs`)

Also no table. `load_processes(slug)` queries `events` filtered by `process.*` types for a given slug, folds them into `ProcessRecord`s keyed by `(slug, pid)`.

```rust
pub struct ProcessRecord {
    pub status: ProcessStatus,  // Spawned, Signaled, Exited, etc.
    pub pid: Option<i32>,
    pub spawned_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub signals: Vec<(DateTime<Utc>, String)>,
    ...
}
```

Multiple records per slug — pipeline restarts, crashes, retries all produce distinct rows. Records with `pid: None` use `(slug, spawned_at)` as a synthetic key.

## Rebuild path

`ship_substrate::db::rebuild::rebuild_projections(ship_dir)`:

1. Load all events from `platform.db` ordered by `id ASC`.
2. Construct a fresh `EventBus`:

   ```rust
   let mut bus = EventBus::new();
   bus.register(Box::new(SessionProjection::new()));
   bus.register(Box::new(ActorProjection::new()));
   ```

3. Call `bus.rebuild(&events, &mut conn)` — truncates all registered projection tables, then replays every event through `apply`.

Returns `RebuildReport { events_replayed, projections_rebuilt, duration_ms }`.

**Only materialized projections are registered.** Computed projections (Job, Process) need no rebuild — they fold from the event log on every read.

`rebuild_projections_default()` is the convenience wrapper: resolves `ship_dir` via `get_global_dir()` and calls `rebuild_projections`.

## Live-path integration

The daemon's startup sequence registers async projections as long-lived tasks. Each task receives events via a kernel `Mailbox` scoped to the projection's actor identity and the relevant namespaces. See the daemon's `ShipRuntime` construction for the exact wiring.

The daemon does not use `EventBus::dispatch` for live events. `EventBus` exists primarily for rebuild and for tests. Live dispatch goes:

1. Event emitted → `KernelRouter::route`
2. `sink.persist(&event)` writes to `events` table
3. Delivery to subscribed mailboxes — async projection tasks consume and apply

The sync `EventBus::dispatch` pattern is still used in paths that want event-append and projection-update in one transaction (rare — mostly tests).

## Failure handling

- **Sync dispatch**: errors in `apply()` are logged via `tracing::warn!`, swallowed. The event is already persisted, so a failed projection update is recoverable via rebuild. `EventBus::dispatch` never returns errors to the caller.
- **Async projections via `spawn_with_failure_counter`**: errors increment the shared `failure_count: AtomicU64`. Daemon supervision can inspect the counter to detect a wedged projection. No automatic restart — supervision policy lives outside the projection code.
- **db_opener failures**: also increment the counter. A projection that can't acquire a connection is effectively dead but doesn't crash.

## Testing

Per-projection tests: `tests.rs`, `tests_actor.rs`, `tests_async.rs`, `tests_daemon_events.rs`, `tests_job.rs`, `tests_process.rs`, `tests_session.rs`. Each constructs a temp DB, writes a sequence of events, runs the projection, asserts the derived state.

Rebuild tests: `crates/core/ship-substrate/src/db/tests_rebuild.rs`. Pattern:

1. Start with a known event log.
2. Let projections build up tables.
3. Mutate the tables directly to simulate drift.
4. Call `rebuild_projections`.
5. Assert tables now match what the event log implies.

## Invariants

- Projections never write events. They consume them.
- Every projection-table row is derivable from the event log. No row with no supporting event.
- Sync and async variants of a projection must produce the same end state for the same event sequence.
- Truncate + replay is safe. If it isn't, the projection has a bug (most often: `apply` reads external state, which replay doesn't have).
- `event_types()` is exhaustive. An event not listed is skipped silently — if you add a new event type that affects a projection, update the list.

## Related

- [Persistence](/runtime/persistence/) — `EventStore`, `SqliteEventStore`, `ActorStore`.
- [Platform DB](/runtime/platform-db/) — tables, migrations, connection pool.
- [Kernel](/architecture/kernel/) — routing contract; where events arrive before projections see them.
