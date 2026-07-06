use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::validator::{CallerKind, EmitContext};
use tempfile::tempdir;

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    }
}

pub(super) fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

pub(super) fn studio_config() -> ActorConfig {
    ActorConfig {
        namespace: "studio".into(),
        write_namespaces: vec!["studio.".into()],
        read_namespaces: vec!["studio.".into(), "session.".into()],
        subscribe_namespaces: vec!["studio.".into()],
        see_directed_events: false,
    }
}

pub(super) fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(crate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    (tmp, router)
}

// ── Actor namespace enforcement via kernel.emit ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_emits_rejects_write_outside_namespace() {
    let (_tmp, mut router) = setup();
    let mcp_ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    router.spawn_actor("studio", studio_config()).unwrap();
    let mut ev_session = ev("session.started");
    ev_session.actor_id = Some("studio".into());
    let err = router
        .emit(ev_session, &mcp_ctx)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("not permitted"),
        "unexpected error: {err}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_emits_allows_write_inside_namespace() {
    let (_tmp, mut router) = setup();
    let mcp_ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    router.spawn_actor("studio", studio_config()).unwrap();
    router
        .emit(ev("studio.canvas_updated"), &mcp_ctx)
        .await
        .unwrap();
}

#[test]
fn kernel_query_rejects_read_outside_namespace() {
    let (_tmp, router) = setup();
    let (_mb, _cfg) = ((), ());
    let _ = _mb;
    let _ = _cfg;
    // No actor with that id registered — query should fail cleanly.
    let err = router
        .query_actor_events("missing", &EventFilter::default())
        .unwrap_err();
    assert!(
        err.to_string().contains("not registered"),
        "unexpected error: {err}"
    );
}

#[test]
fn kernel_query_rejects_namespace_violation() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("studio", studio_config()).unwrap();
    let filter = EventFilter {
        event_type: Some("actor.created".into()),
        ..Default::default()
    };
    let err = router
        .query_actor_events("studio", &filter)
        .unwrap_err();
    assert!(
        err.to_string().contains("read namespace violation"),
        "unexpected error: {err}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_query_allows_read_in_permitted_namespace() {
    let (_tmp, mut router) = setup();
    let mcp_ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    router.spawn_actor("studio", studio_config()).unwrap();
    let mut ev_canvas = ev("studio.canvas_updated");
    ev_canvas.actor_id = Some("studio".into());
    router.emit(ev_canvas, &mcp_ctx).await.unwrap();
    // studio. is in read_namespaces
    let filter = EventFilter {
        event_type: Some("studio.canvas_updated".into()),
        ..Default::default()
    };
    let results = router.query_actor_events("studio", &filter).unwrap();
    assert_eq!(results.len(), 1);
    // session. is also in read_namespaces
    let filter = EventFilter {
        event_type: Some("session.started".into()),
        ..Default::default()
    };
    // The event was not emitted, so the result must be empty, but the call
    // must succeed (no namespace violation).
    let results = router.query_actor_events("studio", &filter).unwrap();
    assert!(results.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_query_unfiltered_scopes_to_actor() {
    let (_tmp, mut router) = setup();
    let mcp_ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    router.spawn_actor("studio", studio_config()).unwrap();
    // Stamp the event with the actor's id so it shows up in the actor's view.
    let mut e = ev("studio.ready");
    e.actor_id = Some("studio".into());
    router.emit(e, &mcp_ctx).await.unwrap();
    let results = router
        .query_actor_events("studio", &EventFilter::default())
        .unwrap();
    assert_eq!(results.len(), 1);
}

// ── Multi-actor isolation ─────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actors_cannot_read_each_others_stores() {
    let (_tmp, mut router) = setup();
    let mcp_ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };

    let agent_cfg = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["agent.".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec![],
        see_directed_events: false,
    };

    router.spawn_actor("agent-a", agent_cfg("agent-a")).unwrap();
    router.spawn_actor("agent-b", agent_cfg("agent-b")).unwrap();

    let mut e = ev("agent.task_started");
    e.actor_id = Some("agent-a".into());
    router.emit(e, &mcp_ctx).await.unwrap();

    // Actor B's query for events scoped to its own actor_id is empty.
    let results = router
        .query_actor_events("agent-b", &EventFilter::default())
        .unwrap();
    assert!(results.is_empty(), "agent B must not see agent A's events");

    // Actor A can see its own event.
    let results = router
        .query_actor_events("agent-a", &EventFilter::default())
        .unwrap();
    assert_eq!(results.len(), 1);
}

