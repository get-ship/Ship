//! Per-view broadcast registry for out-of-band SUI SSE frames.
//!
//! The `<data live />` SSE channel (`/_sui/views/{name}/sse`) historically
//! carried only `patch-signals` events sourced from the kernel-event loop
//! inside [`crate::sse`]. Phase 0.5 adds `patch-elements`:
//! agents and SDK methods POST surgical HTML patches to
//! `/_sui/views/{name}/patch-elements` and the daemon needs a way to fan
//! those frames out to every open SSE connection on the same view.
//!
//! This module owns the bridge. Each view name maps to a single
//! `tokio::sync::broadcast::Sender<String>`; SSE handlers subscribe to
//! get a receiver, publish-side handlers (currently `sui_agent_push`)
//! call [`publish`] to send a pre-encoded SSE frame to all subscribers.
//!
//! # Trust boundary
//!
//! The publish side requires `Scope::Full` (enforced upstream). The
//! frames flowing through here are SSE-formatted text — the runtime
//! handler in `packages/sui/dom/src/attach.ts` is responsible for
//! validating selectors and applying patches against the view-scoped
//! root. This registry is a transport, not a policy enforcer.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;
use tokio::sync::{Mutex, broadcast};

/// Buffer size for the per-view broadcast channel. Small — patches are
/// fire-and-forget, late subscribers only see new frames.
const CHANNEL_CAPACITY: usize = 64;

type Registry = Mutex<HashMap<String, broadcast::Sender<String>>>;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// A subscription receiver that evicts its view's sender from the registry
/// when the LAST subscriber drops (#256-class fix).
///
/// The bare `HashMap<view, Sender>` was append-only: `subscribe` did
/// `entry().or_insert`, and nothing ever removed a sender — so over connection
/// churn (every SSE disconnect/reconnect on a view) the map grew without bound.
/// This guard derefs to the inner [`broadcast::Receiver`] (so call sites are
/// unchanged) and, on drop, spawns a short eviction task: it re-locks the
/// registry and removes the sender IFF it has no remaining receivers. Drop
/// can't `await` the async mutex directly, hence the detached task; the
/// `receiver_count() == 0` re-check under the lock closes the race with a
/// concurrent `subscribe` (which holds the same lock and would have bumped the
/// count, or creates a fresh sender after our removal).
pub struct ViewReceiver {
    inner: broadcast::Receiver<String>,
    view_name: String,
}

impl Deref for ViewReceiver {
    type Target = broadcast::Receiver<String>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ViewReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for ViewReceiver {
    fn drop(&mut self) {
        let view_name = std::mem::take(&mut self.view_name);
        tokio::spawn(async move {
            let mut reg = registry().lock().await;
            // Evict only when no receiver remains. `subscribe` holds this same
            // lock, so a sender we see with `receiver_count() == 0` here has no
            // live subscriber and is safe to drop; a later `subscribe` re-creates
            // it lazily.
            if reg.get(&view_name).is_some_and(|s| s.receiver_count() == 0) {
                reg.remove(&view_name);
            }
        });
    }
}

/// Subscribe to the broadcast channel for `view_name`. Lazily creates
/// the sender on first call; subsequent calls reuse it. Returns a
/// [`ViewReceiver`] that yields every frame published after subscription and
/// evicts the view's sender from the registry when the last subscriber drops.
pub async fn subscribe(view_name: &str) -> ViewReceiver {
    let mut reg = registry().lock().await;
    let sender = reg
        .entry(view_name.to_string())
        .or_insert_with(|| broadcast::channel::<String>(CHANNEL_CAPACITY).0);
    ViewReceiver {
        inner: sender.subscribe(),
        view_name: view_name.to_string(),
    }
}

/// Publish a pre-encoded SSE frame to every subscriber of `view_name`.
/// Returns the number of receivers the frame was delivered to (zero
/// when no SSE connection is open — that's not an error, agent pushes
/// to a non-watched view are still valid; they just have no audience).
pub async fn publish(view_name: &str, frame: String) -> usize {
    let reg = registry().lock().await;
    match reg.get(view_name) {
        Some(sender) => sender.send(frame).unwrap_or(0),
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subscribe_creates_lazy_channel() {
        let mut rx = subscribe("test-view-a").await;
        let n = publish("test-view-a", "frame-1".into()).await;
        assert_eq!(n, 1);
        let got = rx.recv().await.expect("frame");
        assert_eq!(got, "frame-1");
    }

    #[tokio::test]
    async fn publish_to_unsubscribed_view_returns_zero() {
        let n = publish("nobody-listens-here", "x".into()).await;
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn multiple_subscribers_each_receive_frame() {
        let mut rx1 = subscribe("test-view-b").await;
        let mut rx2 = subscribe("test-view-b").await;
        let n = publish("test-view-b", "hello".into()).await;
        assert_eq!(n, 2);
        assert_eq!(rx1.recv().await.unwrap(), "hello");
        assert_eq!(rx2.recv().await.unwrap(), "hello");
    }

    #[tokio::test]
    async fn dropping_last_subscriber_evicts_the_sender() {
        // #256-class: the registry must NOT grow without bound across
        // connection churn. After the last subscriber for a view drops, its
        // sender is evicted (publish to it then counts zero receivers, and the
        // map no longer retains the entry).
        let view = "test-view-evict";
        {
            let _rx = subscribe(view).await;
            assert!(
                registry().lock().await.contains_key(view),
                "sender registered while subscribed"
            );
        }
        // Drop spawns the eviction task; give it a turn to run.
        for _ in 0..50 {
            tokio::task::yield_now().await;
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
            if !registry().lock().await.contains_key(view) {
                break;
            }
        }
        assert!(
            !registry().lock().await.contains_key(view),
            "sender evicted once the last subscriber dropped"
        );
    }

    #[tokio::test]
    async fn sender_retained_while_a_subscriber_remains() {
        // Two subscribers; dropping one must NOT evict — the surviving receiver
        // still needs the sender. Eviction only fires when the LAST drops.
        let view = "test-view-retain";
        let _keep = subscribe(view).await;
        {
            let _drop = subscribe(view).await;
        }
        // Let any (incorrect) eviction task run.
        for _ in 0..10 {
            tokio::task::yield_now().await;
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
        assert!(
            registry().lock().await.contains_key(view),
            "sender retained while a subscriber survives"
        );
        // The surviving subscriber still receives frames.
        assert_eq!(publish(view, "still-here".into()).await, 1);
    }

    #[tokio::test]
    async fn views_are_isolated() {
        let mut rx_a = subscribe("test-view-c").await;
        let _rx_b = subscribe("test-view-d").await;
        publish("test-view-c", "to-c".into()).await;
        let got = rx_a.recv().await.unwrap();
        assert_eq!(got, "to-c");
        // rx_b would block; we don't await it here. The fact that
        // publish counted only one delivery is the cross-check below.
        let n = publish("test-view-c", "again".into()).await;
        assert_eq!(n, 1, "view-d subscriber should not have received view-c frame");
    }
}
