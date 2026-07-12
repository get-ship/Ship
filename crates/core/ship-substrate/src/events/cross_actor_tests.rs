//! Cross-actor routing integration tests.
//!
//! Verifies the actor isolation guarantees from the job spec:
//! - Studio events reach the agent's mailbox (studio → agent routing)
//! - Agent events reach Studio's mailbox (agent → studio routing)
//! - Two agents cannot read each other's stores
//! - studio.message.visual emitted by Studio reaches an agent subscribed to "studio."

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
        binding: ship_types::ScopeBinding::daemon_internal(Some(ship_types::SpaceId::new("feature/actor-test".to_string())), ship_types::Authority::Sdk),
        session_id: None,
    }
}

fn mcp_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(Some(ship_types::SpaceId::new("feature/actor-test".to_string())), ship_types::Authority::Sdk),
        session_id: None,
    }
}

fn agent_config(actor_id: &str) -> ActorConfig {
    ActorConfig {
        namespace: actor_id.to_string(),
        write_namespaces: vec!["".to_string()], // any non-system prefix (enforced by tool)
        read_namespaces: vec!["agent.".to_string()],
        subscribe_namespaces: vec![
            "studio.".to_string(),
            "session.".to_string(),
            "actor.".to_string(),
        ],
        see_directed_events: false,
    }
}

fn studio_config() -> ActorConfig {
    ActorConfig {
        namespace: "studio".to_string(),
        write_namespaces: vec!["studio.".to_string()],
        read_namespaces: vec!["studio.".to_string()],
        subscribe_namespaces: vec!["studio.".to_string(), "agent.".to_string()],
        see_directed_events: false,
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

// ── studio → agent routing ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn studio_visual_message_reaches_agent_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _studio_mb = router.spawn_actor("studio", studio_config()).unwrap();
    let mut agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Studio emits a visual message.
    let event = ev("studio.message.visual");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = agent_mb.try_recv().expect("agent mailbox must receive studio event");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "studio.message.visual");
}

// ── agent → studio routing ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_event_reaches_studio_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let mut studio_mb = router.spawn_actor("studio", studio_config()).unwrap();
    let _agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Agent emits an event in its own namespace (simulating the event tool).
    // write_namespaces is [""] so any event_type is accepted.
    let event = ev("agent.task_started");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = studio_mb.try_recv().expect("studio mailbox must receive agent event");
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
async fn agent_cannot_read_studio_events() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _studio_mb = router.spawn_actor("studio", studio_config()).unwrap();
    let _agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Studio emits an event.
    let mut event = ev("studio.message.visual");
    event.actor_id = Some("studio".into());
    router.emit(event, &mcp_ctx()).await.unwrap();

    // Agent tries to query studio events — read_namespaces = ["agent."]
    let filter = EventFilter {
        event_type: Some("studio.message.visual".into()),
        ..Default::default()
    };
    let err = router
        .query_actor_events("agent.mcp", &filter)
        .unwrap_err();
    assert!(
        err.to_string().contains("read namespace violation"),
        "agent must not be able to query studio events: {err}"
    );
}


// ── artifact event routing ───────────────────────────────────────────────────

/// Agent emits `canvas.artifact_created` → Studio's mailbox receives it.
///
/// Studio subscribes to `canvas.` (the skill's custom namespace) so it can
/// react when the agent produces a new artifact. This verifies the dynamic
/// subscription path added in the split-brain fix.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_skill_event_reaches_studio_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    // Studio subscribes to its own namespace + agent.* + the canvas skill namespace.
    let studio_cfg = ActorConfig {
        namespace: "studio".to_string(),
        write_namespaces: vec!["studio.".to_string()],
        read_namespaces: vec!["studio.".to_string()],
        subscribe_namespaces: vec![
            "studio.".to_string(),
            "agent.".to_string(),
            "canvas.".to_string(),
        ],
        see_directed_events: false,
    };
    let mut studio_mb = router.spawn_actor("studio", studio_cfg).unwrap();
    let _agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    // Agent emits canvas.artifact_created (prefixed with skill id at emit time).
    let event = ev("canvas.artifact_created");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = studio_mb
        .try_recv()
        .expect("studio mailbox must receive canvas.artifact_created");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "canvas.artifact_created");
}

/// Studio emits `studio.feedback` → Agent's mailbox receives it.
///
/// Agents subscribe to `studio.*` in their base subscription, so any Studio
/// UI action (feedback, selection) reaches the agent without needing
/// a per-artifact-type subscription entry.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn studio_feedback_reaches_agent_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let _studio_mb = router.spawn_actor("studio", studio_config()).unwrap();
    let mut agent_mb = router
        .spawn_actor("agent.mcp", agent_config("agent.mcp"))
        .unwrap();

    let event = ev("studio.feedback");
    router.emit(event.clone(), &mcp_ctx()).await.unwrap();

    let received = agent_mb
        .try_recv()
        .expect("agent mailbox must receive studio.feedback");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, "studio.feedback");
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
