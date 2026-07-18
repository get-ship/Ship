//! T1 storage-torture, scenario 1: atomicity under projection failure.
//!
//! `tx_atomicity_tests.rs` proves the single-append case. This file drives
//! MANY appends through a bus that mixes a real projection (`ActorProjection`,
//! which writes the `actors` row) with a data-driven failing projection
//! (`ConditionalFailProjection`, registered AFTER it) — so a failure fires
//! only once the real projection has already written inside the same
//! transaction. The proof is that the ROLLBACK undoes both writes, never
//! leaving an `actors` row whose event was rejected (an orphaned fold) nor an
//! event row whose fold was skipped.
//!
//! The master invariant asserted after every run:
//! `projection_state (the `actors` table) == fold(all COMMITTED events)`.
//! We check this two ways: (1) row-by-row, every committed id has exactly one
//! actors row and every rejected id has none; (2) a full rebuild-from-log
//! (`rebuild_scope_db`) must reproduce byte-identical `actors` table contents
//! — if the live state ever drifted from the log's fold, the rebuild would
//! change it.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use sqlx::{Row, SqliteConnection};
use tempfile::tempdir;

use super::event_store::{EventStore, SqliteEventStore};
use crate::db::{self, block_on};
use crate::projections::{ActorProjection, EventBus, Projection};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types as ev;

/// A projection that reads a `"should_fail"` bool out of its own event's
/// payload and errors when it's `true`. Purely data-driven (no shared mutable
/// state) so the fail/succeed sequence for a whole run is fixed at
/// event-construction time and trivially reproducible from a seed.
struct ConditionalFailProjection;

impl Projection for ConditionalFailProjection {
    fn name(&self) -> &str {
        "torture-conditional-fail"
    }

    fn event_types(&self) -> &[&str] {
        &[ev::ACTOR_CREATED]
    }

    fn apply(&self, event: &EventEnvelope, _conn: &mut SqliteConnection) -> Result<()> {
        let should_fail = serde_json::from_str::<serde_json::Value>(&event.payload_json)
            .ok()
            .and_then(|v| v.get("should_fail").and_then(|b| b.as_bool()))
            .unwrap_or(false);
        if should_fail {
            Err(anyhow!("torture: conditional projection failure for {}", event.id))
        } else {
            Ok(())
        }
    }

    fn truncate(&self, _conn: &mut SqliteConnection) -> Result<()> {
        Ok(())
    }
}

/// Bus with the REAL actor projection first, the conditional-failure hook
/// second. A failure here must roll back the `actors` row `ActorProjection`
/// already wrote in the same transaction, not just skip the event row.
fn store_with_mixed_bus(path: &std::path::Path) -> SqliteEventStore {
    let mut bus = EventBus::new();
    bus.register(Box::new(ActorProjection::new()));
    bus.register(Box::new(ConditionalFailProjection));
    SqliteEventStore::new_with_bus(path, Arc::new(bus)).unwrap()
}

fn actor_created_event(id: &str, should_fail: bool) -> EventEnvelope {
    EventEnvelope::new(
        ev::ACTOR_CREATED,
        id,
        &serde_json::json!({
            "kind": "local",
            "environment_type": "local",
            "should_fail": should_fail,
        }),
    )
    .unwrap()
    .with_actor_id(id)
}

fn event_row_count(path: &std::path::Path) -> i64 {
    let mut conn = db::open_db_at(path).unwrap();
    block_on(async { sqlx::query_scalar("SELECT COUNT(*) FROM events").fetch_one(&mut conn).await }).unwrap()
}

