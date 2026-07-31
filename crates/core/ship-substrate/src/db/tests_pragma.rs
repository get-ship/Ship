//! Pragma discipline tests — census §2 (write discipline per store).
//!
//! Every sqlite open path in this crate must re-issue:
//! - `synchronous = NORMAL` (WAL-appropriate; sqlite's version-dependent
//!   default is unstable so this is set explicitly).
//! - `foreign_keys = ON` (per-connection pragma, NOT persisted in the file
//!   header like `journal_mode`).
//!
//! These tests assert the live connection has both set after going through
//! the public openers.

use crate::db::{
    block_on, ensure_kv_db_at, ensure_kv_db_at_path, kv_db_path, open_account_db, open_db_at,
    open_kv_db_at, open_runtime_db, open_space_db,
};

fn query_synchronous(conn: &mut sqlx::SqliteConnection) -> i64 {
    block_on(async {
        sqlx::query_scalar::<_, i64>("PRAGMA synchronous")
            .fetch_one(conn)
            .await
    })
    .expect("PRAGMA synchronous")
}

fn query_foreign_keys(conn: &mut sqlx::SqliteConnection) -> i64 {
    block_on(async {
        sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
            .fetch_one(conn)
            .await
    })
    .expect("PRAGMA foreign_keys")
}

fn assert_pragmas(conn: &mut sqlx::SqliteConnection, label: &str) {
    assert_eq!(
        query_synchronous(conn),
        1,
        "{label}: synchronous must be NORMAL (1)"
    );
    assert_eq!(
        query_foreign_keys(conn),
        1,
        "{label}: foreign_keys must be ON (1)"
    );
}

#[test]
fn connect_re_issues_synchronous_normal_and_foreign_keys_on() {
    // `connect()` is the per-open path: opens an existing file with
    // `SqliteConnectOptions` + a follow-up `foreign_keys = ON` execute. The
    // open_runtime_db / open_account_db / open_space_db / open_db_at /
    // open_kv_db_at entry points all route through it.
    let tmp = tempfile::tempdir().expect("tempdir");
    let ship_dir = tmp.path().join(".ship");

    // Provision a runtime db so we have a path to open.
    let path = crate::db::runtime_db_path(&ship_dir);
    ensure_kv_db_at(&ship_dir).expect("provision kv"); // any ensure creates parent
    crate::db::ensure_events_db_at_path(&path).expect("provision runtime");

    // connect() path via open_db_at
    let mut conn = open_db_at(&path).expect("open_db_at");
    assert_pragmas(&mut conn, "open_db_at -> connect()");

    // open_runtime_db
    let mut conn = open_runtime_db(&ship_dir).expect("open_runtime_db");
    assert_pragmas(&mut conn, "open_runtime_db -> connect()");

    // open_account_db
    let mut conn = open_account_db(&ship_dir).expect("open_account_db");
    assert_pragmas(&mut conn, "open_account_db -> connect()");

    // open_space_db
    let mut conn = open_space_db(&ship_dir, "s1").expect("open_space_db");
    assert_pragmas(&mut conn, "open_space_db -> connect()");

    // open_kv_db_at routes through connect() too.
    let kv_path = kv_db_path(&ship_dir);
    let mut conn = open_kv_db_at(&kv_path).expect("open_kv_db_at");
    assert_pragmas(&mut conn, "open_kv_db_at -> connect()");
}

#[test]
fn ensure_kv_db_at_path_sets_synchronous_normal_and_foreign_keys_on() {
    // The creation-time path must set both pragmas too (defence-in-depth:
    // the connection returned from a brand-new db file must already enforce
    // them; the connect() re-issue is the per-open follow-up).
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("kv.db");
    ensure_kv_db_at_path(&path).expect("ensure_kv_db_at_path");

    let mut conn = open_kv_db_at(&path).expect("open_kv_db_at");
    assert_pragmas(&mut conn, "ensure_kv_db_at_path");
}

#[test]
fn ensure_db_at_path_sets_synchronous_normal_and_foreign_keys_on() {
    // The per-scope events db creation path (runtime.db / account.db /
    // space.db) — used by ensure_events_db_at_path and the per-scope openers.
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("runtime.db");
    crate::db::ensure_events_db_at_path(&path).expect("ensure_events_db_at_path");

    let mut conn = open_db_at(&path).expect("open_db_at");
    assert_pragmas(
        &mut conn,
        "ensure_db_at_path (via ensure_events_db_at_path)",
    );
}
