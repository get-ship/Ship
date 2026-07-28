//! Cross-actor routing integration tests.
//!
//! Verifies the actor isolation guarantees from the job spec:
//! - Observer events reach the agent's mailbox (observer → agent routing)
//! - Agent events reach Observer's mailbox (agent → observer routing)
//! - Two agents cannot read each other's stores
//! - observer.message.visual emitted by Observer reaches an agent subscribed to "observer."

use tempfile::tempdir;

use ship_types::envelope::EventEnvelope;
use crate::db::{block_on, open_db_at};
use crate::events::filter::EventFilter;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::validator::{CallerKind, EmitContext};
fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(Some(ship_types::SpaceId::new("feature/actor-test".to_string())), agent_schema::Authority::Sdk),
        session_id: None,
    }
}

fn mcp_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(Some(ship_types::SpaceId::new("feature/actor-test".to_string())), agent_schema::Authority::Sdk),
        session_id: None,
    }
}

fn agent_config(actor_id: &str) -> ActorConfig {
    ActorConfig {
        namespace: actor_id.to_string(),
        write_namespaces: vec!["".to_string()], // any non-system prefix (enforced by tool)
        read_namespaces: vec!["agent.".to_string()],
        subscribe_namespaces: vec![
            "observer.".to_string(),
            "session.".to_string(),
            "actor.".to_string(),
        ],
        see_directed_events: false,
    }
}

fn observer_config() -> ActorConfig {
    ActorConfig {
        namespace: "observer".to_string(),
        write_namespaces: vec!["observer.".to_string()],
        read_namespaces: vec!["observer.".to_string()],
        subscribe_namespaces: vec!["observer.".to_string(), "agent.".to_string()],
        see_directed_events: false,
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

// ── observer → agent routing ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn observer_visual_message_reaches_agent_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _observer_mb = router.spawn_actor("observer", observer_config()).unwrap();
    let mut agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Observer emits a visual message.
    let event = ev("observer.message.visual");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = agent_mb.try_recv().expect("agent mailbox must receive observer event");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "observer.message.visual");
}

// ── agent → observer routing ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_event_reaches_observer_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let mut observer_mb = router.spawn_actor("observer", observer_config()).unwrap();
    let _agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Agent emits an event in its own namespace (simulating the event tool).
    // write_namespaces is [""] so any event_type is accepted.
    let event = ev("agent.task_started");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = observer_mb.try_recv().expect("observer mailbox must receive agent event");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "agent.task_started");
}

// ── store isolation (mediated by KernelRouter) ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_agents_cannot_read_each_others_stores() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _mb_a = router
        .spawn_actor("agent.alice", agent_config("agent.alice"))
        .unwrap();
    let _mb_b = router
        .spawn_actor("agent.bob", agent_config("agent.bob"))
        .unwrap();

    // Alice emits an event with her actor_id stamped.
    let mut event = ev("agent.task_a");
    event.actor_id = Some("agent.alice".into());
    router.emit(event, &mcp_ctx()).await.unwrap();

    // Bob's query for events scoped to his own actor_id is empty —
    // query_actor_events forces actor_id = "agent.bob" in the filter.
    let results = router
        .query_actor_events("agent.bob", &EventFilter::default())
        .unwrap();
    assert!(results.is_empty(), "agent B must not see agent A's events");

    // Alice can see her own event.
    let results = router
        .query_actor_events("agent.alice", &EventFilter::default())
        .unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_cannot_read_observer_events() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _observer_mb = router.spawn_actor("observer", observer_config()).unwrap();
    let _agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Observer emits an event.
    let mut event = ev("observer.message.visual");
    event.actor_id = Some("observer".into());
    router.emit(event, &mcp_ctx()).await.unwrap();

    // Agent tries to query observer events — read_namespaces = ["agent."]
    let filter = EventFilter {
        event_type: Some("observer.message.visual".into()),
        ..Default::default()
    };
    let err = router
        .query_actor_events("agent.mcp", &filter)
        .unwrap_err();
    assert!(
        err.to_string().contains("read namespace violation"),
        "agent must not be able to query observer events: {err}"
    );
}


// ── artifact event routing ───────────────────────────────────────────────────

/// Agent emits `canvas.artifact_created` → Observer's mailbox receives it.
///
/// Observer subscribes to `canvas.` (the skill's custom namespace) so it can
/// react when the agent produces a new artifact. This verifies the dynamic
/// subscription path added in the split-brain fix.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_skill_event_reaches_observer_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    // Observer subscribes to its own namespace + agent.* + the canvas skill namespace.
    let observer_cfg = ActorConfig {
        namespace: "observer".to_string(),
        write_namespaces: vec!["observer.".to_string()],
        read_namespaces: vec!["observer.".to_string()],
        subscribe_namespaces: vec![
            "observer.".to_string(),
            "agent.".to_string(),
            "canvas.".to_string(),
        ],
        see_directed_events: false,
    };
    let mut observer_mb = router.spawn_actor("observer", observer_cfg).unwrap();
    let _agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Agent emits canvas.artifact_created (prefixed with skill id at emit time).
    let event = ev("canvas.artifact_created");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = observer_mb
        .try_recv()
        .expect("observer mailbox must receive canvas.artifact_created");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "canvas.artifact_created");
}

/// A neutral observer event reaches the agent mailbox through its explicit
/// `observer.` subscription.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn observer_feedback_reaches_agent_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _observer_mb = router.spawn_actor("observer", observer_config()).unwrap();
    let mut agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    let event = ev("observer.feedback");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = agent_mb
        .try_recv()
        .expect("agent mailbox must receive observer.feedback");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "observer.feedback");
}

// ── kernel audit trail ───────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_store_records_system_events_for_audit() {

    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let event = EventEnvelope::new(
        "actor.created",
        "entity-1",
        &serde_json::json!({ "kind": "test", "environment_type": "local" }),
    )
    .unwrap();
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
            .bind(&event.id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(count, 1, "kernel store must record system events for audit");
}
