use serde::{Deserialize, Serialize};

use agent_schema::{AgentProfile, HookConfig, McpServerConfig};

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
    pub agents: Vec<AgentProfile>,
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
