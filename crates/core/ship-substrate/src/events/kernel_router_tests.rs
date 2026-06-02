use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::validator::{CallerKind, EmitContext};
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::event_registry::EventScope;
use ship_types::identity::{AccountId, DeviceId, SpaceId};
use ship_types::{Authority, ScopeBinding};
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

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

fn studio_config() -> ActorConfig {
    ActorConfig {
        namespace: "studio".into(),
        write_namespaces: vec!["studio.".into()],
        read_namespaces: vec!["studio.".into(), "session.".into()],
        subscribe_namespaces: vec!["studio.".into()],
        see_directed_events: false,
    }
}

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(crate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    (tmp, router)
}

// ── ActorStore namespace enforcement ─────────────────────────────────────────

#[test]
fn actor_store_rejects_write_outside_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    let err = store.append(&ev("session.started")).unwrap_err();
    assert!(
        err.to_string().contains("write namespace violation"),
        "unexpected error: {err}"
    );
}

#[test]
fn actor_store_allows_write_inside_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    store.append(&ev("studio.canvas_updated")).unwrap();
}

#[test]
fn actor_store_rejects_read_outside_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    let filter = EventFilter {
        event_type: Some("workspace.created".into()),
        ..Default::default()
    };
    let err = store.query(&filter).unwrap_err();
    assert!(
        err.to_string().contains("read namespace violation"),
        "unexpected error: {err}"
    );
}

#[test]
fn actor_store_allows_read_in_permitted_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    // studio. is in read_namespaces
    let filter = EventFilter {
        event_type: Some("studio.canvas_updated".into()),
        ..Default::default()
    };
    assert!(store.query(&filter).is_ok());
    // session. is also in read_namespaces
    let filter = EventFilter {
        event_type: Some("session.started".into()),
        ..Default::default()
    };
    assert!(store.query(&filter).is_ok());
}

#[test]
fn actor_store_unfiltered_query_allowed() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    store.append(&ev("studio.ready")).unwrap();
    let results = store.query(&EventFilter::default()).unwrap();
    assert_eq!(results.len(), 1);
}

// ── Multi-actor isolation ─────────────────────────────────────────────────────

#[test]
fn actors_cannot_read_each_others_stores() {
    let (_tmp, mut router) = setup();

    let agent_cfg = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["agent.".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec![],
        see_directed_events: false,
    };

    let (store_a, _mb_a) = router.spawn_actor("agent-a", agent_cfg("agent-a")).unwrap();
    let (store_b, _mb_b) = router.spawn_actor("agent-b", agent_cfg("agent-b")).unwrap();

    store_a.append(&ev("agent.task_started")).unwrap();

    // Actor B's store is empty — it never saw A's write.
    let results = store_b.query(&EventFilter::default()).unwrap();
    assert!(results.is_empty(), "actor B must not see actor A's events");

    // Actor A can see its own event.
    let results = store_a.query(&EventFilter::default()).unwrap();
    assert_eq!(results.len(), 1);
}

// ── KernelRouter routing ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_delivers_to_subscribed_mailbox() {
    let (_tmp, mut router) = setup();
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

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
    let (_store, mut mb) = router.spawn_actor("agent-1", agent_cfg).unwrap();

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

    let (_s1, mut mb1) = router.spawn_actor("watcher-1", sub_cfg("watcher-1")).unwrap();
    let (_s2, mut mb2) = router.spawn_actor("watcher-2", sub_cfg("watcher-2")).unwrap();

    let event = ev("studio.canvas_updated");
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    assert_eq!(mb1.try_recv().unwrap().id, event.id);
    assert_eq!(mb2.try_recv().unwrap().id, event.id);
}

// ── Kernel store ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_store_receives_system_events() {
    let (_tmp, router) = setup();
    let event = ev("session.started");
    // emit persists + delivers. route is delivery-only.
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    // Verify via raw SQL — kernel store has the event.
    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
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
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

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
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

    router.stop_actor("studio").unwrap();

    // Sender was dropped — recv returns None.
    assert!(mb.recv().await.is_none());
}

