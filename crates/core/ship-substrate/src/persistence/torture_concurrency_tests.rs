//! T1 storage-torture, scenario 2 (thread half): concurrent writers, one
//! scope db.
//!
//! Many threads append to the SAME scope db file concurrently, all going
//! through `BEGIN IMMEDIATE`. Assertions: no lost appends, no corruption, no
//! deadlock / permanent lock error, every committed event folded exactly
//! once, and the master invariant (`actors` table == fold(committed log))
//! holds afterward.
//!
//! The child-PROCESS half of this scenario lives in
//! `tests/torture_concurrency_children.rs` — `CARGO_BIN_EXE_*` (needed to
//! locate the `torture_crash_child` helper bin) is only defined for
//! integration tests, not for this crate's unit tests.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::db::{self, block_on};
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types as ev;

fn actor_created_event(id: &str) -> EventEnvelope {
    EventEnvelope::new(
        ev::ACTOR_CREATED,
        id,
        &serde_json::json!({ "kind": "local", "environment_type": "local" }),
    )
    .unwrap()
    .with_actor_id(id)
}

fn actor_row_count(path: &std::path::Path) -> i64 {
    let mut conn = db::open_db_at(path).unwrap();
    block_on(async { sqlx::query_scalar("SELECT COUNT(*) FROM actors").fetch_one(&mut conn).await }).unwrap()
}

fn event_row_count(path: &std::path::Path) -> i64 {
    let mut conn = db::open_db_at(path).unwrap();
    block_on(async { sqlx::query_scalar("SELECT COUNT(*) FROM events").fetch_one(&mut conn).await }).unwrap()
}

fn distinct_actor_ids(path: &std::path::Path) -> i64 {
    let mut conn = db::open_db_at(path).unwrap();
    block_on(async { sqlx::query_scalar("SELECT COUNT(DISTINCT id) FROM actors").fetch_one(&mut conn).await }).unwrap()
}

/// Same-table-contents-before-and-after-rebuild is the master invariant
/// proof used throughout this job (see `torture_atomicity_tests`): if live
/// state had drifted from fold(log), replay would change it.
fn assert_master_invariant(path: &std::path::Path) {
    let mut conn = db::open_db_at(path).unwrap();
    let before: Vec<(String, String)> = block_on(async {
        sqlx::query_as("SELECT id, kind FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    drop(conn);
    crate::db::rebuild::rebuild_scope_db(path).unwrap();
    let mut conn = db::open_db_at(path).unwrap();
    let after: Vec<(String, String)> = block_on(async {
        sqlx::query_as("SELECT id, kind FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(before, after, "master invariant violated after concurrent writers");
}

/// Drive `N` concurrent thread appends against one scope db; return
/// `(ok_count, err_count, elapsed)`.
fn drive_concurrent_appends(db_path: &std::path::Path, n: usize) -> (usize, usize, Duration) {
    let store = Arc::new(SqliteEventStore::new_at_db_path(db_path).unwrap());
    let start = Instant::now();
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let store = Arc::clone(&store);
        handles.push(std::thread::spawn(move || {
            let id = format!("torture-conc-{i}");
            let ev = actor_created_event(&id);
            store.append(&ev)
        }));
    }

    let mut ok = 0usize;
    let mut err = 0usize;
    for h in handles {
        match h.join().expect("writer thread must not panic") {
            Ok(()) => ok += 1,
            Err(_) => err += 1,
        }
    }
    (ok, err, start.elapsed())
}

/// Hundreds of threads, one scope db, pushed hard. No deadlock (bounded
/// elapsed), no corruption, no duplicate ids, every OK append folded exactly
/// once — REGARDLESS of whether every single append returned `Ok` (see the
/// `#[ignore]`d finding below: under heavy REAL system contention some
/// appends legitimately time out with `SQLITE_BUSY`, so this test does not
/// assume `ok == N`).
#[test]
fn hundreds_of_concurrent_thread_appends_never_corrupt_or_orphan() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());

    const N: usize = 300;
    let (ok, err, elapsed) = drive_concurrent_appends(&db_path, N);

    assert_eq!(ok + err, N, "every spawned append thread must resolve (no thread lost, no hang)");
    assert!(
        elapsed < Duration::from_secs(90),
        "no permanent lock / deadlock: {N} concurrent appends took {elapsed:?}"
    );
    assert_eq!(event_row_count(&db_path) as usize, ok, "every OK append has exactly one event row — no orphans among the successes");
    assert_eq!(actor_row_count(&db_path) as usize, ok, "every committed event folded exactly once");
    assert_eq!(distinct_actor_ids(&db_path) as usize, ok, "no corrupted/duplicated actor ids");

    assert_master_invariant(&db_path);
}

/// **FINDING (T1 storage-torture, scenario 2) — real break, do not fix here.**
///
/// Repro: 300 threads in ONE process, all calling `SqliteEventStore::append`
/// against the SAME scope db concurrently (each `append` opens its OWN
/// connection with `busy_timeout(5s)`). Observed error rate is load-
/// dependent and reproduces both ways:
/// - run as part of the full workspace suite (`cargo test --workspace
///   --exclude ship-host --features unstable`, real system-wide CPU
///   contention layered on top of the app-level thread contention): up to
///   ~88/300 (~29%) appends returned `Err("database is locked")`
///   (SQLITE_BUSY).
/// - run BY ITSELF, no competing workspace load, same 300-thread/one-db
///   shape: still observed 35/300 (~12%) failures — so this is not purely a
///   test-environment CI-noise artifact; 300 real OS threads contending a
///   single SQLite writer lock on ordinary hardware exhausts the 5-second
///   busy_timeout on a double-digit fraction of attempts even in isolation.
///
/// `append`'s "no lost appends under concurrency" guarantee is bounded by
/// `busy_timeout`, and `busy_timeout` is a WALL-CLOCK budget that real
/// contention — even purely app-level, no external load required — can
/// exhaust. No orphans or corruption result (the `never_corrupt_or_orphan`
/// test above holds even when this one doesn't) — the break is specifically
/// "some legitimate concurrent appends are rejected", not "the log is
/// inconsistent".
#[test]
#[ignore = "FINDING: 300 concurrent same-process threads against one scope db exceed the 5s busy_timeout on a double-digit percentage of attempts (reproduces even with no other system load) — 'no lost appends' is not unconditional"]
fn hundreds_of_concurrent_thread_appends_lose_nothing_under_full_system_load() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    const N: usize = 300;
    let (ok, err, _elapsed) = drive_concurrent_appends(&db_path, N);
    assert_eq!(err, 0, "no append should fail under contention (busy_timeout covers it) — {err} of {N} failed");
    assert_eq!(ok, N, "no lost appends");
}
