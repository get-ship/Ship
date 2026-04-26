use crate::project::sanitize_file_name;
use anyhow::Result;
use chrono::{DateTime, Utc};

use super::types::*;

// ---- Parsing helpers -------------------------------------------------------

pub(crate) fn parse_datetime(value: &str) -> DateTime<Utc> {
    std::str::FromStr::from_str(value).unwrap_or_else(|_| Utc::now())
}

pub(crate) fn parse_datetime_opt(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|entry| std::str::FromStr::from_str(&entry).ok())
}

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn normalize_agent_ref(agent: &str) -> Option<String> {
    let trimmed = agent.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn normalize_provider_ref(provider: &str) -> Option<String> {
    let trimmed = provider.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// ---- Branch key ------------------------------------------------------------

pub(crate) fn ensure_branch_key(branch: &str) -> Result<&str> {
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("Workspace branch/key cannot be empty"));
    }
    Ok(trimmed)
}

// ---- Workspace construction ------------------------------------------------

pub(crate) fn workspace_id_from_branch(branch: &str) -> String {
    sanitize_file_name(branch)
}

pub(crate) fn new_workspace(branch: &str, now: DateTime<Utc>) -> Workspace {
    Workspace {
        id: workspace_id_from_branch(branch),
        name: branch.to_string(),
        path: None,
        active_agent: None,
        created_at: now,
    }
}

// ---- Agent validation ------------------------------------------------------

pub(crate) fn validate_agent_exists(ship_dir: &std::path::Path, agent_id: &str) -> anyhow::Result<String> {
    let normalized = normalize_agent_ref(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Workspace agent cannot be empty"))?;
    let effective = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    if effective.modes.iter().any(|mode| mode.id == normalized) {
        Ok(normalized)
    } else {
        Err(anyhow::anyhow!("Agent '{}' not found", normalized))
    }
}

