use serde::{Deserialize, Serialize};

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
        let decoded: SessionStarted =
            serde_json::from_str(r#"{"goal": "test"}"#).unwrap();
        assert_eq!(decoded.workspace_id, "");
        assert!(decoded.agent_id.is_none());
    }
}
