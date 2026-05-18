//! ship-kernel: pure event routing with no persistence implementation.
//!
//! This crate defines the kernel's routing core — actor registration,
//! namespace-based subscriptions, event validation, and sink-based persistence.
//! It has zero filesystem or database dependencies.

pub mod mailbox;
pub mod router;
pub mod sink;
pub mod validator;

pub use mailbox::Mailbox;
pub use router::{ActorConfig, KernelRouter};
pub use sink::EventSink;
pub use validator::EventValidator;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("actor '{actor_id}' not found")]
    ActorNotFound { actor_id: String },
    #[error("actor '{actor_id}' already exists")]
    ActorAlreadyExists { actor_id: String },
    #[error("validation failed: {reason}")]
    ValidationFailed { reason: String },
    #[error("namespace violation: '{namespace}' not in write scope for '{actor_id}'")]
    NamespaceViolation { namespace: String, actor_id: String },
    #[error("mailbox full for actor '{actor_id}'")]
    MailboxFull { actor_id: String },
    #[error("persistence error: {0}")]
    Persistence(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        ActorConfig, EventSink, KernelError, KernelRouter, Mailbox,
    };
    use ship_types::envelope::EventEnvelope;

    // ── MockSink ─────────────────────────────────────────────────────────

    struct MockSink {
        events: Mutex<Vec<EventEnvelope>>,
    }

    impl MockSink {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }

        fn captured(&self) -> Vec<EventEnvelope> {
            self.events.lock().unwrap().clone()
        }
    }

    impl EventSink for MockSink {
        fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    fn ev(event_type: &str) -> EventEnvelope {
        EventEnvelope::new(event_type, "entity-1", &serde_json::json!({}))
            .unwrap()
    }

    fn test_config(ns: &str) -> ActorConfig {
        ActorConfig {
            namespace: ns.into(),
            write_namespaces: vec![format!("{ns}.")],
            read_namespaces: vec![format!("{ns}.")],
            subscribe_namespaces: vec![format!("{ns}.")],
            see_directed_events: false,
        }
    }

    fn audience_config(ns: &str) -> ActorConfig {
        ActorConfig {
            namespace: ns.into(),
            write_namespaces: vec![format!("{ns}.")],
            read_namespaces: vec![format!("{ns}.")],
            subscribe_namespaces: vec![format!("{ns}.")],
            see_directed_events: true,
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────

    #[test]
    fn event_sink_trait_exists() {
        let sink = MockSink::new();
        let event = ev("test.hello");
        sink.persist(&event).unwrap();
        assert_eq!(sink.captured().len(), 1);
        assert_eq!(sink.captured()[0].event_type, "test.hello");
    }

    #[test]
    fn kernel_router_new_takes_sink() {
        let sink = Arc::new(MockSink::new());
        let _router = KernelRouter::new(sink);
    }

    #[test]
    fn spawn_actor_returns_mailbox() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let result = router.spawn_actor("test-actor", test_config("test"));
        let _mailbox: Mailbox = result.unwrap();
    }

    #[test]
    fn spawn_duplicate_actor_fails() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        router.spawn_actor("x", test_config("x")).unwrap();
        let err = router.spawn_actor("x", test_config("x")).unwrap_err();
        assert!(matches!(err, KernelError::ActorAlreadyExists { .. }));
    }

    #[test]
    fn stop_actor_removes_from_router() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        router.spawn_actor("x", test_config("x")).unwrap();
        assert!(router.has_actor("x"));
        router.stop_actor("x").unwrap();
        assert!(!router.has_actor("x"));
    }

    #[tokio::test]
    async fn route_persists_via_sink() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink.clone());
        router.spawn_actor("a", test_config("test")).unwrap();
        router.route(ev("test.hello")).await.unwrap();
        assert!(!sink.captured().is_empty());
    }

    #[tokio::test]
    async fn route_delivers_to_subscriber() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb = router.spawn_actor("a", test_config("mesh")).unwrap();
        router.route(ev("mesh.hello")).await.unwrap();
        let received = mb.try_recv().expect("mailbox should have event");
        assert_eq!(received.event_type, "mesh.hello");
    }

    #[tokio::test]
    async fn direct_message_only_reaches_target() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb_a = router.spawn_actor("A", test_config("mesh")).unwrap();
        let mut mb_b = router.spawn_actor("B", test_config("mesh")).unwrap();

        let mut event = ev("mesh.hello");
        event.target_actor_id = Some("B".into());
        router.route(event).await.unwrap();

        assert!(mb_a.try_recv().is_err(), "A should not receive targeted msg");
        assert!(mb_b.try_recv().is_ok(), "B should receive targeted msg");
    }

    #[tokio::test]
    async fn refine_actor_preserves_mailbox() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb = router.spawn_actor("a", test_config("mesh")).unwrap();
        let new_cfg = ActorConfig {
            namespace: "a".into(),
            write_namespaces: vec!["other.".into()],
            read_namespaces: vec!["other.".into()],
            subscribe_namespaces: vec!["other.".into()],
            see_directed_events: false,
        };
        router.refine_actor("a", new_cfg).unwrap();

        let mut event = ev("other.hello");
        event.target_actor_id = Some("a".into());
        router.route(event).await.unwrap();
        let received = mb.try_recv().expect("mailbox should receive directed event");
        assert_eq!(received.event_type, "other.hello");
    }

    #[tokio::test]
    async fn refine_actor_rekeys_subscriptions() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let cfg_a = ActorConfig {
            namespace: "x".into(),
            write_namespaces: vec!["a.".into()],
            read_namespaces: vec!["a.".into()],
            subscribe_namespaces: vec!["a.".into()],
            see_directed_events: false,
        };
        let mut mb = router.spawn_actor("x", cfg_a).unwrap();

        // Confirm initial subscription on a.* works.
        router.route(ev("a.first")).await.unwrap();
        assert!(mb.try_recv().is_ok());

        let cfg_b = ActorConfig {
            namespace: "x".into(),
            write_namespaces: vec!["b.".into()],
            read_namespaces: vec!["b.".into()],
            subscribe_namespaces: vec!["b.".into()],
            see_directed_events: false,
        };
        router.refine_actor("x", cfg_b).unwrap();

        // a.* no longer reaches the actor.
        router.route(ev("a.second")).await.unwrap();
        assert!(mb.try_recv().is_err(), "a.* must not reach after refinement");

        // b.* now reaches the actor.
        router.route(ev("b.first")).await.unwrap();
        let recv = mb.try_recv().expect("b.* must reach after refinement");
        assert_eq!(recv.event_type, "b.first");
    }

    #[test]
    fn refine_actor_errors_on_unknown() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let err = router.refine_actor("ghost", test_config("g")).unwrap_err();
        assert!(matches!(err, KernelError::ActorNotFound { .. }));
    }

    #[test]
    fn ship_kernel_has_no_sqlx() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            !cargo_toml.contains("sqlx"),
            "ship-kernel must not depend on sqlx"
        );
    }

    #[test]
    fn validator_rejects_invalid_events() {
        let sink = Arc::new(MockSink::new());
        let router = KernelRouter::new(sink)
            .with_validator(Box::new(|event: &EventEnvelope| {
                if event.event_type.starts_with("bad.") {
                    Err(KernelError::ValidationFailed {
                        reason: "rejected".into(),
                    })
                } else {
                    Ok(())
                }
            }));

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(router.route(ev("bad.event")));
        assert!(result.is_err());
    }

    // ── see_directed_events audience semantics ───────────────────────────

    #[tokio::test]
    async fn directed_event_reaches_target_and_audience_subscriber() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb_target = router.spawn_actor("target", test_config("mesh")).unwrap();
        let mut mb_audience = router
            .spawn_actor("audit", audience_config("mesh"))
            .unwrap();

        let mut event = ev("mesh.hello");
        event.target_actor_id = Some("target".into());
        router.route(event).await.unwrap();

        assert!(
            mb_target.try_recv().is_ok(),
            "target must receive the directed event"
        );
        assert!(
            mb_audience.try_recv().is_ok(),
            "audience subscriber with see_directed_events must also receive it"
        );
    }

    #[tokio::test]
    async fn directed_event_skips_non_audience_subscriber() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb_target = router.spawn_actor("target", test_config("mesh")).unwrap();
        let mut mb_observer = router
            .spawn_actor("observer", test_config("mesh"))
            .unwrap();

        let mut event = ev("mesh.hello");
        event.target_actor_id = Some("target".into());
        router.route(event).await.unwrap();

        assert!(mb_target.try_recv().is_ok());
        assert!(
            mb_observer.try_recv().is_err(),
            "subscriber WITHOUT see_directed_events must not see directed events (regression: chat actors must stay isolated)"
        );
    }

    #[tokio::test]
    async fn audience_subscriber_namespace_filter_still_applies() {
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb_target = router.spawn_actor("target", test_config("mesh")).unwrap();
        // Audience subscribes to `other.` — directed event on `mesh.`
        // must NOT reach it.
        let mut mb_audience = router
            .spawn_actor("audit", audience_config("other"))
            .unwrap();

        let mut event = ev("mesh.hello");
        event.target_actor_id = Some("target".into());
        router.route(event).await.unwrap();

        assert!(mb_target.try_recv().is_ok());
        assert!(
            mb_audience.try_recv().is_err(),
            "audience must respect its namespace filter even with see_directed_events"
        );
    }

    #[tokio::test]
    async fn broadcast_event_reaches_audience_subscriber_same_as_others() {
        // Sanity: see_directed_events doesn't change broadcast semantics.
        let sink = Arc::new(MockSink::new());
        let mut router = KernelRouter::new(sink);
        let mut mb_a = router.spawn_actor("a", test_config("mesh")).unwrap();
        let mut mb_b = router.spawn_actor("b", audience_config("mesh")).unwrap();
        router.route(ev("mesh.hello")).await.unwrap();
        assert!(mb_a.try_recv().is_ok());
        assert!(mb_b.try_recv().is_ok());
    }
}
