//! Pure-fold tests for the agent-session projection, plus a SQL-backed
//! slug-uniqueness regression.

use super::{project, AgentSession, AgentSessionProjection, AgentSessionStatus, status_for};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::agent_session::{event_types, InvocationMode};
use ship_substrate::projections::registry::Projection;

fn started(session_id: &str, slug: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::AGENT_SESSION_STARTED,
        session_id,
        &serde_json::json!({
            "session_id": session_id,
            "slug": slug,
            "agent": "rust-lane",
            "workspace": "auth-feature",
            "branch": format!("job/{slug}"),
            "open_tab": true,
            "invocation": "pty",
            "keepalive": true,
        }),
    )
    .unwrap()
}

fn dispatched(session_id: &str, slug: &str, pid: u32) -> EventEnvelope {
    EventEnvelope::new(
        event_types::AGENT_SESSION_DISPATCHED,
        session_id,
        &serde_json::json!({
            "session_id": session_id,
            "slug": slug,
            "worktree": "/host/path/wt",
            "harness": "claude",
            "pid": pid,
        }),
    )
    .unwrap()
}

fn completed(session_id: &str, slug: &str, code: i32) -> EventEnvelope {
    EventEnvelope::new(
        event_types::AGENT_SESSION_COMPLETED,
        session_id,
        &serde_json::json!({
            "session_id": session_id,
            "slug": slug,
            "exit_code": code,
        }),
    )
    .unwrap()
}

fn failed(session_id: &str, slug: &str, err: &str, stage: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::AGENT_SESSION_FAILED,
        session_id,
        &serde_json::json!({
            "session_id": session_id,
            "slug": slug,
            "error": err,
            "stage": stage,
        }),
    )
    .unwrap()
}

#[test]
fn started_creates_pending_record() {
    let map = project(&[started("01HV-S1", "auth-001")]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Pending);
    assert_eq!(rec.slug, "auth-001");
    assert_eq!(rec.invocation, InvocationMode::Pty);
    assert!(rec.open_tab);
    assert!(rec.keepalive);
    assert!(rec.worktree.is_none(), "worktree resolves at dispatch time");
    assert!(rec.pid.is_none());
}

#[test]
fn dispatched_flips_status_and_fills_worktree_pid() {
    let map = project(&[
        started("01HV-S1", "auth-001"),
        dispatched("01HV-S1", "auth-001", 12345),
    ]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Dispatched);
    assert_eq!(rec.worktree.as_deref(), Some("/host/path/wt"));
    assert_eq!(rec.pid, Some(12345));
    // Dispatched payload also resolves the harness default chain.
    assert_eq!(rec.harness.as_deref(), Some("claude"));
}

#[test]
fn completed_flips_status_and_sets_completed_at() {
    let map = project(&[
        started("01HV-S1", "auth-001"),
        dispatched("01HV-S1", "auth-001", 12345),
        completed("01HV-S1", "auth-001", 0),
    ]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Completed);
    assert!(rec.completed_at.is_some());
}

#[test]
fn failed_carries_error_message() {
    let map = project(&[
        started("01HV-S1", "auth-001"),
        failed("01HV-S1", "auth-001", "claude binary not on PATH", "spawn_failed"),
    ]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Failed);
    assert_eq!(rec.error.as_deref(), Some("claude binary not on PATH"));
    assert!(rec.completed_at.is_some());
}

#[test]
fn cancelled_carries_reason() {
    let cancel = EventEnvelope::new(
        event_types::AGENT_SESSION_CANCELLED,
        "01HV-S1",
        &serde_json::json!({
            "session_id": "01HV-S1",
            "slug": "auth-001",
            "reason": "operator stop",
        }),
    )
    .unwrap();
    let map = project(&[started("01HV-S1", "auth-001"), cancel]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Cancelled);
    assert_eq!(rec.error.as_deref(), Some("operator stop"));
}

#[test]
fn turn_ended_only_keepalive_path() {
    let turn_ended = EventEnvelope::new(
        event_types::AGENT_SESSION_TURN_ENDED,
        "01HV-S1",
        &serde_json::json!({
            "session_id": "01HV-S1",
            "slug": "auth-001",
            "turn_idx": 1,
            "stop_reason": "end_turn",
        }),
    )
    .unwrap();
    let map = project(&[
        started("01HV-S1", "auth-001"),
        dispatched("01HV-S1", "auth-001", 12345),
        turn_ended,
    ]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::AwaitingPush);
    assert!(!rec.status.is_terminal());
}

#[test]
fn parent_pipeline_run_id_preserved_for_composition() {
    // Pipeline-composed sessions carry parent_pipeline_run_id so the
    // composer can filter completion events by parent. Loss of this
    // field would silently break advance_pipeline.
    let started_with_parent = EventEnvelope::new(
        event_types::AGENT_SESSION_STARTED,
        "01HV-STEP-1",
        &serde_json::json!({
            "session_id": "01HV-STEP-1",
            "slug": "step-1",
            "agent": "rust-lane",
            "workspace": "ws",
            "branch": "job/step-1",
            "open_tab": false,
            "invocation": "pty",
            "keepalive": false,
            "parent_pipeline_run_id": "01HV-PIPELINE",
        }),
    )
    .unwrap();
    let map = project(&[started_with_parent]);
    let rec = map.get("01HV-STEP-1").unwrap();
    assert_eq!(rec.parent_pipeline_run_id.as_deref(), Some("01HV-PIPELINE"));
}

#[test]
fn unknown_event_ignored() {
    // Forward-compat: agent.session.* events the projection doesn't
    // know about yet shouldn't corrupt state.
    let unknown = EventEnvelope::new(
        "agent.session.future_event",
        "01HV-S1",
        &serde_json::json!({"session_id": "01HV-S1"}),
    )
    .unwrap();
    let map = project(&[started("01HV-S1", "auth-001"), unknown]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Pending);
}

