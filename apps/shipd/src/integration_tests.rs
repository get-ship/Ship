//! Integration tests using TestDaemon — SSE delivery, connection lifecycle,
//! and edge cases (concurrency, namespace isolation, mailbox overflow).

use crate::test_harness::TestDaemon;
use ship_substrate::events::{ActorConfig, EventEnvelope};

// ── TestDaemon harness ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_provides_clean_environment() {
    let daemon = TestDaemon::new().await;
    assert!(daemon.temp_dir.path().exists());
    assert!(ship_substrate::ShipHome::project_ship_dir(daemon.temp_dir.path()).exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_get_returns_json() {
    let daemon = TestDaemon::new().await;
    let resp = daemon.get("/api/v1/projects").await;
    assert_eq!(resp.status, 200);
    assert!(resp.body["projects"].is_array());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn harness_post_returns_json() {
    let daemon = TestDaemon::new().await;
    let resp = daemon.register_agent("agent.harness", &["test"]).await;
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body["ok"], true);
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

// ── SSE delivery ────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_delivers_routed_event_to_registered_agent() {
    let daemon = TestDaemon::new().await;

    // Register agent.
    daemon.register_agent("agent.sse-rx", &["test"]).await;

    // Take the mailbox (as SSE endpoint would).
    let mut mailbox = daemon
        .state
        .agent_mailboxes()
        .lock()
        .await
        .remove("agent.sse-rx")
        .expect("mailbox must be stashed after register");

    // Drain any registration events.
    while mailbox.try_recv().is_ok() {}

    // Route a targeted event directly to the agent's mailbox.
    {
        let envelope = EventEnvelope::new(
            "mesh.custom",
            "agent.sse-rx",
            &serde_json::json!({"text": "hello"}),
        )
        .unwrap()
        .with_target("agent.sse-rx");

        let ctx = ship_substrate::events::EmitContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            skill_id: None,
            project_id: None,
            session_id: None,
        };
        daemon.kernel.lock().await.emit(envelope, &ctx).await.unwrap();
    }

    // Give delivery time.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let received = mailbox.try_recv();
    assert!(received.is_ok(), "agent must receive targeted event via mailbox");
    assert_eq!(received.unwrap().event_type, "mesh.custom");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_endpoint_returns_event_stream_content_type() {
    let daemon = TestDaemon::new().await;
    daemon.register_agent("agent.sse-type", &[]).await;

    let resp = daemon.get_raw("/api/v1/mesh/events/agent.sse-type").await;
    assert_eq!(resp.status(), 200);

    let content_type = resp
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(
        content_type.contains("text/event-stream"),
        "SSE endpoint must return text/event-stream, got: {content_type}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_endpoint_not_found_for_unregistered() {
    let daemon = TestDaemon::new().await;
    let resp = daemon.get_raw("/api/v1/mesh/events/agent.ghost").await;
    assert_eq!(resp.status(), 404);
}

// ── Connection lifecycle ────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_connect_register_work_disconnect() {
    let daemon = TestDaemon::new().await;

    // 1. Connect: register agent.
    let resp = daemon.register_agent("agent.worker", &["rust"]).await;
    assert_eq!(resp.status, 200);

    // 2. Verify agent is in kernel.
    assert!(daemon.kernel.lock().await.has_actor("agent.worker"));

    // 3. Work: send a message to another agent.
    daemon.register_agent("agent.reviewer", &["review"]).await;
    let resp = daemon
        .mesh_send("agent.worker", "agent.reviewer", serde_json::json!({"pr": 42}))
        .await;
    assert_eq!(resp.status, 200);

    // 4. Disconnect: stop the actor.
    daemon.kernel.lock().await.stop_actor("agent.worker").unwrap();
    assert!(!daemon.kernel.lock().await.has_actor("agent.worker"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_crash_drops_actor() {
    let daemon = TestDaemon::new().await;
    daemon.register_agent("agent.crashy", &[]).await;

    // Simulate crash by stopping the actor (as ConnectionGuard::drop does).
    daemon
        .kernel
        .lock()
        .await
        .stop_actor("agent.crashy")
        .unwrap();

    // Actor should no longer be in kernel.
    assert!(!daemon.kernel.lock().await.has_actor("agent.crashy"));

    // Mailbox should still be in stash (but unusable since sender dropped).
    let has_mailbox = daemon
        .state
        .agent_mailboxes()
        .lock()
        .await
        .contains_key("agent.crashy");
    assert!(has_mailbox, "mailbox remains stashed until SSE drains it");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multiple_agents_connect_concurrently() {
    let daemon = TestDaemon::new().await;

    // Register 5 agents concurrently.
    let mut handles = Vec::new();
    for i in 0..5 {
        let state = daemon.state.clone();
        handles.push(tokio::spawn(async move {
            let req = crate::routes::rest_api::MeshRegisterReq {
                agent_id: format!("agent.concurrent-{i}"),
                capabilities: vec!["test".into()],
            };
            crate::routes::rest_api::mesh_register(
                axum::extract::State(state),
                axum::Extension(crate::test_harness::TestDaemon::cli_auth()),
                axum::extract::Json(req),
            )
            .await;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    // All 5 should be registered.
    let kernel = daemon.kernel.lock().await;
    for i in 0..5 {
        assert!(
            kernel.has_actor(&format!("agent.concurrent-{i}")),
            "agent.concurrent-{i} must be registered"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_reconnects_after_disconnect() {
    let daemon = TestDaemon::new().await;

    // First connection.
    daemon.register_agent("agent.reconnect", &["v1"]).await;
    assert!(daemon.kernel.lock().await.has_actor("agent.reconnect"));

    // Disconnect.
    daemon
        .kernel
        .lock()
        .await
        .stop_actor("agent.reconnect")
        .unwrap();
    assert!(!daemon.kernel.lock().await.has_actor("agent.reconnect"));

    // Reconnect (re-register).
    daemon.register_agent("agent.reconnect", &["v2"]).await;
    assert!(daemon.kernel.lock().await.has_actor("agent.reconnect"));
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn namespace_isolation_between_actors() {
    let daemon = TestDaemon::new().await;

    // Spawn two actors with different namespaces.
    {
        let mut kernel = daemon.kernel.lock().await;
        let config_a = ActorConfig {
            namespace: "agent-a".into(),
            write_namespaces: vec!["team-a.".into()],
            read_namespaces: vec!["team-a.".into()],
            subscribe_namespaces: vec!["team-a.".into()],
            see_directed_events: false,
        };
        let config_b = ActorConfig {
            namespace: "agent-b".into(),
            write_namespaces: vec!["team-b.".into()],
            read_namespaces: vec!["team-b.".into()],
            subscribe_namespaces: vec!["team-b.".into()],
            see_directed_events: false,
        };
        let (_store_a, _mb_a) = kernel.spawn_actor("agent-a", config_a).unwrap();
        let (store_b, _mb_b) = kernel.spawn_actor("agent-b", config_b).unwrap();

        // Actor B cannot write to team-a namespace.
        let ev = EventEnvelope::new("team-a.secret", "e1", &serde_json::json!({})).unwrap();
        let err = store_b.append(&ev);
        assert!(err.is_err(), "actor B must not write to team-a namespace");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_store_isolation_between_actors() {
    let daemon = TestDaemon::new().await;

    let mut kernel = daemon.kernel.lock().await;
    let config = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["shared.".into()],
        read_namespaces: vec!["shared.".into()],
        subscribe_namespaces: vec![],
        see_directed_events: false,
    };

    let (store_a, _mb_a) = kernel.spawn_actor("iso-a", config("iso-a")).unwrap();
    let (store_b, _mb_b) = kernel.spawn_actor("iso-b", config("iso-b")).unwrap();

    // Write to store A.
    let ev = EventEnvelope::new("shared.data", "x", &serde_json::json!({"key": "secret"})).unwrap();
    store_a.append(&ev).unwrap();

    // Store B cannot see store A's data.
    let filter = ship_substrate::events::EventFilter::default();
    let results = store_b.query(&filter).unwrap();
    assert!(results.is_empty(), "actor B must not see actor A's events");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_overflow_evicts_oldest() {
    use crate::transport::connections::Inbox;

    let inbox = Inbox::new();

    // Push 300 events (cap is 256).
    for i in 0..300 {
        let ev =
            EventEnvelope::new(&format!("overflow.{i}"), "e", &serde_json::json!({})).unwrap();
        inbox.push(ev).await;
    }

    let drained = inbox.drain().await;
    assert_eq!(drained.len(), 256, "inbox must cap at 256");
    // Oldest (0..44) evicted; first should be event 44.
    assert_eq!(drained[0].event_type, "overflow.44");
    assert_eq!(drained[255].event_type, "overflow.299");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_mesh_register_same_agent() {
    let daemon = TestDaemon::new().await;

    // Two concurrent registrations for the same agent.
    let state1 = daemon.state.clone();
    let state2 = daemon.state.clone();

    let h1 = tokio::spawn(async move {
        crate::routes::rest_api::mesh_register(
            axum::extract::State(state1),
            axum::Extension(crate::test_harness::TestDaemon::cli_auth()),
            axum::extract::Json(crate::routes::rest_api::MeshRegisterReq {
                agent_id: "agent.race".into(),
                capabilities: vec!["v1".into()],
            }),
        )
        .await;
    });
    let h2 = tokio::spawn(async move {
        crate::routes::rest_api::mesh_register(
            axum::extract::State(state2),
            axum::Extension(crate::test_harness::TestDaemon::cli_auth()),
            axum::extract::Json(crate::routes::rest_api::MeshRegisterReq {
                agent_id: "agent.race".into(),
                capabilities: vec!["v2".into()],
            }),
        )
        .await;
    });

    h1.await.unwrap();
    h2.await.unwrap();

    // Actor should exist (one of the registrations wins).
    assert!(daemon.kernel.lock().await.has_actor("agent.race"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn broadcast_routes_to_subscribed_actors() {
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
        project_id: None,
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mesh_discover_via_router() {
    let daemon = TestDaemon::new().await;

    // Populate the shared registry directly (no MeshService in test harness).
    {
        let mut reg = daemon.state.mesh_registry().write().await;
        reg.insert(
            "agent.discoverable".into(),
            ship_platform::services::mesh::MeshEntry {
                agent_id: "agent.discoverable".into(),
                label: "agent.discoverable".into(),
                capabilities: vec!["rust".into()],
                registered_at: chrono::Utc::now(),
                status: ship_platform::services::mesh::AgentStatus::Active,
            },
        );
    }

    let resp = daemon.get("/api/v1/mesh/discover").await;
    assert_eq!(resp.status, 200);
    let agents = resp.body["data"]["agents"].as_array().unwrap();
    assert!(
        agents.iter().any(|a| a["agent_id"] == "agent.discoverable"),
        "registered agent must appear in discover"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mesh_send_to_unknown_agent_does_not_crash() {
    let daemon = TestDaemon::new().await;
    daemon.register_agent("agent.loner", &[]).await;

    let resp = daemon
        .mesh_send(
            "agent.loner",
            "agent.nonexistent",
            serde_json::json!({"msg": "hello?"}),
        )
        .await;
    assert_eq!(resp.status, 200, "send to unknown must not crash");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_projects_via_harness_returns_ok() {
    let daemon = TestDaemon::new().await;
    let resp = daemon.get("/api/v1/projects").await;
    assert_eq!(resp.status, 200);
    assert!(resp.body["projects"].is_array());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_sessions_via_harness() {
    let daemon = TestDaemon::new().await;

    let resp = daemon.get("/api/v1/sessions").await;
    assert_eq!(resp.status, 200);
    assert!(resp.body["data"]["sessions"].is_array());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_agents_via_harness() {
    let daemon = TestDaemon::new().await;

    // Populate registry directly.
    {
        let mut reg = daemon.state.mesh_registry().write().await;
        reg.insert(
            "agent.api-test".into(),
            ship_platform::services::mesh::MeshEntry {
                agent_id: "agent.api-test".into(),
                label: "agent.api-test".into(),
                capabilities: vec!["rust".into()],
                registered_at: chrono::Utc::now(),
                status: ship_platform::services::mesh::AgentStatus::Active,
            },
        );
    }

    let resp = daemon.get("/api/v1/agents/active").await;
    assert_eq!(resp.status, 200);
    let agents = resp.body["data"]["agents"].as_array().unwrap();
    assert!(
        agents.iter().any(|a| a["agent_id"] == "agent.api-test"),
        "registered agent must appear in runtime agents API"
    );
}

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

