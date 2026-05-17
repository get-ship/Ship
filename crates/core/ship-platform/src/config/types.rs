use serde::{Deserialize, Serialize};
use specta::Type;

pub use ship_substrate::project::PRIMARY_CONFIG_FILE;

// ─── Data types ───────────────────────────────────────────────────────────────

/// Which lifecycle event triggers a hook.
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookTrigger {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    PostToolUseFailure,
    Notification,
    SubagentStart,
    Stop,
    SubagentStop,
    PreCompact,
    BeforeTool,
    AfterTool,
    BeforeAgent,
    AfterAgent,
    SessionEnd,
    BeforeModel,
    AfterModel,
    BeforeToolSelection,
}

/// A shell command executed on a lifecycle event.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    /// Glob/regex pattern to match tool name (e.g. "Bash", "mcp__*"). Empty = all tools.
    #[serde(default)]
    pub matcher: Option<String>,
    /// Optional timeout for the hook command in milliseconds.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Human-readable intent for editor UX and exported harness config.
    #[serde(default)]
    pub description: Option<String>,
    /// The shell command to run
    pub command: String,
}

/// Mode-scoped tool permission overrides.
/// These overlay canonical `.ship/permissions.jsonc` `tools.allow/deny`
/// when a mode is active.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct PermissionConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AgentLayerConfig {
    /// Skill IDs to load for all sessions in this project.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Legacy alias for global instruction skill IDs.
    /// Prompts are modeled as skills in Ship.
    #[serde(default)]
    pub prompts: Vec<String>,
    /// Context files/folders to preload for agents.
    #[serde(default)]
    pub context: Vec<String>,
}

pub fn is_agent_layer_empty(config: &AgentLayerConfig) -> bool {
    config.skills.is_empty() && config.prompts.is_empty() && config.context.is_empty()
}