#[test]
fn non_agent_session_events_skipped() {
    // The project() fold filters to agent.session.* events. A
    // pipeline.* event in the same stream must not affect any
    // agent_session record.
    let pipeline_event = EventEnvelope::new(
        "pipeline.run.started",
        "j1",
        &serde_json::json!({"pipeline_run_id": "j1", "slug": "auth-001"}),
    )
    .unwrap();
    let map = project(&[started("01HV-S1", "auth-001"), pipeline_event]);
    let rec = map.get("01HV-S1").unwrap();
    assert_eq!(rec.status, AgentSessionStatus::Pending);
}

// ── status_for transitions ──────────────────────────────────────────

#[test]
fn status_for_dispatched_is_dispatched() {
    assert_eq!(
        status_for(event_types::AGENT_SESSION_DISPATCHED),
        Some(AgentSessionStatus::Dispatched)
    );
}

#[test]
fn status_for_completed_is_completed() {
    assert_eq!(
        status_for(event_types::AGENT_SESSION_COMPLETED),
        Some(AgentSessionStatus::Completed)
    );
}

#[test]
fn status_for_failed_is_failed() {
    assert_eq!(
        status_for(event_types::AGENT_SESSION_FAILED),
        Some(AgentSessionStatus::Failed)
    );
}

#[test]
fn status_for_cancelled_is_cancelled() {
    assert_eq!(
        status_for(event_types::AGENT_SESSION_CANCELLED),
        Some(AgentSessionStatus::Cancelled)
    );
}

#[test]
fn status_for_turn_ended_is_awaiting_push() {
    assert_eq!(
        status_for(event_types::AGENT_SESSION_TURN_ENDED),
        Some(AgentSessionStatus::AwaitingPush)
    );
}

#[test]
fn status_for_started_is_none() {
    // STARTED creates the record; transition is implicit (Pending),
    // not in the table.
    assert_eq!(status_for(event_types::AGENT_SESSION_STARTED), None);
}

#[test]
fn status_for_unknown_is_none() {
    assert_eq!(status_for("agent.session.nope"), None);
    assert_eq!(status_for(""), None);
}

#[test]
fn terminal_states() {
    assert!(AgentSessionStatus::Completed.is_terminal());
    assert!(AgentSessionStatus::Failed.is_terminal());
    assert!(AgentSessionStatus::Cancelled.is_terminal());
    assert!(!AgentSessionStatus::Pending.is_terminal());
    assert!(!AgentSessionStatus::Dispatched.is_terminal());
    assert!(!AgentSessionStatus::AwaitingPush.is_terminal());
}

// ── SQL-backed: slug-uniqueness regression (Phase 5b) ───────────────
//
// agent_sessions.slug is UNIQUE. A multi-step pipeline composes N
// sessions that all share the SAME job slug. Phase 5's first cut
// emitted each step's agent.session.started with the bare job slug —
// the second step would violate the UNIQUE constraint inside the
// event-append transaction, potentially rolling back the event write.
// The fix derives a per-step slug (`<run_slug>::<step_id>`). These
// tests pin both halves: same-slug collides (proving the constraint
// is real), distinct-slug both land (proving the fix works).

fn started_full(session_id: &str, slug: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::AGENT_SESSION_STARTED,
        session_id,
        &serde_json::json!({
            "session_id": session_id,
            "slug": slug,
            "agent": "rust-lane",
            "workspace": "ws",
            "branch": "job/ws",
            "open_tab": false,
            "invocation": "pty",
            "keepalive": false,
            "parent_pipeline_run_id": "01HV-PIPELINE",
        }),
    )
    .unwrap()
}

fn fresh_conn() -> (tempfile::TempDir, sqlx::SqliteConnection) {
    let tmp = tempfile::tempdir().unwrap();
    ship_substrate::db::ensure_db_at(tmp.path()).unwrap();
    let conn =
        ship_substrate::db::open_db_at(&ship_substrate::db::platform_db_path(tmp.path())).unwrap();
    (tmp, conn)
}

#[test]
fn sql_two_sessions_same_slug_collide() {
    let (_tmp, mut conn) = fresh_conn();
    let proj = AgentSessionProjection;
    // First step's session — succeeds.
    proj.apply(&started_full("01HV-S1", "ws-job"), &mut conn)
        .expect("first row inserts");
    // Second step reuses the bare job slug — UNIQUE(slug) rejects it.
    let err = proj
        .apply(&started_full("01HV-S2", "ws-job"), &mut conn)
        .expect_err("second row with duplicate slug must violate UNIQUE(slug)");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("unique") || msg.contains("constraint"),
        "expected a UNIQUE-constraint error, got: {err}"
    );
}

#[test]
fn sql_two_sessions_distinct_slug_both_land() {
    let (tmp, mut conn) = fresh_conn();
    let proj = AgentSessionProjection;
    // The fix: each composed step gets `<run_slug>::<step_id>`.
    proj.apply(&started_full("01HV-S1", "ws-job::implement"), &mut conn)
        .expect("step 1 inserts");
    proj.apply(&started_full("01HV-S2", "ws-job::review"), &mut conn)
        .expect("step 2 inserts — distinct slug, no collision");

    let sessions: std::collections::HashMap<String, AgentSession> =
        super::load_sessions_at(tmp.path()).unwrap();
    assert_eq!(sessions.len(), 2, "both composed-step sessions persisted");
    assert!(sessions.contains_key("01HV-S1"));
    assert!(sessions.contains_key("01HV-S2"));
}
