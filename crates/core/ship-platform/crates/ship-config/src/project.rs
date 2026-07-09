//! Space-manifest types and IO — the canonical home for the six symbols that
//! `ship-agents` (C8) needs without pulling in all of `ship-platform`.
//!
//! These types were formerly in `ship-platform/src/config/{project,types,io}.rs`.
//! They live here because ship-config is the L1 config crate that both
//! ship-platform and (soon) ship-agents depend on, making the dep direction clean:
//! ship-agents → ship-config (L1→L1, no cycle).

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use ship_substrate::project::PRIMARY_CONFIG_FILE;

// ─── Hook + permission types ──────────────────────────────────────────────────

/// Which lifecycle event triggers a hook.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PermissionConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentLayerConfig {
    /// Skill IDs to load for all sessions in this project.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Legacy alias for global instruction skill IDs.
    #[serde(default)]
    pub prompts: Vec<String>,
    /// Context files/folders to preload for agents.
    #[serde(default)]
    pub context: Vec<String>,
}

pub fn is_agent_layer_empty(config: &AgentLayerConfig) -> bool {
    config.skills.is_empty() && config.prompts.is_empty() && config.context.is_empty()
}

// ─── Agent / MCP / Project types ─────────────────────────────────────────────

/// The single agent-definition type. Re-exported from the compiler crate (the
/// file-canonical nested schema) so `ship_config::project::AgentProfile` and
/// `compiler::AgentProfile` are one type — no parallel runtime view.
pub use compiler::AgentProfile;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    #[default]
    Stdio,
    Sse,
    Http,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpServerConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// For stdio servers: the binary to run
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// "global" | "project" | "mode"
    #[serde(default = "default_scope")]
    pub scope: String,
    /// Transport type: stdio (default), sse, or http
    #[serde(default)]
    pub server_type: McpServerType,
    /// URL for SSE or HTTP servers (ignored for stdio)
    pub url: Option<String>,
    /// If true, the server is registered but not started
    #[serde(default)]
    pub disabled: bool,
    /// Optional connection timeout in seconds
    pub timeout_secs: Option<u32>,
}

