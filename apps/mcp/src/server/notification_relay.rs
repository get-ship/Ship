use runtime::events::{EventEnvelope, Mailbox};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::push::PushAdapter;

/// A connected agent peer with its allowed event types.
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
    /// Ship chat session id this peer was spawned against (from the
    /// `SHIP_SESSION_ID` env var the daemon sets at spawn time). When
    /// set, events carrying a `session_id` only reach peers whose
    /// `session_id` matches — prevents cross-talk when a user has
    /// multiple chat sessions open in the same workspace.
    pub session_id: Option<String>,
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
        for peer in peers.iter() {
            if self.peer_should_receive(peer, event) {
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
        // Workspace scoping: if the event has a workspace_id and the peer
        // is scoped to a workspace, only deliver if they match.
        if let (Some(event_ws), Some(peer_ws)) =
            (&event.workspace_id, &peer.workspace_branch)
            && event_ws != peer_ws
        {
            return false;
        }

        // Session scoping: session-targeted events only reach the peer
        // spawned for that exact session. Used by `studio.chat_turn` so
        // a user turn routes to the right Claude when several agents
        // share a workspace.
        if let (Some(event_sid), Some(peer_sid)) =
            (&event.session_id, &peer.session_id)
            && event_sid != peer_sid
        {
            return false;
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
