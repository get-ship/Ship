//! Tests for actor crash cleanup and lifecycle edge cases.

use tempfile::tempdir;

use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::validator::{CallerKind, EmitContext};

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(crate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    (tmp, router)
}

fn agent_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec![format!("{id}.")],
        read_namespaces: vec![format!("{id}.")],
        subscribe_namespaces: vec![format!("{id}.")],
        see_directed_events: false,
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    }
}

fn kernel_event_count(router: &KernelRouter, event_type: &str) -> i64 {
    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE event_type = ?")
            .bind(event_type)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap()
}

fn actor_status(router: &KernelRouter, actor_id: &str) -> Option<String> {
    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    block_on(async {
        sqlx::query_scalar("SELECT status FROM actors WHERE id = ?")
            .bind(actor_id)
            .fetch_optional(&mut conn)
            .await
    })
    .unwrap()
}

// ── handle_actor_crash ──────────────────────────────────────────────────────

/// Acceptance: `actor.crashed` must fire the projection bus so the `actors`
/// read model reflects the crash. Before the raw_append→persist fix this row
/// stayed `active` because raw_append bypassed the bus.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn crash_updates_actor_projection() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    // Seed the actors projection row via the canonical bus-firing path.
    let created = EventEnvelope::new(
        "actor.created",
        "dispatch",
        &serde_json::json!({ "kind": "test-worker", "environment_type": "local" }),
    )
    .unwrap();
    router.emit(created, &runtime_ctx()).await.unwrap();
    assert_eq!(
        actor_status(&router, "dispatch").as_deref(),
        Some("created"),
        "actor.created must project a row"
    );

    router.handle_actor_crash("dispatch", "task panicked").unwrap();

    assert_eq!(
        actor_status(&router, "dispatch").as_deref(),
        Some("crashed"),
        "actor.crashed must update the projection (fired via the bus, not raw_append)"
    );
}

#[test]
fn crash_removes_mailbox_and_emits_event() {
    let (_tmp, mut router) = setup();
    let _mb = router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    router.handle_actor_crash("dispatch", "task panicked").unwrap();

    assert_eq!(router.actor_count(), 0, "crashed actor must be removed");
    assert_eq!(
        kernel_event_count(&router, "actor.crashed"),
        1,
        "actor.crashed event must be in kernel store"
    );
}

// `crash_retains_event_store` is retired with the per-actor `ActorStore`.
// Every actor now reads from the shared `runtime.db`; there is no per-actor
// file to retain. Crash cleanup is a no-op on the storage layer.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn crash_removes_subscriptions() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    router.handle_actor_crash("dispatch", "panic").unwrap();

    // Routing to the dead actor's namespace should not panic or deliver.
    let result = router.emit(ev("dispatch.event"), &runtime_ctx()).await;
    assert!(result.is_ok(), "routing after crash cleanup must succeed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn crash_closes_mailbox() {
    let (_tmp, mut router) = setup();
    let mut mb = router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    router.handle_actor_crash("dispatch", "panic").unwrap();

    // Sender was dropped — recv returns None.
    assert!(mb.recv().await.is_none(), "crashed actor mailbox must be closed");
}

#[test]
fn crash_errors_on_unknown_actor() {
    let (_tmp, mut router) = setup();
    let result = router.handle_actor_crash("ghost", "panic");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn crash_payload_contains_reason() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    router.handle_actor_crash("dispatch", "task panicked: division by zero").unwrap();

    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    let payload: String = block_on(async {
        sqlx::query_scalar(
            "SELECT payload_json FROM events WHERE event_type = 'actor.crashed'"
        )
        .fetch_one(&mut conn)
        .await
    })
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
    // The crash event now uses the registered ActorCrashed payload
    // shape (`error` + `restart_count`) — the universal stamp/scope/
    // schema chokepoint validates it before persist, so the reason
    // lands in the schema's `error` field. `actor_id` is the entity
    // id, NOT a payload field.
    assert_eq!(parsed["error"], "task panicked: division by zero");
    assert_eq!(parsed["restart_count"], 0);
}

#[test]
fn crash_event_is_stamped_and_scoped() {
    // The shared chokepoint stamps the crash event with author
    // identity and the Runtime scope key — the same gate every other
    // write path runs. A reconciliation that filters by author or
    // scope must include kernel-lifecycle crash events.
    let (_tmp, mut router) = setup();
    router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    router.handle_actor_crash("dispatch", "boom").unwrap();

    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    let (scope, scope_key, author_device_id): (String, Option<String>, Option<String>) =
        block_on(async {
            sqlx::query_as(
                "SELECT scope, scope_key, author_device_id \
                 FROM events WHERE event_type = 'actor.crashed'",
            )
            .fetch_one(&mut conn)
            .await
        })
        .unwrap();
    assert_eq!(scope, "runtime");
    assert!(scope_key.is_some(), "Runtime scope must be stamped with a key");
    assert!(
        author_device_id.is_some(),
        "crash event must be stamped with author_device_id"
    );
}

// ── Mailbox overflow in stable path ─────────────────────────────────────────
//
// Since route() is delivery-only (deprecated best-effort path), overflow silently
// drops the event and returns Ok. The event is not delivered but no error is
// propagated — backpressure is a separate concern (see job/kernel-backpressure-01).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stable_path_mailbox_full_drops_silently() {
    let (_tmp, mut router) = setup();
    let _mb = router.spawn_actor("studio", ActorConfig {
        namespace: "studio".into(),
        write_namespaces: vec!["studio.".into()],
        read_namespaces: vec!["studio.".into()],
        subscribe_namespaces: vec!["studio.".into()],
        see_directed_events: false,
    }).unwrap();

    // Fill the mailbox.
    for i in 0..256 {
        let event = EventEnvelope::new(
            "studio.tick",
            "entity",
            &serde_json::json!({ "seq": i }),
        )
        .unwrap();
        router.emit(event, &runtime_ctx()).await.unwrap();
    }

    // Overflow event: route() returns Ok (delivery-only, best-effort — drop on full).
    let overflow = ev("studio.overflow");
    let result = router.emit(overflow, &runtime_ctx()).await;
    assert!(result.is_ok(), "route() must not error on mailbox full (best-effort delivery)");
}
