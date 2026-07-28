use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::db::envelope::{EVENT_COLS as COLS, envelope_from_row, insert_event_row};
use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::EventFilter;
use crate::projections::{ActorProjection, EventBus};
use ship_kernel::EventSink;

pub trait EventStore: Send + Sync {
    fn append(&self, event: &EventEnvelope) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<EventEnvelope>>;
    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>>;
    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>>;
}

/// Build the default projection bus (ActorProjection
/// that the L0 substrate ships with no domain coupling). Every
/// `SqliteEventStore` carries this bus, and `append` dispatches matching
/// events to it in the same transaction as the row insert.
///
/// CollectionItemProjection (formerly CollectionProjection) moved to
/// ship-collection at L1 (keystone A, 2026-06-10). It is wired in via
/// ship-platform's `install_extra_projections_factory` call so same-TX
/// semantics are preserved.
fn default_bus() -> Arc<EventBus> {
    let mut bus = EventBus::new();
    bus.register(Box::new(ActorProjection::new()));
    // ArtifactProjection (artifact METADATA; blobs stay on the blob plane)
    // moved to ship-artifacts (L1, keystone-A de-bucket 2026-06-10) — the
    // daemon installs it via `install_extra_projections_factory`, same
    // same-transaction bus as everything here.
    // AnalyticsProjection lives in the `ship-analytics` crate. The
    // daemon registers it at startup via
    // `install_extra_projections_factory` (see apps/shipd/src/lib.rs);
    // substrate stays free of downstream-consumer dependencies.
    for projection in crate::projections::extra_projections() {
        bus.register(projection);
    }
    Arc::new(bus)
}

pub struct SqliteEventStore {
    db_path: PathBuf,
    bus: Arc<EventBus>,
}

impl SqliteEventStore {
    /// Create an event store at an explicit db-file path.
    ///
    /// Takes the concrete db file directly — used by `PartitionedSink` to back
    /// each scope with its own per-scope db (`spaces/<id>/space.db`, `runtime.db`,
    /// etc.). Ensures the events schema exists at that path before returning.
    ///
    /// This is the sole event-store constructor: it can only ever materialize the
    /// db file it is handed, so there is no path by which it resurrects the dead
    /// `platform.db`. Per-scope production paths come from [`crate::db::runtime_db_path`]
    /// / [`crate::db::account_db_path`] / [`crate::db::space_db_path`]; use
    /// [`Self::new_for_scope`] to resolve one from a `ship_dir` + `EventScope`.
    pub fn new_at_db_path(db_path: &Path) -> Result<Self> {
        crate::db::ensure_events_db_at_path(db_path)?;
        Ok(Self {
            db_path: db_path.to_path_buf(),
            bus: default_bus(),
        })
    }

