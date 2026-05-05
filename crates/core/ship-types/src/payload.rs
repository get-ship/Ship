use serde::{Deserialize, Serialize};

// -- Workspace aggregate -------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCreated {
    pub workspace_id: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceActivated {
    pub agent_id: Option<String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceArchived {}

// -- Session aggregate ---------------------------------------------------------

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
    pub primary_provider: Option<String>,
    /// Project this session belongs to. Looked up at start from the
    /// actor's project_id or the workspace path. None for sessions that
    /// have no associated project (daemon services, ephemeral runs).
    #[serde(default)]
    pub project_id: Option<String>,
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
    pub provider: Option<String>,
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorCreated {
    pub kind: String,
    pub environment_type: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub parent_actor_id: Option<String>,
    #[serde(default)]
    pub restart_count: u32,
    /// Agent profile id this actor implements (e.g. "shipwright", "rust-runtime").
    /// None for service actors (mesh, user-inbox, supervisor).
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Provider backing this actor (claude / codex / opencode / gemini / cursor).
    /// None for non-agent actors.
    #[serde(default)]
    pub provider: Option<String>,
    /// Provider model identifier when known at spawn time.
    #[serde(default)]
    pub model: Option<String>,
    /// Transport profile — `pty` for attached/interactive, `headless` for stream-json.
    #[serde(default)]
    pub transport: Option<String>,
    /// Lifecycle profile — `long-lived` for chat/services, `one-shot` for pipeline steps.
    #[serde(default)]
    pub persistence: Option<String>,
    /// SHA-256 of the system prompt or rules concatenation used at spawn.
    /// Lets the dashboard detect prompt drift across restarts.
    #[serde(default)]
    pub system_prompt_hash: Option<String>,
    /// OS pid of the held process. `0` or `None` for in-process services.
    #[serde(default)]
    pub pid: Option<u32>,
    /// Capability ids (skill ids the actor advertises).
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Project id this actor belongs to. Looked up from `runtime::projects`
    /// when None and `cwd` is registered. Daemon services may have neither.
    #[serde(default)]
    pub project_id: Option<String>,
    /// Filesystem cwd the actor runs in. Set explicitly at spawn; falls back
    /// to project root or supervisor cwd when omitted (legacy callers).
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorWoke {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorSlept {
    pub idle_secs: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorStopped {
    pub reason: String,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
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

// -- Artifact events -----------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactCreated {
    pub id: String,
    pub scope: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub owner_actor: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactUpdated {
    pub id: String,
    pub scope: String,
    pub changed: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactMoved {
    pub id: String,
    pub from_scope: String,
    pub to_scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactDeleted {
    pub id: String,
    pub scope: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
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

// -- Supervisor aggregate ------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SupervisorShuttingDown {
    #[serde(default)]
    pub reason: Option<String>,
}

// -- Compile aggregate --------------------------------------------------------

/// Payload for `compile.completed`. Records what was compiled for the
/// `project_compile_state` projection. `entity_id` on the envelope is the
/// project_id; this payload carries the compile-run details.
#[derive(Debug, Serialize, Deserialize)]
pub struct CompileCompleted {
    /// Active agent id at compile time, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_agent: Option<String>,
    /// Compile time as RFC 3339 UTC.
    pub compiled_at: String,
    /// Ship version that produced this compile.
    pub ship_version: String,
    /// Primary provider compiled for (e.g. "claude").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Plugin IDs the active agent declared and that this compile installed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins_installed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_created_payload_deserializes() {
        let original = WorkspaceCreated {
            workspace_id: "ws-1".to_string(),
            path: None,
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
