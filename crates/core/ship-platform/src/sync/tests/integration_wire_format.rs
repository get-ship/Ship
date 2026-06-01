/// Integration tests — sync wire-format invariants.
///
/// Covers the canonical wire-format contracts identified in the audit:
///
/// 1. `x-project-id` header model — space_id must NOT appear in PushRequest body
/// 2. EventEnvelope field layout round-trips correctly through PushRequest JSON
/// 3. `sync.push.completed` payload shape — F4 fix: uses `events_pushed` + `duration_ms`
///    (not `accepted`; the PushResponse.accepted field is the transport ack count,
///    but the event payload uses the canonical field names from SyncPushCompleted)
/// 4. PullResponse envelope body preserves all EventEnvelope fields intact
/// 5. `elevated` flag survives round-trip through PushRequest → JSON
///
/// All tests are pure ser/de — no filesystem, no network, no tempdir needed.
use chrono::Utc;
use ship_substrate::events::EventEnvelope;
use ship_types::payload::SyncPushCompleted;

use crate::sync::types::{PullResponse, PushRequest, PushResponse};

fn elevated_envelope(id: &str, event_type: &str) -> EventEnvelope {
    EventEnvelope {
        id: id.to_string(),
        event_type: event_type.to_string(),
        entity_id: "entity-1".to_string(),
        actor: "ship".to_string(),
        payload_json: r#"{"k":"v"}"#.to_string(),
        version: 1,
        causation_id: None,
        scope: ship_substrate::events::EventScope::Space,
        scope_key: Some("sp-wire-test".to_string()),
        session_id: None,
        actor_id: None,
        parent_actor_id: None,
        target_actor_id: None,
        elevated: true,
        created_at: Utc::now(),
    }
}

// ── Header auth model — space_id placement ────────────────────────────────────

#[test]
fn sync_wire_format_push_request_no_top_level_space_id() {
    // The gateway's x-project-id middleware (`apps/gateway/src/middleware/
    // project-auth.ts:23–24`) is the auth surface — space_id must not be
    // duplicated at the PushRequest body level.
    let events = vec![elevated_envelope("01JAA", "workspace.created")];
    let req = PushRequest { events };
    let json = serde_json::to_value(&req).unwrap();

    assert!(
        json.get("space_id").is_none(),
        "PushRequest body must NOT contain a top-level space_id field (auth via x-project-id header)"
    );
    assert!(
        json.get("project_id").is_none(),
        "PushRequest body must NOT contain project_id — auth is header-only"
    );
}

#[test]
fn sync_wire_format_push_request_scope_key_in_envelope_not_duplicated() {
    // The partition key appears exactly once per event (inside the
    // EventEnvelope), not at the PushRequest wrapper level.
    let events = vec![elevated_envelope("01JAA", "workspace.created")];
    let req = PushRequest { events };
    let json_str = serde_json::to_string(&req).unwrap();

    // Should appear exactly once (inside the envelope object).
    let count = json_str.matches("\"scope_key\"").count();
    assert_eq!(
        count, 1,
        "\"scope_key\" must appear exactly once in the serialised PushRequest (inside the envelope, not duplicated); json: {json_str}"
    );
}

// ── EventEnvelope field layout ────────────────────────────────────────────────

#[test]
fn sync_wire_format_elevated_flag_preserved_in_push_request() {
    let mut env = elevated_envelope("01JAA", "gate.approved");
    env.elevated = true;
    let req = PushRequest { events: vec![env] };
    let json = serde_json::to_value(&req).unwrap();

    let elevated = json["events"][0]["elevated"].as_bool().unwrap();
    assert!(elevated, "elevated flag must be preserved in serialised PushRequest");
}

#[test]
fn sync_wire_format_non_elevated_flag_preserved_in_push_request() {
    let mut env = elevated_envelope("01JAA", "session.heartbeat");
    env.elevated = false;
    let req = PushRequest { events: vec![env] };
    let json = serde_json::to_value(&req).unwrap();

    let elevated = json["events"][0]["elevated"].as_bool().unwrap();
    assert!(!elevated, "elevated=false must be preserved in serialised PushRequest");
}

#[test]
fn sync_wire_format_event_id_and_type_survive_push_request_round_trip() {
    let env = elevated_envelope("01JFIXED-ID-01", "pipeline.run.completed");
    let req = PushRequest { events: vec![env] };
    let json = serde_json::to_value(&req).unwrap();

    assert_eq!(json["events"][0]["id"].as_str().unwrap(), "01JFIXED-ID-01");
    assert_eq!(
        json["events"][0]["event_type"].as_str().unwrap(),
        "pipeline.run.completed"
    );
}

#[test]
fn sync_wire_format_causation_id_null_when_none() {
    let env = elevated_envelope("01JAA", "workspace.created");
    assert!(env.causation_id.is_none());
    let req = PushRequest { events: vec![env] };
    let json = serde_json::to_value(&req).unwrap();

    // null is the canonical JSON form of an absent causation_id.
    assert!(
        json["events"][0]["causation_id"].is_null(),
        "causation_id must be null when None"
    );
}

