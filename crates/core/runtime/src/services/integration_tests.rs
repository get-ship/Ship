//! Integration tests — mesh service wired to a real KernelRouter.
//!
//! Each test spins up a KernelRouter, spawns MeshService via spawn_service,
//! spawns agent actors with directed subscriptions, and drives the full
//! register → send → receive cycle.

#[cfg(feature = "unstable")]
mod mesh_kernel {
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc;

    use ship_types::identity::{ActorIdentity, ActorType};
    use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
    use ship_substrate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use ship_substrate::events::validator::{CallerKind, EmitContext};
    use ship_substrate::events::{EventEnvelope, Mailbox};
    use ship_platform::services::mesh::MeshService;
    use ship_platform::services::{ServiceHandle, spawn_service};

    // ── Fixtures ──────────────────────────────────────────────────────────────

    fn setup() -> (tempfile::TempDir, KernelRouter) {
        let tmp = tempdir().unwrap();
        let router = KernelRouter::new(ship_substrate::ShipHome::project_ship_dir(tmp.path())).unwrap();
        (tmp, router)
    }

    fn mesh_config() -> ActorConfig {
        ActorConfig {
            namespace: "mesh".into(),
            write_namespaces: vec!["mesh.".into()],
            read_namespaces: vec!["mesh.".into()],
            subscribe_namespaces: vec!["mesh.".into()],
        }
    }

