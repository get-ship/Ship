use super::io::{get_config, get_effective_config, save_config};
use super::project::{AgentProfile, McpServerConfig};
use super::runtime_settings::{default_color_for, id_to_name};
use super::types::StatusConfig;
use anyhow::{Result, anyhow};
use std::path::PathBuf;

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub fn get_project_statuses(project_dir: Option<PathBuf>) -> Result<Vec<String>> {
    let config = get_config(project_dir)?;
    Ok(config.statuses.iter().map(|s| s.id.clone()).collect())
}

fn emit_config_event(subject: &str, detail: Option<String>) -> Result<()> {
    use ship_substrate::db::block_on_anyhow;
    use ship_types::envelope::EventEnvelope;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    use ship_substrate::events::types::event_types;
    use ship_substrate::events::types::ConfigChanged;
    use ship_substrate::events::validator::{CallerKind, EmitContext};
    let store = SqliteEventStore::new()?;
    let payload = ConfigChanged {
        subject: subject.to_string(),
        detail,
    };
    let envelope = EventEnvelope::new(event_types::CONFIG_CHANGED, subject, &payload)?;
    store.append(&envelope)?;

    let ctx = EmitContext {
        caller_kind: CallerKind::Cli,
        skill_id: None,
        project_id: None,
        session_id: None,
    };
    if let Some(kr) = ship_substrate::events::kernel_router()
        && let Err(e) = block_on_anyhow(async { kr.lock().await.emit(envelope, &ctx).await })
    {
        tracing::warn!("config::crud: kernel route failed: {e}");
    }
    Ok(())
}

pub fn add_status(project_dir: Option<PathBuf>, status: &str) -> Result<()> {
    let sanitized = status.to_lowercase().replace(' ', "-");
    let mut config = get_config(project_dir.clone())?;
    if !config.statuses.iter().any(|s| s.id == sanitized) {
        config.statuses.push(StatusConfig {
            id: sanitized.clone(),
            name: id_to_name(&sanitized),
            color: default_color_for(&sanitized),
        });
        save_config(&config, project_dir.clone())?;
        emit_config_event("status", Some(format!("id={}", sanitized)))?;
    }
    Ok(())
}

pub fn remove_status(project_dir: Option<PathBuf>, status: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.statuses.retain(|s| s.id != status);
    save_config(&config, project_dir.clone())?;
    emit_config_event("status", Some(format!("id={}", status)))?;
    Ok(())
}

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

// `get_active_agent` reads `ship.jsonc:active_agent` and resolves it to
// the matching `AgentProfile`. Today MCP's tool-gate is the only live
// reader; the writer side (`add_agent` / `remove_agent` /
// `set_active_agent` / `ship mode set`) was scaffolded but never wired
// to a CLI surface, so the field is effectively read-only-and-always-None
// in production. Kept for the read path; rebuild the writer when a real
// `ship use --persist` style UX lands.
pub fn get_active_agent(project_dir: Option<PathBuf>) -> Result<Option<AgentProfile>> {
    let config = get_config(project_dir)?;
    Ok(config
        .active_agent
        .as_ref()
        .and_then(|id| config.modes.into_iter().find(|m| &m.id == id)))
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