// ── sync.push.completed payload shape (F4 fix) ───────────────────────────────

#[test]
fn sync_wire_format_push_completed_payload_has_events_pushed_field() {
    // F4 fix: the sync.push.completed event payload uses `events_pushed`
    // (not `accepted`). `accepted` is the PushResponse transport ack field;
    // `events_pushed` is the canonical payload field (SyncPushCompleted).
    let payload = SyncPushCompleted {
        events_pushed: 7,
        duration_ms: 42,
    };
    let json = serde_json::to_value(&payload).unwrap();

    assert!(
        json.get("events_pushed").is_some(),
        "sync.push.completed payload must have `events_pushed` field (F4 fix)"
    );
    assert_eq!(
        json["events_pushed"].as_u64().unwrap(),
        7,
        "events_pushed must match the count passed to the payload"
    );
}

#[test]
fn sync_wire_format_push_completed_payload_has_duration_ms_field() {
    let payload = SyncPushCompleted {
        events_pushed: 3,
        duration_ms: 150,
    };
    let json = serde_json::to_value(&payload).unwrap();

    assert!(
        json.get("duration_ms").is_some(),
        "sync.push.completed payload must have `duration_ms` field"
    );
    assert_eq!(json["duration_ms"].as_u64().unwrap(), 150);
}

#[test]
fn sync_wire_format_push_completed_payload_has_no_accepted_field() {
    // `accepted` is PushResponse's transport ack field — it must NOT bleed
    // into the event payload. The payload has its own canonical field name.
    let payload = SyncPushCompleted {
        events_pushed: 5,
        duration_ms: 30,
    };
    let json = serde_json::to_value(&payload).unwrap();

    assert!(
        json.get("accepted").is_none(),
        "sync.push.completed payload must NOT have an `accepted` field — use `events_pushed`"
    );
}

#[test]
fn sync_wire_format_push_completed_payload_round_trips() {
    let payload = SyncPushCompleted {
        events_pushed: 12,
        duration_ms: 999,
    };
    let json_str = serde_json::to_string(&payload).unwrap();
    let decoded: SyncPushCompleted = serde_json::from_str(&json_str).unwrap();

    assert_eq!(decoded.events_pushed, 12);
    assert_eq!(decoded.duration_ms, 999);
}

// ── PushResponse / PullResponse field contract ────────────────────────────────

#[test]
fn sync_wire_format_push_response_cursor_is_advancement_token() {
    // The cursor returned by PushResponse is the canonical cursor for the
    // next push cycle (stored via cursors::set). It must be preserved exactly
    // without transformation.
    let json = r#"{"accepted": 4, "cursor": "01JCURSOR-EXACT"}"#;
    let resp: PushResponse = serde_json::from_str(json).unwrap();
    assert_eq!(
        resp.cursor, "01JCURSOR-EXACT",
        "cursor must be preserved verbatim from the gateway response"
    );
}

#[test]
fn sync_wire_format_pull_response_preserves_all_envelope_fields() {
    // All EventEnvelope fields survive the pull response round-trip.
    // The gateway returns the full EventEnvelope shape — nothing is stripped.
    // Using serde_json::json! macro avoids raw-string delimiter edge cases.
    let pull_json = serde_json::json!({
        "events": [{
            "id": "01JPRESERVE",
            "event_type": "workspace.archived",
            "entity_id": "ws-preserve",
            "actor": "ship",
            "payload_json": "{\"archived\":true}",
            "version": 3,
            "causation_id": "01JPREV",
            "scope": "space",
            "scope_key": "sp-preserve",
            "session_id": "sess-1",
            "actor_id": "actor-1",
            "parent_actor_id": "parent-1",
            "target_actor_id": null,
            "elevated": false,
            "created_at": "2026-05-30T12:00:00Z"
        }],
        "cursor": "01JPRESERVE"
    });
    let json = serde_json::to_string(&pull_json).unwrap();

    let resp: PullResponse = serde_json::from_str(&json).unwrap();
    let env = &resp.events[0];

    assert_eq!(env.id, "01JPRESERVE");
    assert_eq!(env.event_type, "workspace.archived");
    assert_eq!(env.entity_id, "ws-preserve");
    assert_eq!(env.version, 3);
    assert_eq!(env.causation_id.as_deref(), Some("01JPREV"));
    assert_eq!(env.space_id(), Some("sp-preserve"));
    assert_eq!(env.scope, ship_substrate::events::EventScope::Space);
    assert_eq!(env.session_id.as_deref(), Some("sess-1"));
    assert_eq!(env.actor_id.as_deref(), Some("actor-1"));
    assert_eq!(env.parent_actor_id.as_deref(), Some("parent-1"));
    assert!(!env.elevated, "elevated must be false as sent");
}
