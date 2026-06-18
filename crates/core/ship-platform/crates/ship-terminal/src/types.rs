//! Session types — shared between the registry, lifecycle functions,
//! and eventually the HTTP API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type SessionId = String;

/// What kind of process backs this session. Drives whether we expect a
/// harness adapter, how the parser runs, and how the UI labels it.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    /// A Ship agent running a harness CLI (Claude, Codex, …). Has an
    /// agent id and a harness id, output parses through the harness
    /// adapter.
    Agent,
    /// A pipeline step's child process. Registered so the pipelines
    /// view can live-stream output. Does not take user stdin.
    Pipeline,
    /// A plain shell (`$SHELL`) for the dock's terminal tab. No agent,
    /// no harness, no parsing — just raw passthrough.
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
    pub harness: Option<String>,
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
    pub harness_session_id: Option<String>,
}

/// Incoming spawn request. Agent-kind sessions require `agent_id` +
/// `harness`; shell kinds require none; pipeline kinds are registered
/// internally by the orchestrator, not over HTTP.
#[derive(Clone, Debug, Deserialize)]
pub struct SpawnRequest {
    pub workspace_id: String,
    pub kind: SessionKind,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
}

pub mod private {
    use super::*;
    use ship_harnesses::harness_adapter::HarnessEvent;

    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tokio::sync::broadcast;

    /// Sidecar metadata for per-turn agent sessions registered on
    /// `SessionRegistry`. Holds only what the platform layer needs:
    /// metadata, the output and event broadcast channels, and a log
    /// path. The actual child process + PTY are owned by the substrate
    /// supervisor (`ChildHandle` via `PtyTransport` / `HeadlessTransport`)
    /// — this struct never holds a `MasterPty` or `Child` directly.
    pub struct Inner {
        pub meta: Mutex<SessionMeta>,
        /// Raw-bytes broadcast for subscribers (terminal view, debug logs).
        /// `run_headless_turn` republishes the substrate handle's bytes here.
        pub output_tx: broadcast::Sender<Vec<u8>>,
        /// Parsed `HarnessEvent` broadcast — populated by the reader
        /// pumping the substrate handle's output through the harness
        /// adapter.
        pub event_tx: broadcast::Sender<HarnessEvent>,
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

    /// Subscribe to parsed harness events (agent sessions only).
    /// Shell sessions return a receiver that will never produce.
    pub fn subscribe_events(
        &self,
    ) -> tokio::sync::broadcast::Receiver<ship_harnesses::harness_adapter::HarnessEvent> {
        self.0.event_tx.subscribe()
    }
}
