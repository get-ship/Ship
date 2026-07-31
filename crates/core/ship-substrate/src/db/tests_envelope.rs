//! Tests for S1: shared envelope_from_row handles both platform and actor DB schemas.
//!
//! The platform.db events table has a `target_actor_id` column.
//! The job projection query omits `target_actor_id` (13-col SELECT).
//! A shared `envelope_from_row` must handle both schemas correctly.

use crate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::Stamped;
use crate::testutil::init_project_minimal as init_project;
use ship_types::{envelope::EventEnvelope, event_registry::EventScope};
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();
    (tmp, ship_dir)
}

/// Seed into runtime.db — the db `list_all_events` reads after Phase 4d.
fn runtime_store() -> SqliteEventStore {
    SqliteEventStore::new_for_scope(
        &crate::project::get_global_dir().unwrap(),
        ship_types::event_registry::EventScope::Runtime,
        None,
    )
    .unwrap()
}

fn sample_event(target: Option<&str>) -> EventEnvelope {
    let mut ev =
        EventEnvelope::new("test.envelope", "entity-1", &serde_json::json!({"k": "v"})).unwrap();
    if let Some(t) = target {
        ev = ev.with_target(t);
    }
    ev
}

#[test]
fn envelope_from_row_platform_with_target_actor_id() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();

    let ev = sample_event(Some("actor-42"));
    store.append(&Stamped::unchecked(ev.clone())).unwrap();

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
    let store = runtime_store();

    let ev = sample_event(None);
    store.append(&Stamped::unchecked(ev.clone())).unwrap();

    let all = crate::db::events::list_all_events().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[0].target_actor_id, None,
        "platform.db events with no target must have target_actor_id = None"
    );
}

#[test]
fn envelope_round_trip_preserves_all_fields() {
    let (_tmp, _ship_dir) = setup();
    let store = runtime_store();

    let mut ev = EventEnvelope::new("test.roundtrip", "entity-rt", &serde_json::json!({"x": 1}))
        .unwrap()
        .with_causation("cause-1")
        .with_scope(EventScope::Space, Some("ws-1"))
        .with_actor_id("actor-a")
        .with_parent_actor_id("actor-parent")
        .with_target("actor-target")
        .elevate();
    ev.session_id = Some("sess-1".to_string());

    store.append(&Stamped::unchecked(ev.clone())).unwrap();

    let all = crate::db::events::list_all_events().unwrap();
    assert_eq!(all.len(), 1);
    let loaded = &all[0];

    assert_eq!(loaded.id, ev.id);
    assert_eq!(loaded.event_type, "test.roundtrip");
    assert_eq!(loaded.entity_id, "entity-rt");
    assert_eq!(loaded.actor, "ship");
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.causation_id.as_deref(), Some("cause-1"));
    assert_eq!(loaded.space_id(), Some("ws-1"));
    assert_eq!(loaded.scope, ship_types::event_registry::EventScope::Space);
    assert_eq!(loaded.session_id.as_deref(), Some("sess-1"));
    assert_eq!(loaded.actor_id.as_deref(), Some("actor-a"));
    assert_eq!(loaded.parent_actor_id.as_deref(), Some("actor-parent"));
    assert_eq!(loaded.target_actor_id.as_deref(), Some("actor-target"));
    assert!(loaded.elevated);
}
