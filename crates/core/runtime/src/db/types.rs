//! Shared types for the platform database layer.

pub type WorkspaceDbRow = (
    String,         // 0: id
    Option<String>, // 1: path
    Option<String>, // 2: created_at
);

pub type WorkspaceDbListRow = (
    String,         // 0: branch (name)
    String,         // 1: id
    Option<String>, // 2: path
    Option<String>, // 3: created_at
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionDb {
    pub id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub agent_id: Option<String>,
    pub primary_provider: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub compiled_at: Option<String>,
    pub compile_error: Option<String>,
    pub config_generation_at_start: Option<i64>,
    pub tool_call_count: i64,
    pub drained_at: Option<String>,
    pub mcp_provider: Option<String>,
    pub last_activity_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionRecordDb {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub duration_secs: Option<i64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub agent_id: Option<String>,
    pub files_changed: Option<i64>,
    pub gate_result: Option<String>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
    pub tool_calls: Option<i64>,
    pub exit_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledAssetCacheDb {
    pub uuid: String,
    pub kind: String,
    pub external_id: String,
    pub name: String,
    pub source_path: String,
    pub content_hash: String,
}

/// Backwards-compatible alias — remove after all callers migrate.
#[allow(dead_code)]
pub type AgentArtifactRegistryDb = CompiledAssetCacheDb;