    fn ctx() -> EmitContext {
        EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, project_id: None, session_id: None }
    }

    fn identity(label: &str) -> ActorIdentity {
        ActorIdentity { instance_id: ulid::Ulid::new().to_string(), label: label.into(), actor_type: ActorType::Agent }
    }

    fn agent_perms() -> ActorPermissions {
        ActorPermissions {
            emit: vec!["mesh.".into()],
            subscribe: vec![PermittedSubscription { namespace: "mesh.".into(), scope: DeliveryScope::Directed }],
        }
    }

    fn observer_perms() -> ActorPermissions {
        ActorPermissions {
            emit: vec!["mesh.".into()],
            subscribe: vec![PermittedSubscription { namespace: "mesh.".into(), scope: DeliveryScope::Global }],
        }
    }

    fn ev(t: &str, entity: &str, p: serde_json::Value) -> EventEnvelope {
        EventEnvelope::new(t, entity, &p).unwrap()
    }

    fn register(id: &str, caps: &[&str]) -> EventEnvelope {
        let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
        ev("mesh.register", id, serde_json::json!({ "agent_id": id, "capabilities": caps }))
    }

    fn send(from: &str, to: &str, body: serde_json::Value) -> EventEnvelope {
        ev("mesh.send", from, serde_json::json!({ "from": from, "to": to, "body": body }))
    }

    fn broadcast(from: &str, body: serde_json::Value, cap: Option<&str>) -> EventEnvelope {
        let mut p = serde_json::json!({ "from": from, "body": body });
        if let Some(c) = cap { p["capability_filter"] = serde_json::json!(c); }
        ev("mesh.broadcast", from, p)
    }

    /// Yield to the mesh task, then drain outbox and re-route all queued events.
    ///
    /// When routing a `mesh.message` fails due to a full mailbox, generates
    /// a `mesh.send.failed` event with reason `"mailbox_full"` and routes it
    /// back to the sender — implementing backpressure at the routing layer.
    async fn flush(
        outbox: &mut mpsc::UnboundedReceiver<EventEnvelope>,
        router: &KernelRouter,
        ctx: &EmitContext,
    ) {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let events: Vec<EventEnvelope> = std::iter::from_fn(|| outbox.try_recv().ok()).collect();
        for e in events {
            match router.route(e.clone(), ctx).await {
                Ok(()) => {}
                Err(err) if err.to_string().contains("mailbox full") => {
                    if e.event_type == "mesh.message" {
                        let payload: serde_json::Value =
                            serde_json::from_str(&e.payload_json).unwrap_or_default();
                        let sender = payload["from"].as_str().unwrap_or("unknown");
                        let target = e.target_actor_id.as_deref().unwrap_or("unknown");
                        let fail = EventEnvelope::new(
                            "mesh.send.failed",
                            target,
                            &serde_json::json!({
                                "to": target,
                                "reason": "mailbox_full",
                            }),
                        )
                        .unwrap()
                        .with_causation(&e.id)
                        .with_target(sender);
                        let _ = router.route(fail, ctx).await;
                    }
                }
                Err(err) => panic!("route error: {err}"),
            }
        }
    }

    /// Receive from a mailbox with a 1 s deadline.
    async fn recv(mb: &mut Mailbox) -> EventEnvelope {
        tokio::time::timeout(Duration::from_secs(1), mb.recv())
            .await
            .expect("timeout waiting for mailbox event")
            .expect("mailbox closed")
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn agent_register_send_receive() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_s1, mut mb_sender) = router.spawn_actor_with_permissions(identity("agent.sender"), agent_perms()).unwrap();
        let (_s2, mut mb_receiver) = router.spawn_actor_with_permissions(identity("agent.receiver"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.sender", &[]), &ctx).await.unwrap();
        router.route(register("agent.receiver", &[]), &ctx).await.unwrap();
        router.route(send("agent.sender", "agent.receiver", serde_json::json!({"text": "hello"})), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let msg = recv(&mut mb_receiver).await;
        assert_eq!(msg.event_type, "mesh.message");
        let p: serde_json::Value = serde_json::from_str(&msg.payload_json).unwrap();
        assert_eq!(p["body"]["text"], "hello");
        assert!(mb_sender.try_recv().is_err(), "sender must not receive its own message");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn agent_send_and_respond() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_sa, mut mb_a) = router.spawn_actor_with_permissions(identity("agent.alpha"), agent_perms()).unwrap();
        let (_sb, mut mb_b) = router.spawn_actor_with_permissions(identity("agent.beta"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.alpha", &[]), &ctx).await.unwrap();
        router.route(register("agent.beta", &[]), &ctx).await.unwrap();

        router.route(send("agent.alpha", "agent.beta", serde_json::json!({"msg": "ping"})), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;
        let req = recv(&mut mb_b).await;
        let rp: serde_json::Value = serde_json::from_str(&req.payload_json).unwrap();
        assert_eq!(rp["body"]["msg"], "ping");

        router.route(send("agent.beta", "agent.alpha", serde_json::json!({"msg": "pong"})), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;
        let resp = recv(&mut mb_a).await;
        let rp2: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
        assert_eq!(rp2["body"]["msg"], "pong");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn broadcast_reaches_all_subscribers() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_sa, _mb_a) = router.spawn_actor_with_permissions(identity("agent.a"), agent_perms()).unwrap();
        let (_sb, mut mb_b) = router.spawn_actor_with_permissions(identity("agent.b"), agent_perms()).unwrap();
        let (_sc, mut mb_c) = router.spawn_actor_with_permissions(identity("agent.c"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.a", &["rust"]), &ctx).await.unwrap();
        router.route(register("agent.b", &["rust", "go"]), &ctx).await.unwrap();
        router.route(register("agent.c", &["go"]), &ctx).await.unwrap();
        router.route(broadcast("agent.a", serde_json::json!({}), Some("rust")), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let msg = recv(&mut mb_b).await;
        assert_eq!(msg.event_type, "mesh.message");
        assert!(mb_c.try_recv().is_err(), "agent.c lacks 'rust' capability — must not receive");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn discover_returns_live_agents() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_sa, mut mb_a) = router.spawn_actor_with_permissions(identity("agent.a"), agent_perms()).unwrap();
        let (_sb, _mb_b) = router.spawn_actor_with_permissions(identity("agent.b"), agent_perms()).unwrap();
        let (_sc, _mb_c) = router.spawn_actor_with_permissions(identity("agent.c"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.a", &[]), &ctx).await.unwrap();
        router.route(register("agent.b", &[]), &ctx).await.unwrap();
        router.route(register("agent.c", &[]), &ctx).await.unwrap();
        router.route(ev("mesh.status", "agent.b", serde_json::json!({ "agent_id": "agent.b", "status": "busy" })), &ctx).await.unwrap();
        router.route(ev("mesh.discover.request", "agent.a", serde_json::json!({ "from": "agent.a", "status": "active" })), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let resp = recv(&mut mb_a).await;
        assert_eq!(resp.event_type, "mesh.discover.response");
        let p: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
        let ids: Vec<&str> = p["agents"].as_array().unwrap()
            .iter().map(|a| a["agent_id"].as_str().unwrap()).collect();
        assert!(!ids.contains(&"agent.b"), "busy agent.b must not appear in active results");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_to_offline_agent_fails_gracefully() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_sa, mut mb_a) = router.spawn_actor_with_permissions(identity("agent.a"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.a", &[]), &ctx).await.unwrap();
        router.route(send("agent.a", "agent.ghost", serde_json::json!({})), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let fail = recv(&mut mb_a).await;
        assert_eq!(fail.event_type, "mesh.send.failed");
        let p: serde_json::Value = serde_json::from_str(&fail.payload_json).unwrap();
        assert_eq!(p["reason"], "agent not found");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mesh_handles_rapid_fire_messages() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_sa, _mb_a) = router.spawn_actor_with_permissions(identity("agent.a"), agent_perms()).unwrap();
        let (_sb, mut mb_b) = router.spawn_actor_with_permissions(identity("agent.b"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.a", &[]), &ctx).await.unwrap();
        router.route(register("agent.b", &[]), &ctx).await.unwrap();
        for i in 0..100u32 {
            router.route(send("agent.a", "agent.b", serde_json::json!({ "n": i })), &ctx).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        while let Ok(e) = outbox.try_recv() {
            router.route(e, &ctx).await.unwrap();
        }

        let mut count = 0u32;
        while mb_b.try_recv().is_ok() { count += 1; }
        assert_eq!(count, 100, "B must receive exactly 100 messages, got {count}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn service_lifecycle_start_stop() {
        let (_tmp, mut router) = setup();
        let (tx, _outbox) = mpsc::unbounded_channel();
        let ServiceHandle { handle, .. } =
            spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let ctx = ctx();

        router.route(register("agent.test", &[]), &ctx).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(router); // closes the mesh mailbox sender → service sees None and exits

        tokio::time::timeout(Duration::from_secs(2), handle)
            .await
            .expect("service did not exit within 2 s")
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn malformed_payload_does_not_crash_service() {
        let (_tmp, mut router) = setup();
        let (tx, _outbox) = mpsc::unbounded_channel();
        let ServiceHandle { handle, .. } =
            spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let ctx = ctx();

        let bad = EventEnvelope::new("mesh.send", "x", &serde_json::json!({})).unwrap();
        router.route(bad, &ctx).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(!handle.is_finished(), "service must survive a malformed payload");

        // Valid register still processed after the error
        router.route(register("agent.ok", &[]), &ctx).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!handle.is_finished(), "service must still be running after valid event");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_broadcast_no_recipients() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let ctx = ctx();

        router.route(register("agent.only", &[]), &ctx).await.unwrap();
        // Drain the mesh.registered event from registration before testing broadcast
        flush(&mut outbox, &router, &ctx).await;

        router.route(broadcast("agent.only", serde_json::json!({}), None), &ctx).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(outbox.try_recv().is_err(), "sole agent broadcasting to itself must produce no messages");
    }

    // ── v0.2.0: Event emission through kernel ───────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn register_emits_mesh_registered_through_kernel() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();

        // Observer subscribes with Global scope to see untargeted mesh.registered events
        let (_so, mut mb_observer) =
            router.spawn_actor_with_permissions(identity("observer"), observer_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.rust", &["rust", "testing"]), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        // Observer should receive mesh.registered event
        let mut found_registered = false;
        while let Ok(msg) = mb_observer.try_recv() {
            if msg.event_type == "mesh.registered" {
                found_registered = true;
                let p: serde_json::Value = serde_json::from_str(&msg.payload_json).unwrap();
                assert_eq!(p["agent_id"], "agent.rust");
                assert_eq!(p["capabilities"], serde_json::json!(["rust", "testing"]));
            }
        }
        assert!(found_registered, "mesh.registered must be routable through the kernel");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deregister_emits_mesh_deregistered_through_kernel() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();
        let (_so, mut mb_observer) =
            router.spawn_actor_with_permissions(identity("observer"), observer_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.rust", &["rust"]), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;
        // Drain observer's register event
        while mb_observer.try_recv().is_ok() {}

        router.route(ev("mesh.deregister", "agent.rust", serde_json::json!({ "agent_id": "agent.rust" })), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let mut found = false;
        while let Ok(msg) = mb_observer.try_recv() {
            if msg.event_type == "mesh.deregistered" {
                found = true;
                let p: serde_json::Value = serde_json::from_str(&msg.payload_json).unwrap();
                assert_eq!(p["agent_id"], "agent.rust");
            }
        }
        assert!(found, "mesh.deregistered must be routable through the kernel");
    }

    // ── v0.2.0: Backpressure ────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn backpressure_signals_mailbox_full_to_sender() {
        use ship_substrate::events::kernel_router::MAILBOX_CAPACITY;

        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();

        let (_sa, mut mb_sender) =
            router.spawn_actor_with_permissions(identity("agent.sender"), agent_perms()).unwrap();
        // Target actor: we register it but DON'T drain its mailbox
        let (_st, _mb_target) =
            router.spawn_actor_with_permissions(identity("agent.target"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.sender", &[]), &ctx).await.unwrap();
        router.route(register("agent.target", &[]), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        // Fill target's mailbox to capacity.
        // The mailbox gets the register event too, so we need (MAILBOX_CAPACITY - 1) more
        // messages to fill it completely. Send enough to ensure it's full.
        for i in 0..MAILBOX_CAPACITY {
            let msg = send("agent.sender", "agent.target", serde_json::json!({ "n": i }));
            router.route(msg, &ctx).await.unwrap();
        }
        flush(&mut outbox, &router, &ctx).await;

        // Now send one more — target mailbox is full, mesh should signal back
        router.route(
            send("agent.sender", "agent.target", serde_json::json!({ "overflow": true })),
            &ctx,
        ).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        // Sender should receive mesh.send.failed with reason "mailbox_full"
        let mut found_backpressure = false;
        while let Ok(msg) = mb_sender.try_recv() {
            if msg.event_type == "mesh.send.failed" {
                let p: serde_json::Value = serde_json::from_str(&msg.payload_json).unwrap();
                if p["reason"] == "mailbox_full" {
                    found_backpressure = true;
                }
            }
        }
        assert!(
            found_backpressure,
            "sender must receive mesh.send.failed with reason mailbox_full when target is at capacity"
        );
    }

    // ── v0.2.0: ConnectionGuard regression ──────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deregister_cleans_up_agent_completely() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();

        let (_sa, mut mb_a) =
            router.spawn_actor_with_permissions(identity("agent.a"), agent_perms()).unwrap();
        let (_sb, _mb_b) =
            router.spawn_actor_with_permissions(identity("agent.b"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.a", &["rust"]), &ctx).await.unwrap();
        router.route(register("agent.b", &["rust"]), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        // Deregister agent.b (simulates ConnectionGuard cleanup)
        router.route(
            ev("mesh.deregister", "agent.b", serde_json::json!({ "agent_id": "agent.b" })),
            &ctx,
        ).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        // agent.a discovers — should only see itself
        router.route(
            ev("mesh.discover.request", "agent.a", serde_json::json!({ "from": "agent.a" })),
            &ctx,
        ).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let resp = recv(&mut mb_a).await;
        assert_eq!(resp.event_type, "mesh.discover.response");
        let p: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
        let agents = p["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1, "only agent.a should remain after agent.b deregistered");
        assert_eq!(agents[0]["agent_id"], "agent.a");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_to_deregistered_agent_fails() {
        let (_tmp, mut router) = setup();
        let (tx, mut outbox) = mpsc::unbounded_channel();
        spawn_service(&mut router, "mesh", mesh_config(), Box::new(MeshService::new(tx))).unwrap();

        let (_sa, mut mb_a) =
            router.spawn_actor_with_permissions(identity("agent.a"), agent_perms()).unwrap();
        let (_sb, _mb_b) =
            router.spawn_actor_with_permissions(identity("agent.b"), agent_perms()).unwrap();
        let ctx = ctx();

        router.route(register("agent.a", &[]), &ctx).await.unwrap();
        router.route(register("agent.b", &[]), &ctx).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        // Deregister agent.b
        router.route(
            ev("mesh.deregister", "agent.b", serde_json::json!({ "agent_id": "agent.b" })),
            &ctx,
        ).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;
        // Drain any pending events for agent.a
        while mb_a.try_recv().is_ok() {}

        // Try to send to deregistered agent.b
        router.route(
            send("agent.a", "agent.b", serde_json::json!({ "text": "hello" })),
            &ctx,
        ).await.unwrap();
        flush(&mut outbox, &router, &ctx).await;

        let fail = recv(&mut mb_a).await;
        assert_eq!(fail.event_type, "mesh.send.failed");
        let p: serde_json::Value = serde_json::from_str(&fail.payload_json).unwrap();
        assert_eq!(p["reason"], "agent not found");
    }
}
