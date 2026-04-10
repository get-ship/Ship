// ship-kernel: pure event routing with no persistence implementation.
// This crate defines the kernel interface. Implementation follows.

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
}
