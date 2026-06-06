use super::io::{get_config, save_config};
use super::project::{AgentProfile, McpServerConfig};
use anyhow::{Result, anyhow};
use std::path::PathBuf;

/// Create the standard `.ship/` subdirectories used by built-in modules.
/// Replaces the old configurable `namespaces` registry — every project
/// gets the same baseline tree.
pub fn ensure_standard_dirs(ship_path: &std::path::Path) -> Result<()> {
    let base = ship_substrate::project::ship_dir_from_path(ship_path)
        .unwrap_or_else(|| ship_path.to_path_buf());
    for sub in ["project", "agents"] {
        std::fs::create_dir_all(base.join(sub))?;
    }
    Ok(())
}

/// Resolve the `AgentProfile` for the project's currently-active agent.
/// "Active" is defined as the agent recorded by the latest `compile.completed`
/// event materialized in `project_compile_state`. Returns None when the
/// project has never been compiled or when the recorded agent id no longer
/// exists in the project's agents list. Callers pass the ship_dir
/// (`<project>/.ship/`).
pub fn get_active_agent(project_dir: Option<PathBuf>) -> Result<Option<AgentProfile>> {
    let Some(ship_dir) = project_dir.clone() else {
        return Ok(None);
    };
    let space_id = ship_substrate::project::ensure_space_id(&ship_dir)?;
    let Some(state) = crate::services::compile::projection::load(&space_id)? else {
        return Ok(None);
    };
    let Some(active_id) = state.active_agent else {
        return Ok(None);
    };
    let config = get_config(project_dir)?;
    Ok(config.agents.into_iter().find(|a| a.id == active_id))
}

// ─── MCP Server Registry CRUD ─────────────────────────────────────────────────

pub fn add_mcp_server(project_dir: Option<PathBuf>, server: McpServerConfig) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if config.mcp_servers.iter().any(|s| s.id == server.id) {
        return Err(anyhow!("MCP server '{}' already exists", server.id));
    }
    config.mcp_servers.push(server);
    save_config(&config, project_dir)
}

pub fn remove_mcp_server(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.mcp_servers.retain(|s| s.id != id);
    save_config(&config, project_dir)
}

pub fn list_mcp_servers(project_dir: Option<PathBuf>) -> Result<Vec<McpServerConfig>> {
    let config = get_config(project_dir)?;
    Ok(config.mcp_servers)
}
