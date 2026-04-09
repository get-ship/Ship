//! Tests for mesh message scoping — directed delivery, broadcast filtering,
//! and write namespace enforcement at the router level.

#[cfg(feature = "unstable")]
mod mesh_scope {
    use crate::events::envelope::EventEnvelope;
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::KernelRouter;
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use crate::events::validator::{CallerKind, EmitContext};
    use tempfile::tempdir;

    fn id(label: &str) -> ActorIdentity {
        ActorIdentity { instance_id: ulid::Ulid::new().to_string(), label: label.into(), actor_type: ActorType::Agent }
    }
    fn ev(t: &str) -> EventEnvelope { EventEnvelope::new(t, "e", &serde_json::json!({})).unwrap() }
    fn rt() -> EmitContext { EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, workspace_id: None, session_id: None } }
    fn mcp() -> EmitContext { EmitContext { caller_kind: CallerKind::Mcp, skill_id: None, workspace_id: None, session_id: None } }
    fn setup() -> (tempfile::TempDir, KernelRouter) { let t = tempdir().unwrap(); let r = KernelRouter::new(t.path().join(".ship")).unwrap(); (t, r) }

    fn sub(ns: &str, scope: DeliveryScope) -> PermittedSubscription {
        PermittedSubscription { namespace: ns.into(), scope }
    }
    fn perms(emit: &[&str], subs: Vec<PermittedSubscription>) -> ActorPermissions {
        ActorPermissions { emit: emit.iter().map(|s| s.to_string()).collect(), subscribe: subs }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn direct_message_only_reaches_target() {
        let (_t, mut r) = setup();
        let p = perms(&[], vec![sub("mesh.", DeliveryScope::Global)]);
        let (_, mut ma) = r.spawn_actor_with_permissions(id("a"), p.clone()).unwrap();
        let (_, mut mb) = r.spawn_actor_with_permissions(id("b"), p.clone()).unwrap();
        let (_, mut mc) = r.spawn_actor_with_permissions(id("c"), p).unwrap();

        let msg = ev("mesh.hello").with_target("b");
        r.route(msg.clone(), &rt()).await.unwrap();

        assert!(ma.try_recv().is_err(), "A must not receive targeted message");
        assert_eq!(mb.try_recv().expect("B must receive").id, msg.id);
        assert!(mc.try_recv().is_err(), "C must not receive targeted message");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn broadcast_reaches_all_subscribers() {
        let (_t, mut r) = setup();
        let p = perms(&[], vec![sub("mesh.", DeliveryScope::Global)]);
        let (_, mut ma) = r.spawn_actor_with_permissions(id("a"), p.clone()).unwrap();
        let (_, mut mb) = r.spawn_actor_with_permissions(id("b"), p).unwrap();

        let msg = ev("mesh.update");
        r.route(msg.clone(), &rt()).await.unwrap();

        assert_eq!(ma.try_recv().unwrap().id, msg.id);
        assert_eq!(mb.try_recv().unwrap().id, msg.id);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_namespace_enforced() {
        let (_t, mut r) = setup();
        r.spawn_actor_with_permissions(id("a"), perms(&["mesh.a."], vec![])).unwrap();

        let event = ev("mesh.b.hello").with_actor_id("a");
        let result = r.route(event, &mcp()).await;
        assert!(result.is_err(), "must reject write outside permitted namespace");
        assert!(result.unwrap_err().to_string().contains("not permitted"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_to_own_namespace_succeeds() {
        let (_t, mut r) = setup();
        r.spawn_actor_with_permissions(id("a"), perms(&["mesh.a."], vec![])).unwrap();

        let result = r.route(ev("mesh.a.hello").with_actor_id("a"), &mcp()).await;
        assert!(result.is_ok(), "must allow write to own namespace");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn direct_message_to_nonexistent_actor_fails_gracefully() {
        let (_t, mut r) = setup();
        r.spawn_actor_with_permissions(id("a"), perms(&[], vec![sub("mesh.", DeliveryScope::Global)])).unwrap();

        let targeted = ev("mesh.ping").with_target("nonexistent");
        // Must not panic — either Ok (silently dropped) or Err with a clear message.
        let result = r.route(targeted, &rt()).await;
        assert!(result.is_ok() || !result.unwrap_err().to_string().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscription_namespace_filters_broadcasts() {
        let (_t, mut r) = setup();
        let (_, mut ma) = r.spawn_actor_with_permissions(id("a"), perms(&[], vec![sub("mesh.team-a.", DeliveryScope::Global)])).unwrap();
        let (_, mut mb) = r.spawn_actor_with_permissions(id("b"), perms(&[], vec![sub("mesh.team-b.", DeliveryScope::Global)])).unwrap();

        r.route(ev("mesh.team-a.update"), &rt()).await.unwrap();
        assert!(ma.try_recv().is_ok(), "A must receive team-a event");
        assert!(mb.try_recv().is_err(), "B must not receive team-a event");

        r.route(ev("mesh.team-b.update"), &rt()).await.unwrap();
        assert!(mb.try_recv().is_ok(), "B must receive team-b event");
        assert!(ma.try_recv().is_err(), "A must not receive team-b event");
    }
}
