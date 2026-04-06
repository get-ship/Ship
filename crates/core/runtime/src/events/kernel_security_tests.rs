//! Tests for kernel_security.rs — enforce_emit_permissions, should_deliver,
//! event_matches_actor_scope, and deliver_with_scope edge cases.
//!
//! Complements scoped_delivery_tests (scope variants) and permission_tests
//! (emit/subscribe enforcement) with edge cases specific to the security module.

#[cfg(feature = "unstable")]
mod kernel_security {
    use crate::events::envelope::EventEnvelope;
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::{KernelRouter, MAILBOX_CAPACITY};
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use crate::events::validator::{CallerKind, EmitContext};
    use tempfile::tempdir;

    fn identity(label: &str) -> ActorIdentity {
        ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: label.into(),
            actor_type: ActorType::Agent,
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

    fn mcp_ctx() -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        }
    }

    fn setup() -> (tempfile::TempDir, KernelRouter) {
        let tmp = tempdir().unwrap();
        let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
        (tmp, router)
    }

    // ── enforce_emit_permissions edge cases ──────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn runtime_caller_no_actor_id_bypasses_emit_check() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![],
        };
        router
            .spawn_actor_with_permissions(identity("agent.x"), perms)
            .unwrap();

        // Runtime caller with no actor_id on the event — bypasses check.
        let event = ev("studio.anything");
        let result = router.route(event, &runtime_ctx()).await;
        assert!(result.is_ok(), "runtime caller with no actor_id must bypass emit check");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mcp_caller_unknown_actor_id_passes_through() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec!["studio.".into()],
            subscribe: vec![],
        };
        router
            .spawn_actor_with_permissions(identity("agent.known"), perms)
            .unwrap();

        // MCP caller with actor_id not in actor_meta — no meta means no restriction.
        let event = ev("studio.hack").with_actor_id("agent.unknown");
        let result = router.route(event, &mcp_ctx()).await;
        assert!(result.is_ok(), "unknown actor_id with no meta must pass through");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mcp_caller_with_no_actor_id_passes_through() {
        let (_tmp, router) = setup();

        // MCP caller, no actor_id on the event — the enforce_emit_permissions
        // only checks when actor_id is Some.
        let event = ev("custom.event");
        let result = router.route(event, &mcp_ctx()).await;
        assert!(result.is_ok(), "MCP caller with no actor_id must pass through");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_with_multiple_emit_namespaces() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec!["studio.".into(), "agent.".into()],
            subscribe: vec![],
        };
        router
            .spawn_actor_with_permissions(identity("agent.multi"), perms)
            .unwrap();

        let ev1 = ev("studio.annotation").with_actor_id("agent.multi");
        let ev2 = ev("agent.started").with_actor_id("agent.multi");
        let ev3 = ev("session.started").with_actor_id("agent.multi");

        assert!(router.route(ev1, &mcp_ctx()).await.is_ok());
        assert!(router.route(ev2, &mcp_ctx()).await.is_ok());
        assert!(router.route(ev3, &mcp_ctx()).await.is_err(), "must reject namespace outside both emit lists");
    }

    // ── should_deliver / event_matches_actor_scope edge cases ────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_with_multiple_subscriptions_different_scopes() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![
                PermittedSubscription {
                    namespace: "studio.".into(),
                    scope: DeliveryScope::Global,
                },
                PermittedSubscription {
                    namespace: "agent.".into(),
                    scope: DeliveryScope::Directed,
                },
            ],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.multi_sub"), perms)
            .unwrap();

        // Global studio event — should be delivered.
        let studio_ev = ev("studio.feedback");
        router.route(studio_ev.clone(), &runtime_ctx()).await.unwrap();

        // Undirected agent event — should NOT be delivered (directed scope).
        let agent_ev = ev("agent.task_started");
        router.route(agent_ev, &runtime_ctx()).await.unwrap();

        // Directed agent event TO this actor — should be delivered.
        let mut directed = ev("agent.message");
        directed.target_actor_id = Some("agent.multi_sub".into());
        router.route(directed.clone(), &runtime_ctx()).await.unwrap();

        let r1 = mb.try_recv().expect("must receive global studio event");
        assert_eq!(r1.id, studio_ev.id);
        let r2 = mb.try_recv().expect("must receive directed agent event");
        assert_eq!(r2.id, directed.id);
        assert!(mb.try_recv().is_err(), "must not receive undirected agent event");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn event_outside_all_subscription_namespaces_not_delivered() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.studio_only"), perms)
            .unwrap();

        // Route an event in a namespace the actor isn't subscribed to.
        router.route(ev("config.changed"), &runtime_ctx()).await.unwrap();

        assert!(mb.try_recv().is_err(), "event outside subscribed namespaces must not be delivered");
    }

    // ── deliver_with_scope: mailbox full returns error ────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mailbox_full_returns_error_and_emits_event() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };
        // Don't read from mailbox — let it fill up.
        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("agent.slow"), perms)
            .unwrap();

        // Fill the mailbox to capacity.
        for i in 0..MAILBOX_CAPACITY {
            let event = EventEnvelope::new(
                "studio.tick",
                "entity",
                &serde_json::json!({ "seq": i }),
            )
            .unwrap();
            router.route(event, &runtime_ctx()).await.unwrap();
        }

        // Next event should fail with mailbox full error.
        let overflow = ev("studio.overflow");
        let result = router.route(overflow, &runtime_ctx()).await;
        assert!(result.is_err(), "must return error when mailbox is full");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("mailbox full"), "error must mention mailbox full: {err_msg}");

        // Verify actor.mailbox_full event in kernel store.
        let mut conn = crate::db::open_db_at(router.kernel_store_path()).unwrap();
        let count: i64 = crate::db::block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE event_type = 'actor.mailbox_full'")
                .fetch_one(&mut conn)
                .await
        })
        .unwrap();
        assert!(count >= 1, "actor.mailbox_full event must be in kernel store");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mailbox_full_still_delivers_to_other_actors() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };

        // Actor A: will be full.
        let (_store_a, _mb_a) = router
            .spawn_actor_with_permissions(identity("agent.full"), perms.clone())
            .unwrap();
        // Actor B: will have capacity.
        let (_store_b, mut mb_b) = router
            .spawn_actor_with_permissions(identity("agent.fast"), perms)
            .unwrap();

        // Fill actor A's mailbox.
        for i in 0..MAILBOX_CAPACITY {
            let event = EventEnvelope::new(
                "studio.tick",
                "entity",
                &serde_json::json!({ "seq": i }),
            )
            .unwrap();
            // Drain B to keep it with capacity.
            let _ = router.route(event, &runtime_ctx()).await;
            let _ = mb_b.try_recv();
        }

        // Route one more — A is full, B has capacity.
        let event = ev("studio.final");
        let result = router.route(event.clone(), &runtime_ctx()).await;
        assert!(result.is_err(), "must error because A is full");

        // But B still received the event.
        let received = mb_b.try_recv().expect("actor B must still receive the event");
        assert_eq!(received.id, event.id);
    }
}
