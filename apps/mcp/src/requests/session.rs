use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct StartSessionRequest {
    /// Branch the session is for. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// Optional goal for this session.
    pub goal: Option<String>,
    /// Optional agent override for this session.
    pub agent_id: Option<String>,
    /// Optional primary provider for this session.
    pub provider_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EndSessionRequest {
    /// Branch the session is for. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// End-of-session summary — what was accomplished, what changed.
    pub summary: Option<String>,
    /// Branches updated during this session (used to invalidate downstream work).
    pub updated_workspace_ids: Option<Vec<String>>,
    /// Model ID used during the session (e.g. "claude-opus-4-20250514").
    pub model: Option<String>,
    /// Count of files modified during the session.
    pub files_changed: Option<i64>,
    /// Gate result for gated sessions: "pass", "fail", or null.
    pub gate_result: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LogProgressRequest {
    /// Progress note — what you did, decided, or got blocked on.
    pub note: String,
    /// Branch the session is on. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// Escalation level: "blocker", "warning", or "question". Sets elevated flag.
    pub escalation: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetSessionRequest {
    /// Branch the session is on. If omitted, resolves from current git branch.
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListSessionsRequest {
    /// Filter by branch. If omitted, returns sessions across all branches.
    pub branch: Option<String>,
    /// Maximum number of sessions to return. Default 20, max 100.
    pub limit: Option<u32>,
}
