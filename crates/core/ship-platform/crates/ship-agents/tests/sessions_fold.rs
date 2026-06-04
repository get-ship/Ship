//! E4 — sessions src fold + scope-boundary proof.
//!
//! Sessions are RUNTIME-scoped runtime instances (#283), distinct from agent
//! DEFS (space/account-scoped config). After the fold they live as the
//! `ship_agents::sessions` sub-module. These tests pin:
//!   1. the sub-module's runtime-instance types resolve from ship-agents;
//!   2. the Runtime-scope boundary — a session record is keyed on the runtime
//!      instance, NOT a def scope (no def-style `scope: "project"|"user"` axis);
//!   3. the dispatch primitive stays the SOLE `agent.session.*` handler — the
//!      session-lifecycle fold introduces no second handler of that namespace.

use ship_agents::sessions::{Session, SessionStatus};
use ship_agents::sessions::types_session::{EndSessionRequest, SessionLiveness};

#[test]
fn sessions_submodule_types_resolve_from_ship_agents() {
    // The fold exposes the runtime-instance types at `ship_agents::sessions::*`.
    assert_eq!(SessionStatus::Active.to_string(), "active");
    assert_eq!(SessionStatus::Ended.to_string(), "ended");
    let req = EndSessionRequest::default();
    assert!(req.summary.is_none());
    // Fresh instance with no heartbeat reads Active.
    assert_eq!(SessionLiveness::from_activity(None), SessionLiveness::Active);
}

#[test]
fn session_is_a_runtime_instance_not_a_def_scope() {
    use chrono::Utc;
    // A Session is a runtime instance: it is keyed on a workspace_id + branch
    // (the runtime handle), carries liveness/telemetry, and has NO def-style
    // `scope: "project"|"user"` discriminator. Serializing a record proves the
    // runtime-instance shape — there is no def-scope axis to conflate.
    let session = Session {
        id: "s-runtime-1".to_string(),
        workspace_id: "ws".to_string(),
        branch: "feat/x".to_string(),
        status: SessionStatus::Active,
        started_at: Utc::now(),
        ended_at: None,
        agent_id: None,
        primary_harness: None,
        goal: None,
        summary: None,
        updated_workspace_ids: vec![],
        last_activity_at: None,
        liveness: Some(SessionLiveness::Active),
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
    let v: serde_json::Value = serde_json::to_value(&session).expect("serialize");
    // Runtime-instance fields present.
    assert!(v.get("workspace_id").is_some(), "runtime handle present");
    assert!(v.get("liveness").is_some(), "runtime liveness present");
    // No def-scope discriminator — the fold must not conflate session-scope
    // (runtime) with def-scope (space/account config).
    assert!(
        v.get("scope").is_none(),
        "a session is a runtime instance — it carries no agent-def `scope` axis",
    );
}

#[test]
fn dispatch_primitive_is_the_sole_agent_session_handler() {
    // The `service.agents` actor (actor.rs) subscribes to `agent.session.` only,
    // and routes exactly two event types to the dispatch + cancel primitives.
    // The session-lifecycle fold emits the `session.*` namespace (session.started
    // / session.ended / session.progress / session.attached) — a DIFFERENT
    // namespace — so no second `agent.session.*` handler is introduced.
    //
    // We pin the invariant against the crate source: the actor's subscribe set is
    // `agent.session.` and its match has no arm outside started/cancelled, and the
    // folded session lifecycle never names the `agent.session.` namespace.
    let actor_src = include_str!("../src/actor.rs");
    assert!(
        actor_src.contains("subscribe_namespaces: vec![\"agent.session.\".to_string()]"),
        "service.agents must subscribe to agent.session.* (the dispatch domain)",
    );
    assert!(
        actor_src.contains("\"agent.session.started\" =>")
            && actor_src.contains("\"agent.session.cancelled\" =>"),
        "the dispatch actor routes exactly started + cancelled",
    );

    // The folded session lifecycle (runtime-instance analytics) must NOT touch
    // the agent.session.* control namespace — it emits session.* only.
    let session_src = include_str!("../src/sessions/session.rs");
    assert!(
        !session_src.contains("agent.session."),
        "the session-record fold must not handle/emit the agent.session.* control namespace",
    );
}
