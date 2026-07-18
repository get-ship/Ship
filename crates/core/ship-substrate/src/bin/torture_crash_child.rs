//! T1 storage-torture, scenario 3: the child half of the real-SIGKILL
//! crash-mid-append test.
//!
//! Gated behind the `unstable` feature (see `Cargo.toml`) — this binary never
//! builds, let alone ships, outside the torture-test gate. It opens the
//! SAME scope db the parent test created, begins a transaction that mirrors
//! `SqliteEventStore::append` (event-row insert + a simulated projection
//! write to `actors`, both on one connection), and — depending on the
//! requested crash point — either signals "ready to be killed" before
//! `COMMIT` or signals "committed" after `COMMIT` returns, then parks
//! forever so the parent's SIGKILL is the only thing that ends it.
//!
//! Deliberately reimplements the insert here (rather than reaching into
//! `ship_substrate`'s `pub(crate)` internals) since a `[[bin]]` target is a
//! separate crate from the library and only sees its `pub` surface.
//!
//! Usage: torture_crash_child <db_path> <event_id> <entity_id> <pre_commit|post_commit> <marker_path>

use std::env;
use std::str::FromStr;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteSynchronous};
use sqlx::{Connection, SqliteConnection};

fn sqlite_url(path: &str) -> String {
    let mut raw = path.replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    format!("sqlite://{raw}")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let db_path = &args[1];
    let event_id = &args[2];
    let entity_id = &args[3];
    let crash_point = &args[4];
    let marker_path = &args[5];

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build current-thread runtime");

    rt.block_on(async move {
        let url = sqlite_url(db_path);
        let opts = SqliteConnectOptions::from_str(&url)
            .expect("valid sqlite url")
            .busy_timeout(Duration::from_secs(5))
            .synchronous(SqliteSynchronous::Normal);
        let mut conn = SqliteConnection::connect_with(&opts)
            .await
            .expect("connect to scope db");

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut conn)
            .await
            .expect("enable foreign keys");

        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut conn)
            .await
            .expect("BEGIN IMMEDIATE");

        let created_at = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              scope, actor_id, elevated, created_at) \
             VALUES (?, 'actor.created', ?, 'ship', \
              '{\"kind\":\"local\",\"environment_type\":\"local\"}', 1, \
              'runtime', ?, 0, ?)",
        )
        .bind(event_id)
        .bind(entity_id)
        .bind(entity_id)
        .bind(&created_at)
        .execute(&mut conn)
        .await
        .expect("insert event row");

        // Simulated fold — same transaction, mirrors ActorProjection::apply_created
        // writing the actors row inside the SAME BEGIN IMMEDIATE as the event
        // row. Whether this row survives is exactly the all-or-nothing question
        // the crash point is designed to probe.
        sqlx::query("INSERT INTO actors (id, kind) VALUES (?, 'local')")
            .bind(entity_id)
            .execute(&mut conn)
            .await
            .expect("insert actors row (simulated fold)");

        if crash_point == "pre_commit" {
            std::fs::write(marker_path, "ready-pre-commit").expect("write ready marker");
            park_forever();
        }

        sqlx::query("COMMIT")
            .execute(&mut conn)
            .await
            .expect("COMMIT");

        std::fs::write(marker_path, "committed").expect("write committed marker");

        if crash_point == "post_commit" {
            park_forever();
        }
    });
}

/// Sleep until SIGKILLed. Never returns under normal operation — the parent
/// test is expected to have killed this process well before any of these
/// sleeps individually elapse.
fn park_forever() -> ! {
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}
