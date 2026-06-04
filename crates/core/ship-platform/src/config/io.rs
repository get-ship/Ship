use super::mcp::{get_mcp_config, save_mcp_config};
use super::merge::{
    merge_agents, merge_mcp_servers, merge_string_lists, write_project_core_config,
};
use super::project::{AgentProfile, ProjectConfig, default_harnesses};
use super::types::PRIMARY_CONFIG_FILE;
use ship_substrate::config_doc;
use ship_substrate::project::get_global_dir;
use anyhow::Result;
use std::path::PathBuf;

/// Project the FULL nested `compiler::AgentProfile` (THE reader's output) into
/// the platform's flat [`AgentProfile`] runtime view. Lossy by design — the
/// platform UI does not surface the nested type's hand-authored fields; the
/// fields the platform owns (`rules`, `hooks`, `permissions`, `target_agents`,
/// `isolation`) are derived elsewhere / default here, matching the prior
/// projection.
fn project_canonical(src: &compiler::AgentProfile) -> AgentProfile {
    AgentProfile {
        id: src.agent.id.clone(),
        name: src.agent.name.clone(),
        description: src.agent.description.clone(),
        role: src.agent.role.clone(),
        tools_preset: src.agent.tools_preset.clone(),
        active_tools: src.agent.active_tools.clone(),
        mcp_servers: src.mcp.servers.clone(),
        skills: src.skills.refs.clone(),
        rules: Vec::new(),
        hooks: Vec::new(),
        permissions: super::types::PermissionConfig::default(),
        target_agents: Vec::new(),
        isolation: None,
    }
}

/// Read the project's agent profiles via THE agent-def reader (FsBroker,
/// ship-config), projected to the flat platform view. `config_dir` is the
/// `.ship` directory the reader walks `agents/` under.
fn read_project_agents(config_dir: &std::path::Path) -> Result<Vec<AgentProfile>> {
    let fs = ship_substrate::ports::LocalFsBroker::new();
    Ok(ship_config::agents::read_agent_profiles(&fs, config_dir)?
        .iter()
        .map(project_canonical)
        .collect())
}

/// Project the platform's nested [`AgentProfile`] to the config crate's flat
/// write view (`ship_config::types::AgentProfile`) — the platform-managed
/// fields the batch write door persists. The nested-only fields (`hooks`,
/// `permissions`, `isolation`) are NOT platform-owned at the file tier and are
/// preserved on disk by the door's re-save merge, not re-emitted here.
fn to_config_profile(p: &AgentProfile) -> ship_config::types::AgentProfile {
    ship_config::types::AgentProfile {
        id: p.id.clone(),
        name: p.name.clone(),
        description: p.description.clone(),
        role: p.role.clone(),
        tools_preset: p.tools_preset.clone(),
        active_tools: p.active_tools.clone(),
        mcp_servers: p.mcp_servers.clone(),
        skills: p.skills.clone(),
        rules: p.rules.clone(),
        target_agents: p.target_agents.clone(),
    }
}

/// Persist the project's agent profiles through THE agent-def write door
/// (FsBroker batch, ship-config) — the same gated door the per-agent CRUD uses.
/// `config_dir` is the `.ship` dir the door authors `agents/<id>.jsonc` under.
fn save_project_agents(config_dir: &std::path::Path, agents: &[AgentProfile]) -> Result<()> {
    let fs = ship_substrate::ports::LocalFsBroker::new();
    let mapped: Vec<_> = agents.iter().map(to_config_profile).collect();
    ship_config::agents_write::save_agents_config(&fs, config_dir, &mapped)?;
    Ok(())
}

pub fn get_config(project_dir: Option<PathBuf>) -> Result<ProjectConfig> {
    let is_project = project_dir.is_some();
    let config_dir = match project_dir {
        Some(dir) => dir,
        None => get_global_dir()?,
    };

    let primary_path = config_dir.join(PRIMARY_CONFIG_FILE);

    let mut config: ProjectConfig =
        config_doc::read_doc(&primary_path)?.unwrap_or_default();

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
pub fn get_effective_config(project_dir: Option<PathBuf>) -> Result<ProjectConfig> {
    let global = get_config(None)?;
    let Some(project_dir) = project_dir else {
        return Ok(global);
    };

    let mut project = get_config(Some(project_dir))?;

    project.agent.skills = merge_string_lists(&global.agent.skills, &project.agent.skills);
    project.agent.prompts = merge_string_lists(&global.agent.prompts, &project.agent.prompts);
    project.agent.context = merge_string_lists(&global.agent.context, &project.agent.context);

    // Project harnesses win; fall back to global when project does not specify a real override.
    if (project.harnesses.is_empty() || project.harnesses == default_harnesses())
        && !global.harnesses.is_empty()
    {
        project.harnesses = global.harnesses;
    }

    project.agents = merge_agents(&global.agents, &project.agents);
    project.mcp_servers = merge_mcp_servers(&global.mcp_servers, &project.mcp_servers);

    Ok(project)
}

pub fn save_config(config: &ProjectConfig, project_dir: Option<PathBuf>) -> Result<()> {
    let config_dir = if let Some(p_dir) = project_dir.clone() {
        p_dir
    } else {
        get_global_dir()?
    };
    let path = config_dir.join(PRIMARY_CONFIG_FILE);

    if project_dir.is_none() {
        // Global config is JSONC. The home-layout migrator converts any
        // pre-existing legacy TOML user-config into the settings/local pair.
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
        // Lenient: a corrupt existing ship.jsonc must not block re-save; we
        // only try to preserve a prior id when one parses out.
        let parsed: MinConfig = config_doc::read_doc_or_default(&path);
        if !parsed.id.trim().is_empty() {
            effective.id = parsed.id;
        }
    }
    if effective.id.trim().is_empty() {
        effective.id = ship_substrate::gen_nanoid();
    }

    // Bootstrap ship.jsonc with project identity before any SQLite-backed writes.
    if !path.exists() {
        write_project_core_config(&path, &effective)?;
    }

    // File-backed catalog state (mcp/skills/rules) is indexed into SQLite for agent refs.
    save_mcp_config(&config_dir, &effective.mcp_servers)?;
    save_project_agents(&config_dir, &effective.agents)?;

    // Keep ship.jsonc focused on stable project identity only. Project
    // harnesses default via the field's serde default; if the user wants to
    // narrow at the project level, they edit ship.jsonc directly. (No more
    // KV side-table.)
    write_project_core_config(&path, &effective)?;
    Ok(())
}
