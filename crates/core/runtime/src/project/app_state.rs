//! Global app state stored at `~/.ship/app_state.json`: active project + recents.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::global::get_global_dir;
use super::registry::normalize_registry_project_path;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AppState {
    pub active_project: Option<PathBuf>,
    pub recent_projects: Vec<PathBuf>,
}

pub fn load_app_state() -> Result<AppState> {
    let path = get_global_dir()?.join("app_state.json");
    if !path.exists() {
        return Ok(AppState::default());
    }
    let content = fs::read_to_string(path)?;
    let mut state: AppState =
        serde_json::from_str(&content).context("Failed to parse app state")?;
    if normalize_app_state_paths(&mut state) {
        let _ = save_app_state(&state);
    }
    Ok(state)
}

pub fn save_app_state(state: &AppState) -> Result<()> {
    let path = get_global_dir()?.join("app_state.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn set_active_project_global(path: PathBuf) -> Result<()> {
    let mut state = load_app_state()?;
    state.active_project = Some(path.clone());

    // Add to recent projects if not already there
    if !state.recent_projects.contains(&path) {
        state.recent_projects.insert(0, path);
        // Keep only last 10
        state.recent_projects.truncate(10);
    }
    save_app_state(&state)
}

pub fn get_active_project_global() -> Result<Option<PathBuf>> {
    let state = load_app_state()?;
    Ok(state.active_project)
}

pub fn get_recent_projects_global() -> Result<Vec<PathBuf>> {
    let state = load_app_state()?;
    Ok(state.recent_projects)
}

pub(crate) fn normalize_app_state_paths(state: &mut AppState) -> bool {
    let mut changed = false;

    if let Some(active) = state.active_project.clone() {
        let normalized = normalize_registry_project_path(&active);
        if normalized != active {
            state.active_project = Some(normalized);
            changed = true;
        }
    }

    let mut normalized_recent = Vec::with_capacity(state.recent_projects.len());
    for path in &state.recent_projects {
        let normalized = normalize_registry_project_path(path);
        if normalized != *path {
            changed = true;
        }
        if !normalized_recent.contains(&normalized) {
            normalized_recent.push(normalized);
        } else {
            changed = true;
        }
    }

    if normalized_recent != state.recent_projects {
        state.recent_projects = normalized_recent;
        changed = true;
    }

    changed
}
