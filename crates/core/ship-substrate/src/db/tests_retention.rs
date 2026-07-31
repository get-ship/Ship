//! Retention-sweep tests (T4-lite) — the DELETE mechanics, crash-safety,
//! Durable safety, and sweep-under-concurrent-appends.
//!
//! This L0 test binary links no `Retention::Windowed` registration (the
//! windowed tier lives in L1 domain crates), so the mechanism tests drive
//! [`retention::sweep_types_at`] with explicit type lists; the registry-
//! driven entry (`sweep_windowed_events_at`) is exercised for its no-
//! windowed-registrations safety property. The re-triaged types' own
//! projections are proven in `ship-platform/tests/retention_replay.rs`,
//! where those domains link.

use chrono::{Duration as ChronoDuration, Utc};
use tempfile::tempdir;

use crate::db::{self, retention};
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::envelope::EventEnvelope;
use ship_types::Stamped;

const WINDOWED_TICK: &str = "test.windowed.tick";
const WINDOWED_TOCK: &str = "test.windowed.tock";
const TTL_DAYS: u16 = 7;

/// An envelope of `event_type` backdated `age_days` into the past.
fn ev_aged(event_type: &str, entity: &str, age_days: i64) -> EventEnvelope {
    let mut e =
        EventEnvelope::new(event_type, entity, &serde_json::json!({})).expect("envelope builds");
    e.created_at = Utc::now() - ChronoDuration::days(age_days);
    e
}

