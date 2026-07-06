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

// ─── Root space manifest (ship.toml) ─────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpaceManifest {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub harnesses: Vec<String>,
    #[serde(default)]
    pub agents: Vec<super::agent_profile::AgentProfile>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub namespaces: Vec<NamespaceConfig>,
    #[serde(skip, default)]
    pub agent: AgentLayerConfig,
}

impl Default for SpaceManifest {
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

