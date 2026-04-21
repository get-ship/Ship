//! Tests for actor crash cleanup and lifecycle edge cases.

use tempfile::tempdir;

use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::validator::{CallerKind, EmitContext};

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
    (tmp, router)
}

fn agent_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec![format!("{id}.")],
        read_namespaces: vec![format!("{id}.")],
        subscribe_namespaces: vec![format!("{id}.")],
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn kernel_event_count(router: &KernelRouter, event_type: &str) -> i64 {
    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE event_type = ?")
            .bind(event_type)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap()
}

// ── handle_actor_crash ──────────────────────────────────────────────────────

#[test]
fn crash_removes_mailbox_and_emits_event() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();
    store.append(&ev("dispatch.started")).unwrap();

    router.handle_actor_crash("dispatch", "task panicked").unwrap();

    assert_eq!(router.actor_count(), 0, "crashed actor must be removed");
    assert_eq!(
        kernel_event_count(&router, "actor.crashed"),
        1,
        "actor.crashed event must be in kernel store"
    );
}

#[test]
fn crash_retains_event_store() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();
    let db_path = store.db_path().to_path_buf();
    store.append(&ev("dispatch.started")).unwrap();

    router.handle_actor_crash("dispatch", "panic").unwrap();

    assert!(db_path.exists(), "actor event store must be retained after crash");
}

#[test]
fn crash_removes_subscriptions() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

    router.handle_actor_crash("dispatch", "panic").unwrap();

    // Routing to the dead actor's namespace should not panic or deliver.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let result = router.route(ev("dispatch.event"), &runtime_ctx()).await;
        assert!(result.is_ok(), "routing after crash cleanup must succeed");
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn crash_closes_mailbox() {
    let (_tmp, mut router) = setup();
    let (_store, mut mb) = router.spawn_actor("dispatch", agent_config("dispatch")).unwrap();

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

    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    let payload: String = block_on(async {
        sqlx::query_scalar(
            "SELECT payload_json FROM events WHERE event_type = 'actor.crashed'"
        )
        .fetch_one(&mut conn)
        .await
    })
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(parsed["reason"], "task panicked: division by zero");
    assert_eq!(parsed["actor_id"], "dispatch");
}

// ── Mailbox overflow in stable path ─────────────────────────────────────────
//
// Since route() is delivery-only (deprecated best-effort path), overflow silently
// drops the event and returns Ok. The event is not delivered but no error is
// propagated — backpressure is a separate concern (see job/kernel-backpressure-01).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stable_path_mailbox_full_drops_silently() {
    let (_tmp, mut router) = setup();
    let (_store, _mb) = router.spawn_actor("studio", ActorConfig {
        namespace: "studio".into(),
        write_namespaces: vec!["studio.".into()],
        read_namespaces: vec!["studio.".into()],
        subscribe_namespaces: vec!["studio.".into()],
    }).unwrap();

    // Fill the mailbox.
    for i in 0..256 {
        let event = EventEnvelope::new(
            "studio.tick",
            "entity",
            &serde_json::json!({ "seq": i }),
        )
        .unwrap();
        router.route(event, &runtime_ctx()).await.unwrap();
    }

    // Overflow event: route() returns Ok (delivery-only, best-effort — drop on full).
    let overflow = ev("studio.overflow");
    let result = router.route(overflow, &runtime_ctx()).await;
    assert!(result.is_ok(), "route() must not error on mailbox full (best-effort delivery)");
}
