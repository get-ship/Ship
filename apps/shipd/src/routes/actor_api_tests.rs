//! `/api/v1/actor/spawn` register-or-refine HTTP behavior.

use axum::{Extension, Json, extract::State};
use axum::response::IntoResponse;

use crate::routes::actor_api::{ActorSpawnReq, actor_spawn};
use crate::test_harness::TestDaemon;
use ship_substrate::events::{ActorConfig, EventEnvelope};

fn cfg(ns: &str) -> ActorConfig {
    ActorConfig {
        namespace: ns.into(),
        write_namespaces: vec![format!("{ns}.")],
        read_namespaces: vec![format!("{ns}.")],
        subscribe_namespaces: vec![format!("{ns}.")],
        see_directed_events: false,
    }
}

/// ULID-shaped id so actor_spawn doesn't warn about the backwards-compat
/// path. 26 chars, Crockford alphabet.
const ULID_FIXTURE: &str = "01ABCDEFGHJKMNPQRSTVWXYZ23";

#[tokio::test(flavor = "multi_thread")]
async fn actor_spawn_refines_existing_actor_and_preserves_mailbox() {
    let daemon = TestDaemon::new().await;

    // Pre-register the actor as if substrate had spawned it — hold the
    // mailbox so we can prove refinement doesn't replace the sender.
    let mailbox = {
        let mut kernel = daemon.kernel.lock().await;
        let (_store, mb) = kernel
            .spawn_actor(ULID_FIXTURE, cfg("preregistered"))
            .expect("substrate-style pre-register");
        mb
    };

    // The MCP-style caller comes in with a refined config.
    let resp = actor_spawn(
        State(daemon.state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(ActorSpawnReq {
            actor_id: Some(ULID_FIXTURE.to_string()),
            config: cfg("refined"),
            capabilities: None,
        }),
    )
    .await
    .into_response();
    assert!(
        resp.status().is_success(),
        "refine HTTP status = {}",
        resp.status()
    );

    // Kernel's stored config now reflects the refined namespace.
    {
        let kernel = daemon.kernel.lock().await;
        let stored = kernel
            .list_actors()
            .into_iter()
            .find(|(id, _)| id == ULID_FIXTURE)
            .map(|(_, c)| c)
            .expect("actor still registered after refine");
        assert_eq!(
            stored.subscribe_namespaces,
            vec!["refined.".to_string()],
            "refine_actor must replace subscribe_namespaces"
        );
    }

    // The original Mailbox still receives a directed event — the
    // mpsc::Sender substrate's `spawn_actor` planted in the kernel
    // is unchanged.
    let mut mailbox = mailbox;
    let mut event = EventEnvelope::new(
        "refined.hello",
        "src",
        &serde_json::json!({}),
    )
    .unwrap();
    event.target_actor_id = Some(ULID_FIXTURE.to_string());
    daemon
        .kernel
        .lock()
        .await
        .emit(event, &ship_substrate::events::EmitContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            skill_id: None,
            project_id: None,
            session_id: None,
        })
        .await
        .expect("emit");
    let received = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        mailbox.recv(),
    )
    .await
    .expect("mailbox recv timed out")
    .expect("mailbox sender dropped");
    assert_eq!(received.event_type, "refined.hello");
}

#[tokio::test(flavor = "multi_thread")]
async fn actor_spawn_registers_fresh_when_no_existing_actor() {
    let daemon = TestDaemon::new().await;

    let resp = actor_spawn(
        State(daemon.state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(ActorSpawnReq {
            actor_id: Some(ULID_FIXTURE.to_string()),
            config: cfg("fresh"),
            capabilities: None,
        }),
    )
    .await
    .into_response();
    assert!(
        resp.status().is_success(),
        "fresh register HTTP status = {}",
        resp.status()
    );

    assert!(
        daemon
            .kernel
            .lock()
            .await
            .has_actor(ULID_FIXTURE),
        "actor must be registered after fresh spawn"
    );
    assert!(
        daemon
            .state
            .agent_mailboxes()
            .lock()
            .await
            .contains_key(ULID_FIXTURE),
        "fresh register must populate agent_mailboxes for SSE drain"
    );
}