fn actor_row_exists(path: &std::path::Path, id: &str) -> bool {
    let mut conn = db::open_db_at(path).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM actors WHERE id = ?")
            .bind(id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    count > 0
}

/// Dump every `actors` row as a sorted, comparable string vector — used to
/// prove `rebuild_scope_db` is a no-op fixpoint when the invariant already
/// holds (the master-invariant check).
fn dump_actors(path: &std::path::Path) -> Vec<String> {
    let mut conn = db::open_db_at(path).unwrap();
    let rows = block_on(async {
        sqlx::query("SELECT id, kind, status, restart_count FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    rows.iter()
        .map(|r| {
            format!(
                "{}|{}|{}|{}",
                r.get::<String, _>(0),
                r.get::<String, _>(1),
                r.get::<String, _>(2),
                r.get::<i64, _>(3)
            )
        })
        .collect()
}

/// Deterministic xorshift64 PRNG — no `rand` dependency in this crate; a
/// fixed seed makes a "random" failure run byte-reproducible across CI runs.
struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    fn next_bool(&mut self) -> bool {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x & 1 == 1
    }
}

/// THE MASTER INVARIANT, asserted generically: after any sequence of
/// appends, replaying the WHOLE log through a fresh projection bus
/// (`rebuild_scope_db`) must reproduce the exact same `actors` contents. If
/// live state had drifted from fold(log) — an orphan row, a missing row — the
/// rebuild would change the table and this would fail.
fn assert_master_invariant(path: &std::path::Path) {
    let before = dump_actors(path);
    let (_replayed, _names) = crate::db::rebuild::rebuild_scope_db(path).unwrap();
    let after = dump_actors(path);
    assert_eq!(
        before, after,
        "master invariant violated: live actors state != fold(committed log) at {}",
        path.display()
    );
}

/// Scenario 1, part A: EVERY append fails. Proves the degenerate case first —
/// zero appends should ever land a row, in either table.
#[test]
fn every_append_fails_leaves_zero_rows() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = store_with_mixed_bus(&db_path);

    let mut ok_count = 0;
    for i in 0..50 {
        let id = format!("torture-allfail-{i}");
        let ev = actor_created_event(&id, true);
        if store.append(&ev).is_ok() {
            ok_count += 1;
        }
        assert!(!actor_row_exists(&db_path, &id), "actor row must not survive a failed projection ({id})");
    }
    assert_eq!(ok_count, 0, "a projection that always fails must never report Ok");
    assert_eq!(event_row_count(&db_path), 0, "no event row may survive when every projection failed");
    assert_master_invariant(&db_path);
}

/// Scenario 1, part B: EVERY append succeeds — the rollback machinery must
/// not be a blanket refusal to ever commit.
#[test]
fn every_append_succeeds_commits_all_rows() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = store_with_mixed_bus(&db_path);

    for i in 0..50 {
        let id = format!("torture-allok-{i}");
        let ev = actor_created_event(&id, false);
        store.append(&ev).unwrap();
        assert!(actor_row_exists(&db_path, &id), "actor row must be present after a clean append ({id})");
    }
    assert_eq!(event_row_count(&db_path), 50);
    assert_master_invariant(&db_path);
}

/// Scenario 1, part C: random failures (deterministic seed) across hundreds
/// of appends. This is the attempt to produce an orphan event — a row
/// present whose fold was skipped, or a fold present whose event was rolled
/// back. Every single iteration is checked, not just an aggregate count.
#[test]
fn random_projection_failures_never_produce_an_orphan() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = store_with_mixed_bus(&db_path);

    let mut rng = Xorshift64::new(0xC0FFEE_u64);
    let mut expect_committed: Vec<String> = Vec::new();
    let mut expect_rejected: Vec<String> = Vec::new();

    const N: usize = 500;
    for i in 0..N {
        let should_fail = rng.next_bool();
        let id = format!("torture-rand-{i}");
        let ev = actor_created_event(&id, should_fail);
        let result = store.append(&ev);

        if should_fail {
            assert!(result.is_err(), "projection marked should_fail=true must reject the append ({id})");
            expect_rejected.push(id);
        } else {
            assert!(result.is_ok(), "projection marked should_fail=false must accept the append ({id}): {result:?}");
            expect_committed.push(id);
        }
    }

    // Row-by-row: every committed id has exactly an event row + an actor
    // row; every rejected id has neither.
    for id in &expect_committed {
        assert!(actor_row_exists(&db_path, id), "committed id {id} must have an actor row");
    }
    for id in &expect_rejected {
        assert!(!actor_row_exists(&db_path, id), "rejected id {id} must NOT have an actor row (orphan fold)");
    }

    assert_eq!(
        event_row_count(&db_path) as usize,
        expect_committed.len(),
        "event row count must equal exactly the accepted appends — no orphan events, no lost commits"
    );

    assert_master_invariant(&db_path);
}

/// Scenario 1, part D: "fail at every point" — for a fixed-size run, cycle
/// the single failure position through every index 0..N so each position in
/// the sequence gets to be the (sole) failure at least once across the sweep.
/// Each position is its own fresh db (isolated), so this proves the rollback
/// is correct regardless of WHERE in a sequence of otherwise-successful
/// appends the failure lands — start, middle, end.
#[test]
fn failure_at_every_position_in_a_run_rolls_back_only_that_position() {
    const N: usize = 20;
    for fail_at in 0..N {
        let tmp = tempdir().unwrap();
        let db_path = db::runtime_db_path(tmp.path());
        let store = store_with_mixed_bus(&db_path);

        for i in 0..N {
            let should_fail = i == fail_at;
            let id = format!("torture-pos-{i}");
            let ev = actor_created_event(&id, should_fail);
            let result = store.append(&ev);
            assert_eq!(result.is_ok(), !should_fail, "position {i} (fail_at={fail_at}) unexpected result: {result:?}");
        }

        assert_eq!(
            event_row_count(&db_path) as usize,
            N - 1,
            "exactly one position (fail_at={fail_at}) must be rejected, the rest committed"
        );
        let failed_id = format!("torture-pos-{fail_at}");
        assert!(!actor_row_exists(&db_path, &failed_id), "the failed position must not leave an actor row");

        assert_master_invariant(&db_path);
    }
}
