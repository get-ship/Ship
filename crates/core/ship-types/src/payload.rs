use serde::{Deserialize, Serialize};

// -- Workspace aggregate -------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCreated {
    pub workspace_id: String,
    pub workspace_type: String,
    pub status: String,
    #[serde(default)]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub is_worktree: bool,
    #[serde(default)]
    pub worktree_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceDeleted {
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceStatusChanged {
    pub old_status: String,
    pub new_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceActivated {
    pub agent_id: Option<String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCompiled {
    pub config_generation: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCompileFailed {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceArchived {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceAgentChanged {
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceReconciled {
    pub is_worktree: bool,
    pub worktree_path: Option<String>,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceTmuxAssigned {
    pub tmux_session_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceStarted {
    pub worktree_path: String,
    pub tmux_session_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceWorktreeAdded {
    pub worktree_path: String,
    #[serde(default)]
    pub detected_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceWorktreeRemoved {
    #[serde(default)]
    pub previous_path: Option<String>,
    #[serde(default)]
    pub detected_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceBranchSwitched {
    #[serde(default)]
    pub previous_branch: Option<String>,
    pub new_branch: String,
    #[serde(default)]
    pub detected_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConfigChanged {
    #[serde(default)]
    pub changed_path: Option<String>,
    #[serde(default)]
    pub detected_by: String,
}

// -- Session aggregate ---------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionStarted {
    pub goal: Option<String>,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub workspace_branch: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub primary_provider: Option<String>,
    #[serde(default)]
    pub config_generation_at_start: Option<i64>,
    #[serde(default)]
    pub compiled_at: Option<String>,
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
    #[serde(default)]
    pub compile_error: Option<String>,
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
pub struct SessionAttached {
    pub existing_session_id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionRecorded {
    pub record_id: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_workspace_ids: Vec<String>,
    pub duration_secs: Option<i64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub agent_id: Option<String>,
    pub files_changed: Option<i64>,
    pub gate_result: Option<String>,
    pub workspace_branch: String,
}

// -- Skill aggregate -----------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillStarted {
    pub skill_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCompleted {
    pub skill_id: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillFailed {
    pub skill_id: String,
    pub error: String,
}

// -- Actor aggregate (v0.2.0 kernel) -------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorCreated {
    pub kind: String,
    pub environment_type: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub parent_actor_id: Option<String>,
    #[serde(default)]
    pub restart_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorWoke {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorSlept {
    pub idle_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorStopped {
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorCrashed {
    pub error: String,
    pub restart_count: u32,
}

// -- Gate aggregate ------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct GatePassed {
    pub evidence: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateFailed {
    pub evidence: String,
}

// -- Job aggregate -------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCreated {
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobClaimed {
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCompleted {
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobFailed {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobDispatched {
    pub agent_id: Option<String>,
}

// -- Config / Log aggregates ---------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigChanged {
    pub subject: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectLog {
    pub action: String,
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_created_payload_deserializes() {
        let original = WorkspaceCreated {
            workspace_id: "ws-1".to_string(),
            workspace_type: "feature".to_string(),
            status: "active".to_string(),
            active_agent: None,
            providers: vec![],
            mcp_servers: vec![],
            skills: vec![],
            is_worktree: false,
            worktree_path: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: WorkspaceCreated = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.workspace_id, "ws-1");
    }

    #[test]
    fn session_started_payload_has_defaults() {
        let decoded: SessionStarted = serde_json::from_str(r#"{"goal": "test"}"#).unwrap();
        assert_eq!(decoded.workspace_id, "");
        assert!(decoded.agent_id.is_none());
    }
}
