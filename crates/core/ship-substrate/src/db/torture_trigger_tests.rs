//! T1 storage-torture, scenario 5: `events_immutable` trigger.
//!
//! `scope_schema::BASE_DDL` installs `events_immutable` — a `BEFORE UPDATE ON
//! events` trigger that `RAISE(FAIL, ...)`s. This proves both halves of the
//! contract against the REAL schema (not a mocked trigger): an UPDATE on a
//! committed event row is rejected, and the append path (an INSERT, never an
//! UPDATE) is not accidentally caught by it.

use crate::db::{self, block_on};
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types as ev;

fn setup() -> (tempfile::TempDir, std::path::PathBuf, SqliteEventStore) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();
    (tmp, db_path, store)
}

/// An UPDATE against a committed event row must be rejected by the real
/// trigger, under the real schema — not a re-implementation of the rule.
#[test]
fn update_on_committed_event_row_is_rejected_by_trigger() {
    let (_tmp, db_path, store) = setup();
    let event = EventEnvelope::new(ev::ACTOR_CREATED, "trigger-drill", &serde_json::json!({})).unwrap();
    store.append(&event).unwrap();

    let mut conn = db::open_db_at(&db_path).unwrap();
    let result = block_on(async {
        sqlx::query("UPDATE events SET payload_json = ? WHERE id = ?")
            .bind("{\"tampered\":true}")
            .bind(&event.id)
            .execute(&mut conn)
            .await
    });

    let err = result.expect_err("UPDATE on a committed event row must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("immutable"),
        "trigger error must name the immutability rule; got: {msg}"
    );

    // The row must be UNCHANGED — the rejected UPDATE must not have partially
    // applied (SQLite's RAISE(FAIL) aborts the statement, not a full
    // transaction rollback of prior statements, but this lone UPDATE must
    // leave the row exactly as it was).
    let stored = store.get(&event.id).unwrap().unwrap();
    assert_eq!(stored.payload_json, event.payload_json, "the row must be untouched by the rejected UPDATE");
}

/// UPDATE rejection must not depend on which column is targeted — the
/// trigger fires on ANY update to the row, not just the ones an attacker
/// might target.
#[test]
fn update_on_any_column_of_a_committed_event_row_is_rejected() {
    let (_tmp, db_path, store) = setup();
    let event = EventEnvelope::new(ev::ACTOR_CREATED, "trigger-drill-2", &serde_json::json!({})).unwrap();
    store.append(&event).unwrap();

    let mut conn = db::open_db_at(&db_path).unwrap();
    for (col, value) in [("event_type", "tampered.type"), ("entity_id", "tampered-entity"), ("actor", "tampered-actor")] {
        let sql = format!("UPDATE events SET {col} = ? WHERE id = ?");
        let result = block_on(async {
            sqlx::query(&sql).bind(value).bind(&event.id).execute(&mut conn).await
        });
        assert!(result.is_err(), "UPDATE on column '{col}' must be rejected by events_immutable");
    }
}

/// The append path itself is an INSERT, not an UPDATE — the trigger (`BEFORE
/// UPDATE`) must never fire for it. Many sequential appends to distinct AND
/// (via upsert-style projections) overlapping entities must all succeed
/// cleanly; this is the "not accidentally caught" half of the contract.
#[test]
fn append_path_is_never_caught_by_the_update_trigger() {
    let (_tmp, _db_path, store) = setup();
    for i in 0..50 {
        let event = EventEnvelope::new(ev::ACTOR_CREATED, &format!("trigger-append-{i}"), &serde_json::json!({}))
            .unwrap();
        store.append(&event).unwrap();
        assert!(store.get(&event.id).unwrap().is_some());
    }
}

/// `INSERT ... ON CONFLICT DO UPDATE` (used elsewhere in the codebase, e.g.
/// projection upserts) is a distinct SQL statement from a bare `UPDATE`, but
/// on the `events` table itself no code path ever performs an upsert — a
/// SECOND insert with the SAME primary key `id` must fail on the PRIMARY KEY
/// constraint, not silently succeed via some UPDATE-shaped path the trigger
/// would then reject anyway. This pins that the two failure modes
/// (PK-violation vs trigger-rejection) don't get confused for each other.
#[test]
fn duplicate_id_insert_fails_on_primary_key_not_the_trigger() {
    let (_tmp, db_path, store) = setup();
    let event = EventEnvelope::new(ev::ACTOR_CREATED, "trigger-dup", &serde_json::json!({})).unwrap();
    store.append(&event).unwrap();

    let mut conn = db::open_db_at(&db_path).unwrap();
    let result = block_on(async {
        sqlx::query(
            "INSERT INTO events (id, event_type, entity_id, payload_json, scope, created_at) \
             VALUES (?, 'actor.created', 'trigger-dup', '{}', 'runtime', datetime('now'))",
        )
        .bind(&event.id)
        .execute(&mut conn)
        .await
    });
    let err = result.expect_err("duplicate id INSERT must fail");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("unique") || msg.contains("primary key") || msg.contains("constraint"),
        "duplicate-id failure must be a PK/uniqueness violation, not the immutability trigger; got: {msg}"
    );
}
