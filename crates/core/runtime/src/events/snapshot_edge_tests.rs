//! Snapshot edge case tests — zero events, many events, stats accuracy,
//! serialization limits, and restore from empty snapshots.

use chrono::Utc;
use tempfile::tempdir;

use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::snapshot::ActorSnapshot;

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
    (tmp, router)
}

fn agent_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec![format!("{id}.")],
        read_namespaces: vec![format!("{id}.")],
        subscribe_namespaces: vec![format!("{id}.")],
    }
}

fn ev(id: &str, event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, id, &serde_json::json!({})).unwrap()
}

#[test]
fn snapshot_with_zero_events() {
    let (_tmp, mut router) = setup();
    let (_store, _mb) = router
        .spawn_actor("agent-empty", agent_config("agent-empty"))
        .unwrap();

    let snap = router.snapshot("agent-empty").unwrap();
    assert_eq!(snap.event_count, 0);
    assert!(snap.last_event_id.is_none());
    assert!(!snap.db_bytes.is_empty(), "DB bytes must exist even with 0 events");
}

#[test]
fn snapshot_with_many_events() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router
        .spawn_actor("agent-busy", agent_config("agent-busy"))
        .unwrap();

    for i in 0..500u32 {
        let event = EventEnvelope::new(
            "agent-busy.tick",
            &format!("entity-{i}"),
            &serde_json::json!({ "seq": i }),
        )
        .unwrap();
        store.append(&event).unwrap();
    }

    let snap = router.snapshot("agent-busy").unwrap();
    assert_eq!(snap.event_count, 500);
    assert!(snap.last_event_id.is_some());
}

#[test]
fn event_stats_accuracy_after_append_cycles() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router
        .spawn_actor("agent-cycle", agent_config("agent-cycle"))
        .unwrap();

    for i in 0..10u32 {
        store
            .append(
                &EventEnvelope::new(
                    "agent-cycle.step",
                    "entity",
                    &serde_json::json!({ "seq": i }),
                )
                .unwrap(),
            )
            .unwrap();
    }

    let snap = router.snapshot("agent-cycle").unwrap();
    assert_eq!(snap.event_count, 10);

    for i in 10..15u32 {
        store
            .append(
                &EventEnvelope::new(
                    "agent-cycle.step",
                    "entity",
                    &serde_json::json!({ "seq": i }),
                )
                .unwrap(),
            )
            .unwrap();
    }

    let snap2 = router.snapshot("agent-cycle").unwrap();
    assert_eq!(snap2.event_count, 15);
    assert_ne!(
        snap.last_event_id, snap2.last_event_id,
        "last_event_id must change after new appends"
    );
}

#[test]
fn restore_from_zero_event_snapshot() {
    let (_tmp, mut router) = setup();
    let (_store, _mb) = router
        .spawn_actor("agent-empty", agent_config("agent-empty"))
        .unwrap();

    let snap = router.suspend("agent-empty").unwrap();
    assert_eq!(snap.event_count, 0);

    let (restored_store, _mb) = router.restore(snap).unwrap();
    let events = restored_store.query(&EventFilter::default()).unwrap();
    assert_eq!(events.len(), 0);

    restored_store
        .append(&ev("entity-1", "agent-empty.first_event"))
        .unwrap();
    let events = restored_store.query(&EventFilter::default()).unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn snapshot_serialization_roundtrip_with_large_payload() {
    let big_payload = "x".repeat(100_000);
    let snap = ActorSnapshot {
        actor_id: "agent-big".into(),
        namespace: "agent-big".into(),
        config: agent_config("agent-big"),
        db_bytes: big_payload.as_bytes().to_vec(),
        created_at: Utc::now(),
        event_count: 1000,
        last_event_id: Some("01JXYZ".into()),
    };

    let bytes = snap.to_bytes().unwrap();
    let restored = ActorSnapshot::from_bytes(&bytes).unwrap();
    assert_eq!(restored.db_bytes.len(), 100_000);
    assert_eq!(restored.event_count, 1000);
}

#[test]
fn snapshot_deserialization_rejects_garbage() {
    let result = ActorSnapshot::from_bytes(b"not valid msgpack");
    assert!(result.is_err());
}
