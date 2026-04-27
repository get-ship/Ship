//! Tests for `migration.rs` and `migration_tables.rs`.

use super::*;
use crate::collections::builtin::{register_builtin_schemas, ADRS_ID, NOTES_ID, SPECS_ID};
use crate::collections::query::QueryRequest;
use crate::collections::store;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::str::FromStr;

fn empty_query(collection: &str) -> QueryRequest {
    QueryRequest {
        collection: collection.to_string(),
        filter: Default::default(),
        sort: Default::default(),
        limit: 50,
        offset: 0,
    }
}

fn setup_data_service() -> (DataService, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    // Use open_raw to bypass the auto-startup — these tests exercise
    // migration directly rather than letting it run as a side effect.
    let mut svc = DataService::open_raw(&ship_dir).unwrap();
    register_builtin_schemas(&mut svc).unwrap();
    (svc, tmp)
}

fn open_platform_rw(path: &std::path::Path) -> SqliteConnection {
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let opts = SqliteConnectOptions::from_str(&url).unwrap().create_if_missing(true);
    crate::db::block_on(async { SqliteConnection::connect_with(&opts).await }).unwrap()
}

fn seed_platform_db(path: &Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut conn = open_platform_rw(path);

    crate::db::block_on(async {
        sqlx::query(
            "CREATE TABLE note (
                id TEXT PRIMARY KEY, title TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                tags_json TEXT NOT NULL DEFAULT '[]',
                branch TEXT, synced_at TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
    })
    .unwrap();
    crate::db::block_on(async {
        sqlx::query(
            "CREATE TABLE adr (
                id TEXT PRIMARY KEY, title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'proposed',
                date TEXT NOT NULL,
                context TEXT NOT NULL DEFAULT '',
                decision TEXT NOT NULL DEFAULT '',
                tags_json TEXT NOT NULL DEFAULT '[]',
                supersedes_id TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
    })
    .unwrap();
    crate::db::block_on(async {
        sqlx::query(
            "CREATE TABLE target (
                id TEXT PRIMARY KEY, kind TEXT NOT NULL,
                title TEXT NOT NULL, description TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                goal TEXT, phase TEXT, due_date TEXT,
                body_markdown TEXT, file_scope_json TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
    })
    .unwrap();
    crate::db::block_on(async {
        sqlx::query(
            "CREATE TABLE capability (
                id TEXT PRIMARY KEY, target_id TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'aspirational',
                evidence TEXT, milestone_id TEXT, phase TEXT,
                acceptance_criteria TEXT NOT NULL DEFAULT '[]',
                assigned_to TEXT, priority INTEGER NOT NULL DEFAULT 0,
                file_scope TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
    })
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO note (id, title, content, tags_json, branch, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("n1")
        .bind("First note")
        .bind("hello world")
        .bind(r#"["dev","todo"]"#)
        .bind("main")
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })
    .unwrap();
    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO adr (id, title, status, date, context, decision, tags_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("a1")
        .bind("Use SQLite")
        .bind("accepted")
        .bind(&now)
        .bind("Need durable storage")
        .bind("SQLite with WAL")
        .bind("[]")
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })
    .unwrap();
    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO target (id, kind, title, description, status, body_markdown,
                                 file_scope_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("t1")
        .bind("feature")
        .bind("Ship it")
        .bind("High-level desc")
        .bind("active")
        .bind("Body text")
        .bind(r#"["src/foo.rs"]"#)
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })
    .unwrap();
    for (id, title) in [("c1", "Acceptance one"), ("c2", "Acceptance two")] {
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO capability (id, target_id, title, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind("t1")
            .bind(title)
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
    }
    crate::db::block_on(async { conn.close().await }).unwrap();
}

// ── Migration tests ──────────────────────────────────────────────────────────

#[test]
fn migration_without_platform_db_marks_complete() {
    let (mut svc, tmp) = setup_data_service();
    let missing = tmp.path().join("nope").join("platform.db");

    let report = migrate_platform_to_collections(&mut svc, &missing).unwrap();
    assert!(report.source_missing);
    assert!(!report.already_migrated);
    assert_eq!(report.notes_migrated, 0);

    // Second run short-circuits.
    let report2 = migrate_platform_to_collections(&mut svc, &missing).unwrap();
    assert!(report2.already_migrated);
}

#[test]
fn migration_copies_rows_and_is_idempotent() {
    let (mut svc, tmp) = setup_data_service();
    let platform = tmp.path().join("platform.db");
    seed_platform_db(&platform);

    let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert_eq!(report.notes_migrated, 1);
    assert_eq!(report.adrs_migrated, 1);
    assert_eq!(report.specs_migrated, 1);

    let notes = store::query_items(&mut svc, &builtin::notes_schema(), &empty_query(NOTES_ID)).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "n1");
    assert_eq!(notes[0].data["title"], "First note");
    assert_eq!(notes[0].data["content"], "hello world");
    assert_eq!(notes[0].data["category"], "main");

    let adrs = store::query_items(&mut svc, &builtin::adrs_schema(), &empty_query(ADRS_ID)).unwrap();
    assert_eq!(adrs.len(), 1);
    assert_eq!(adrs[0].data["status"], "accepted");

    let specs = store::query_items(&mut svc, &builtin::specs_schema(), &empty_query(SPECS_ID)).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].data["status"], "launched");
    // Body content (description + acceptance bullets) is written to the
    // file at body_path on migration. We can't read it from this test
    // because crate::project::get_project_dir(None) won't find a project
    // — the file write is skipped in that case. body_path is still set.
    assert!(
        specs[0].data["body_path"].as_str().unwrap().ends_with(".md"),
        "body_path should be set after migration"
    );

    // Running again should not duplicate anything.
    let report2 = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert!(report2.already_migrated);

    let notes_again = store::query_items(&mut svc, &builtin::notes_schema(), &empty_query(NOTES_ID)).unwrap();
    assert_eq!(notes_again.len(), 1, "no duplicates on second run");
}

#[test]
fn migration_empty_tables_completes_cleanly() {
    let (mut svc, tmp) = setup_data_service();
    let platform = tmp.path().join("platform.db");
    let mut conn = open_platform_rw(&platform);
    for ddl in [
        "CREATE TABLE note (id TEXT PRIMARY KEY, title TEXT, content TEXT, tags_json TEXT, branch TEXT)",
        "CREATE TABLE adr (id TEXT PRIMARY KEY, title TEXT, status TEXT, context TEXT, decision TEXT, tags_json TEXT)",
        "CREATE TABLE target (id TEXT PRIMARY KEY, kind TEXT, title TEXT, description TEXT, status TEXT, body_markdown TEXT, file_scope_json TEXT)",
        "CREATE TABLE capability (id TEXT PRIMARY KEY, target_id TEXT, title TEXT, priority INTEGER, created_at TEXT)",
    ] {
        crate::db::block_on(async { sqlx::query(ddl).execute(&mut conn).await }).unwrap();
    }
    crate::db::block_on(async { conn.close().await }).unwrap();

    let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert_eq!(report.notes_migrated, 0);
    assert_eq!(report.adrs_migrated, 0);
    assert_eq!(report.specs_migrated, 0);
    assert!(!report.source_missing);
}

#[test]
fn status_mapping() {
    use super::super::migration_tables::{map_adr_status, map_spec_status};
    assert_eq!(map_adr_status("accepted"), "accepted");
    assert_eq!(map_adr_status("ACTIVE"), "accepted");
    assert_eq!(map_adr_status("retired"), "deprecated");
    assert_eq!(map_adr_status("replaced"), "superseded");
    assert_eq!(map_adr_status("whatever"), "proposed");

    // Specs schema enum is now [draft, ready, launched, archived].
    // active/completed fold into "launched"; cancelled into "archived".
    assert_eq!(map_spec_status("active"), "launched");
    assert_eq!(map_spec_status("completed"), "launched");
    assert_eq!(map_spec_status("cancelled"), "archived");
    assert_eq!(map_spec_status("unknown"), "draft");
}

#[test]
fn migration_handles_realistic_volume() {
    let (mut svc, tmp) = setup_data_service();
    let platform = tmp.path().join("platform.db");
    seed_platform_db(&platform);

    // Append 500 notes on top of the base seed (which inserts 1).
    let mut conn = open_platform_rw(&platform);
    let now = chrono::Utc::now().to_rfc3339();
    for i in 0..500 {
        let id = format!("bulk-{i:04}");
        crate::db::block_on(async {
            sqlx::query(
                "INSERT INTO note (id, title, content, tags_json, branch, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(format!("bulk note {i}"))
            .bind("bulk content")
            .bind("[]")
            .bind("main")
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
    }
    crate::db::block_on(async { conn.close().await }).unwrap();

    let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert_eq!(report.notes_migrated, 501);

    let report2 = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert!(report2.already_migrated);
}

#[test]
fn migration_resumes_after_partial_failure() {
    let (mut svc, tmp) = setup_data_service();
    let platform = tmp.path().join("platform.db");
    seed_platform_db(&platform);

    // Simulate a prior partial run: migrate just notes, but don't mark complete.
    let notes_schema = builtin::notes_schema();
    let mut conn_ro = open_platform(&platform, true).unwrap().unwrap();
    let first_pass_notes = tables::migrate_notes(&mut svc, &notes_schema, &mut conn_ro).unwrap();
    assert_eq!(first_pass_notes, 1);
    crate::db::block_on(async { conn_ro.close().await }).unwrap();

    // Full migration: notes already present; adrs/specs still need copying.
    let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert!(!report.already_migrated);
    assert_eq!(report.notes_migrated, 0, "notes already present — none re-copied");
    assert_eq!(report.adrs_migrated, 1);
    assert_eq!(report.specs_migrated, 1);

    let notes = store::query_items(&mut svc, &builtin::notes_schema(), &empty_query(NOTES_ID)).unwrap();
    assert_eq!(notes.len(), 1, "no note duplicates after resume");
}

// ── Prune tests ──────────────────────────────────────────────────────────────

#[test]
fn prune_skips_when_migration_not_complete() {
    let (mut svc, tmp) = setup_data_service();
    let platform = tmp.path().join("platform.db");
    seed_platform_db(&platform);
    // Do NOT run migrate_platform_to_collections — no v1 flag set.
    ensure_migration_state_table(&mut svc).unwrap();

    let report = prune_legacy_source_rows(&mut svc, &platform).unwrap();
    assert!(report.skipped_not_migrated);
    assert_eq!(report.notes_deleted, 0);
}

#[test]
fn prune_deletes_source_rows_after_migration() {
    let (mut svc, tmp) = setup_data_service();
    let platform = tmp.path().join("platform.db");
    seed_platform_db(&platform);

    // Run migration first.
    migrate_platform_to_collections(&mut svc, &platform).unwrap();

    let report = prune_legacy_source_rows(&mut svc, &platform).unwrap();
    assert!(!report.skipped_not_migrated);
    assert!(!report.already_pruned);
    assert_eq!(report.notes_deleted, 1);
    assert_eq!(report.adrs_deleted, 1);
    assert_eq!(report.specs_deleted, 1);

    // Second call short-circuits.
    let report2 = prune_legacy_source_rows(&mut svc, &platform).unwrap();
    assert!(report2.already_pruned);
}

#[test]
fn prune_is_idempotent_on_missing_platform_db() {
    let (mut svc, tmp) = setup_data_service();
    let missing = tmp.path().join("nope").join("platform.db");

    // Mark migration complete manually (as if platform.db was absent at migration time).
    ensure_migration_state_table(&mut svc).unwrap();
    mark_migration_complete(&mut svc, MIGRATION_KEY).unwrap();

    let report = prune_legacy_source_rows(&mut svc, &missing).unwrap();
    assert!(!report.skipped_not_migrated);
    assert_eq!(report.notes_deleted, 0);

    // Second call → already_pruned.
    let report2 = prune_legacy_source_rows(&mut svc, &missing).unwrap();
    assert!(report2.already_pruned);
}

// ── End-to-end: drop migration removes legacy tables after copy ─────────────

/// Fixture-based upgrade path test: seed a `platform.db` that contains all
/// five legacy user-content tables with N rows each, run the collections
/// migration + the sqlx drop migration, and assert (a) every row landed in
/// its destination collection and (b) the five tables no longer exist.
#[test]
fn drop_migration_removes_all_legacy_tables_after_copy() {
    let (mut svc, tmp) = setup_data_service();
    // Seed at the same path ensure_db_at will open later, so migration 0006
    // runs against the same file the test seeds.
    let platform = crate::db::platform_db_path(tmp.path());
    std::fs::create_dir_all(platform.parent().unwrap()).unwrap();
    seed_platform_db(&platform);

    // Additionally seed branch_context — its destination is /dev/null, we only
    // need to prove the drop removes it.
    let mut seed_conn = open_platform_rw(&platform);
    crate::db::block_on(async {
        sqlx::query(
            "CREATE TABLE branch_context (
                branch TEXT PRIMARY KEY, link_type TEXT NOT NULL,
                link_id TEXT NOT NULL, last_synced TEXT NOT NULL
            )",
        )
        .execute(&mut seed_conn)
        .await
    })
    .unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO branch_context (branch, link_type, link_id, last_synced)
             VALUES (?, ?, ?, ?)",
        )
        .bind("feature/x")
        .bind("spec")
        .bind("t1")
        .bind(&now)
        .execute(&mut seed_conn)
        .await
    })
    .unwrap();
    crate::db::block_on(async { seed_conn.close().await }).unwrap();

    // Copy rows into collections.
    let report = migrate_platform_to_collections(&mut svc, &platform).unwrap();
    assert_eq!(report.notes_migrated, 1);
    assert_eq!(report.adrs_migrated, 1);
    assert_eq!(report.specs_migrated, 1);

    // Run sqlx migrations (which includes 0006 drop) against the same platform.db.
    // ensure_db_at uses ship_dir.join("platform.db") — point it at our fixture.
    let fake_ship_dir = tmp.path().to_path_buf();
    crate::db::ensure_db_at(&fake_ship_dir).unwrap();

    // Verify: every legacy table is gone.
    let mut conn = open_platform_rw(&platform);
    for table in ["adr", "note", "target", "capability", "branch_context"] {
        let row: Option<(String,)> = crate::db::block_on(async {
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name = ?")
                .bind(table)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        assert!(
            row.is_none(),
            "table `{table}` should be dropped by migration 0006"
        );
    }
    crate::db::block_on(async { conn.close().await }).unwrap();

    // Collections still contain the copied rows.
    let notes = store::query_items(&mut svc, &builtin::notes_schema(), &empty_query(NOTES_ID)).unwrap();
    assert_eq!(notes.len(), 1);
    let adrs = store::query_items(&mut svc, &builtin::adrs_schema(), &empty_query(ADRS_ID)).unwrap();
    assert_eq!(adrs.len(), 1);
    let specs = store::query_items(&mut svc, &builtin::specs_schema(), &empty_query(SPECS_ID)).unwrap();
    assert_eq!(specs.len(), 1);
}
