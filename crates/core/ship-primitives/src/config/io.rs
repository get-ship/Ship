use super::mcp::{get_mcp_config, save_mcp_config};
use super::merge::{
    merge_agents, merge_mcp_servers, merge_string_lists, write_project_core_config,
};
use super::agents::{get_agents_config, save_agents_config};
use super::project::{ProjectConfig, default_providers};
use super::runtime_settings::{
    get_runtime_settings, migrate_json_config, save_runtime_settings,
};
use super::types::PRIMARY_CONFIG_FILE;
use ship_substrate::project::get_global_dir;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn get_config(project_dir: Option<PathBuf>) -> Result<ProjectConfig> {
    let is_project = project_dir.is_some();
    let config_dir = match project_dir {
        Some(dir) => dir,
        None => get_global_dir()?,
    };

    let primary_path = config_dir.join(PRIMARY_CONFIG_FILE);
    let json_path = config_dir.join("config.json");

    let mut config = None;
    if primary_path.exists() {
        let content = fs::read_to_string(&primary_path)?;
        config = Some(serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(&content))?);
    }

    let mut config = if let Some(config) = config {
        config
    } else if json_path.exists() {
        // Legacy JSON config — read what we can and migrate.
        migrate_json_config(&json_path).unwrap_or_default()
    } else {
        ProjectConfig::default()
    };

    if is_project {
        if let Some(providers) = get_runtime_settings(&config_dir)? {
            config.providers = providers;
        }
    }

    if is_project {
        let agents = get_agents_config(&config_dir)?;
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

    // Project providers win; fall back to global when project does not specify a real override.
    if (project.providers.is_empty() || project.providers == default_providers())
        && !global.providers.is_empty()
    {
        project.providers = global.providers;
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

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if project_dir.is_none() {
        // Global config is JSONC. The home-layout migrator converts any
        // pre-existing legacy TOML user-config into the settings/local pair.
        let json = serde_json::to_string_pretty(config)?;
        ship_substrate::fs_util::write_atomic(&path, json)?;
        return Ok(());
    }

    let mut effective = config.clone();
    if effective.id.trim().is_empty() {
        #[derive(serde::Deserialize, Default)]
        struct MinConfig {
            #[serde(default)]
            id: String,
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let parsed: MinConfig = serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(&content)).unwrap_or_default();
            if !parsed.id.trim().is_empty() {
                effective.id = parsed.id;
            }
        }
    }
    if effective.id.trim().is_empty() {
        effective.id = ship_substrate::gen_nanoid();
    }

    // Bootstrap ship.jsonc with project identity before any SQLite-backed writes.
    // save_runtime_settings/open_project_db resolve the DB key from ship.jsonc:id.
    if !path.exists() {
        write_project_core_config(&path, &effective)?;
    }

    // Project runtime settings + agent bindings live in SQLite.
    save_runtime_settings(&config_dir, &effective)?;
    // File-backed catalog state (mcp/skills/rules) is indexed into SQLite for agent refs.
    save_mcp_config(&config_dir, &effective.mcp_servers)?;
    save_agents_config(&config_dir, &effective.agents)?;

    // Keep ship.jsonc focused on stable project identity only.
    write_project_core_config(&path, &effective)?;
    Ok(())
}
