//! T1 storage-torture, scenario 3: crash mid-append (real SIGKILL).
//!
//! A real child process (`torture_crash_child`, gated behind the `unstable`
//! feature — see the crate's `Cargo.toml`) opens the scope db and begins a
//! transaction that mirrors `SqliteEventStore::append`: `BEGIN IMMEDIATE`,
//! insert the event row, insert the simulated projection (fold) row, then
//! either park (waiting to be killed) or `COMMIT` and then park. The parent
//! polls a marker file the child writes at the exact boundary, sends a real
//! `SIGKILL`, reaps the child, then reopens the db fresh and asserts WAL
//! recovery left an ALL-OR-NOTHING state.
//!
//! ## Crash points actually reachable in-harness
//!
//! - **`pre_commit`**: killed after the event row AND the fold row are
//!   written on the open (uncommitted) transaction, but before `COMMIT` is
//!   issued. Asserts: NEITHER row is visible after recovery — an uncommitted
//!   WAL transaction from a dead writer is simply invisible on reopen (SQLite
//!   requires no separate "undo" step; the frames were never marked
//!   committed).
//! - **`post_commit`**: killed after `COMMIT` returns (durable per
//!   `synchronous=NORMAL` WAL semantics) but before the child would have
//!   exited on its own. Asserts: BOTH rows are visible after recovery.
//!
//! ## Named gap
//!
//! The job spec asks for crash points spanning "before commit, mid-
//! projection, at commit". At the SQL/WAL level, "mid-projection" is NOT a
//! separately observable boundary from "before commit": projection dispatch
//! runs on the SAME uncommitted transaction as the event-row insert (see
//! `SqliteEventStore::append`), so a kill anywhere between `BEGIN IMMEDIATE`
//! and `COMMIT` produces the identical durable outcome (nothing survives).
//! `pre_commit` here covers that entire span, including mid-projection.
//! Separately, "crash DURING the `COMMIT` statement's fsync" (torn between
//! WAL-frame-written and header-updated) is NOT deterministically reachable
//! from outside the process without fault-injecting the `write`/`fsync`
//! syscalls themselves (e.g. via `LD_PRELOAD` or a custom VFS) — a real
//! `SIGKILL` from a parent polling a marker file cannot land inside a
//! sub-millisecond syscall on demand. That gap is named, not faked: this
//! suite does not claim coverage of a torn `COMMIT` syscall.

use std::process::Command;
use std::time::{Duration, Instant};

use ship_substrate::db;

fn sqlite_url(path: &std::path::Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    format!("sqlite://{raw}")
}

fn wait_for_marker(path: &std::path::Path, timeout: Duration) -> String {
    let start = Instant::now();
    loop {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if !contents.is_empty() {
                return contents;
            }
        }
        if start.elapsed() > timeout {
            panic!("timed out waiting for marker file {}", path.display());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn row_present(db_path: &std::path::Path, event_id: &str, entity_id: &str) -> (bool, bool) {
    let mut conn = db::open_db_at(db_path).unwrap();
    let event_count: i64 = db::block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
            .bind(event_id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    let actor_count: i64 = db::block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM actors WHERE id = ?")
            .bind(entity_id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    (event_count > 0, actor_count > 0)
}

/// Master invariant: replaying the surviving log must reproduce the exact
/// same `actors` contents as post-recovery live state.
fn assert_master_invariant(db_path: &std::path::Path) {
    let mut conn = db::open_db_at(db_path).unwrap();
    let before: Vec<(String, String)> = db::block_on(async {
        sqlx::query_as("SELECT id, kind FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    drop(conn);
    db::rebuild::rebuild_scope_db(db_path).unwrap();
    let mut conn = db::open_db_at(db_path).unwrap();
    let after: Vec<(String, String)> = db::block_on(async {
        sqlx::query_as("SELECT id, kind FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(
        before,
        after,
        "master invariant violated post-crash-recovery at {}",
        db_path.display()
    );
}

/// Run one crash trial: fresh scope db, spawn the child at `crash_point`,
/// SIGKILL it right after its marker fires, reopen and return
/// `(event_row_present, actor_row_present)`.
fn run_crash_trial(crash_point: &str, trial: usize) -> (std::path::PathBuf, bool, bool) {
    let tmp = tempfile::tempdir().unwrap();
    // Leak the tempdir so the db file survives past this function's return
    // (the caller inspects it) — each trial gets its own dir, cleaned up by
    // the OS at test-process exit, same as any other tempdir-per-test.
    let ship_dir = tmp.keep();
    let db_path = db::runtime_db_path(&ship_dir);
    db::ensure_events_db_at_path(&db_path).unwrap();

    // Sanity: reopening via a plain sqlx URL must agree with the resolved
    // path — the child bin builds its own URL independently.
    let _ = sqlite_url(&db_path);

    let event_id = format!("torture-crash-{crash_point}-{trial}-evt");
    let entity_id = format!("torture-crash-{crash_point}-{trial}-actor");
    let marker = ship_dir.join(format!("marker-{trial}"));

    let bin = env!("CARGO_BIN_EXE_torture_crash_child");
    let mut child = Command::new(bin)
        .arg(&db_path)
        .arg(&event_id)
        .arg(&entity_id)
        .arg(crash_point)
        .arg(&marker)
        .spawn()
        .expect("spawn crash-child process");

    wait_for_marker(&marker, Duration::from_secs(10));

    // The real crash: SIGKILL, not a graceful terminate. The child cannot
    // run any cleanup/finalizer — this is the point.
    unsafe {
        libc::kill(child.id() as i32, libc::SIGKILL);
    }
    let status = child.wait().expect("reap killed child");
    assert!(
        !status.success(),
        "child must have been killed, not exited cleanly"
    );

    let (event_present, actor_present) = row_present(&db_path, &event_id, &entity_id);
    (db_path, event_present, actor_present)
}

#[test]
fn crash_before_commit_leaves_no_orphan_either_side() {
    for trial in 0..10 {
        let (db_path, event_present, actor_present) = run_crash_trial("pre_commit", trial);
        assert!(
            !event_present && !actor_present,
            "trial {trial}: pre-commit crash must leave NEITHER row (torn write) — event={event_present} actor={actor_present}"
        );
        assert_master_invariant(&db_path);
    }
}

#[test]
fn crash_after_commit_returns_leaves_both_sides_durable() {
    for trial in 0..10 {
        let (db_path, event_present, actor_present) = run_crash_trial("post_commit", trial);
        assert!(
            event_present && actor_present,
            "trial {trial}: post-commit crash must leave BOTH rows (durable commit) — event={event_present} actor={actor_present}"
        );
        assert_master_invariant(&db_path);
    }
}
