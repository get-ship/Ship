//! Per-connection lifecycle: event relay and cleanup guard. The
//! in-shipd mesh.* op service was removed along with the mesh
//! REST/CLI surface; the kernel mailbox primitive
//! (`EventEnvelope.target_actor_id`) is the canonical inter-agent path.

use async_trait::async_trait;
use rmcp::model::CustomNotification;
use rmcp::{Peer, RoleServer, model::ServerNotification};
use ship_substrate::events::{EventEnvelope, Mailbox};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
// ---- Inbox ----

/// Shared inbox buffer for polling-based message retrieval.
/// The EventRelay writes here; `mesh_inbox` drains it.
pub struct Inbox {
    buf: RwLock<VecDeque<EventEnvelope>>,
}

impl Inbox {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { buf: RwLock::new(VecDeque::new()) })
    }

    pub async fn push(&self, event: EventEnvelope) {
        let mut buf = self.buf.write().await;
        buf.push_back(event);
        // Cap at 256 to prevent unbounded growth
        while buf.len() > 256 {
            buf.pop_front();
        }
    }

    /// Drain all pending messages.
    pub async fn drain(&self) -> Vec<EventEnvelope> {
        self.buf.write().await.drain(..).collect()
    }
}

// ---- EventSink ----

#[async_trait]
pub trait EventSink: Send + Sync + 'static {
    async fn send_event(&self, event: &EventEnvelope);
}

pub struct McpEventSink {
    peer: Peer<RoleServer>,
}

impl McpEventSink {
    pub fn new(peer: Peer<RoleServer>) -> Self {
        Self { peer }
    }
}

#[async_trait]
impl EventSink for McpEventSink {
    async fn send_event(&self, event: &EventEnvelope) {
        let params = match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => {
                warn!("failed to serialize event for MCP notification: {e}");
                return;
            }
        };
        let notification = CustomNotification::new("ship/event", Some(params));
        let server_notif = ServerNotification::CustomNotification(notification);
        if let Err(e) = self.peer.send_notification(server_notif).await {
            warn!("failed to send ship/event notification: {e}");
        }
    }
}

// ---- EventRelay ----

pub struct PeerHandle {
    pub sink: Box<dyn EventSink>,
}

/// Routes events from an actor mailbox to the connected MCP peer and inbox buffer.
pub struct EventRelay {
    peers: Arc<RwLock<Vec<PeerHandle>>>,
    inbox: Option<Arc<Inbox>>,
}

impl Default for EventRelay {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRelay {
    pub fn new() -> Self {
        Self { peers: Arc::new(RwLock::new(Vec::new())), inbox: None }
    }

    /// Attach an inbox so messages are buffered for polling via `mesh_inbox`.
    pub fn with_inbox(mut self, inbox: Arc<Inbox>) -> Self {
        self.inbox = Some(inbox);
        self
    }

    pub async fn add_peer(&self, handle: PeerHandle) {
        self.peers.write().await.push(handle);
    }

    /// Consume the mailbox in a background task, forwarding all events to peers.
    /// The task exits when the mailbox closes (actor stopped or sender dropped).
    pub fn spawn(self, mut mailbox: Mailbox) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(env) = mailbox.recv().await {
                // Buffer for polling
                if let Some(ref inbox) = self.inbox {
                    inbox.push(env.clone()).await;
                }
                // Push to SSE peers (best-effort)
                let peers = self.peers.read().await;
                for peer in peers.iter() {
                    peer.sink.send_event(&env).await;
                }
            }
        })
    }
}

// ---- ConnectionGuard ----

/// Cleanup guard held (via Arc) by every clone of a NetworkServer connection.
///
/// When the last clone is dropped (i.e. the HTTP session ends), Drop emits
/// `mesh.deregister` and removes the actor from the kernel router.
pub struct ConnectionGuard {
    pub actor_id: std::sync::Mutex<Option<String>>,
    pub relay_handle: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub kernel: Arc<tokio::sync::Mutex<ship_substrate::events::KernelRouter>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        // Abort relay task
        if let Ok(mut h) = self.relay_handle.lock()
            && let Some(handle) = h.take()
        {
            handle.abort();
        }
        // Emit deregister + stop actor (requires tokio runtime)
        let id = match self.actor_id.lock().ok().and_then(|mut g| g.take()) {
            Some(id) => id,
            None => return,
        };
        let kernel = self.kernel.clone();
        let id_clone = id.clone();
        let Ok(rt) = tokio::runtime::Handle::try_current() else { return };
        rt.spawn(async move {
            // Directed at the leaving actor — the actor itself sees
            // its own deregister confirmation; other peers don't get
            // a push about it. Symmetric with mesh.register.
            if let Ok(env) = EventEnvelope::new(
                "mesh.deregister",
                &id,
                &serde_json::json!({ "agent_id": &id }),
            )
            .map(|e| e.with_actor_id(&id).with_target(&id))
            {
                let ctx = ship_substrate::events::EmitContext {
                    caller_kind: ship_substrate::events::CallerKind::Mcp,
                    skill_id: None,
                    binding: ship_types::ScopeBinding::daemon_internal(
                        None,
                        ship_types::Authority::Sdk,
                    ),
                    session_id: None,
                };
                if let Err(e) = kernel.lock().await.emit(env, &ctx).await {
                    tracing::warn!(actor_id = %id_clone, "connection close: kernel emit failed: {e}");
                }
            }
            let _ = kernel.lock().await.stop_actor(&id_clone);
        });
    }
}

#[cfg(test)]
#[path = "connections_tests.rs"]
mod tests;