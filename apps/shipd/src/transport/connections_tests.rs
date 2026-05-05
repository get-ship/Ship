//! Tests for per-connection lifecycle: Inbox, EventRelay, ConnectionGuard.

use ship_substrate::events::EventEnvelope;
use std::sync::Arc;

use crate::transport::connections::Inbox;

fn test_event(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

// ── Inbox ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn inbox_drain_returns_all_pushed_events() {
    let inbox = Inbox::new();
    inbox.push(test_event("mesh.message")).await;
    inbox.push(test_event("mesh.broadcast")).await;

    let drained = inbox.drain().await;
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].event_type, "mesh.message");
    assert_eq!(drained[1].event_type, "mesh.broadcast");
}

#[tokio::test]
async fn inbox_drain_empties_buffer() {
    let inbox = Inbox::new();
    inbox.push(test_event("mesh.message")).await;
    inbox.drain().await;

    let second = inbox.drain().await;
    assert!(second.is_empty(), "drain must empty the buffer");
}

#[tokio::test]
async fn inbox_caps_at_256() {
    let inbox = Inbox::new();
    for i in 0..300 {
        inbox
            .push(test_event(&format!("mesh.event.{i}")))
            .await;
    }

    let drained = inbox.drain().await;
    assert_eq!(drained.len(), 256, "inbox must cap at 256 events");
    // Oldest events should have been evicted.
    assert_eq!(
        drained[0].event_type, "mesh.event.44",
        "oldest events must be evicted first"
    );
}

#[tokio::test]
async fn inbox_preserves_insertion_order() {
    let inbox = Inbox::new();
    inbox.push(test_event("first")).await;
    inbox.push(test_event("second")).await;
    inbox.push(test_event("third")).await;

    let drained = inbox.drain().await;
    assert_eq!(drained[0].event_type, "first");
    assert_eq!(drained[1].event_type, "second");
    assert_eq!(drained[2].event_type, "third");
}

#[tokio::test]
async fn inbox_new_returns_arc() {
    let inbox = Inbox::new();
    let clone = Arc::clone(&inbox);
    clone.push(test_event("shared")).await;

    let drained = inbox.drain().await;
    assert_eq!(drained.len(), 1);
}

// ── EventRelay ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_relay_forwards_to_inbox() {
    use crate::transport::connections::EventRelay;
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel::<EventEnvelope>(16);
    let mailbox = ship_substrate::events::Mailbox::from_receiver(rx);

    let inbox = Inbox::new();
    let relay = EventRelay::new().with_inbox(Arc::clone(&inbox));
    let handle = relay.spawn(mailbox);

    tx.send(test_event("relayed.event")).await.unwrap();

    // Give the relay task time to process.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let drained = inbox.drain().await;
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].event_type, "relayed.event");

    drop(tx);
    handle.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_relay_exits_when_mailbox_closes() {
    use crate::transport::connections::EventRelay;
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel::<EventEnvelope>(16);
    let mailbox = ship_substrate::events::Mailbox::from_receiver(rx);

    let relay = EventRelay::new();
    let handle = relay.spawn(mailbox);

    drop(tx);
    // The relay task should exit cleanly.
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        handle,
    )
    .await;
    assert!(result.is_ok(), "relay must exit when mailbox closes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_relay_forwards_multiple_events() {
    use crate::transport::connections::EventRelay;
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel::<EventEnvelope>(16);
    let mailbox = ship_substrate::events::Mailbox::from_receiver(rx);

    let inbox = Inbox::new();
    let relay = EventRelay::new().with_inbox(Arc::clone(&inbox));
    let handle = relay.spawn(mailbox);

    for i in 0..5 {
        tx.send(test_event(&format!("event.{i}"))).await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let drained = inbox.drain().await;
    assert_eq!(drained.len(), 5);

    drop(tx);
    handle.await.unwrap();
}

// ── EventRelay with peers ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_relay_adds_peer_and_forwards() {
    use crate::transport::connections::{EventRelay, EventSink, PeerHandle};
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    struct TestSink {
        events: Arc<tokio::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl EventSink for TestSink {
        async fn send_event(&self, event: &EventEnvelope) {
            self.events.lock().await.push(event.event_type.clone());
        }
    }

    let (tx, rx) = mpsc::channel::<EventEnvelope>(16);
    let mailbox = ship_substrate::events::Mailbox::from_receiver(rx);

    let relay = EventRelay::new();
    let sink_events = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    relay
        .add_peer(PeerHandle {
            sink: Box::new(TestSink {
                events: sink_events.clone(),
            }),
        })
        .await;
    let handle = relay.spawn(mailbox);

    tx.send(test_event("peer.event")).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let received = sink_events.lock().await;
    assert_eq!(received.len(), 1);
    assert_eq!(received[0], "peer.event");

    drop(tx);
    handle.await.unwrap();
}
