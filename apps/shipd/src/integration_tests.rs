//! Integration tests using TestDaemon — kernel actor delivery, connection
//! lifecycle, and edge cases. The mesh.* op surface tests were removed with
//! the mesh.*/actor.* op cut.

use crate::test_harness::TestDaemon;
use ship_substrate::events::{ActorConfig, EventEnvelope};

/// Standard mesh actor config used by actor/spawn registrations in tests —
/// mirrors the namespaces the retired `mesh/register` shim applied.
fn mesh_actor_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["mesh.".into()],
        read_namespaces: vec!["mesh.".into()],
        subscribe_namespaces: vec!["mesh.".into()],
        see_directed_events: false,
    }
}

// ── TestDaemon harness ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_provides_clean_environment() {
    let daemon = TestDaemon::new().await;
    assert!(daemon.temp_dir.path().exists());
    assert!(ship_substrate::ShipHome::project_ship_dir(daemon.temp_dir.path()).exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_teardown_cleans_temp_dir() {
    let path;
    {
        let daemon = TestDaemon::new().await;
        path = daemon.temp_dir.path().to_path_buf();
        assert!(path.exists());
    }
    // TempDir dropped — path should be cleaned up.
    assert!(!path.exists(), "temp dir must be cleaned up on drop");
}

// ── Kernel actor delivery ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_receives_targeted_kernel_event() {
    let daemon = TestDaemon::new().await;

    let (_store, mut mailbox) = daemon
        .kernel
        .lock()
        .await
        .spawn_actor(
            "agent.target",
            ActorConfig {
                namespace: "agent.target".into(),
                write_namespaces: vec![],
                read_namespaces: vec![],
                subscribe_namespaces: vec![],
                see_directed_events: false,
            },
        )
        .unwrap();

    // Drain any registration events.
    while mailbox.try_recv().is_ok() {}

    // Route a targeted event directly to the actor's mailbox.
    {
        let envelope = EventEnvelope::new(
            "test.custom",
            "agent.target",
            &serde_json::json!({"text": "hello"}),
        )
        .unwrap()
        .with_target("agent.target");

        let ctx = ship_substrate::events::EmitContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            skill_id: None,
            binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
            session_id: None,
        };
        daemon.kernel.lock().await.emit(envelope, &ctx).await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let received = mailbox.try_recv();
    assert!(received.is_ok(), "actor must receive targeted event via mailbox");
    assert_eq!(received.unwrap().event_type, "test.custom");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn broadcast_reaches_subscribed_actors() {
    let daemon = TestDaemon::new().await;

    // Spawn two actors subscribed to "mesh." namespace.
    let mut kernel = daemon.kernel.lock().await;
    let config = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["mesh.".into()],
        read_namespaces: vec!["mesh.".into()],
        subscribe_namespaces: vec!["mesh.".into()],
        see_directed_events: false,
    };
    let (_s1, mut mb1) = kernel.spawn_actor("bcast.sender", config("bcast.sender")).unwrap();
    let (_s2, mut mb2) = kernel.spawn_actor("bcast.listener", config("bcast.listener")).unwrap();

    // Route a broadcast event.
    let ev = EventEnvelope::new("mesh.broadcast", "all", &serde_json::json!({"data": "hi"}))
        .unwrap();
    let ctx = ship_substrate::events::EmitContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    kernel.emit(ev.clone(), &ctx).await.unwrap();
    drop(kernel);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Both actors should receive the broadcast (they both subscribe to mesh.).
    let r1 = mb1.try_recv();
    let r2 = mb2.try_recv();
    assert!(r1.is_ok(), "sender must receive broadcast");
    assert!(r2.is_ok(), "listener must receive broadcast");
}

// ── Connection lifecycle (kernel-level) ────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_crash_drops_actor() {
    let daemon = TestDaemon::new().await;
    daemon
        .kernel
        .lock()
        .await
        .spawn_actor(
            "agent.crashy",
            ActorConfig {
                namespace: "agent.crashy".into(),
                write_namespaces: vec![],
                read_namespaces: vec![],
                subscribe_namespaces: vec![],
                see_directed_events: false,
            },
        )
        .unwrap();

    // Simulate crash by stopping the actor (as ConnectionGuard::drop does).
    daemon.kernel.lock().await.stop_actor("agent.crashy").unwrap();

    // Actor should no longer be in kernel.
    assert!(!daemon.kernel.lock().await.has_actor("agent.crashy"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_actor_spawn_same_id() {
    let daemon = TestDaemon::new().await;
    let state = daemon.state.clone();
    let kernel = daemon.kernel.clone();

    // Spawn two actors with the same id concurrently — one must win without panic.
    let k1 = kernel.clone();
    let k2 = kernel.clone();
    let h1 = tokio::spawn(async move {
        let _ = k1.lock().await.spawn_actor(
            "agent.race",
            ActorConfig {
                namespace: "agent.race".into(),
                write_namespaces: vec![],
                read_namespaces: vec![],
                subscribe_namespaces: vec![],
                see_directed_events: false,
            },
        );
    });
    let h2 = tokio::spawn(async move {
        let _ = k2.lock().await.spawn_actor(
            "agent.race",
            ActorConfig {
                namespace: "agent.race".into(),
                write_namespaces: vec![],
                read_namespaces: vec![],
                subscribe_namespaces: vec![],
                see_directed_events: false,
            },
        );
    });
    let _ = tokio::join!(h1, h2);

    // At least one registered successfully.
    assert!(
        state.kernel().lock().await.has_actor("agent.race"),
        "at least one spawn must succeed"
    );
}

// ── events/route endpoint ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_route_via_harness() {
    let daemon = TestDaemon::new().await;

    let envelope =
        EventEnvelope::new("test.custom", "entity-1", &serde_json::json!({"key": "value"}))
            .unwrap();

    let resp = daemon
        .post(
            "/api/v1/events/route",
            &serde_json::json!({
                "envelope": envelope,
                "workspace_id": null,
                "session_id": null,
            }),
        )
        .await;
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body["ok"], true);
}
