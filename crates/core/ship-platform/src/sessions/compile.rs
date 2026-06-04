use crate::agents::config::{
    ProviderSettings, resolve_harness_settings_with_agent_override,
};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

// The session helpers folded into the ship-agents crate (agents-arc E4); the
// compiler bridge stays here (it names `crate::agents::config`, an L1 type).
use ship_agents::sessions::helpers::*;

// ---- Agent config resolution -----------------------------------------------

pub(crate) fn resolve_session_agent_config(
    ship_dir: &Path,
    agent_id: Option<&str>,
) -> Result<ProviderSettings> {
    let agent_id = agent_id
        .and_then(normalize_agent_ref)
        .map(|a| a.to_string());

    resolve_harness_settings_with_agent_override(
        ship_dir,
        None,
        agent_id.as_deref(),
    )
}

pub(crate) fn resolve_session_harnesses(
    ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<Vec<String>> {
    let resolved = resolve_session_agent_config(ship_dir, agent_id)?;
    if resolved.harnesses.is_empty() {
        return Err(anyhow!(
            "No valid harnesses resolved for session on branch '{}'",
            branch
        ));
    }
    Ok(resolved.harnesses)
}

// ---- Context root and missing configs --------------------------------------

/// Resolve the on-disk root where harness config files should be written.
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

// ---- Compile session context ---------------------------------------------

/// Ensure harness config files exist in the project root before session launch.
///
/// Runs `ship compile` as a subprocess — best-effort. Compilation errors are
/// logged as warnings but do not block session start (configs may already exist
/// from a prior compile run). Returns the resolved harness list.
pub(crate) fn compile_session_context(
    ship_dir: &Path,
    branch: &str,
    cwd: Option<&str>,
    agent_id_override: Option<&str>,
) -> Result<Vec<String>> {
    let agent_id = agent_id_override
        .map(|a| a.to_string())
        .and_then(|value| normalize_optional_text(Some(value)));
    let harnesses = resolve_session_harnesses(ship_dir, branch, agent_id.as_deref())?;

    let context_root = resolve_session_context_root(ship_dir, cwd);
    let project_root = ship_dir.parent().unwrap_or(ship_dir);

    let mut cmd = std::process::Command::new("ship");
    cmd.arg("compile")
        .arg("--path")
        .arg(project_root)
        .current_dir(&context_root);
    match cmd.output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            tracing::warn!(
                "ship compile failed for session on branch '{}': {}",
                branch,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Err(e) => {
            tracing::debug!(
                "ship binary not found at session start for branch '{}': {}",
                branch,
                e
            );
        }
    }

    Ok(harnesses)
}