// ── D-enforce: emit chokepoint stamps + rejects from the binding ───────────────
//
// These go THROUGH `KernelRouter::emit(...).await` (not the pure `enforce_scope`
// fn) and assert the PERSISTED envelope, proving the wiring — not just the rule.

/// A session binding bound to `space`, carrying device + account identity.
fn space_binding(space: &str) -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        Some(SpaceId::new(space)),
        Authority::Space {
            space_id: SpaceId::new(space),
        },
    )
}

/// A space-unbound session binding (device + account, no space).
fn spaceless_binding() -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        None,
        Authority::Sdk,
    )
}

fn ctx_with(binding: ScopeBinding) -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding,
        session_id: None,
    }
}

/// Read the persisted envelope back from platform.db at the router's base dir.
fn read_back(tmp: &tempfile::TempDir, id: &str) -> Option<EventEnvelope> {
    let store = SqliteEventStore::new_at(&crate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    store.get(id).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_stamps_space_key_and_author_from_binding() {
    let (tmp, router) = setup();
    // Space event with NO scope_key, emitted under a space-bound binding.
    let event = ev("studio.canvas_updated").with_scope(EventScope::Space, None);
    let id = event.id.clone();
    router
        .emit(event, &ctx_with(space_binding("space-A")))
        .await
        .unwrap();

    let persisted = read_back(&tmp, &id).expect("event must be persisted");
    assert_eq!(persisted.scope, EventScope::Space);
    assert_eq!(persisted.scope_key.as_deref(), Some("space-A"));
    assert_eq!(persisted.author_device_id.as_deref(), Some("dev-1"));
    assert_eq!(persisted.author_account_id.as_deref(), Some("acct-1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_rejects_cross_space_and_does_not_persist() {
    let (tmp, mut router) = setup();
    // Subscribe an actor so we can also assert non-delivery.
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

    // Space event keyed to space-B, but binding is space-A → CrossSpace reject.
    let event = ev("studio.canvas_updated").with_scope(EventScope::Space, Some("space-B"));
    let id = event.id.clone();
    let err = router
        .emit(event, &ctx_with(space_binding("space-A")))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("cross-space"),
        "unexpected error: {err}"
    );

    assert!(read_back(&tmp, &id).is_none(), "rejected event must not persist");
    assert!(mb.try_recv().is_err(), "rejected event must not be delivered");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_rejects_space_event_from_space_unbound_binding() {
    let (tmp, router) = setup();
    let event = ev("studio.canvas_updated").with_scope(EventScope::Space, None);
    let id = event.id.clone();
    let err = router
        .emit(event, &ctx_with(spaceless_binding()))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("space-unbound"),
        "unexpected error: {err}"
    );
    assert!(read_back(&tmp, &id).is_none(), "rejected event must not persist");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_stamps_non_space_keys_without_cross_check() {
    let (tmp, router) = setup();
    let binding = space_binding("space-A");

    // Account → account key.
    let acct = ev("studio.canvas_updated").with_scope(EventScope::Account, None);
    let acct_id = acct.id.clone();
    router.emit(acct, &ctx_with(binding.clone())).await.unwrap();
    let p = read_back(&tmp, &acct_id).unwrap();
    assert_eq!(p.scope_key.as_deref(), Some("acct-1"));

    // Device → device key.
    let dev = ev("studio.canvas_updated").with_scope(EventScope::Device, None);
    let dev_id = dev.id.clone();
    router.emit(dev, &ctx_with(binding.clone())).await.unwrap();
    let p = read_back(&tmp, &dev_id).unwrap();
    assert_eq!(p.scope_key.as_deref(), Some("dev-1"));

    // System → None, no cross-check.
    let sys = ev("studio.canvas_updated").with_scope(EventScope::System, None);
    let sys_id = sys.id.clone();
    router.emit(sys, &ctx_with(binding)).await.unwrap();
    let p = read_back(&tmp, &sys_id).unwrap();
    assert_eq!(p.scope_key, None);
    assert_eq!(p.author_device_id.as_deref(), Some("dev-1"));
}
