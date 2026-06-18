//! In-memory session registry.
//!
//! Thread-safe, cloneable, lives for the lifetime of the daemon.
//! Exposes CRUD operations over session handles. Persistence across
//! daemon restart is deferred — a future sqlite mirror will capture
//! spawn metadata + log paths so a restart can reattach to orphaned
//! harness processes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::types::{PtySession, SessionHandle, SessionId, SessionMeta};

#[derive(Clone, Default)]
pub struct SessionRegistry {
    inner: Arc<Mutex<HashMap<SessionId, SessionHandle>>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, id: SessionId, handle: SessionHandle) {
        self.inner.lock().unwrap().insert(id, handle);
    }

    pub fn get(&self, id: &str) -> Option<PtySession> {
        self.inner
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .map(PtySession)
    }

    pub fn remove(&self, id: &str) -> Option<PtySession> {
        self.inner
            .lock()
            .unwrap()
            .remove(id)
            .map(PtySession)
    }

    /// List all active session metadata snapshots, sorted by most
    /// recent activity first.
    pub fn list(&self) -> Vec<SessionMeta> {
        let guard = self.inner.lock().unwrap();
        let mut metas: Vec<SessionMeta> = guard
            .values()
            .map(|h| h.meta.lock().unwrap().clone())
            .collect();
        metas.sort_by_key(|m| std::cmp::Reverse(m.last_activity));
        metas
    }

    /// Find an existing session for a `(workspace, agent, harness)`
    /// tuple. Used by spawn-or-attach flows so we don't duplicate.
    pub fn find_agent_session(
        &self,
        workspace_id: &str,
        agent_id: &str,
        harness: &str,
    ) -> Option<PtySession> {
        let guard = self.inner.lock().unwrap();
        for handle in guard.values() {
            let meta = handle.meta.lock().unwrap();
            if meta.workspace_id == workspace_id
                && meta.agent_id.as_deref() == Some(agent_id)
                && meta.harness.as_deref() == Some(harness)
            {
                return Some(PtySession(handle.clone()));
            }
        }
        None
    }

    /// Reverse-lookup: find the Ship session whose captured
    /// `harness_session_id` matches the harness's own session id.
    /// Used by hook endpoints to route `PreToolUse` / `PostToolUse`
    /// events (which carry the harness's session id) to the
    /// corresponding Ship chat bubble.
    pub fn find_by_harness_session(&self, harness_session_id: &str) -> Option<PtySession> {
        let guard = self.inner.lock().unwrap();
        for handle in guard.values() {
            let meta = handle.meta.lock().unwrap();
            if meta.harness_session_id.as_deref() == Some(harness_session_id) {
                return Some(PtySession(handle.clone()));
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
