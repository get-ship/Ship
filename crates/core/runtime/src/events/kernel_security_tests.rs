//! Tests for kernel_security — identity, permissions, scoped delivery,
//! emit enforcement, and cursor-based replay.

#[cfg(feature = "unstable")]
mod kernel_security {
    use crate::events::envelope::EventEnvelope;
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::KernelRouter;
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use crate::events::validator::{CallerKind, EmitContext};
    use tempfile::tempdir;

    fn identity(label: &str, actor_type: ActorType) -> ActorIdentity {
        ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: label.into(),
            actor_type,
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

    fn cli_ctx() -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Cli,
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

    fn agent_perms(emit_ns: &[&str], sub_ns: &[(&str, DeliveryScope)]) -> ActorPermissions {
        ActorPermissions {
            emit: emit_ns.iter().map(|s| s.to_string()).collect(),
            subscribe: sub_ns
                .iter()
                .map(|(ns, scope)| PermittedSubscription {
                    namespace: ns.to_string(),
                    scope: scope.clone(),
                })
                .collect(),
        }
    }

    // ── Spawn with permissions ──────────────────────────────────────────────

    #[test]
    fn spawn_with_permissions_creates_actor() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&["agent."], &[("agent.", DeliveryScope::Global)]);
        let result = router.spawn_actor_with_permissions(
            identity("agent.test", ActorType::Agent),
            perms,
        );
        assert!(result.is_ok());
        assert!(router.has_actor("agent.test"));
    }

    #[test]
    fn spawn_with_permissions_rejects_duplicate() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&["agent."], &[("agent.", DeliveryScope::Global)]);
        router
            .spawn_actor_with_permissions(identity("agent.dup", ActorType::Agent), perms.clone())
            .unwrap();
        let result = router
            .spawn_actor_with_permissions(identity("agent.dup", ActorType::Agent), perms);
        match result {
            Err(e) => assert!(e.to_string().contains("already exists")),
            Ok(_) => panic!("expected error for duplicate actor"),
        }
    }

    #[test]
    fn spawn_with_permissions_validates_empty_emit_namespace() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec!["".to_string()],
            subscribe: vec![],
        };
        let result = router
            .spawn_actor_with_permissions(identity("agent.bad", ActorType::Agent), perms);
        match result {
            Err(e) => assert!(e.to_string().contains("empty namespace")),
            Ok(_) => panic!("expected validation error"),
        }
    }

    #[test]
    fn spawn_with_permissions_validates_empty_subscribe_namespace() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "".to_string(),
                scope: DeliveryScope::Global,
            }],
        };
        let result = router
            .spawn_actor_with_permissions(identity("agent.bad", ActorType::Agent), perms);
        match result {
            Err(e) => assert!(e.to_string().contains("empty namespace")),
            Ok(_) => panic!("expected validation error"),
        }
    }

    #[test]
    fn spawn_in_workspace_binds_workspace_id() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("studio.", DeliveryScope::Workspace)]);
        let result = router.spawn_actor_with_permissions_in_workspace(
            identity("agent.ws", ActorType::Agent),
            perms,
            "feature/login",
        );
        assert!(result.is_ok());
    }

    // ── Emit permission enforcement ─────────────────────────────────────────

    #[test]
    fn enforce_emit_cli_caller_bypasses() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[]); // No emit permissions at all
        router
            .spawn_actor_with_permissions(identity("agent.cli", ActorType::Agent), perms)
            .unwrap();

        let event = ev("workspace.created").with_actor_id("agent.cli");
        let result = router.enforce_emit_permissions(&event, &cli_ctx());
        assert!(result.is_ok(), "CLI callers must bypass emit enforcement");
    }

    #[test]
    fn enforce_emit_runtime_without_actor_id_bypasses() {
        let (_tmp, router) = setup();
        let event = ev("workspace.created"); // No actor_id set
        let result = router.enforce_emit_permissions(&event, &runtime_ctx());
        assert!(result.is_ok(), "Runtime events without actor_id must bypass");
    }

    #[test]
    fn enforce_emit_rejects_unpermitted_namespace() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&["agent."], &[]);
        router
            .spawn_actor_with_permissions(identity("agent.limited", ActorType::Agent), perms)
            .unwrap();

        let event = ev("workspace.created").with_actor_id("agent.limited");
        let result = router.enforce_emit_permissions(&event, &mcp_ctx());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not permitted"));
    }

    #[test]
    fn enforce_emit_allows_permitted_namespace() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&["agent."], &[]);
        router
            .spawn_actor_with_permissions(identity("agent.allowed", ActorType::Agent), perms)
            .unwrap();

        let event = ev("agent.task_done").with_actor_id("agent.allowed");
        let result = router.enforce_emit_permissions(&event, &mcp_ctx());
        assert!(result.is_ok());
    }

    #[test]
    fn enforce_emit_unknown_actor_passes() {
        let (_tmp, router) = setup();
        // Actor not registered with permissions — no meta to check
        let event = ev("agent.task").with_actor_id("agent.unknown");
        let result = router.enforce_emit_permissions(&event, &mcp_ctx());
        assert!(result.is_ok(), "unknown actors have no restrictions to enforce");
    }

    // ── event_matches_actor_scope ────────────────────────────────────────────

    #[test]
    fn scope_match_returns_none_for_legacy_actor() {
        let (_tmp, mut router) = setup();
        // Use spawn_actor (not spawn_actor_with_permissions) — no meta
        let config = crate::events::ActorConfig {
            namespace: "agent".into(),
            write_namespaces: vec!["agent.".into()],
            read_namespaces: vec!["agent.".into()],
            subscribe_namespaces: vec!["agent.".into()],
        };
        router.spawn_actor("agent.legacy", config).unwrap();

        let event = ev("agent.task");
        assert!(router.event_matches_actor_scope(&event, "agent.legacy").is_none());
    }

    #[test]
    fn scope_match_global_returns_true_for_any_event() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("workspace.", DeliveryScope::Global)]);
        router
            .spawn_actor_with_permissions(identity("sync", ActorType::Service), perms)
            .unwrap();

        let event = ev("workspace.created").with_context(Some("ws-a"), None);
        assert_eq!(router.event_matches_actor_scope(&event, "sync"), Some(true));
    }

    #[test]
    fn scope_match_workspace_matches_correct_workspace() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("workspace.", DeliveryScope::Workspace)]);
        router
            .spawn_actor_with_permissions_in_workspace(
                identity("agent.ws", ActorType::Agent),
                perms,
                "ws-mine",
            )
            .unwrap();

        let matching = ev("workspace.activated").with_context(Some("ws-mine"), None);
        assert_eq!(
            router.event_matches_actor_scope(&matching, "agent.ws"),
            Some(true)
        );

        let other = ev("workspace.activated").with_context(Some("ws-other"), None);
        assert_eq!(
            router.event_matches_actor_scope(&other, "agent.ws"),
            Some(false)
        );
    }

    #[test]
    fn scope_match_directed_matches_target_actor() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("mesh.", DeliveryScope::Directed)]);
        router
            .spawn_actor_with_permissions(identity("agent.target", ActorType::Agent), perms)
            .unwrap();

        let mut targeted = ev("mesh.message");
        targeted.target_actor_id = Some("agent.target".into());
        assert_eq!(
            router.event_matches_actor_scope(&targeted, "agent.target"),
            Some(true)
        );

        let untargeted = ev("mesh.message");
        assert_eq!(
            router.event_matches_actor_scope(&untargeted, "agent.target"),
            Some(false)
        );
    }

    #[test]
    fn scope_match_elevated_matches_only_elevated() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("workspace.", DeliveryScope::Elevated)]);
        router
            .spawn_actor_with_permissions(identity("sync-svc", ActorType::Service), perms)
            .unwrap();

        let elevated = ev("workspace.created").elevate();
        assert_eq!(
            router.event_matches_actor_scope(&elevated, "sync-svc"),
            Some(true)
        );

        let normal = ev("workspace.created");
        assert_eq!(
            router.event_matches_actor_scope(&normal, "sync-svc"),
            Some(false)
        );
    }

    #[test]
    fn scope_match_returns_false_for_unsubscribed_namespace() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("agent.", DeliveryScope::Global)]);
        router
            .spawn_actor_with_permissions(identity("agent.narrow", ActorType::Agent), perms)
            .unwrap();

        let event = ev("workspace.created"); // Not in agent. namespace
        assert_eq!(
            router.event_matches_actor_scope(&event, "agent.narrow"),
            Some(false)
        );
    }

    // ── should_deliver ──────────────────────────────────────────────────────

    #[test]
    fn should_deliver_legacy_actor_always_true() {
        let (_tmp, mut router) = setup();
        let config = crate::events::ActorConfig {
            namespace: "agent".into(),
            write_namespaces: vec!["agent.".into()],
            read_namespaces: vec!["agent.".into()],
            subscribe_namespaces: vec!["agent.".into()],
        };
        router.spawn_actor("agent.legacy", config).unwrap();

        let event = ev("agent.task");
        assert!(router.should_deliver(&event, "agent.legacy"));
    }

    #[test]
    fn should_deliver_permission_actor_respects_scope() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&[], &[("workspace.", DeliveryScope::Workspace)]);
        router
            .spawn_actor_with_permissions_in_workspace(
                identity("agent.scoped", ActorType::Agent),
                perms,
                "ws-1",
            )
            .unwrap();

        let matching = ev("workspace.created").with_context(Some("ws-1"), None);
        assert!(router.should_deliver(&matching, "agent.scoped"));

        let other = ev("workspace.created").with_context(Some("ws-2"), None);
        assert!(!router.should_deliver(&other, "agent.scoped"));
    }

    // ── deliver_with_scope ──────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deliver_targeted_event_only_to_target() {
        let (_tmp, mut router) = setup();
        let perms_a = agent_perms(&[], &[("mesh.", DeliveryScope::Global)]);
        let perms_b = agent_perms(&[], &[("mesh.", DeliveryScope::Global)]);

        let (_sa, mut mb_a) = router
            .spawn_actor_with_permissions(identity("agent.a", ActorType::Agent), perms_a)
            .unwrap();
        let (_sb, mut mb_b) = router
            .spawn_actor_with_permissions(identity("agent.b", ActorType::Agent), perms_b)
            .unwrap();

        let targeted = ev("mesh.message").with_target("agent.b");
        router.deliver_with_scope(&targeted).await;

        assert!(mb_a.try_recv().is_err(), "non-target must not receive");
        let received = mb_b.try_recv().expect("target must receive");
        assert_eq!(received.id, targeted.id);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deliver_broadcast_respects_workspace_scope() {
        let (_tmp, mut router) = setup();

        let perms_a = agent_perms(&[], &[("workspace.", DeliveryScope::Workspace)]);
        let perms_b = agent_perms(&[], &[("workspace.", DeliveryScope::Workspace)]);

        let (_sa, mut mb_a) = router
            .spawn_actor_with_permissions_in_workspace(
                identity("agent.ws1", ActorType::Agent),
                perms_a,
                "ws-alpha",
            )
            .unwrap();
        let (_sb, mut mb_b) = router
            .spawn_actor_with_permissions_in_workspace(
                identity("agent.ws2", ActorType::Agent),
                perms_b,
                "ws-beta",
            )
            .unwrap();

        let event = ev("workspace.activated").with_context(Some("ws-alpha"), None);
        router.deliver_with_scope(&event).await;

        let received = mb_a.try_recv().expect("workspace-matching actor must receive");
        assert_eq!(received.id, event.id);
        assert!(
            mb_b.try_recv().is_err(),
            "other-workspace actor must not receive"
        );
    }

    // ── Permission-based actor receives events via route ────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn permission_actor_receives_events_via_route() {
        let (_tmp, mut router) = setup();

        let perms = agent_perms(&[], &[("workspace.", DeliveryScope::Global)]);
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(
                identity("sync", ActorType::Service),
                perms,
            )
            .unwrap();

        let event = ev("workspace.created").with_context(Some("ws-1"), None);
        router.route(event.clone(), &runtime_ctx()).await.unwrap();

        let received = mb.try_recv().expect("permission actor must receive routed event");
        assert_eq!(received.id, event.id);
    }

    // ── Multiple permissions ────────────────────────────────────────────────

    #[test]
    fn actor_with_multiple_emit_namespaces() {
        let (_tmp, mut router) = setup();
        let perms = agent_perms(&["agent.", "mesh."], &[]);
        router
            .spawn_actor_with_permissions(identity("agent.multi", ActorType::Agent), perms)
            .unwrap();

        let ev_agent = ev("agent.task").with_actor_id("agent.multi");
        assert!(router.enforce_emit_permissions(&ev_agent, &mcp_ctx()).is_ok());

        let ev_mesh = ev("mesh.register").with_actor_id("agent.multi");
        assert!(router.enforce_emit_permissions(&ev_mesh, &mcp_ctx()).is_ok());

        let ev_ws = ev("workspace.created").with_actor_id("agent.multi");
        assert!(router.enforce_emit_permissions(&ev_ws, &mcp_ctx()).is_err());
    }

    #[test]
    fn actor_with_multiple_subscribe_scopes() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![
                PermittedSubscription {
                    namespace: "workspace.".into(),
                    scope: DeliveryScope::Global,
                },
                PermittedSubscription {
                    namespace: "mesh.".into(),
                    scope: DeliveryScope::Directed,
                },
            ],
        };
        router
            .spawn_actor_with_permissions(identity("agent.dual", ActorType::Agent), perms)
            .unwrap();

        // Global scope for workspace events — always matches.
        let ws_event = ev("workspace.created");
        assert_eq!(
            router.event_matches_actor_scope(&ws_event, "agent.dual"),
            Some(true)
        );

        // Directed scope for mesh events — only matches with target.
        let mesh_targeted = ev("mesh.message").with_target("agent.dual");
        assert_eq!(
            router.event_matches_actor_scope(&mesh_targeted, "agent.dual"),
            Some(true)
        );

        let mesh_untargeted = ev("mesh.message");
        assert_eq!(
            router.event_matches_actor_scope(&mesh_untargeted, "agent.dual"),
            Some(false)
        );
    }
}

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
