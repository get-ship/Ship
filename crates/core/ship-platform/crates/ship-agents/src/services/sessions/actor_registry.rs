//! Actor-backed session registry.
//!
//! Holds [`ship_substrate::supervisor::ProcessHandle`] (the unified spawn primitive)
//! plus a session-flavored [`SessionMeta`] sidecar. This is the destination
//! registry — `SessionRegistry` is the legacy parallel system being retired
//! kind-by-kind. When the last consumer flips, this is renamed to
//! `Registry` and `SessionRegistry` is deleted.
//!
//! Shape: `actor_id -> (ProcessHandle, Arc<Mutex<SessionMeta>>)`. The handle
//! owns process I/O (write_input, subscribe_output, stop). The sidecar
//! holds session-flavored state the HTTP layer reports — workspace,
//! agent/harness, status, exit code, harness_session_id — which doesn't
//! belong inside the supervisor primitive.
//!
//! Status updates flow from per-kind reader middleware (auto-dismiss for
//! agents, raw passthrough for shells) into the sidecar via the entry's
//! `meta` mutex.
//!
//! Insert is keyed by `ProcessHandle::actor_id`; the meta's `id` field must
//! match. Lookup, list, and find-by-* mirror `SessionRegistry` so HTTP
//! handlers can query whichever registry holds the session without
//! restructuring routes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use ship_harnesses::harness_adapter::HarnessEvent;
use ship_substrate::supervisor::ProcessHandle;
use std::path::PathBuf;
use tokio::sync::broadcast;

use ship_terminal::types::{SessionId, SessionMeta};

#[derive(Clone)]
pub struct ActorEntry {
    pub handle: ProcessHandle,
    pub meta: Arc<Mutex<SessionMeta>>,
    /// Harness-event broadcast for agent kinds. Reader middleware
    /// (harness adapter `parse_output`) publishes here; SSE consumers
    /// subscribe via `/api/sessions/{id}/events`. Shell entries leave
    /// this `None`; consumers receive `Closed` immediately.
    pub event_tx: Option<broadcast::Sender<HarnessEvent>>,
}

impl ActorEntry {
    /// Snapshot the session-flavored metadata.
    pub fn meta_snapshot(&self) -> SessionMeta {
        self.meta.lock().unwrap().clone()
    }

    /// Subscribe to raw PTY bytes — replaces `PtySession::subscribe`.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.handle.subscribe_output()
    }

    pub fn write_input(&self, bytes: &[u8]) -> Result<()> {
        self.handle.write_input(bytes)
    }

    pub fn stop(&self) -> Result<()> {
        self.handle.stop()
    }

    pub fn log_path(&self) -> PathBuf {
        self.handle.child.log_path().clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<HarnessEvent> {
        match &self.event_tx {
            Some(tx) => tx.subscribe(),
            None => {
                // No agent-side event source — return a closed receiver
                // (created from a fresh sender that's immediately dropped).
                let (tx, rx) = broadcast::channel::<HarnessEvent>(1);
                drop(tx);
                rx
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct ActorRegistry {
    inner: Arc<Mutex<HashMap<SessionId, ActorEntry>>>,
}

impl ActorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, handle: ProcessHandle, meta: SessionMeta) -> ActorEntry {
        self.insert_with(handle, meta, None)
    }

    pub fn insert_with(
        &self,
        handle: ProcessHandle,
        meta: SessionMeta,
        event_tx: Option<broadcast::Sender<HarnessEvent>>,
    ) -> ActorEntry {
        debug_assert_eq!(handle.actor_id, meta.id, "handle/meta id mismatch");
        let entry = ActorEntry {
            handle,
            meta: Arc::new(Mutex::new(meta)),
            event_tx,
        };
        self.inner
            .lock()
            .unwrap()
            .insert(entry.handle.actor_id.clone(), entry.clone());
        entry
    }

    pub fn get(&self, id: &str) -> Option<ActorEntry> {
        self.inner.lock().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &str) -> Option<ActorEntry> {
        self.inner.lock().unwrap().remove(id)
    }

    /// Snapshot every entry's meta, sorted by most recent activity.
    pub fn list(&self) -> Vec<SessionMeta> {
        let guard = self.inner.lock().unwrap();
        let mut metas: Vec<SessionMeta> = guard
            .values()
            .map(|e| e.meta.lock().unwrap().clone())
            .collect();
        metas.sort_by_key(|m| std::cmp::Reverse(m.last_activity));
        metas
    }

    pub fn find_agent_session(
        &self,
        workspace_id: &str,
        agent_id: &str,
        harness: &str,
    ) -> Option<ActorEntry> {
        let guard = self.inner.lock().unwrap();
        for entry in guard.values() {
            let meta = entry.meta.lock().unwrap();
            if meta.workspace_id == workspace_id
                && meta.agent_id.as_deref() == Some(agent_id)
                && meta.harness.as_deref() == Some(harness)
            {
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn find_by_harness_session(&self, harness_session_id: &str) -> Option<ActorEntry> {
        let guard = self.inner.lock().unwrap();
        for entry in guard.values() {
            let meta = entry.meta.lock().unwrap();
            if meta.harness_session_id.as_deref() == Some(harness_session_id) {
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }
}
