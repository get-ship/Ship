//! Tests for S5 and S6: projection rebuild from events, rebuild idempotency.

use crate::db::{block_on, ensure_db_at, open_db_at, rebuild};
use ship_types::envelope::EventEnvelope;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

fn workspace_created_event(branch: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::WORKSPACE_CREATED,
        branch,
        &serde_json::json!({
            "workspace_id": branch,
            "workspace_type": "feature",
            "active_agent": null,
            "providers": [],
            "mcp_servers": [],
            "skills": [],
            "path": null
        }),
    )
    .unwrap()
    .with_context(Some(branch), None)
    .elevate()
}

fn workspace_archived_event(branch: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::WORKSPACE_ARCHIVED,
        branch,
        &serde_json::json!({}),
    )
    .unwrap()
    .with_context(Some(branch), None)
    .elevate()
}

fn query_workspace_count(ship_dir: &std::path::Path) -> i64 {
    let mut conn = open_db_at(&ship_dir.join("platform.db")).unwrap();
    let (count,): (i64,) = block_on(async {
        sqlx::query_as("SELECT COUNT(*) FROM workspace")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    count
}

fn query_workspace_exists(
    ship_dir: &std::path::Path,
    branch: &str,
) -> bool {
    let mut conn = open_db_at(&ship_dir.join("platform.db")).unwrap();
    let (count,): (i64,) = block_on(async {
        sqlx::query_as("SELECT COUNT(*) FROM workspace WHERE branch = ?")
            .bind(branch)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    count > 0
}

#[test]
fn rebuild_from_events_produces_correct_state() {
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    // Emit events through the store (bypassing projections).
    // workspace.archived is now a no-op (status removed), so both workspaces
    // survive rebuild — we just verify they exist.
    let events = vec![
        workspace_created_event("feature/rebuild-a"),
        workspace_created_event("feature/rebuild-b"),
        workspace_archived_event("feature/rebuild-b"),
    ];
    for ev in &events {
        store.append(ev).unwrap();
    }

    let report = rebuild::rebuild_projections(&ship_dir).unwrap();

    assert_eq!(report.events_replayed, 3);
    assert!(report.projections_rebuilt.len() >= 3);
    assert!(report.duration_ms < 10_000);

    assert!(query_workspace_exists(&ship_dir, "feature/rebuild-a"));
    assert!(query_workspace_exists(&ship_dir, "feature/rebuild-b"));
}

#[test]
fn rebuild_is_idempotent() {
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    let events = vec![
        workspace_created_event("feature/idem-a"),
        workspace_created_event("feature/idem-b"),
    ];
    for ev in &events {
        store.append(ev).unwrap();
    }

    // First rebuild.
    rebuild::rebuild_projections(&ship_dir).unwrap();
    let count_before = query_workspace_count(&ship_dir);

    // Second rebuild — state must be identical.
    rebuild::rebuild_projections(&ship_dir).unwrap();
    let count_after = query_workspace_count(&ship_dir);

    assert_eq!(count_before, count_after, "row count must not change");
    assert!(query_workspace_exists(&ship_dir, "feature/idem-a"));
    assert!(query_workspace_exists(&ship_dir, "feature/idem-b"));
}

#[test]
fn rebuild_empty_event_log() {
    let (_tmp, ship_dir) = setup();
    let report = rebuild::rebuild_projections(&ship_dir).unwrap();
    assert_eq!(report.events_replayed, 0);
    assert!(report.projections_rebuilt.len() >= 3);
}

#[test]
fn rebuild_report_lists_all_projections() {
    let (_tmp, ship_dir) = setup();
    let report = rebuild::rebuild_projections(&ship_dir).unwrap();
    let names = &report.projections_rebuilt;
    assert!(
        names.contains(&"workspace_state".to_string()),
        "report must include workspace_state"
    );
    assert!(
        names.contains(&"session_state".to_string()),
        "report must include session_state"
    );
    assert!(
        names.contains(&"actor_state".to_string()),
        "report must include actor_state"
    );
}
