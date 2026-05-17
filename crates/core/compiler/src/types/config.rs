use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ─── Hooks ────────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookTrigger {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
    SubagentStop,
    PreCompact,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    #[serde(default)]
    pub matcher: Option<String>,
    pub command: String,
    /// Raw Cursor event name; when set, bypasses trigger mapping and emits directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_event: Option<String>,
    /// Raw Gemini event name; when set, bypasses trigger mapping and emits directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini_event: Option<String>,
}

// ─── Modes ────────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PermissionConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Legacy role for tool gating defaults (`worker`, `lead`, `commander`).
    /// Retained for back-compat with profiles that haven't migrated to
    /// `tools_preset`. Mapping:
    ///   worker → preset "worker"
    ///   lead | commander → preset "orchestrator"
    /// `tools_preset` takes precedence when both are set.
    #[serde(default)]
    pub role: Option<String>,
    /// Tool preset name. Controls which platform-shipped tools this agent
    /// sees before `active_tools` is applied. Known presets:
    ///   "worker" → platform protocol tools only
    ///   "orchestrator" → worker tools + create_job + mesh_* tools
    /// Unset falls back to `role` mapping, then to "worker".
    #[serde(default)]
    pub tools_preset: Option<String>,
    #[serde(default)]
    pub active_tools: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub permissions: PermissionConfig,
    #[serde(default)]
    pub target_agents: Vec<String>,
}

// ─── MCP servers ──────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    #[default]
    Stdio,
    Sse,
    Http,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpServerConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default)]
    pub server_type: McpServerType,
    pub url: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    pub timeout_secs: Option<u32>,
    // ── Codex per-server fields ────────────────────────────────────────────────
    /// Codex: tool names to enable for this server.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codex_enabled_tools: Vec<String>,
    /// Codex: tool names to disable for this server.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codex_disabled_tools: Vec<String>,
    // ── Gemini per-server fields ───────────────────────────────────────────────
    /// Gemini: whether this server is trusted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini_trust: Option<bool>,
    /// Gemini: tool names to include from this server.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gemini_include_tools: Vec<String>,
    /// Gemini: tool names to exclude from this server.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gemini_exclude_tools: Vec<String>,
    /// Gemini: per-server timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini_timeout_ms: Option<u32>,
    // ── Cursor per-server fields ───────────────────────────────────────────────
    /// Cursor: path to an env file to load for this stdio server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_env_file: Option<String>,
}

fn default_scope() -> String {
    "global".to_string()
}

// ─── Agent layer ──────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentLayerConfig {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub prompts: Vec<String>,
    #[serde(default)]
    pub context: Vec<String>,
}

// ─── Namespaces ───────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NamespaceConfig {
    pub id: String,
    pub path: String,
    pub owner: String,
}

// ─── Root project config (ship.toml) ─────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub harnesses: Vec<String>,
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub namespaces: Vec<NamespaceConfig>,
    #[serde(skip, default)]
    pub agent: AgentLayerConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            id: String::new(),
            name: None,
            description: None,
            harnesses: Vec::new(),
            agents: Vec::new(),
            mcp_servers: Vec::new(),
            hooks: Vec::new(),
            namespaces: Vec::new(),
            agent: AgentLayerConfig::default(),
        }
    }
}

fn default_version() -> String {
    "1".to_string()
}

