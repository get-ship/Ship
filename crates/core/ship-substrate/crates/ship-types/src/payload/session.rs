// -- Session aggregate ---------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionStarted {
    pub goal: Option<String>,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub primary_harness: Option<String>,
    /// Project this session belongs to. Looked up at start from the
    /// actor's space_id or the workspace path. None for sessions that
    /// have no associated project (daemon services, ephemeral runs).
    #[serde(default)]
    pub space_id: Option<String>,
    /// Entry-point discriminator: `pipeline_step | chat_thread |
    /// direct_attach`. Defaults to `direct_attach` when not specified.
    #[serde(default)]
    pub kind: Option<String>,
    /// Set when this session is a step within a pipeline run.
    #[serde(default)]
    pub pipeline_run_id: Option<String>,
    /// Set when this session is a chat-thread invocation.
    #[serde(default)]
    pub chat_thread_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionProgress {
    #[serde(default)]
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionEnded {
    pub summary: Option<String>,
    pub duration_secs: Option<u64>,
    pub gate_result: Option<String>,
    #[serde(default)]
    pub updated_workspace_ids: Vec<String>,
    // Telemetry fields — populated at session end
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub files_changed: Option<i64>,
    #[serde(default)]
    pub lines_added: Option<i64>,
    #[serde(default)]
    pub lines_removed: Option<i64>,
    #[serde(default)]
    pub tool_calls: Option<i64>,
    #[serde(default)]
    pub exit_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDrainStarted {
    pub drained_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDrainCompleted {
    pub ended_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDrainAborted {
    pub resumed_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionToolCountIncremented {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHeartbeat {
    pub last_activity_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionAttached {
    pub existing_session_id: String,
    pub workspace_id: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusChanged {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTurnFailed {
    /// Free-text error message from the spawn / run path.
    pub error: String,
}
