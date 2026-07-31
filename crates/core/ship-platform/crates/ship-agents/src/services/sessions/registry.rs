//! Live-session registry. Holds a [`bridge::ProcessHandle`]
//! (the unified spawn primitive) plus a session-flavored [`SessionRecord`]
//! sidecar per entry. Insert is keyed by `ProcessHandle::actor_id`; the
//! record's `id` field must match.
//!
//! Shape: `actor_id -> (ProcessHandle, Arc<Mutex<SessionRecord>>)`. The
//! handle owns process I/O (write_input, subscribe_output, stop). The
//! sidecar holds session-flavored state the HTTP layer reports —
//! workspace, agent/harness, status, exit code, harness_session_id —
//! which doesn't belong inside the supervisor primitive.
//!
//! `SessionRecord` is process-lifecycle bookkeeping only, kept as one
//! shape because both nouns are, underneath, OS processes the supervisor
//! tracks the same way. It is NOT the query surface: callers that want
//! "the agent sessions" or "the terminal sessions" go through
//! [`Registry::list_agent_sessions`] / [`Registry::list_terminal_sessions`],
//! which hand back the distinct [`super::types::AgentSessionMeta`] /
//! [`super::types::TerminalSessionMeta`] — never a mixed `Vec`, never a
//! kind tag the caller has to branch on.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bridge::HarnessEvent;
use bridge::ProcessHandle;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use tokio::sync::broadcast;

use super::types::{AgentSessionMeta, SessionId, SessionStatus, TerminalSessionMeta};

/// Process-lifecycle sidecar. Agent sessions carry `agent_id`/`harness`
/// (always both `Some` together); terminal sessions carry neither. The
/// noun is derived structurally from that shape — there is no kind tag.
#[derive(Clone, Debug)]
pub struct SessionRecord {
    pub id: SessionId,
    pub agent_id: Option<String>,
    pub harness: Option<String>,
    pub pid: Option<u32>,
    pub status: SessionStatus,
    pub log_path: String,
    pub spawned_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub harness_session_id: Option<String>,
}

impl SessionRecord {
    /// Build the agent-session query type. `None` for a terminal record.
    pub fn as_agent_meta(&self) -> Option<AgentSessionMeta> {
        Some(AgentSessionMeta {
            id: self.id.clone(),
            agent_id: self.agent_id.clone()?,
            harness: self.harness.clone()?,
            pid: self.pid,
            status: self.status,
            log_path: self.log_path.clone(),
            spawned_at: self.spawned_at,
            last_activity: self.last_activity,
            exit_code: self.exit_code,
            harness_session_id: self.harness_session_id.clone(),
        })
    }

    /// Build the terminal-session query type. `None` for an agent record.
    pub fn as_terminal_meta(&self) -> Option<TerminalSessionMeta> {
        if self.agent_id.is_some() {
            return None;
        }
        Some(TerminalSessionMeta {
            id: self.id.clone(),
            pid: self.pid,
            status: self.status,
            log_path: self.log_path.clone(),
            spawned_at: self.spawned_at,
            last_activity: self.last_activity,
            exit_code: self.exit_code,
        })
    }
}

#[derive(Clone)]
pub struct Entry {
    pub handle: ProcessHandle,
    pub meta: Arc<Mutex<SessionRecord>>,
    /// Harness-event broadcast for agent sessions. Reader middleware
    /// (harness adapter `parse_output`) publishes here; SSE consumers
    /// subscribe via `/api/sessions/{id}/events`. Terminal entries leave
    /// this `None`; consumers receive `Closed` immediately.
    pub event_tx: Option<broadcast::Sender<HarnessEvent>>,
}

impl Entry {
    /// Snapshot the session-flavored metadata.
    pub fn meta_snapshot(&self) -> SessionRecord {
        self.meta.lock().clone()
    }

    /// Subscribe to raw PTY bytes.
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
                let (tx, rx) = broadcast::channel::<HarnessEvent>(1);
                drop(tx);
                rx
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct Registry {
    inner: Arc<Mutex<HashMap<SessionId, Entry>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, handle: ProcessHandle, meta: SessionRecord) -> Entry {
        self.insert_with(handle, meta, None)
    }

    pub fn insert_with(
        &self,
        handle: ProcessHandle,
        meta: SessionRecord,
        event_tx: Option<broadcast::Sender<HarnessEvent>>,
    ) -> Entry {
        debug_assert_eq!(handle.actor_id, meta.id, "handle/meta id mismatch");
        let entry = Entry {
            handle,
            meta: Arc::new(Mutex::new(meta)),
            event_tx,
        };
        self.inner
            .lock()
            .insert(entry.handle.actor_id.clone(), entry.clone());
        entry
    }

    pub fn get(&self, id: &str) -> Option<Entry> {
        self.inner.lock().get(id).cloned()
    }

    pub fn remove(&self, id: &str) -> Option<Entry> {
        self.inner.lock().remove(id)
    }

    /// Snapshot every live session's raw record, sorted by most recent
    /// activity, spanning BOTH nouns. Reserved for the wire-boundary
    /// adapter that genuinely needs a heterogeneous "everything running"
    /// view (the terminal dock's process list) and for kind-agnostic
    /// lifecycle sweeps. Any caller that wants "the agent sessions" or
    /// "the terminal sessions" must use the noun-specific list below —
    /// this is not that entry point.
    pub fn list_all(&self) -> Vec<SessionRecord> {
        let guard = self.inner.lock();
        let mut records: Vec<SessionRecord> =
            guard.values().map(|e| e.meta.lock().clone()).collect();
        records.sort_by_key(|m| std::cmp::Reverse(m.last_activity));
        records
    }

    /// Snapshot every **agent session**, sorted by most recent activity.
    /// Never returns a terminal session.
    pub fn list_agent_sessions(&self) -> Vec<AgentSessionMeta> {
        let guard = self.inner.lock();
        let mut metas: Vec<AgentSessionMeta> = guard
            .values()
            .filter_map(|e| e.meta.lock().as_agent_meta())
            .collect();
        metas.sort_by_key(|m| std::cmp::Reverse(m.last_activity));
        metas
    }

    /// Snapshot every **terminal session**, sorted by most recent
    /// activity. Never returns an agent session.
    pub fn list_terminal_sessions(&self) -> Vec<TerminalSessionMeta> {
        let guard = self.inner.lock();
        let mut metas: Vec<TerminalSessionMeta> = guard
            .values()
            .filter_map(|e| e.meta.lock().as_terminal_meta())
            .collect();
        metas.sort_by_key(|m| std::cmp::Reverse(m.last_activity));
        metas
    }

    pub fn find_agent_session(&self, agent_id: &str, harness: &str) -> Option<Entry> {
        let guard = self.inner.lock();
        for entry in guard.values() {
            let meta = entry.meta.lock();
            if meta.agent_id.as_deref() == Some(agent_id)
                && meta.harness.as_deref() == Some(harness)
            {
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn find_by_harness_session(&self, harness_session_id: &str) -> Option<Entry> {
        let guard = self.inner.lock();
        for entry in guard.values() {
            let meta = entry.meta.lock();
            if meta.harness_session_id.as_deref() == Some(harness_session_id) {
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }
}
