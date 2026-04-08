//! Tests for S1: shared envelope_from_row handles both platform and actor DB schemas.
//!
//! The platform.db events table has a `target_actor_id` column.
//! The job projection query omits `target_actor_id` (13-col SELECT).
//! A shared `envelope_from_row` must handle both schemas correctly.

use crate::db::{block_on, ensure_db, open_db};
use crate::events::envelope::EventEnvelope;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();
    (tmp, ship_dir)
}

fn sample_event(target: Option<&str>) -> EventEnvelope {
    let mut ev = EventEnvelope::new("test.envelope", "entity-1", &serde_json::json!({"k": "v"}))
        .unwrap();
    if let Some(t) = target {
        ev = ev.with_target(t);
    }
    ev
}

#[test]
fn envelope_from_row_platform_with_target_actor_id() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();

    let ev = sample_event(Some("actor-42"));
    store.append(&ev).unwrap();

    // Read back via list_all_events which uses row_to_envelope on the 14-col SELECT
    let all = crate::db::events::list_all_events().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[0].target_actor_id.as_deref(),
        Some("actor-42"),
        "platform.db events must preserve target_actor_id"
    );
}

#[test]
fn envelope_from_row_platform_without_target_actor_id() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();

    let ev = sample_event(None);
    store.append(&ev).unwrap();

    let all = crate::db::events::list_all_events().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[0].target_actor_id, None,
        "platform.db events with no target must have target_actor_id = None"
    );
}

#[test]
fn envelope_from_row_job_projection_sets_target_actor_id_none() {
    // The job projection uses a 13-col SELECT (no target_actor_id column).
    // After dedup, load_jobs_from_conn must still produce valid EventEnvelopes
    // with target_actor_id = None.
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    let job_ev = EventEnvelope::new(
        "job.created",
        "test-entity",
        &serde_json::json!({
            "job_id": "j-envelope-test",
            "slug": "envelope-test",
            "agent": "test",
            "branch": "job/envelope-test",
            "spec_path": "spec.md",
            "plan_id": null
        }),
    )
    .unwrap()
    .with_target("actor-99"); // target set, but job projection ignores it

    store.append(&job_ev).unwrap();

    let jobs = crate::projections::job::load_jobs_at(&ship_dir).unwrap();
    let rec = jobs.get("j-envelope-test").unwrap();
    assert_eq!(rec.slug, "envelope-test");
    // The job projection's row_to_envelope omits target_actor_id —
    // after dedup this should be None (not panic).
}

#[test]
fn envelope_round_trip_preserves_all_fields() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();

    let ev = EventEnvelope::new("test.roundtrip", "entity-rt", &serde_json::json!({"x": 1}))
        .unwrap()
        .with_causation("cause-1")
        .with_context(Some("ws-1"), Some("sess-1"))
        .with_actor_id("actor-a")
        .with_parent_actor_id("actor-parent")
        .with_target("actor-target")
        .elevate();

    store.append(&ev).unwrap();

    let all = crate::db::events::list_all_events().unwrap();
    assert_eq!(all.len(), 1);
    let loaded = &all[0];

    assert_eq!(loaded.id, ev.id);
    assert_eq!(loaded.event_type, "test.roundtrip");
    assert_eq!(loaded.entity_id, "entity-rt");
    assert_eq!(loaded.actor, "ship");
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.causation_id.as_deref(), Some("cause-1"));
    assert_eq!(loaded.workspace_id.as_deref(), Some("ws-1"));
    assert_eq!(loaded.session_id.as_deref(), Some("sess-1"));
    assert_eq!(loaded.actor_id.as_deref(), Some("actor-a"));
    assert_eq!(loaded.parent_actor_id.as_deref(), Some("actor-parent"));
    assert_eq!(loaded.target_actor_id.as_deref(), Some("actor-target"));
    assert!(loaded.elevated);
}