    /// The concrete db file this store reads/writes. Used by `PartitionedSink`
    /// to enumerate mounted scope dbs for the shutdown WAL flush.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// The enforced off-kernel write chokepoint.
    ///
    /// Every off-kernel emit site (the `db::*_events` helpers, `log`, the
    /// platform CRUD/audit emitters) writes through here, NOT through the raw
    /// `EventStore::append`. It runs the SAME stamp-or-reject rule the kernel
    /// `emit` chokepoint runs — `apply_stamp_chokepoint` (stamp author,
    /// enforce scope, validate payload schema) against the authoritative
    /// `binding` — BEFORE the row is persisted, so a rejected
    /// (cross-space / space-unbound / malformed-payload) event is never
    /// half-written. Closing blueprint #91 by construction: if the kernel
    /// path validates payloads, so does everyone.
    ///
    /// Returns the stamped envelope so callers deliver exactly the enforced row
    /// (scope_key + author) to actor mailboxes rather than the pre-stamp draft.
    ///
    /// `apply_stamp_chokepoint` rejects on:
    /// - chosen scope ∉ the type's allowed-set (`ScopeNotAllowed`),
    /// - a Space event whose key ≠ the binding's space (`CrossSpace`),
    /// - a Space event from a space-unbound binding (`NoSpaceForSpaceEvent`),
    /// - a malformed payload for a known event type (`SchemaViolation`).
    pub fn append_enforced(
        &self,
        event: &EventEnvelope,
        binding: &ship_types::ScopeBinding,
    ) -> Result<EventEnvelope> {
        let mut stamped = event.clone();
        crate::events::apply_stamp_chokepoint(
            &mut stamped,
            binding,
            &crate::events::off_kernel_ctx(binding.clone()),
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        self.append(&stamped)?;
        Ok(stamped)
    }

    /// Create an event store bound to a single scope's db (Phase 4d).
    ///
    /// Resolves the per-scope db path under `ship_dir` exactly as
    /// `PartitionedSink::route` does, so an off-kernel `append()` lands in the
    /// same db the kernel sink would have routed the event to:
    ///
    /// - `Runtime` → `runtime.db`
    /// - `Account` → `account.db`
    /// - `Space`   → `spaces/<scope_key>/space.db` (a `scope_key` is required;
    ///   a Space write with no space id has no home and is rejected — the
    ///   caller must resolve the space first or emit under Runtime)
    ///
    /// This is the bounded fix for the off-kernel emit sites that previously
    /// constructed via `new()`/`new_at()` and wrote a `platform.db` nobody
    /// projects or reads. Full kernel-routing of these helpers is deferred to
    /// the log-as-truth arc (#91).
    pub fn new_for_scope(
        ship_dir: &Path,
        scope: ship_types::event_registry::EventScope,
        scope_key: Option<&str>,
    ) -> Result<Self> {
        use ship_types::event_registry::EventScope;
        let db_path = match scope {
            EventScope::Runtime => crate::db::runtime_db_path(ship_dir),
            EventScope::Account => crate::db::account_db_path(ship_dir),
            EventScope::Space => {
                let key = scope_key.ok_or_else(|| {
                    anyhow::anyhow!(
                        "new_for_scope: Space store requires a space_id scope_key"
                    )
                })?;
                crate::db::space_db_path(ship_dir, key)
            }
        };
        Self::new_at_db_path(&db_path)
    }

    /// Construct a store at `db_path` with an explicit projection bus.
    ///
    /// Test-only seam: the production constructors always wire `default_bus()`,
    /// which cannot be perturbed to make a handler fail. The transaction-
    /// atomicity proof needs a store whose bus contains a deliberately-failing
    /// projection so it can assert that `append` rolls the event row back when
    /// dispatch errors. No production path constructs a store this way.
    #[cfg(test)]
    pub(crate) fn new_with_bus(db_path: &Path, bus: Arc<EventBus>) -> Result<Self> {
        crate::db::ensure_events_db_at_path(db_path)?;
        Ok(Self {
            db_path: db_path.to_path_buf(),
            bus,
        })
    }

    /// One attempt at the durable append transaction (BEGIN IMMEDIATE ..
    /// COMMIT). Never partially commits: a failure at any step rolls back,
    /// so re-invoking this on retry re-runs the whole transaction rather than
    /// resuming a partial one.
    fn append_once(&self, event: &EventEnvelope) -> Result<()> {
        let mut conn = open_db_at(&self.db_path)?;

        block_on(async { sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await })?;

        if let Err(e) = insert_event_row(event, &mut conn) {
            // Census §2: a ROLLBACK failure here is NOT the same as the
            // original event-insert failure, but it deserves an error-level
            // breadcrumb — the kernel has now been told the transaction is
            // in flight, and a stuck BEGIN is a stuck writer. Surface both
            // errors (the original `e` and the rollback error) so the caller
            // can log them with full context, and never silently drop the
            // rollback outcome.
            if let Err(rollback_err) =
                block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await })
            {
                tracing::error!(
                    error = %e,
                    rollback_error = %rollback_err,
                    db_path = %self.db_path.display(),
                    "ROLLBACK failed after event-insert error; transaction may be stuck"
                );
            }
            return Err(e);
        }

        if let Err(e) = self.bus.dispatch(event, &mut conn) {
            if let Err(rollback_err) =
                block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await })
            {
                tracing::error!(
                    error = %e,
                    rollback_error = %rollback_err,
                    db_path = %self.db_path.display(),
                    "ROLLBACK failed after projection-dispatch error; transaction may be stuck"
                );
            }
            return Err(e);
        }

        block_on(async { sqlx::query("COMMIT").execute(&mut conn).await })?;
        Ok(())
    }
}

/// Bounded retry on transient SQLITE_BUSY/SQLITE_LOCKED contention only.
/// Each attempt already blocks up to the connection's own 5s `busy_timeout`
/// before it can even fail transient, so the retry budget has to cover
/// several such waits, not fractions of one: 8 attempts (1 initial + 7
/// retries), full-jitter backoff doubling from 20ms up to a 400ms
/// per-attempt cap between attempts, bounded by a 45s total wall-clock cap
/// so a single append can retry through several lock hand-off windows
/// under the 300-thread torture test without masking a genuinely stuck
/// writer as a slow one.
const APPEND_RETRY_MAX_ATTEMPTS: u32 = 8;
const APPEND_RETRY_BASE_BACKOFF_MS: u64 = 20;
const APPEND_RETRY_MAX_BACKOFF_MS: u64 = 400;
const APPEND_RETRY_TOTAL_CAP: Duration = Duration::from_secs(45);

/// Extended sqlite result codes are `(family << 8) | primary`; SQLITE_BUSY
/// (5) and SQLITE_LOCKED (6) are the transient-contention primaries, e.g.
/// SQLITE_BUSY_TIMEOUT is `773 == (3 << 8) | 5`.
fn is_transient_sqlite_busy(err: &anyhow::Error) -> bool {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<sqlx::Error>())
        .and_then(|e| e.as_database_error())
        .and_then(|db_err| db_err.code())
        .and_then(|code| code.parse::<i32>().ok())
        .map(|code| matches!(code & 0xff, 5 | 6))
        .unwrap_or(false)
}

