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
            "status": "active",
            "active_agent": null,
            "providers": [],
            "mcp_servers": [],
            "skills": [],
            "is_worktree": false,
            "worktree_path": null
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

fn query_workspace_status(
    ship_dir: &std::path::Path,
    branch: &str,
) -> Option<String> {
    let mut conn = open_db_at(&ship_dir.join("platform.db")).unwrap();
    let rows: Vec<(String,)> = block_on(async {
        sqlx::query_as("SELECT status FROM workspace WHERE branch = ?")
            .bind(branch)
            .fetch_all(&mut conn)
            .await
    })
    .unwrap();
    rows.first().map(|r| r.0.clone())
}

#[test]
fn rebuild_from_events_produces_correct_state() {
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    // Emit events through the store (bypassing projections).
    let events = vec![
        workspace_created_event("feature/rebuild-a"),
        workspace_created_event("feature/rebuild-b"),
        workspace_archived_event("feature/rebuild-b"),
    ];
    for ev in &events {
        store.append(ev).unwrap();
    }

    // No projections applied yet — workspace table should be empty or stale.
    // Run rebuild to reconstruct from events.
    let report = rebuild::rebuild_projections(&ship_dir).unwrap();

    assert_eq!(report.events_replayed, 3);
    assert!(report.projections_rebuilt.len() >= 3);
    assert!(report.duration_ms < 10_000); // sanity check

    // Verify rebuilt state matches what events describe.
    assert_eq!(
        query_workspace_status(&ship_dir, "feature/rebuild-a"),
        Some("active".to_string())
    );
    assert_eq!(
        query_workspace_status(&ship_dir, "feature/rebuild-b"),
        Some("archived".to_string())
    );
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
    let status_a = query_workspace_status(&ship_dir, "feature/idem-a");
    let status_b = query_workspace_status(&ship_dir, "feature/idem-b");

    // Second rebuild — state must be identical.
    rebuild::rebuild_projections(&ship_dir).unwrap();
    let count_after = query_workspace_count(&ship_dir);
    let status_a2 = query_workspace_status(&ship_dir, "feature/idem-a");
    let status_b2 = query_workspace_status(&ship_dir, "feature/idem-b");

    assert_eq!(count_before, count_after, "row count must not change");
    assert_eq!(status_a, status_a2, "workspace A status must be stable");
    assert_eq!(status_b, status_b2, "workspace B status must be stable");
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