fn default_scope() -> String {
    "global".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub mcp: McpSection,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct McpSection {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HarnessProjectOverrides {
    /// Project-level override for the user-KV `harness.default` setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Project-level override for the user-KV `harness.default_model` setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
}

impl HarnessProjectOverrides {
    pub fn is_empty(&self) -> bool {
        self.default.is_none() && self.default_model.is_none()
    }
}

fn is_harness_overrides_empty(h: &Option<HarnessProjectOverrides>) -> bool {
    h.as_ref().map(|o| o.is_empty()).unwrap_or(true)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpaceManifest {
    #[serde(default = "default_version")]
    pub version: String,
    /// Stable project identity.
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<AgentProfile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default, skip_serializing_if = "is_agent_layer_empty")]
    pub agent: AgentLayerConfig,
    /// Which agent harnesses to generate config for on branch checkout.
    #[serde(default = "default_harnesses", skip_serializing_if = "Vec::is_empty")]
    pub harnesses: Vec<String>,
    #[serde(default, skip_serializing_if = "is_harness_overrides_empty")]
    pub harness: Option<HarnessProjectOverrides>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct ProjectCoreFile {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub fn default_version() -> String {
    "1".to_string()
}

pub fn default_harnesses() -> Vec<String> {
    vec!["claude".to_string()]
}

impl Default for SpaceManifest {
    fn default() -> Self {
        Self {
            version: default_version(),
            id: String::new(),
            name: None,
            description: None,
            agents: Vec::new(),
            mcp_servers: Vec::new(),
            agent: AgentLayerConfig::default(),
            harnesses: default_harnesses(),
            harness: None,
        }
    }
}

// ─── MCP IO ───────────────────────────────────────────────────────────────────

pub fn get_mcp_config(ship_dir: &std::path::Path) -> Result<Vec<McpServerConfig>> {
    use std::fs;
    let path = ship_substrate::project::mcp_config_path(ship_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let raw: McpConfig = serde_json::from_str(
        &ship_substrate::fs_util::strip_jsonc_comments(&content)
    )?;

    let mut servers = Vec::new();
    for (id, mut server) in raw.mcp.servers {
        server.id = id;
        servers.push(server);
    }
    servers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(servers)
}

pub(crate) fn save_mcp_config(ship_dir: &std::path::Path, servers: &[McpServerConfig]) -> Result<()> {
    use std::fs;
    use ship_substrate::fs_util::write_atomic;

    let path = ship_substrate::project::mcp_config_path(ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut by_id: HashMap<String, McpServerConfig> = HashMap::new();
    for server in servers {
        let mut cloned = server.clone();
        cloned.id.clear();
        by_id.insert(server.id.clone(), cloned);
    }

    let raw = McpConfig {
        mcp: McpSection { servers: by_id },
    };
    write_atomic(&path, serde_json::to_string_pretty(&raw)?)?;
    Ok(())
}

// ─── Merge helpers ────────────────────────────────────────────────────────────

pub(crate) fn merge_string_lists(base: &[String], overlay: &[String]) -> Vec<String> {
    use std::collections::HashSet;
    let mut merged = Vec::new();
    let mut seen = HashSet::new();
    for item in base.iter().chain(overlay.iter()) {
        if seen.insert(item.clone()) {
            merged.push(item.clone());
        }
    }
    merged
}

pub(crate) fn merge_agents(base: &[AgentProfile], overlay: &[AgentProfile]) -> Vec<AgentProfile> {
    let mut merged = base.to_vec();
    for agent in overlay {
        if let Some(existing) = merged.iter_mut().find(|a| a.agent.id == agent.agent.id) {
            *existing = agent.clone();
        } else {
            merged.push(agent.clone());
        }
    }
    merged
}

pub(crate) fn merge_mcp_servers(
    base: &[McpServerConfig],
    overlay: &[McpServerConfig],
) -> Vec<McpServerConfig> {
    let mut merged = base.to_vec();
    for server in overlay {
        if let Some(existing) = merged.iter_mut().find(|s| s.id == server.id) {
            *existing = server.clone();
        } else {
            merged.push(server.clone());
        }
    }
    merged
}

pub(crate) fn write_space_core_config(path: &std::path::Path, config: &SpaceManifest) -> Result<()> {
    use ship_substrate::fs_util::write_atomic;
    let core = ProjectCoreFile {
        version: config.version.clone(),
        id: config.id.clone(),
        name: config.name.clone(),
        description: config.description.clone(),
    };
    let json_str = serde_json::to_string_pretty(&core)?;
    write_atomic(path, json_str)?;
    Ok(())
}

// ─── Config IO ───────────────────────────────────────────────────────────────

fn read_project_agents(config_dir: &std::path::Path) -> Result<Vec<AgentProfile>> {
    let fs = ship_substrate::ports::LocalFsBroker::new();
    Ok(crate::agents::read_agent_profiles(&fs, config_dir)?)
}

fn save_project_agents(config_dir: &std::path::Path, agents: &[AgentProfile]) -> Result<()> {
    let fs = ship_substrate::ports::LocalFsBroker::new();
    crate::agents_write::save_agents_config(&fs, config_dir, agents)?;
    Ok(())
}

/// Read the project config from `project_dir` (or global if `None`).
pub fn get_config(project_dir: Option<PathBuf>) -> Result<SpaceManifest> {
    use ship_substrate::config_doc;
    let is_project = project_dir.is_some();
    let config_dir = match project_dir {
        Some(dir) => dir,
        None => ship_substrate::project::get_global_dir()?,
    };

    let primary_path = config_dir.join(PRIMARY_CONFIG_FILE);
    let mut config: SpaceManifest = config_doc::read_doc(&primary_path)?.unwrap_or_default();

    if is_project {
        let agents = read_project_agents(&config_dir)?;
        if !agents.is_empty() {
            config.agents = agents;
        }
    }

    let servers = get_mcp_config(&config_dir)?;
    if !servers.is_empty() {
        config.mcp_servers = servers;
    }

    Ok(config)
}

/// Returns a merged view of global + project config.
/// Project values win; missing project AI/agent/MCP values inherit from global.
pub fn get_effective_config(project_dir: Option<PathBuf>) -> Result<SpaceManifest> {
    let global = get_config(None)?;
    let Some(project_dir) = project_dir else {
        return Ok(global);
    };

    let mut project = get_config(Some(project_dir))?;

    project.agent.skills = merge_string_lists(&global.agent.skills, &project.agent.skills);
    project.agent.prompts = merge_string_lists(&global.agent.prompts, &project.agent.prompts);
    project.agent.context = merge_string_lists(&global.agent.context, &project.agent.context);

    if (project.harnesses.is_empty() || project.harnesses == default_harnesses())
        && !global.harnesses.is_empty()
    {
        project.harnesses = global.harnesses;
    }

    project.agents = merge_agents(&global.agents, &project.agents);
    project.mcp_servers = merge_mcp_servers(&global.mcp_servers, &project.mcp_servers);

    Ok(project)
}

/// Persist project config to `project_dir` (or global if `None`).
pub fn save_config(config: &SpaceManifest, project_dir: Option<PathBuf>) -> Result<()> {
    use ship_substrate::config_doc;
    let config_dir = if let Some(p_dir) = project_dir.clone() {
        p_dir
    } else {
        ship_substrate::project::get_global_dir()?
    };
    let path = config_dir.join(PRIMARY_CONFIG_FILE);

    if project_dir.is_none() {
        config_doc::write_doc(&path, config)?;
        return Ok(());
    }

    let mut effective = config.clone();
    if effective.id.trim().is_empty() {
        #[derive(serde::Deserialize, Default)]
        struct MinConfig {
            #[serde(default)]
            id: String,
        }
        let parsed: MinConfig = config_doc::read_doc_or_default(&path);
        if !parsed.id.trim().is_empty() {
            effective.id = parsed.id;
        }
    }
    if effective.id.trim().is_empty() {
        effective.id = ship_substrate::gen_nanoid();
    }
    if !path.exists() {
        write_space_core_config(&path, &effective)?;
    }
    save_mcp_config(&config_dir, &effective.mcp_servers)?;
    save_project_agents(&config_dir, &effective.agents)?;
    write_space_core_config(&path, &effective)?;
    Ok(())
}
