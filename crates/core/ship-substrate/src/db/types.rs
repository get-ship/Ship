//! Shared types for the platform database layer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDb {
    pub id: String,
    pub workspace_id: String,
    pub branch: String,
    pub project_id: Option<String>,
    pub kind: String,
    pub pipeline_run_id: Option<String>,
    pub chat_thread_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub agent_id: Option<String>,
    pub primary_harness: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub tool_call_count: i64,
    pub drained_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub duration_secs: Option<i64>,
    pub model: Option<String>,
    pub files_changed: Option<i64>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
    pub tool_calls: Option<i64>,
    pub gate_result: Option<String>,
    pub exit_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

