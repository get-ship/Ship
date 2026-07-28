//! Tests: CLI-originated events route through KernelRouter.
//!
//! Uses local KernelRouter instances (not the global OnceLock singleton)
//! to keep tests isolated. Each test spins up its own router and verifies
//! that the routing path works end-to-end.

use tempfile::tempdir;

use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::types::event_types;
use crate::events::validator::{CallerKind, EmitContext};

fn cli_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Cli,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    }
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    }
}

fn agent_config() -> ActorConfig {
    ActorConfig {
        namespace: "agent.mcp".into(),
        write_namespaces: vec!["".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec![
            "session.".into(),
            "gate.".into(),
        ],
        see_directed_events: false,
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-cli", &serde_json::json!({})).unwrap()
}

fn session_ev() -> EventEnvelope {
    EventEnvelope::new(agent_schema::canonical_session::event_types::TURN_STARTED, "entity-cli", &serde_json::json!({
        "session_id": "entity-cli",
        "idx": 0,
    })).unwrap()
}

// ── system event reaches kernel store ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn system_event_reaches_kernel_store() {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let event = EventEnvelope::new(
        event_types::ACTOR_CREATED,
        "entity-cli",
        &serde_json::json!({ "kind": "test", "environment_type": "local" }),
    )
    .unwrap();
    // emit persists + delivers. route is delivery-only.
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
            .bind(&event.id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(count, 1, "system event must reach kernel store");
}

// ── session event delivered to subscribed actor mailbox ───────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_event_delivered_to_subscribed_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let mut mailbox = router.spawn_actor("agent.mcp", agent_config()).unwrap();

    let event = session_ev();
    router.emit(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mailbox.try_recv().expect("session event must reach subscribed actor mailbox");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, agent_schema::canonical_session::event_types::TURN_STARTED);
}

// ── CLI-context event reaches kernel store ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_context_event_reaches_kernel_store() {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let event = session_ev();
    // CallerKind::Cli is trusted — must not be rejected by validators.
    router.emit(event.clone(), &cli_ctx()).await.unwrap();

    let mut conn = open_db_at(&router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
            .bind(&event.id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(count, 1, "cli-context event must reach kernel store");
}

// ── CLI-context event delivered to subscribed mailbox ────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_context_event_delivered_to_subscribed_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let mut mailbox = router.spawn_actor("agent.mcp", agent_config()).unwrap();

    let event = session_ev();
    router.emit(event.clone(), &cli_ctx()).await.unwrap();

    let received = mailbox.try_recv().expect("cli-context event must reach subscribed actor mailbox");
    assert_eq!(received.id, event.id);
}

// ── fallback: no router → events go straight to the store db ─────────────────
//
// This test verifies the fallback path by directly calling SqliteEventStore,
// which is what workspace_events/session_events/actor_events do when
// kernel_router() returns None.

#[test]
fn event_persisted_to_store_without_router() {
    use crate::db::runtime_db_path;
    use crate::persistence::event_store::{EventStore, SqliteEventStore};
    use crate::testutil::init_project_minimal as init_project;

    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();

    let store = SqliteEventStore::new_at_db_path(&runtime_db_path(&ship_dir)).unwrap();
    let event = ev(event_types::ACTOR_CREATED);
    store.append(&event).unwrap();

    let got = store.get(&event.id).unwrap().expect("event must be in the store db");
    assert_eq!(got.id, event.id);
    assert_eq!(got.event_type, event_types::ACTOR_CREATED);
}
