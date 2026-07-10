use crate::db::runtime_db_path;
use ship_types::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::events::types::SpaceOpened;
use crate::testutil::init_project_minimal as init_project;
use tempfile::tempdir;
use serde::{Serialize, Deserialize};

// Local test stand-ins for canonical session types (moved to agent-def)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestSession {
    session_id: String,
    agent_id: String,
    harness: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    space_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestTurn {
    session_id: String,
    idx: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestCompleted {
    session_id: String,
}

fn setup() -> (tempfile::TempDir, SqliteEventStore) {
    let tmp = tempdir().unwrap();
    let ship = init_project(tmp.path()).unwrap();
    let store = SqliteEventStore::new_at_db_path(&runtime_db_path(&ship)).unwrap();
    (tmp, store)
}

fn session_started(id: &str) -> TestSession {
    TestSession {
        session_id: id.into(),
        agent_id: "agent-a".into(),
        harness: "claude".into(),
        model: None,
        space_id: "ws-1".into(),
        input: None,
        parent: None,
    }
}

#[test]
fn test_actor_fields_round_trip() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "ws-1", &session_started("ws-1"))?
        .with_actor_id("actor-ws-001")
        .with_parent_actor_id("actor-system-000")
        .elevate();

    store.append(&ev)?;
    let got = store.get(&ev.id)?.unwrap();

    assert_eq!(got.actor_id.as_deref(), Some("actor-ws-001"));
    assert_eq!(got.parent_actor_id.as_deref(), Some("actor-system-000"));
    assert!(got.elevated);
    Ok(())
}

#[test]
fn test_filter_by_actor_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    for i in 0..2_u32 {
        let ev = EventEnvelope::new(event_types::canonical_session::TURN_STARTED, "entity-1", &TestTurn { session_id: "entity-1".into(), idx: i })?
            .with_actor_id("actor-a");
        store.append(&ev)?;
    }
    let ev_b = EventEnvelope::new(event_types::canonical_session::TURN_STARTED, "entity-1", &TestTurn { session_id: "entity-1".into(), idx: 2 })?
        .with_actor_id("actor-b");
    store.append(&ev_b)?;

    let results = store.query(&EventFilter { actor_id: Some("actor-a".into()), ..Default::default() })?;
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.actor_id.as_deref() == Some("actor-a")));

    let results_b = store.query(&EventFilter { actor_id: Some("actor-b".into()), ..Default::default() })?;
    assert_eq!(results_b.len(), 1);
    Ok(())
}

#[test]
fn test_elevated_filter() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "elevation-entity";

    for i in 0..3_u32 {
        let ev = EventEnvelope::new(event_types::canonical_session::TURN_STARTED, entity, &TestTurn { session_id: entity.into(), idx: i })?;
        store.append(&ev)?;
    }
    let ev_end = EventEnvelope::new(event_types::canonical_session::SESSION_COMPLETED, entity, &TestCompleted { session_id: entity.into() })?
        .elevate();
    store.append(&ev_end)?;
    let ev_ws = EventEnvelope::new(
        event_types::SPACE_OPENED,
        entity,
        &SpaceOpened {
            path: "/test/path".into(),
            name: "test".into(),
            opened_at: "2026-01-01T00:00:00Z".into(),
        },
    )?
    .elevate();
    store.append(&ev_ws)?;

    let all = store.query_aggregate(entity)?;
    assert_eq!(all.len(), 5);

    let elevated = store.query(&EventFilter { entity_id: Some(entity.into()), elevated_only: true, ..Default::default() })?;
    assert_eq!(elevated.len(), 2);
    assert!(elevated.iter().all(|e| e.elevated));
    Ok(())
}
#[test]
fn supervisor_queries_elevated_child_events_by_parent_actor_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    let parent_ev = EventEnvelope::new(event_types::ACTOR_CREATED, "supervisor-1", &crate::events::types::ActorCreated { kind: "supervisor".into(), environment_type: "local".into(), parent_actor_id: None, restart_count: 0, ..Default::default() })?
        .with_actor_id("supervisor-1")
        .elevate();
    store.append(&parent_ev)?;

    let child1 = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "child-entity-1", &session_started("child-entity-1"))?
        .with_actor_id("child-actor-1")
        .with_parent_actor_id("supervisor-1")
        .elevate();
    store.append(&child1)?;

    let child2 = EventEnvelope::new(event_types::canonical_session::SESSION_COMPLETED, "child-entity-2", &TestCompleted { session_id: "child-entity-2".into() })?
        .with_actor_id("child-actor-2")
        .with_parent_actor_id("supervisor-1")
        .elevate();
    store.append(&child2)?;

    let child3 = EventEnvelope::new(event_types::canonical_session::TURN_STARTED, "child-entity-3", &TestTurn { session_id: "child-entity-3".into(), idx: 0 })?
        .with_actor_id("child-actor-3")
        .with_parent_actor_id("supervisor-1");
    store.append(&child3)?;

    let results = store.query(&EventFilter {
        parent_actor_id: Some("supervisor-1".into()),
        elevated_only: true,
        ..Default::default()
    })?;

    assert_eq!(results.len(), 2, "expected exactly 2 elevated child events, got {}", results.len());
    assert!(results.iter().all(|e| e.elevated), "all returned events must be elevated");
    assert!(results.iter().all(|e| e.parent_actor_id.as_deref() == Some("supervisor-1")));
    Ok(())
}

#[test]
fn test_hierarchy_traversal_via_parent_actor_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    let ws_ev = EventEnvelope::new(
        event_types::SPACE_OPENED,
        "ws-entity",
        &SpaceOpened {
            path: "/test/path".into(),
            name: "ws-entity".into(),
            opened_at: "2026-01-01T00:00:00Z".into(),
        },
    )?
    .with_actor_id("actor-ws-1");
    store.append(&ws_ev)?;

    let sess_ev = EventEnvelope::new(event_types::canonical_session::SESSION_STARTED, "sess-entity", &session_started("sess-entity"))?
        .with_actor_id("actor-sess-1")
        .with_parent_actor_id("actor-ws-1");
    store.append(&sess_ev)?;

    let children = store.query(&EventFilter {
        actor_id: Some("actor-sess-1".into()),
        ..Default::default()
    })?;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].parent_actor_id.as_deref(), Some("actor-ws-1"));
    Ok(())
}
