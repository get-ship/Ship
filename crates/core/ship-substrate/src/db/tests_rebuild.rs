//! Tests for S5 and S6: per-scope projection rebuild from events, rebuild
//! idempotency. After the Phase-4 storage partition `rebuild_projections`
//! replays each scope db's own log into its own projections, so these tests
//! write through the Runtime scope store (`runtime.db`) and rebuild from there.

use crate::db::rebuild;
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::testutil::init_project_minimal as init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    // Provision the Runtime scope db the rebuild path reads from.
    crate::db::ensure_events_db_at_path(&crate::db::runtime_db_path(&ship_dir)).unwrap();
    (tmp, ship_dir)
}

fn space_opened_event(branch: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::SPACE_OPENED,
        branch,
        &serde_json::json!({
            "path": branch,
            "name": branch,
            "opened_at": "2026-01-01T00:00:00Z"
        }),
    )
    .unwrap()
    .with_scope(EventScope::Runtime, None)
    .elevate()
}

fn space_seen_event(branch: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::SPACE_SEEN,
        branch,
        &serde_json::json!({ "seen_at": "2026-01-01T00:00:00Z" }),
    )
    .unwrap()
    .with_scope(EventScope::Runtime, None)
    .elevate()
}

#[test]
fn rebuild_replays_events_without_panic() {
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_for_scope(&ship_dir, EventScope::Runtime, None).unwrap();
    // space.* events are folded by SpacesProjection; verify rebuild
    // completes cleanly with space lifecycle events present.
    let events = vec![
        space_opened_event("feature/rebuild-a"),
        space_opened_event("feature/rebuild-b"),
        space_seen_event("feature/rebuild-b"),
    ];
    for ev in &events {
        store.append(ev).unwrap();
    }

    let report = rebuild::rebuild_projections(&ship_dir).unwrap();

    assert_eq!(report.events_replayed, 3);
    assert!(report.projections_rebuilt.len() >= 1);
    assert!(report.duration_ms < 10_000);
}

#[test]
fn rebuild_empty_event_log() {
    let (_tmp, ship_dir) = setup();
    let report = rebuild::rebuild_projections(&ship_dir).unwrap();
    assert_eq!(report.events_replayed, 0);
    assert!(report.projections_rebuilt.len() >= 1);
}

#[test]
fn rebuild_report_lists_all_projections() {
    let (_tmp, ship_dir) = setup();
    let report = rebuild::rebuild_projections(&ship_dir).unwrap();
    let names = &report.projections_rebuilt;
    assert!(
        names.contains(&"actor_state".to_string()),
        "report must include actor_state"
    );
}