// ── KernelRouter routing ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_delivers_to_subscribed_mailbox() {
    let (_tmp, mut router) = setup();
    let mut mb = router.spawn_actor("studio", studio_config()).unwrap();

    let event = ev("studio.canvas_updated");
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb.try_recv().expect("mailbox should have an event");
    assert_eq!(received.id, event.id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_skips_unsubscribed_mailbox() {
    let (_tmp, mut router) = setup();
    let agent_cfg = ActorConfig {
        namespace: "agent".into(),
        write_namespaces: vec!["agent.".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec!["agent.".into()],
        see_directed_events: false,
    };
    let mut mb = router.spawn_actor("agent-1", agent_cfg).unwrap();

    // studio event — agent-1 is not subscribed to "studio."
    router
        .emit(ev("studio.canvas_updated"), &runtime_ctx())
        .await
        .unwrap();

    assert!(
        mb.try_recv().is_err(),
        "unsubscribed actor must not receive the event"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_delivers_to_multiple_subscribers() {
    let (_tmp, mut router) = setup();

    let sub_cfg = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec![format!("{id}.")],
        read_namespaces: vec![format!("{id}."), "studio.".into()],
        subscribe_namespaces: vec!["studio.".into()],
        see_directed_events: false,
    };

    let mut mb1 = router.spawn_actor("watcher-1", sub_cfg("watcher-1")).unwrap();
    let mut mb2 = router.spawn_actor("watcher-2", sub_cfg("watcher-2")).unwrap();

    let event = ev("studio.canvas_updated");
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    assert_eq!(mb1.try_recv().unwrap().id, event.id);
    assert_eq!(mb2.try_recv().unwrap().id, event.id);
}

// ── Kernel store ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_store_receives_system_events() {
    let (_tmp, router) = setup();
    let event = EventEnvelope::new(
        "actor.created",
        "entity-1",
        &serde_json::json!({ "kind": "test", "environment_type": "local" }),
    )
    .unwrap();
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    // Verify via raw SQL — kernel store has the event.
    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE id = ?",
        )
        .bind(&event.id)
        .fetch_one(&mut conn)
        .await
    })
    .unwrap();
    assert_eq!(count, 1, "system event must be in kernel store");
}

// `kernel_store_does_not_receive_actor_events` was retired along with
// `KernelRouter::route()` — it validated the now-gone delivery-only
// path where actor events bypassed the kernel store. `emit()` is the
// single emit path and DOES persist; actor-vs-kernel-store partitioning
// is enforced by the persistence layer itself, not by the router.

// ── Actor lifecycle ───────────────────────────────────────────────────────────

#[test]
fn stop_actor_removes_mailbox_and_subscriptions() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("studio", studio_config()).unwrap();
    assert_eq!(router.actor_count(), 1);

    router.stop_actor("studio").unwrap();
    assert_eq!(router.actor_count(), 0);
}

#[test]
fn stop_actor_errors_on_unknown_actor() {
    let (_tmp, mut router) = setup();
    assert!(router.stop_actor("ghost").is_err());
}

#[test]
fn spawn_actor_rejects_duplicate_id() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("studio", studio_config()).unwrap();
    assert!(router.spawn_actor("studio", studio_config()).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refine_actor_preserves_mailbox_through_wrapper() {
    let (_tmp, mut router) = setup();
    let mut mb = router.spawn_actor("studio", studio_config()).unwrap();

    // Refine to a different namespace; the wrapper must keep the sender
    // intact so the original Mailbox handle stays receivable.
    let new_cfg = ActorConfig {
        namespace: "studio".into(),
        write_namespaces: vec!["other.".into()],
        read_namespaces: vec!["other.".into()],
        subscribe_namespaces: vec!["other.".into()],
        see_directed_events: false,
    };
    router.refine_actor("studio", new_cfg).unwrap();

    let mut event = ev("other.hello");
    event.target_actor_id = Some("studio".into());
    router.emit(event, &runtime_ctx()).await.unwrap();
    let recv = mb.try_recv().expect("mailbox should receive directed event");
    assert_eq!(recv.event_type, "other.hello");
}

#[test]
fn refine_actor_errors_on_unknown_through_wrapper() {
    let (_tmp, mut router) = setup();
    let err = router.refine_actor("ghost", studio_config()).unwrap_err();
    assert!(matches!(
        err,
        crate::errors::KernelError::ActorNotFound { .. }
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_actor_mailbox_is_closed() {
    let (_tmp, mut router) = setup();
    let mut mb = router.spawn_actor("studio", studio_config()).unwrap();

    router.stop_actor("studio").unwrap();

    // Sender was dropped — recv returns None.
    assert!(mb.recv().await.is_none());
}
