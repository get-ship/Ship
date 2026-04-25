use runtime::events::{EventEnvelope, Mailbox};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::push::PushAdapter;

/// A connected agent peer with its allowed event types.
///
/// The peer's actor identity lives on its mailbox in the kernel —
/// directed-delivery events (those with `target_actor_id` set) only
/// land in the matching actor's mailbox, so per-peer session/actor
/// filtering at the relay is unnecessary. The relay only filters by
/// workspace and skill namespaces.
pub struct PeerHandle {
    pub adapter: Box<dyn PushAdapter>,
    /// Event types this agent is allowed to receive, derived from
    /// the agent's active skills' declared `in` events.
    /// Empty = system peer, receives all (e.g. internal projections).
    pub allowed_events: HashSet<String>,
    /// Workspace branch this peer is scoped to. System events with a
    /// workspace_id are only delivered if it matches this field.
    /// None = unscoped, receives system events for all workspaces.
    pub workspace_branch: Option<String>,
}

/// Routes events from an actor's `Mailbox` to connected agent peers.
///
/// Filtering happens HERE, not at the receiver. Each peer only receives
/// events that match its agent's active skill declarations. This enforces
/// least privilege — an agent without `visual-brainstorm` never sees
/// `visual-brainstorm.annotation_created`.
pub struct EventRelay {
    peers: Arc<RwLock<Vec<PeerHandle>>>,
}

impl EventRelay {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn peers(&self) -> Arc<RwLock<Vec<PeerHandle>>> {
        self.peers.clone()
    }

    pub async fn add_peer(&self, handle: PeerHandle) {
        self.peers.write().await.push(handle);
    }

    /// Spawn the relay loop consuming from a `Mailbox`.
    ///
    /// Returns `None` when the mailbox closes (all senders dropped).
    pub fn spawn(self, mut mailbox: Mailbox) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(env) = mailbox.recv().await {
                self.handle_event(&env).await;
            }
        })
    }

    async fn handle_event(&self, event: &EventEnvelope) {
        let peers = self.peers.read().await;
        tracing::info!(
            event_type = %event.event_type,
            target = ?event.target_actor_id,
            peers_n = peers.len(),
            "relay: handle_event"
        );
        for peer in peers.iter() {
            let receive = self.peer_should_receive(peer, event);
            tracing::info!(
                event_type = %event.event_type,
                adapter = peer.adapter.adapter_name(),
                receive,
                "relay: peer decision"
            );
            if receive {
                peer.adapter.push_event(event).await;
            }
        }
    }

    /// Filter at the relay, not at the receiver.
    ///
    /// System events (workspace.*, session.*, actor.*, config.*, gate.*)
    /// are scoped by workspace — a peer only receives system events that
    /// match its workspace_branch, or events with no workspace_id (global).
    ///
    /// Skill-namespaced events (e.g. visual-brainstorm.annotation_created)
    /// are only delivered if the peer's allowed_events contains that type.
    ///
    /// If allowed_events is empty, the peer is a system peer (receives all
    /// system events for its workspace).
    fn peer_should_receive(&self, peer: &PeerHandle, event: &EventEnvelope) -> bool {
        // Directed events are pre-filtered by the kernel router via
        // `target_actor_id` — only the matching actor's mailbox sees them.
        // No further workspace check is needed (and would be wrong: the
        // event's `workspace_id` is the slugged form, while the peer's
        // `workspace_branch` is the raw git branch name — they don't
        // compare).
        if event.target_actor_id.is_none() {
            // Broadcast event: apply workspace scoping so cross-workspace
            // peers don't see each other's traffic.
            if let (Some(event_ws), Some(peer_ws)) =
                (&event.workspace_id, &peer.workspace_branch)
                && event_ws != peer_ws
            {
                return false;
            }
        }

        // System peers receive all (workspace-matched) events
        if peer.allowed_events.is_empty() {
            return true;
        }

        // System namespace events go to all (workspace-matched) peers
        if is_system_event(&event.event_type) {
            return true;
        }

        // Skill-namespaced events: check if peer's skills declared this as `in`
        peer.allowed_events.contains(&event.event_type)
    }
}

/// System namespaces that all agents receive regardless of skill set.
/// `studio.` is included so Studio→agent events (visual messages, canvas
/// feedback) reach connected peers without skill declarations.
const SYSTEM_PREFIXES: &[&str] = &[
    "workspace.", "session.", "actor.", "config.",
    "gate.", "runtime.", "sync.", "project.",
    "studio.", "mesh.",
];

pub(crate) fn is_system_event(event_type: &str) -> bool {
    SYSTEM_PREFIXES.iter().any(|p| event_type.starts_with(p))
}

#[cfg(test)]
#[path = "notification_relay_tests.rs"]
mod tests;
