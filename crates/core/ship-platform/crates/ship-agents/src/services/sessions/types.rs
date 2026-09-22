//! Session types — shared between the registry, lifecycle functions,
//! and the HTTP API.
//!
//! Two distinct nouns live here, never one shared struct behind a kind
//! tag: an **agent session** (a Ship agent running a harness CLI — has
//! an agent id and a harness id, output parses through the harness
//! adapter) and a **terminal session** (a plain `$SHELL` for the dock's
//! terminal tab — no agent, no harness, no parsing, just raw
//! passthrough). A query for one noun must never return the other.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type SessionId = String;

/// Runtime status derived from PTY activity + exit state. Updated by
/// the reader thread / lifecycle machinery. Shared vocabulary — not
/// overloaded — both nouns above go through the same lifecycle states.
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

/// Serializable metadata snapshot for an **agent session** — a Ship
/// agent running a harness CLI (Claude, Codex, …). Returned only by
/// agent-session query entry points; never mixed with terminal
/// sessions in a result set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSessionMeta {
    pub id: SessionId,
    pub agent_id: String,
    pub harness: String,
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

/// Serializable metadata snapshot for a **terminal session** — a plain
/// `$SHELL` backing the dock's terminal tab. No agent id, no harness,
/// no harness-session id: raw passthrough only.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalSessionMeta {
    pub id: SessionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub status: SessionStatus,
    pub log_path: String,
    pub spawned_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}
