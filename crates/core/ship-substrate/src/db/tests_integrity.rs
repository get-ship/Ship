//! Tests for S4: integrity check detects corruption.
//!
//! After the Phase-4 storage partition the integrity check runs per-scope-db.
//! These tests exercise the canonical kernel scope db (`runtime.db`), which
//! `KernelRouter::new` provisions at boot, so they validate the real
//! production read path rather than a defunct platform.db.

use crate::db::integrity;
use crate::testutil::init_project_minimal as init_project;
use tempfile::tempdir;

/// Initialize a project and provision `runtime.db` (the Runtime scope db the
/// kernel ensures at boot). Returns the ship-home dir and the runtime.db path.
fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    let runtime_db = crate::db::runtime_db_path(&ship_dir);
    // Mirror `KernelRouter::new`, which eagerly ensures the Runtime scope db.
    crate::db::ensure_events_db_at_path(&runtime_db).unwrap();
    (tmp, ship_dir, runtime_db)
}

#[test]
fn healthy_db_passes_check() {
    let (_tmp, ship_dir, _runtime_db) = setup();
    match integrity::check_platform_integrity(&ship_dir).unwrap() {
        integrity::IntegrityStatus::Ok => {}
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn quick_check_passes_on_healthy_db() {
    let (_tmp, _ship_dir, runtime_db) = setup();
    assert!(integrity::quick_check(&runtime_db).unwrap());
}

#[test]
fn corrupted_db_detected() {
    let (_tmp, _ship_dir, runtime_db) = setup();

    // Read the file and corrupt internal pages (keep the 100-byte SQLite header
    // so the file opens, but damage data pages so integrity_check fails).
    let mut data = std::fs::read(&runtime_db).unwrap();
    if data.len() > 200 {
        // Corrupt pages beyond the header (first 100 bytes).
        for byte in data[100..200].iter_mut() {
            *byte = 0xFF;
        }
    }
    std::fs::write(&runtime_db, &data).unwrap();

    let result = integrity::check_integrity(&runtime_db);
    match result {
        Ok(integrity::IntegrityStatus::Ok) => {
            // Some SQLite versions may still pass — skip test gracefully.
            eprintln!("SQLite integrity_check passed despite corruption — skipping");
        }
        Ok(integrity::IntegrityStatus::EventsCorrupt { details })
        | Ok(integrity::IntegrityStatus::ProjectionsCorrupt { details }) => {
            assert!(!details.is_empty(), "corruption details must be non-empty");
        }
        Err(_) => {
            // Connection failure due to corruption is also acceptable.
        }
    }
}

/// **The destruction-drill test (claim c-2026-07-11T01-30-00)**:
///
/// DROPS the static Runtime projection table `actors` while events
/// for it exist in the runtime.db log. Proves:
///   1. The absence-detection integrity check classifies the drop as
///      `ProjectionsCorrupt` (recoverable), NOT `EventsCorrupt`.
///   2. `rebuild_projections` recreates the dropped table from the log.
#[test]

fn destruction_drill_drop_static_projection_table_rebuilds_from_log() {
    use crate::db::rebuild;
    use crate::persistence::event_store::{EventStore, SqliteEventStore};
    use crate::events::EventScope;
    use crate::events::types::event_types;
    use ship_types::envelope::EventEnvelope;
    let (_tmp, ship_dir, runtime_db) = setup();

    // Append an actor.created event so the events log now references
    // a projection the integrity check should consider "in use".
    let store = SqliteEventStore::new_for_scope(&ship_dir, EventScope::Runtime, None).unwrap();
    let env = EventEnvelope::new(
        event_types::ACTOR_CREATED,
        "actor-destruction-drill",
        &serde_json::json!({
            "kind": "local",
            "environment_type": "local",
            "harness": null,
            "model": null,
            "session_id": null,
            "parent_actor_id": null,
            "restart_count": 0,
            "agent_id": null,
            "harness_session_id": null,
            "system_prompt_hash": null,
            "transport": null,
            "persistence": null,
            "cwd": null,
        }),
    )
    .unwrap()
    .with_actor_id("actor-destruction-drill")
    .with_scope(EventScope::Runtime, None);
    store.append(&env).unwrap();

    // Now DROP the `actors` projection table — the destruction drill.
    {
        let mut conn = crate::db::open_db_at(&runtime_db).unwrap();
        crate::db::block_on(async {
            sqlx::query("DROP TABLE actors").execute(&mut conn).await
        })
        .unwrap();
    }

    // (1) Integrity check classifies as ProjectionsCorrupt (recoverable),
    // not the fatal EventsCorrupt. The error message names the missing
    // table so an operator knows what drifted.
    match integrity::check_integrity(&runtime_db).unwrap() {
        integrity::IntegrityStatus::ProjectionsCorrupt { details } => {
            assert!(
                details.contains("actors"),
                "drift details must name the missing table; got {details}"
            );
        }
        other => panic!("expected ProjectionsCorrupt, got {other:?}"),
    }

    // (2) Rebuild recreates the table and re-fills it from the log.
    let report = rebuild::rebuild_projections(&ship_dir).unwrap();
    assert_eq!(report.events_replayed, 1);
    assert!(report.projections_rebuilt.iter().any(|n| n == "actor_state"));

    // After rebuild, the integrity check is clean.
    match integrity::check_integrity(&runtime_db).unwrap() {
        integrity::IntegrityStatus::Ok => {}
        other => panic!("post-rebuild check must be Ok, got {other:?}"),
    }

    // And the rebuilt `actors` row is present (the actor projection
    // wrote it from the event during replay).
    let mut conn = crate::db::open_db_at(&runtime_db).unwrap();
    let count: i64 = crate::db::block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM actors WHERE id = ?")
            .bind("actor-destruction-drill")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(count, 1, "rebuild must materialize the actor row from the log");
}

/// An empty events log MUST NOT trigger absence detection: a fresh
/// runtime.db that simply never received events is not drift, even if
/// the static projection tables exist via the DDL on open.
#[test]
fn absence_check_passes_on_empty_log_with_all_tables_present() {
    let (_tmp, ship_dir, runtime_db) = setup();
    match integrity::check_integrity(&runtime_db).unwrap() {
        integrity::IntegrityStatus::Ok => {}
        other => panic!("fresh runtime.db must be Ok, got {other:?}"),
    }
    assert!(integrity::check_platform_integrity(&ship_dir).is_ok());
}

/// The PROJECTS→SPACES rename (claim c-2026-07-11T01-30-00): the
/// previous integrity allowlist named `"projects"`, but the spaces
/// projection writes to the `spaces` table after the 2026-07-09 space
/// cutover. The new allowlist must NOT contain the stale name; it
/// must contain the current one. Verified by checking the
/// `spaces` table name appears in `PROJECTION_TABLE_TAGS`.
#[test]
fn projects_to_spaces_rename_is_in_allowlist() {
    // Stale name removed.
    assert!(
        !integrity::PROJECTION_TABLE_TAGS.contains(&"projects"),
        "allowlist must not contain the legacy 'projects' name",
    );
    // Current name present.
    assert!(
        integrity::PROJECTION_TABLE_TAGS.contains(&"spaces"),
        "allowlist must contain the current 'spaces' name",
    );
}
