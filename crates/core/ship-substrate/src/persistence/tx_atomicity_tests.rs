//! Transaction-atomicity proofs (S7).
//!
//! `SqliteEventStore::append` opens one `BEGIN IMMEDIATE`, inserts the event
//! row, dispatches the projection bus on the SAME connection, then commits.
//! The contract is all-or-nothing: if a projection handler errors, the event
//! row must NOT survive — there must be no orphaned event whose derived state
//! was skipped.
//!
//! These tests register a deliberately-failing projection on a test event and
//! assert that `append` returns `Err` AND leaves zero rows in `events`. The
//! mailbox-non-delivery half of the contract is structural: the kernel router
//! only `deliver()`s after `sink.persist()?` succeeds (see
//! `ship_kernel::KernelRouter::route`), so a persist that returns `Err` is
//! never delivered — proven here by asserting the `Err` and separately covered
//! by the kernel `route` tests.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use sqlx::SqliteConnection;
use tempfile::tempdir;

use super::event_store::{EventStore, SqliteEventStore};
use crate::db::{self, block_on};
use crate::projections::{EventBus, Projection};
use ship_types::envelope::EventEnvelope;

const FAIL_TYPE: &str = "test.tx-fail";

/// A projection that always errors on `FAIL_TYPE`. Stands in for any handler
/// that rejects an event it was asked to apply.
struct FailingProjection;

impl Projection for FailingProjection {
    fn name(&self) -> &str {
        "failing-test-projection"
    }

    fn event_types(&self) -> &[&str] {
        &[FAIL_TYPE]
    }

    fn apply(&self, _event: &EventEnvelope, _conn: &mut SqliteConnection) -> Result<()> {
        Err(anyhow!("deliberate projection-handler failure"))
    }

    fn truncate(&self, _conn: &mut SqliteConnection) -> Result<()> {
        Ok(())
    }
}

/// Count rows in the `events` table at `path`.
fn event_row_count(path: &std::path::Path) -> i64 {
    let mut conn = db::open_db_at(path).unwrap();
    block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap()
}

fn store_with_failing_projection(path: &std::path::Path) -> SqliteEventStore {
    let mut bus = EventBus::new();
    bus.register(Box::new(FailingProjection));
    SqliteEventStore::new_with_bus(path, Arc::new(bus)).unwrap()
}

/// THE PROOF: a projection-handler error must roll the event row back.
///
/// Pre-S7 the bus swallowed the handler error (`tracing::warn!`), `append`
/// returned `Ok`, and the event row stayed committed — an orphan whose
/// projection write was skipped. The fix makes `dispatch` propagate the error
/// and `append` roll back, so the write is all-or-nothing.
#[test]
fn append_rolls_back_event_row_when_projection_handler_fails() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = store_with_failing_projection(&db_path);

    let event = EventEnvelope::new(FAIL_TYPE, "entity-fail", &serde_json::json!({})).unwrap();

    // 1. append must surface the handler error.
    let result = store.append(&event);
    assert!(
        result.is_err(),
        "append must return Err when a projection handler fails (got Ok — error was swallowed)"
    );

    // 2. the event row must NOT be persisted: a failed projection leaves no
    //    orphaned event.
    assert_eq!(
        event_row_count(&db_path),
        0,
        "the event row must be rolled back when its projection handler fails"
    );
    assert!(
        store.get(&event.id).unwrap().is_none(),
        "the rolled-back event must not be readable by id"
    );
}

/// A handler that succeeds still commits the row — the rollback is scoped to
/// the failure path, not a blanket refusal to ever commit.
#[test]
fn append_commits_event_row_when_projections_succeed() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    // Default bus has no handler for `test.ok`, so dispatch is a no-op success.
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();

    let event = EventEnvelope::new("test.ok", "entity-ok", &serde_json::json!({})).unwrap();
    store.append(&event).unwrap();

    assert_eq!(event_row_count(&db_path), 1, "a clean append must commit one row");
    assert!(store.get(&event.id).unwrap().is_some());
}

/// `rebuild` must surface a handler error rather than silently warn — a drifted
/// projection that errors on replay must fail loud, not pretend it rebuilt.
#[test]
fn rebuild_surfaces_projection_handler_errors() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    db::ensure_events_db_at_path(&db_path).unwrap();
    let mut conn = db::open_db_at(&db_path).unwrap();

    let mut bus = EventBus::new();
    bus.register(Box::new(FailingProjection));

    let events = vec![EventEnvelope::new(FAIL_TYPE, "entity-fail", &serde_json::json!({})).unwrap()];

    let result = bus.rebuild(&events, &mut conn);
    assert!(
        result.is_err(),
        "rebuild must surface a projection-handler error during replay"
    );
}