/// Full-jitter backoff: a pseudo-random delay in `[0, cap_ms]`, seeded from
/// the wall clock and thread id (no external RNG dependency needed for a
/// bounded-retry backoff).
fn append_retry_jitter_ms(attempt: u32, cap_ms: u64) -> u64 {
    use std::hash::{Hash, Hasher};
    if cap_ms == 0 {
        return 0;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    attempt.hash(&mut hasher);
    nanos.hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    hasher.finish() % (cap_ms + 1)
}

/// Query a single DB file with the given filter. Returns empty vec if the DB
/// does not have an events table (e.g. an uninitialized file).
fn query_one_db(path: &Path, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
    crate::persistence::build_event_query(path, filter)
}

impl EventStore for SqliteEventStore {
    /// Persist an event row and apply registered projections in a single
    /// transaction. This is the canonical event write path; `kernel.emit`
    /// reaches it via `EventSink::persist`.
    ///
    /// Atomicity: the event row and any matching projection writes commit
    /// together, or roll back together. A projection-dispatch error fails the
    /// transaction — the `BEGIN IMMEDIATE` is rolled back and the event row is
    /// NOT committed, so a rejected projection leaves no orphaned event whose
    /// derived state was skipped. The error is returned to the caller (the
    /// kernel emit path then rejects, and never delivers to mailboxes). See
    /// `EventBus::dispatch` for the first-error-wins handler semantics.
    fn append(&self, event: &EventEnvelope) -> Result<()> {
        let start = Instant::now();
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self.append_once(event) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    let retryable = attempt < APPEND_RETRY_MAX_ATTEMPTS
                        && start.elapsed() < APPEND_RETRY_TOTAL_CAP
                        && is_transient_sqlite_busy(&e);
                    if !retryable {
                        return Err(e);
                    }
                    let cap_ms = APPEND_RETRY_BASE_BACKOFF_MS
                        .saturating_mul(1u64 << (attempt - 1).min(6))
                        .min(APPEND_RETRY_MAX_BACKOFF_MS);
                    std::thread::sleep(Duration::from_millis(append_retry_jitter_ms(
                        attempt, cap_ms,
                    )));
                }
            }
        }
    }

    fn get(&self, id: &str) -> Result<Option<EventEnvelope>> {
        let mut conn = open_db_at(&self.db_path)?;
        let row = block_on(async {
            sqlx::query(&format!("SELECT {COLS} FROM events WHERE id = ?"))
                .bind(id)
                .fetch_optional(&mut conn)
                .await
        })?;
        match row {
            Some(r) => envelope_from_row(&r, true).map(Some),
            None => Ok(None),
        }
    }

    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        query_one_db(&self.db_path, filter)
    }

    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>> {
        self.query(&EventFilter {
            entity_id: Some(entity_id.to_string()),
            ..Default::default()
        })
    }

}

impl EventSink for SqliteEventStore {
    /// Persist an event to the SQLite events table.
    ///
    /// Delegates to the same append path used by `EventStore::append`, but
    /// satisfies the `ship_kernel::EventSink` trait so `ship_kernel::KernelRouter`
    /// can use `SqliteEventStore` as its persistence back-end.
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        self.append(event)
    }
}

#[cfg(test)]
mod event_sink_tests {
    use super::*;
    use ship_kernel::EventSink;
    use tempfile::tempdir;

    #[test]
    fn sqlite_event_store_persist_writes_to_events_table() {
        let tmp = tempdir().unwrap();
        let store =
            SqliteEventStore::new_at_db_path(&crate::db::runtime_db_path(tmp.path())).unwrap();
        let event = EventEnvelope::new("test.sink", "entity-1", &serde_json::json!({})).unwrap();

        store.persist(&event).unwrap();

        let retrieved = store.get(&event.id).unwrap().expect("event must be in DB after persist");
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.event_type, "test.sink");
    }

    #[test]
    fn sqlite_event_store_implements_event_sink_trait() {
        let tmp = tempdir().unwrap();
        let store =
            SqliteEventStore::new_at_db_path(&crate::db::runtime_db_path(tmp.path())).unwrap();
        // Verify the type can be used as Arc<dyn EventSink>
        let sink: std::sync::Arc<dyn EventSink> = std::sync::Arc::new(store);
        let event = EventEnvelope::new("test.trait", "entity-2", &serde_json::json!({})).unwrap();
        sink.persist(&event).unwrap();
    }

    #[test]
    fn append_enforced_rejects_malformed_payload_for_known_event_type() {
        // Blueprint #91: off-kernel writes used to skip the payload
        // validator. The shared chokepoint now runs the SAME gate the
        // kernel emit path runs — so a malformed payload for a known
        // event type is rejected before persist.

        let tmp = tempdir().unwrap();
        let store =
            SqliteEventStore::new_at_db_path(&crate::db::runtime_db_path(tmp.path())).unwrap();
        let event = EventEnvelope::new(
            "actor.created",
            "actor-1",
            &serde_json::json!({ "garbage": true }),
        )
        .unwrap();
        let binding = ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk);
        let err = store.append_enforced(&event, &binding).unwrap_err();
        assert!(
            err.to_string().contains("schema")
                || err.to_string().contains("Schema")
                || err.to_string().contains("missing field"),
            "unexpected error: {err}"
        );
    }
}

