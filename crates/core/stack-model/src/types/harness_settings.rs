//! Per-harness typed settings deltas — DATA, not codegen. These ride
//! inside [`crate::resolve::ResolvedConfig`], which is why they live in
//! the stack model: the model owns the resolved-config shape; the
//! harness-config-compiler crate consumes these types to emit.

/// Claude's typed settings delta from the Ship agent.
#[derive(Debug, Clone, Default)]
pub struct ClaudeSettings {
    pub settings_extra: Option<serde_json::Value>,
    /// Legacy team-agent passthrough: `.ship/agents/teams/claude/<n>.md`.
    pub team_agents: Vec<(String, String)>,
    pub theme: Option<String>,
    pub auto_updates: Option<bool>,
    pub include_co_authored_by: Option<bool>,
    /// Reasoning effort written as `effortLevel` in `.claude/settings.json`.
    /// Project-scope override of the operator's `~/.claude/settings.json`
    /// `effortLevel`; the worktree value wins on merge, so a dispatched
    /// agent never inherits the human's `xhigh` default. Claude rejects
    /// empty / unknown values — the compile path is permissive; the
    /// harness validates.
    pub effort: Option<String>,
}

/// Codex's typed settings delta from the Ship agent.
#[derive(Debug, Clone, Default)]
pub struct CodexSettings {
    pub sandbox: Option<String>,
    pub approval_policy: Option<String>,
    pub reasoning_effort: Option<String>,
    pub max_threads: Option<u32>,
    pub max_depth: Option<u32>,
    pub job_max_runtime_seconds: Option<u64>,
    pub shell_env_policy: Option<String>,
    pub notify: Option<serde_json::Value>,
    pub settings_extra: Option<serde_json::Value>,
}

/// OpenCode's typed settings delta from the Ship agent. Owned here so all
/// OpenCode knowledge lives in this module; `ResolvedConfig` holds one of these.
#[derive(Debug, Clone, Default)]
pub struct OpenCodeSettings {
    /// Pass-through extra merged into opencode.json.
    pub settings_extra: Option<serde_json::Value>,
}
