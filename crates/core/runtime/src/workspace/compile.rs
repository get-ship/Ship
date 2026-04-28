use crate::agents::config::{
    ProviderSettings, resolve_provider_settings_with_agent_override,
};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use super::helpers::*;
use super::types_session::*;

// ---- Agent config resolution -----------------------------------------------

pub(crate) fn resolve_session_agent_config(
    ship_dir: &Path,
    agent_id: Option<&str>,
) -> Result<ProviderSettings> {
    let agent_id = agent_id
        .and_then(normalize_agent_ref)
        .map(|a| a.to_string());

    resolve_provider_settings_with_agent_override(
        ship_dir,
        None,
        agent_id.as_deref(),
    )
}

pub(crate) fn resolve_session_providers(
    ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<Vec<String>> {
    let resolved = resolve_session_agent_config(ship_dir, agent_id)?;
    if resolved.providers.is_empty() {
        return Err(anyhow!(
            "No valid providers resolved for session on branch '{}'",
            branch
        ));
    }
    Ok(resolved.providers)
}

// ---- Provider matrix -------------------------------------------------------

pub(crate) fn build_session_provider_matrix(
    ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix> {
    let resolved_agent_id = agent_id.and_then(normalize_agent_ref);
    let resolved = resolve_session_agent_config(ship_dir, agent_id)?;
    let providers = resolved.providers;

    let supported_providers = crate::agents::export::list_providers(ship_dir)?
        .into_iter()
        .map(|provider| provider.id)
        .collect::<Vec<_>>();

    let resolution_error = if providers.is_empty() {
        Some(format!(
            "No valid providers resolved for session on branch '{}'",
            branch
        ))
    } else {
        None
    };

    Ok(WorkspaceProviderMatrix {
        workspace_branch: branch.to_string(),
        agent_id: resolved_agent_id,
        source: if resolved.active_agent.is_some() {
            "agent/config".to_string()
        } else {
            "config/default".to_string()
        },
        allowed_providers: providers,
        supported_providers,
        resolution_error,
    })
}

// ---- Context root and missing configs --------------------------------------

/// Resolve the on-disk root where provider config files should be written.
/// Caller passes the worktree path explicitly (or None to fall back to the
/// project root, computed from `ship_dir.parent()`).
pub(crate) fn resolve_session_context_root(
    ship_dir: &Path,
    explicit_path: Option<&str>,
) -> PathBuf {
    if let Some(path) = explicit_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    ship_dir.parent().unwrap_or(ship_dir).to_path_buf()
}

pub(crate) fn missing_provider_configs_for_session(
    context_root: &Path,
    providers: &[String],
) -> Vec<String> {
    providers
        .iter()
        .filter_map(|provider| {
            let desc = crate::agents::export::get_provider(provider)?;
            let target = context_root.join(desc.project_config);
            if target.exists() {
                None
            } else {
                Some(provider.clone())
            }
        })
        .collect()
}

// ---- Compile session context ---------------------------------------------

/// Compile session context and return the resolved provider list.
///
/// Writes provider config files to the worktree as a side effect. Returns the
/// providers that were exported.
pub(crate) fn compile_session_context(
    ship_dir: &Path,
    branch: &str,
    cwd: Option<&str>,
    agent_id_override: Option<&str>,
) -> Result<Vec<String>> {
    let agent_id = agent_id_override
        .map(|a| a.to_string())
        .and_then(|value| normalize_optional_text(Some(value)));
    let resolved_agent =
        resolve_session_agent_config(ship_dir, agent_id.as_deref())?;
    let providers = resolved_agent.providers.clone();
    if providers.is_empty() {
        return Err(anyhow!(
            "No valid providers resolved for session on branch '{}'",
            branch
        ));
    }

    let mcp_server_filter = resolved_agent
        .mcp_servers
        .iter()
        .map(|server| server.id.clone())
        .collect::<Vec<_>>();
    let skill_filter = resolved_agent
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect::<Vec<_>>();

    let context_root = resolve_session_context_root(ship_dir, cwd);
    for provider in &providers {
        crate::agents::export::export_to_filtered_with_mode_override_and_skills_at_root(
            ship_dir.to_path_buf(),
            provider,
            Some(mcp_server_filter.as_slice()),
            Some(skill_filter.as_slice()),
            agent_id.as_deref(),
            &context_root,
        )
        .map_err(|error| {
            error.context(format!(
                "Failed to compile provider '{}' for branch '{}'",
                provider, branch
            ))
        })?;
    }

    Ok(providers)
}
