use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

// ---- Session types ---------------------------------------------------------

/// Computed liveness state derived from last_activity_at.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum SessionLiveness {
    /// Tool calls within TTL window.
    Active,
    /// No tool calls for TTL period, process may be alive.
    Idle,
    /// Session ended by scanner due to process death.
    Abandoned,
}

/// Default TTL for session liveness: 10 minutes.
const SESSION_TTL_SECS: i64 = 600;

impl SessionLiveness {
    /// Compute liveness from last_activity_at for an active session.
    pub fn from_activity(last_activity_at: Option<&DateTime<Utc>>) -> Self {
        let Some(last_activity) = last_activity_at else {
            return SessionLiveness::Active; // No heartbeat yet, just started
        };
        let elapsed = Utc::now().signed_duration_since(*last_activity);
        if elapsed.num_seconds() > SESSION_TTL_SECS {
            SessionLiveness::Idle
        } else {
            SessionLiveness::Active
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    #[default]
    Active,
    Ended,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Ended => write!(f, "ended"),
        }
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "active" => Ok(SessionStatus::Active),
            "ended" => Ok(SessionStatus::Ended),
            _ => Err(anyhow::anyhow!(
                "Invalid workspace session status: {}",
                value
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Session {
    pub id: String,
    pub workspace_id: String,
    pub branch: String,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_harness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_workspace_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liveness: Option<SessionLiveness>,
    // ── End-of-session telemetry (set on session.ended; None while active) ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_changed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_added: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_removed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---- Request / report types ------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct EndSessionRequest {
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub model: Option<String>,
    pub files_changed: Option<i64>,
    pub gate_result: Option<String>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
    pub tool_calls: Option<i64>,
    pub exit_reason: Option<String>,
}

