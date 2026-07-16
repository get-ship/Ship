//! Session types — shared between the registry, lifecycle functions,
//! and the HTTP API.

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
/// live `Entry` holds non-serializable bits (the supervisor
/// handle, the harness-event broadcast) that the API never returns
/// verbatim.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: SessionId,
    pub kind: SessionKind,
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
