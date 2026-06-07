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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_actor_evicts_rate_counter() {
    // #256: `emit` inserts a `RateCounter` per distinct originator/target id;
    // `stop_actor` must evict it so heavy fanout (short-lived ids) doesn't
    // strand counters forever. Mirrors the `actor_meta.remove` discipline.
    let (_tmp, mut router) = setup();
    let (_store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();

    // Emit an event AUTHORED by `studio` so its rate counter is bumped.
    let event = ev("studio.canvas_updated").with_actor_id("studio");
    router.emit(event, &runtime_ctx()).await.unwrap();
    assert!(router.msg_rate("studio") > 0, "emit must bump the originator's counter");

    router.stop_actor("studio").unwrap();
    assert_eq!(
        router.msg_rate("studio"),
        0,
        "stop_actor must evict the per-actor rate counter"
    );
}

// #256 (the regression the rate_counter batch MISSED): `stop_actor` (and
// `handle_actor_crash`) used to UNCONDITIONALLY `actor_cursors.insert(id, ..)`.
// The ONLY removal was on re-spawn of the identical id via `spawn_actor`.
// SSE-source actors (`sui-sse-{name}-{ulid}`) and the git-signal updater
// (`git-signal-updater-{ulid}`) are spawned via the legacy `spawn_actor`
// path with single-use ULID ids that are stopped on disconnect and NEVER
// re-spawned — so every disconnect stranded one cursor permanently
// (unbounded in connection churn). Cursors are only ever READ on re-spawn
// in `spawn_actor_with_permissions_inner` (kernel_security.rs), so only
// permissioned/restartable actors can meaningfully replay; legacy actors
// strand a cursor that is never consumed. The fix mirrors the sibling
// `actor_meta.remove`/`rate_counters.remove` discipline: retain a cursor
// only for actors that carry `actor_meta` (the permissioned/replay path).
#[cfg(feature = "unstable")]
#[test]
fn stop_actor_does_not_strand_cursor_for_single_use_actor() {
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use ship_types::identity::{ActorIdentity, ActorType};

    let (_tmp, mut router) = setup();

    // --- Single-use legacy actor (the SSE/git-signal shape): unique id,
    //     spawned via `spawn_actor`, stopped, never re-spawned. ---
    let single_use_id = "sui-sse-git-01J0000000000000000000SSEX";
    router
        .spawn_actor(single_use_id, studio_config())
        .unwrap();
    router.stop_actor(single_use_id).unwrap();
    assert!(
        !router.actor_cursors.contains_key(single_use_id),
        "stop_actor must NOT strand a cursor for a single-use (non-permissioned) actor"
    );

    // --- Restartable/permissioned actor: cursor MUST be retained so a
    //     re-spawn through spawn_actor_with_permissions can replay events
    //     emitted while it was down. ---
    let restartable_id = "agent-restartable";
    let identity = ActorIdentity {
        instance_id: restartable_id.to_string(),
        label: restartable_id.to_string(),
        actor_type: ActorType::Agent,
    };
    let permissions = ActorPermissions {
        emit: vec!["agent.".into()],
        subscribe: vec![PermittedSubscription {
            namespace: "agent.".into(),
            scope: DeliveryScope::Global,
        }],
    };
    router
        .spawn_actor_with_permissions(identity, permissions)
        .unwrap();
    router.stop_actor(restartable_id).unwrap();
    assert!(
        router.actor_cursors.contains_key(restartable_id),
        "stop_actor must RETAIN a cursor for a restartable/permissioned actor (replay-on-respawn)"
    );
}
