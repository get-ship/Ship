//! Integration tests — PushRequest, PushResponse, PullResponse contracts.
//!
//! Covers:
//! - PushRequest serialises events correctly, space_id in envelope body
//! - PushResponse deserialises accepted count + cursor
//! - PullResponse deserialises event list + cursor
//! - PullResponse with empty events list
//! - PushResponse cursor field is the canonical advancement token
//! - EventEnvelope round-trip through PushRequest → JSON → back
//!
//! These are pure ser/de tests — no filesystem, no network. Hermetic.
//!
//! Originally nested under `ship-platform/src/sync/tests/integration_types.rs`;
//! carved into a top-level integration test for `ship-cloud` so the
//! wire-format contract is enforced at the crate boundary.

use chrono::Utc;
use ship_substrate::events::EventEnvelope;
use ship_cloud::types::{PullResponse, PushRequest, PushResponse};

fn sample_envelope(id: &str, event_type: &str, space_id: &str) -> EventEnvelope {
    EventEnvelope {
        id: id.to_string(),
        event_type: event_type.to_string(),
        entity_id: "entity-1".to_string(),
        actor: "ship".to_string(),
        payload_json: r#"{"key":"val"}"#.to_string(),
        version: 1,
        causation_id: None,
        scope: ship_substrate::events::EventScope::Space,
        scope_key: Some(space_id.to_string()),
        session_id: None,
        actor_id: None,
        parent_actor_id: None,
        target_actor_id: None,
        elevated: true,
        created_at: Utc::now(),
        author_account_id: None,
        author_device_id: None,
        signature: None,
    }
}

// ── PushRequest ───────────────────────────────────────────────────────────────

#[test]
fn sync_types_push_request_serializes_events_array() {
    let events = vec![
        sample_envelope("01JAAA", "actor.created", "sp-1"),
        sample_envelope("01JBBB", "agent.task.complete", "sp-1"),
    ];
    let req = PushRequest { events };
    let json = serde_json::to_value(&req).unwrap();

    let arr = json["events"].as_array().expect("events must be an array");
    assert_eq!(arr.len(), 2, "both events must be serialised");
}

#[test]
fn sync_types_push_request_has_no_top_level_space_id() {
    // The space_id auth model uses the x-project-id header, not the request
    // body. This is the canonical wire-format invariant documented in types.rs.
    let events = vec![sample_envelope("01JAAA", "actor.created", "sp-xyz")];
    let req = PushRequest { events };
    let json = serde_json::to_value(&req).unwrap();

    assert!(
        json.get("space_id").is_none(),
        "space_id must NOT appear at the PushRequest top level — it belongs in the x-project-id header"
    );
}

#[test]
fn sync_types_push_request_space_id_in_envelope_body() {
    // space_id lives inside each EventEnvelope, not duplicated into the request
    // wrapper. Confirmed by the audit's wire-format note (types.rs:54–63).
    let events = vec![sample_envelope("01JAAA", "actor.created", "sp-canonical")];
    let req = PushRequest { events };
    let json_str = serde_json::to_string(&req).unwrap();

    assert!(
        json_str.contains("sp-canonical"),
        "space_id must appear in the envelope body; json: {json_str}"
    );
}

#[test]
fn sync_types_push_request_event_type_preserved() {
    let events = vec![sample_envelope("01JAAA", "pipeline.run.completed", "sp-1")];
    let req = PushRequest { events };
    let json = serde_json::to_value(&req).unwrap();

    let event_type = json["events"][0]["event_type"].as_str().unwrap();
    assert_eq!(event_type, "pipeline.run.completed");
}

#[test]
fn sync_types_push_request_empty_events_serializes() {
    let req = PushRequest { events: vec![] };
    let json = serde_json::to_value(&req).unwrap();
    let arr = json["events"].as_array().unwrap();
    assert!(arr.is_empty(), "empty events array must serialise as []");
}

// ── PushResponse ──────────────────────────────────────────────────────────────

#[test]
fn sync_types_push_response_deserializes_accepted_and_cursor() {
    let json = r#"{"accepted": 5, "cursor": "01JCCCCCCCCCCCCCCCCCCCCCCCCC"}"#;
    let resp: PushResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.accepted, 5, "accepted count must match");
    assert_eq!(resp.cursor, "01JCCCCCCCCCCCCCCCCCCCCCCCCC", "cursor must match");
}

#[test]
fn sync_types_push_response_accepted_zero() {
    // A zero-accepted response is valid — e.g., duplicate batch detection.
    let json = r#"{"accepted": 0, "cursor": "01JDDDDDDDDDDDDDDDDDDDDDDDDD"}"#;
    let resp: PushResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.accepted, 0);
}

#[test]
fn sync_types_push_response_cursor_is_non_empty_after_success() {
    let json = r#"{"accepted": 3, "cursor": "01JEEEEEEEEEEEEEEEEEEEEEEEEE"}"#;
    let resp: PushResponse = serde_json::from_str(json).unwrap();
    assert!(
        !resp.cursor.is_empty(),
        "cursor must be non-empty after a successful push"
    );
}

// ── PullResponse ──────────────────────────────────────────────────────────────

#[test]
fn sync_types_pull_response_deserializes_events_and_cursor() {
    let json = r#"{
        "events": [{
            "id": "01JFFFFF",
            "event_type": "actor.created",
            "entity_id": "ws-1",
            "actor": "ship",
            "payload_json": "{}",
            "version": 1,
            "causation_id": null,
            "scope": "space",
            "scope_key": "sp-1",
            "session_id": null,
            "actor_id": null,
            "parent_actor_id": null,
            "target_actor_id": null,
            "elevated": true,
            "created_at": "2026-03-01T00:00:00Z"
        }],
        "cursor": "01JFFFFF"
    }"#;
    let resp: PullResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.events.len(), 1, "one event must deserialise");
    assert_eq!(resp.events[0].event_type, "actor.created");
    assert_eq!(resp.cursor, "01JFFFFF");
}

#[test]
fn sync_types_pull_response_empty_events() {
    let json = r#"{"events": [], "cursor": ""}"#;
    let resp: PullResponse = serde_json::from_str(json).unwrap();
    assert!(resp.events.is_empty(), "empty events array must deserialise");
    assert_eq!(resp.cursor, "");
}

#[test]
fn sync_types_pull_response_space_id_preserved_in_envelope() {
    // space_id round-trips through the pull response envelope — it is NOT
    // stripped by the gateway before returning to the client.
    let json = r#"{
        "events": [{
            "id": "01JGGGGG",
            "event_type": "sync.push.completed",
            "entity_id": "sync",
            "actor": "ship",
            "payload_json": "{}",
            "version": 1,
            "causation_id": null,
            "scope": "space",
            "scope_key": "sp-preserved",
            "session_id": null,
            "actor_id": null,
            "parent_actor_id": null,
            "target_actor_id": null,
            "elevated": false,
            "created_at": "2026-05-01T00:00:00Z"
        }],
        "cursor": "01JGGGGG"
    }"#;
    let resp: PullResponse = serde_json::from_str(json).unwrap();
    assert_eq!(
        resp.events[0].space_id(),
        Some("sp-preserved"),
        "scope_key in envelope must be preserved through pull response"
    );
}