fn ids_in(db_path: &std::path::Path) -> Vec<String> {
    let mut conn = db::open_db_at(db_path).unwrap();
    let rows: Vec<(String,)> = db::block_on(async {
        sqlx::query_as("SELECT id FROM events ORDER BY id")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    rows.into_iter().map(|(id,)| id).collect()
}

#[test]
fn sweep_deletes_only_named_windowed_type_past_horizon() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();

    let tick_old = ev_aged(WINDOWED_TICK, "e1", 10);
    let tick_recent = ev_aged(WINDOWED_TICK, "e2", 1);
    // A registered-Durable row OLDER than the horizon: inside the sweep's
    // deletion time window, and it must survive (deletion is by type,
    // never by age alone).
    let durable_old = ev_aged("daemon.startup_completed", "daemon", 10);
    // An unrelated type the sweep was never told about.
    let other_old = ev_aged("test.other.event", "e3", 10);

    for e in [&tick_old, &tick_recent, &durable_old, &other_old] {
        store.append(&Stamped::unchecked(e.clone())).unwrap();
    }

    let swept =
        retention::sweep_types_at(&db_path, &[(WINDOWED_TICK, TTL_DAYS)], Utc::now()).unwrap();
    assert_eq!(swept.len(), 1);
    assert_eq!(swept[0].event_type, WINDOWED_TICK);
    assert_eq!(swept[0].rows_deleted, 1);

    let surviving = ids_in(&db_path);
    assert!(
        !surviving.contains(&tick_old.id),
        "out-of-horizon windowed row must be swept"
    );
    assert!(
        surviving.contains(&tick_recent.id),
        "in-horizon windowed row must survive"
    );
    assert!(
        surviving.contains(&durable_old.id),
        "Durable row inside the sweep's time window must never be deleted"
    );
    assert!(
        surviving.contains(&other_old.id),
        "unlisted type must never be touched"
    );
}

#[test]
fn sweep_with_no_windowed_registrations_deletes_nothing() {
    // This binary's registry has no Windowed entry, so the registry-driven
    // sweep must be a no-op regardless of row age — the type list comes
    // from registrations, never from row inspection.
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();
    store
        .append(&Stamped::unchecked(ev_aged("daemon.startup_completed", "daemon", 400)))
        .unwrap();
    store
        .append(&Stamped::unchecked(ev_aged("test.other.event", "e", 400)))
        .unwrap();

    let windowed = retention::windowed_types();
    assert!(
        windowed.is_empty(),
        "L0 fixture invariant: no Windowed registration links here (got {windowed:?})"
    );

    let report = retention::sweep_windowed_events_at(tmp.path(), Utc::now()).unwrap();
    assert_eq!(report.rows_deleted(), 0);
    assert_eq!(report.dbs_visited, 1);
    assert_eq!(
        ids_in(&db_path).len(),
        2,
        "every row survives a no-type sweep"
    );
}

#[test]
fn interrupted_sweep_leaves_consistent_log_and_next_pass_finishes() {
    // Crash-safety: each per-type DELETE is one autocommit statement, so a
    // kill can only land BETWEEN types. That intermediate state — here
    // produced by sweeping a one-type prefix of the list — must be a
    // consistent, merely-unswept log that a later full pass finishes.
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();

    let tick_old = ev_aged(WINDOWED_TICK, "e1", 10);
    let tock_old = ev_aged(WINDOWED_TOCK, "e2", 10);
    let tock_recent = ev_aged(WINDOWED_TOCK, "e3", 1);
    for e in [&tick_old, &tock_old, &tock_recent] {
        store.append(&Stamped::unchecked(e.clone())).unwrap();
    }

    // "Killed" sweep: only the first type got its DELETE in.
    retention::sweep_types_at(&db_path, &[(WINDOWED_TICK, TTL_DAYS)], Utc::now()).unwrap();

    let mid = ids_in(&db_path);
    assert!(!mid.contains(&tick_old.id));
    assert!(
        mid.contains(&tock_old.id),
        "unswept type intact after interrupted pass"
    );
    assert!(mid.contains(&tock_recent.id));
    match db::integrity::check_integrity(&db_path).unwrap() {
        db::integrity::IntegrityStatus::Ok => {}
        other => panic!("interrupted sweep must leave an Ok db, got {other:?}"),
    }

    // Next full pass finishes the job; a further pass is a no-op.
    let full = [(WINDOWED_TICK, TTL_DAYS), (WINDOWED_TOCK, TTL_DAYS)];
    let swept = retention::sweep_types_at(&db_path, &full, Utc::now()).unwrap();
    assert_eq!(swept.iter().map(|s| s.rows_deleted).sum::<u64>(), 1);
    let again = retention::sweep_types_at(&db_path, &full, Utc::now()).unwrap();
    assert!(
        again.is_empty(),
        "sweep is idempotent once the horizon is clear"
    );

    let final_ids = ids_in(&db_path);
    assert_eq!(final_ids, vec![tock_recent.id.clone()]);
}

#[test]
fn sweep_never_touches_projection_tables_and_swept_log_rebuilds_cleanly() {
    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();

    // A Durable projection-feeding event OLDER than the horizon: its row
    // and its fold must both survive sweep + rebuild.
    let mut actor_created = EventEnvelope::new(
        "actor.created",
        "actor-1",
        &serde_json::json!({ "kind": "agent-session", "environment_type": "local" }),
    )
    .unwrap();
    actor_created.created_at = Utc::now() - ChronoDuration::days(10);
    store.append(&Stamped::unchecked(actor_created.clone())).unwrap();

    let tick_old = ev_aged(WINDOWED_TICK, "e1", 10);
    let tick_recent = ev_aged(WINDOWED_TICK, "e2", 1);
    store.append(&Stamped::unchecked(tick_old.clone())).unwrap();
    store.append(&Stamped::unchecked(tick_recent.clone())).unwrap();

    let actors_before: Vec<(String, String)> = {
        let mut conn = db::open_db_at(&db_path).unwrap();
        db::block_on(async {
            sqlx::query_as("SELECT id, status FROM actors ORDER BY id")
                .fetch_all(&mut conn)
                .await
        })
        .unwrap()
    };
    assert_eq!(actors_before.len(), 1, "fixture: actor.created folded");

    retention::sweep_types_at(&db_path, &[(WINDOWED_TICK, TTL_DAYS)], Utc::now()).unwrap();

    // The sweep deletes event rows only — projection state is bit-identical.
    let actors_after: Vec<(String, String)> = {
        let mut conn = db::open_db_at(&db_path).unwrap();
        db::block_on(async {
            sqlx::query_as("SELECT id, status FROM actors ORDER BY id")
                .fetch_all(&mut conn)
                .await
        })
        .unwrap()
    };
    assert_eq!(
        actors_before, actors_after,
        "sweep must never touch projections"
    );

    // Replay honesty: the swept log rebuilds without error, replaying
    // exactly the survivors, and the Durable fold is intact.
    let (replayed, _) = db::rebuild::rebuild_scope_db(&db_path).unwrap();
    assert_eq!(
        replayed, 2,
        "rebuild replays the surviving rows (durable + in-horizon)"
    );
    let actors_rebuilt: Vec<(String, String)> = {
        let mut conn = db::open_db_at(&db_path).unwrap();
        db::block_on(async {
            sqlx::query_as("SELECT id, status FROM actors ORDER BY id")
                .fetch_all(&mut conn)
                .await
        })
        .unwrap()
    };
    assert_eq!(
        actors_rebuilt.len(),
        1,
        "Durable fold survives a swept-log rebuild"
    );
}

#[test]
fn sweep_under_concurrent_appends_loses_nothing_inside_horizon() {
    const WRITERS: usize = 4;
    const APPENDS_PER_WRITER: usize = 25;

    let tmp = tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    let store = std::sync::Arc::new(SqliteEventStore::new_at_db_path(&db_path).unwrap());

    // Seed out-of-horizon rows the sweeps will be deleting meanwhile.
    for i in 0..20 {
        store
            .append(&Stamped::unchecked(ev_aged(WINDOWED_TICK, &format!("old-{i}"), 10)))
            .unwrap();
    }

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let sweeper = {
        let db_path = db_path.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                retention::sweep_types_at(&db_path, &[(WINDOWED_TICK, TTL_DAYS)], Utc::now())
                    .expect("sweep during concurrent appends");
            }
        })
    };

    let mut appended: Vec<String> = Vec::new();
    let mut handles = Vec::new();
    for w in 0..WRITERS {
        let store = store.clone();
        handles.push(std::thread::spawn(move || {
            let mut ids = Vec::new();
            for i in 0..APPENDS_PER_WRITER {
                let e = ev_aged(WINDOWED_TICK, &format!("live-{w}-{i}"), 0);
                store.append(&Stamped::unchecked(e.clone())).expect("append during sweep");
                ids.push(e.id);
            }
            ids
        }));
    }
    for h in handles {
        appended.extend(h.join().expect("writer thread"));
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    sweeper.join().expect("sweeper thread");

    // One final sweep after all appends: still nothing in-horizon lost.
    retention::sweep_types_at(&db_path, &[(WINDOWED_TICK, TTL_DAYS)], Utc::now()).unwrap();

    let surviving: std::collections::HashSet<String> = ids_in(&db_path).into_iter().collect();
    for id in &appended {
        assert!(
            surviving.contains(id),
            "in-horizon append {id} lost by concurrent sweep"
        );
    }
    assert_eq!(
        surviving.len(),
        WRITERS * APPENDS_PER_WRITER,
        "exactly the in-horizon appends survive (all seeded old rows swept)"
    );
}
