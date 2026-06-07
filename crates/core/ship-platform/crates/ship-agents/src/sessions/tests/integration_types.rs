/// Integration tests — sessions types: SessionStatus Display/FromStr,
/// SessionLiveness::from_activity, and EndSessionRequest defaults.
///
/// All tests are hermetic pure-type tests — no substrate I/O needed.
use chrono::Utc;

use crate::sessions::{Session, SessionStatus};
use crate::sessions::types_session::{EndSessionRequest, SessionLiveness};

// ── SessionStatus Display ─────────────────────────────────────────────────────

#[test]
fn sessions_types_session_status_display_active() {
    assert_eq!(SessionStatus::Active.to_string(), "active");
}

#[test]
fn sessions_types_session_status_display_ended() {
    assert_eq!(SessionStatus::Ended.to_string(), "ended");
}

// ── SessionStatus FromStr ─────────────────────────────────────────────────────

#[test]
fn sessions_types_session_status_from_str_active_lowercase() {
    let status: SessionStatus = "active".parse().expect("must parse");
    assert_eq!(status, SessionStatus::Active);
}

#[test]
fn sessions_types_session_status_from_str_ended_lowercase() {
    let status: SessionStatus = "ended".parse().expect("must parse");
    assert_eq!(status, SessionStatus::Ended);
}

#[test]
fn sessions_types_session_status_from_str_case_insensitive() {
    let status: SessionStatus = "ACTIVE".parse().expect("must parse uppercase");
    assert_eq!(status, SessionStatus::Active);
    let status2: SessionStatus = "ENDED".parse().expect("must parse uppercase");
    assert_eq!(status2, SessionStatus::Ended);
}

#[test]
fn sessions_types_session_status_from_str_invalid_returns_error() {
    let err = "unknown".parse::<SessionStatus>().unwrap_err();
    assert!(err.to_string().contains("Invalid"));
}

// ── SessionStatus round-trip ──────────────────────────────────────────────────

#[test]
fn sessions_types_session_status_display_from_str_round_trip() {
    for status in [SessionStatus::Active, SessionStatus::Ended] {
        let s = status.to_string();
        let back: SessionStatus = s.parse().expect("round-trip must succeed");
        assert_eq!(back, status, "round-trip failed for {status}");
    }
}

// ── SessionLiveness::from_activity ───────────────────────────────────────────

#[test]
fn sessions_types_liveness_active_when_no_heartbeat() {
    // No last_activity_at → freshly-started session → Active.
    let liveness = SessionLiveness::from_activity(None);
    assert_eq!(liveness, SessionLiveness::Active);
}

#[test]
fn sessions_types_liveness_active_when_recent_heartbeat() {
    // Heartbeat 5 seconds ago → within TTL → Active.
    let recent = Utc::now() - chrono::Duration::seconds(5);
    let liveness = SessionLiveness::from_activity(Some(&recent));
    assert_eq!(liveness, SessionLiveness::Active);
}

#[test]
fn sessions_types_liveness_idle_when_heartbeat_expired() {
    // Heartbeat 11 minutes ago → past the 10-minute TTL → Idle.
    let old = Utc::now() - chrono::Duration::minutes(11);
    let liveness = SessionLiveness::from_activity(Some(&old));
    assert_eq!(liveness, SessionLiveness::Idle);
}

// ── EndSessionRequest defaults ────────────────────────────────────────────────

#[test]
fn sessions_types_end_session_request_default_all_none() {
    let req = EndSessionRequest::default();
    assert!(req.summary.is_none());
    assert!(req.model.is_none());
    assert!(req.files_changed.is_none());
    assert!(req.gate_result.is_none());
    assert!(req.lines_added.is_none());
    assert!(req.lines_removed.is_none());
    assert!(req.tool_calls.is_none());
    assert!(req.exit_reason.is_none());
    assert!(req.updated_workspace_ids.is_empty());
}

// ── Session serialization ─────────────────────────────────────────────────────

#[test]
fn sessions_types_session_serializes_without_none_fields() {
    // Construct a minimal Session to verify serde skip_serializing_if works.
    let session = Session {
        id: "test-id".to_string(),
        workspace_id: "ws".to_string(),
        branch: "main".to_string(),
        status: SessionStatus::Active,
        started_at: Utc::now(),
        ended_at: None,
        agent_id: None,
        primary_harness: None,
        goal: None,
        summary: None,
        updated_workspace_ids: vec![],
        last_activity_at: None,
        liveness: None,
        duration_secs: None,
        model: None,
        files_changed: None,
        lines_added: None,
        lines_removed: None,
        tool_calls: None,
        gate_result: None,
        exit_reason: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&session).expect("serialize must succeed");
    let value: serde_json::Value = serde_json::from_str(&json).expect("must parse back");

    // None fields with skip_serializing_if must be absent from JSON output.
    assert!(value.get("ended_at").is_none(), "ended_at must be absent");
    assert!(value.get("agent_id").is_none(), "agent_id must be absent");
    assert!(value.get("goal").is_none(), "goal must be absent");
    assert!(value.get("summary").is_none(), "summary must be absent");
    assert!(value.get("duration_secs").is_none(), "duration_secs must be absent");
}
