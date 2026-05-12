//! Session types — shared between the registry, lifecycle functions,
//! and eventually the HTTP API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type SessionId = String;

/// What kind of process backs this session. Drives whether we expect a
/// provider adapter, how the parser runs, and how the UI labels it.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    /// A Ship agent running a provider CLI (Claude, Codex, …). Has an
    /// agent id and a provider id, output parses through the provider
    /// adapter.
    Agent,
    /// A pipeline step's child process. Registered so the pipelines
    /// view can live-stream output. Does not take user stdin.
    Pipeline,
    /// A plain shell (`$SHELL`) for the dock's terminal tab. No agent,
    /// no provider, no parsing — just raw passthrough.
    Shell,
}

/// Runtime status derived from PTY activity + exit state. Updated by
/// the reader thread / lifecycle machinery.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Process spawning but not yet running. Transient.
    Spawning,
    /// Waiting at a prompt, ready for input.
    Idle,
    /// Actively producing output (mid-response for agents).
    Thinking,
    /// Process ended successfully or unsuccessfully.
    Ended,
}

/// Serializable metadata snapshot for list / detail endpoints. The
/// live `PtySession` struct holds non-serializable bits (master handle,
/// child, subscribers) that the API never returns verbatim.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: SessionId,
    pub kind: SessionKind,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub status: SessionStatus,
    pub log_path: String,
    pub spawned_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Claude-side conversation id captured from the first
    /// SessionStarted event of the most recent turn. Used by the next
    /// `/message` call to continue the conversation via --resume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
}

/// Incoming spawn request. Agent-kind sessions require `agent_id` +
/// `provider`; shell kinds require none; pipeline kinds are registered
/// internally by the orchestrator, not over HTTP.
#[derive(Clone, Debug, Deserialize)]
pub struct SpawnRequest {
    pub workspace_id: String,
    pub kind: SessionKind,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
}

pub(crate) mod private {
    use super::*;
    use portable_pty::MasterPty;
    use ship_platform::services::provider_adapter::ProviderEvent;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tokio::sync::broadcast;

    /// Live session handle. Non-serializable; the API returns a
    /// [`SessionMeta`] snapshot built from this plus the header fields.
    pub struct Inner {
        pub meta: Mutex<SessionMeta>,
        /// Writer to the PTY master (stdin from the child's view).
        /// Shell sessions hold a live writer; agent sessions spawn
        /// fresh per-turn processes so they leave this `None`.
        /// portable-pty's `take_writer()` is single-use on the master,
        /// so we take it once at spawn time and hold it here.
        pub writer: Mutex<Option<Box<dyn std::io::Write + Send>>>,
        /// Keep the PTY master alive for the lifetime of the session.
        /// `None` for agent sessions (no PTY).
        pub _master: Mutex<Option<Box<dyn MasterPty + Send>>>,
        /// Raw-bytes broadcast for subscribers that want the full PTY
        /// stream (terminal view, debug logs).
        pub output_tx: broadcast::Sender<Vec<u8>>,
        /// Parsed-event broadcast for Agent sessions. Reader thread
        /// feeds lines through the provider adapter and publishes
        /// extracted `ProviderEvent`s here. Shell sessions ignore it.
        pub event_tx: broadcast::Sender<ProviderEvent>,
        /// Child process handle, used for graceful stop / try_wait().
        pub child: Mutex<Option<Box<dyn portable_pty::Child + Send + Sync>>>,
        /// Absolute path to the tee'd log file for replay.
        pub log_path: PathBuf,
    }

    pub type SessionHandle = Arc<Inner>;
}

pub use private::{Inner, SessionHandle};

/// Public read-only view of a session. Used wherever code needs to
/// observe or subscribe to a session without mutating its process
/// state.
#[derive(Clone)]
pub struct PtySession(pub SessionHandle);

impl PtySession {
    pub fn meta(&self) -> SessionMeta {
        self.0.meta.lock().unwrap().clone()
    }

    /// Subscribe to a live byte stream. Returns a receiver that will
    /// yield every chunk the reader thread publishes from now on.
    /// Historical bytes are in the log file (`meta().log_path`).
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Vec<u8>> {
        self.0.output_tx.subscribe()
    }

    /// Subscribe to parsed provider events (agent sessions only).
    /// Shell sessions return a receiver that will never produce.
    pub fn subscribe_events(
        &self,
    ) -> tokio::sync::broadcast::Receiver<ship_platform::services::provider_adapter::ProviderEvent> {
        self.0.event_tx.subscribe()
    }
}
