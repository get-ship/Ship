//! T1 storage-torture, scenario 2 (child-process half): concurrent writers,
//! one scope db, spanning processes not just threads.
//!
//! Spawns real child processes (the `torture_crash_child` helper bin, S3's
//! helper reused here in its non-crashing shape) that append into the SAME
//! scope db file while this process's own threads also append concurrently.
//! Lives in `tests/` (not `src/`) because `CARGO_BIN_EXE_*` — needed to
//! locate the helper bin — is only defined for integration test binaries.

use std::process::{Child, Command};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ship_substrate::db;
use ship_substrate::persistence::{EventStore, SqliteEventStore};
use ship_types::envelope::EventEnvelope;
use ship_types::Stamped;
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
    db::block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM actors")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap()
}

fn event_row_count(path: &std::path::Path) -> i64 {
    let mut conn = db::open_db_at(path).unwrap();
    db::block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap()
}

/// Master invariant: replaying the whole log must reproduce the exact same
/// `actors` contents as the live state — if the two writer populations
/// (threads + processes) had corrupted or dropped each other's writes, the
/// rebuild would change the table.
fn assert_master_invariant(path: &std::path::Path) {
    let mut conn = db::open_db_at(path).unwrap();
    let before: Vec<(String, String)> = db::block_on(async {
        sqlx::query_as("SELECT id, kind FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    drop(conn);
    db::rebuild::rebuild_scope_db(path).unwrap();
    let mut conn = db::open_db_at(path).unwrap();
    let after: Vec<(String, String)> = db::block_on(async {
        sqlx::query_as("SELECT id, kind FROM actors ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(
        before, after,
        "master invariant violated after multi-process concurrent writers"
    );
}

/// `post_commit` commits for real, writes a "committed" marker, then parks
/// (by the crash-child's design) — we wait for the marker (proof it wrote
/// for real) then SIGKILL to reap it; this is not itself the crash test.
fn spawn_child_writer(db_path: &std::path::Path, id: &str, marker_path: &std::path::Path) -> Child {
    let bin = env!("CARGO_BIN_EXE_torture_crash_child");
    Command::new(bin)
        .arg(db_path)
        .arg(format!("evt-{id}"))
        .arg(id)
        .arg("post_commit")
        .arg(marker_path)
        .spawn()
        .expect("spawn child writer process")
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
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn concurrent_child_processes_and_threads_on_same_scope_db() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = db::runtime_db_path(tmp.path());
    // Provision the schema before any writer (thread or child process) races
    // to create it — mirrors production, where the schema is ensured once at
    // boot rather than raced by first-writer.
    db::ensure_events_db_at_path(&db_path).unwrap();
    let store = Arc::new(SqliteEventStore::new_at_db_path(&db_path).unwrap());

    const N_THREADS: usize = 150;
    const N_CHILDREN: usize = 5;

    let mut thread_handles = Vec::with_capacity(N_THREADS);
    for i in 0..N_THREADS {
        let store = Arc::clone(&store);
        thread_handles.push(std::thread::spawn(move || {
            let id = format!("torture-conc-thread-{i}");
            let ev = actor_created_event(&id);
            store.append(&Stamped::unchecked(ev.clone()))
        }));
    }

    let mut children = Vec::with_capacity(N_CHILDREN);
    for i in 0..N_CHILDREN {
        let id = format!("torture-conc-child-{i}");
        let marker = tmp.path().join(format!("marker-{i}"));
        let child = spawn_child_writer(&db_path, &id, &marker);
        children.push((child, marker));
    }

    let mut thread_ok = 0usize;
    for h in thread_handles {
        if h.join().expect("writer thread must not panic").is_ok() {
            thread_ok += 1;
        }
    }
    assert_eq!(
        thread_ok, N_THREADS,
        "no lost thread appends while child processes wrote concurrently"
    );

    for (mut child, marker) in children {
        let status = wait_for_marker(&marker, Duration::from_secs(20));
        assert_eq!(
            status, "committed",
            "child writer must have committed before parking"
        );
        unsafe {
            libc::kill(child.id() as i32, libc::SIGKILL);
        }
        let _ = child.wait();
    }

    assert_eq!(
        event_row_count(&db_path) as usize,
        N_THREADS + N_CHILDREN,
        "every writer (thread and child process) landed exactly one event row"
    );
    assert_eq!(
        actor_row_count(&db_path) as usize,
        N_THREADS + N_CHILDREN,
        "every committed event folded exactly once, across processes"
    );

    assert_master_invariant(&db_path);
}
