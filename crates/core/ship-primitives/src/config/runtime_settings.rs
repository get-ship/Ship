use super::project::ProjectConfig;
use super::types::{AiConfig, GitConfig, StatusConfig};
use ship_substrate::db::kv;
use ship_substrate::db::kv::KvNamespace;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Return the default value for a well-known setting key.
///
/// Defaults are reserved for non-identity infrastructure choices (terminal
/// program, daemon port). Identity-shaped values like "which AI provider"
/// or "which model" never default — those need explicit user input on first
/// run, since silently picking an AI the user hasn't installed or paid for
/// is a footgun.
pub fn setting_default(key: &str) -> Option<&'static str> {
    match key {
        "terminal.program" => Some("tmux"),
        "daemon.port" => Some("9315"),
        _ => None,
    }
}

/// Get a single setting, falling back to the default.
pub fn get_setting(key: &str) -> Result<Option<serde_json::Value>> {
    if let Some(val) = kv::get(KvNamespace::Settings, key)? {
        return Ok(Some(val));
    }
    Ok(setting_default(key).map(|d| serde_json::Value::String(d.to_string())))
}

/// Set a single setting.
pub fn set_setting(key: &str, value: serde_json::Value) -> Result<()> {
    kv::set(KvNamespace::Settings, key, &value)
}

/// Return all persisted settings merged with defaults.
pub fn get_all_settings() -> Result<HashMap<String, serde_json::Value>> {
    let mut map = HashMap::new();
    // Seed only with non-identity infrastructure defaults.
    for (key, default) in &[
        ("terminal.program", "tmux"),
        ("daemon.port", "9315"),
    ] {
        map.insert(
            (*key).to_string(),
            serde_json::Value::String((*default).to_string()),
        );
    }
    // Overlay persisted values.
    for (key, value) in kv::list_all(KvNamespace::Settings)? {
        map.insert(key, value);
    }
    Ok(map)
}

/// Derive a project-scoped KV namespace from the ship_dir path.
/// Each project gets its own namespace so settings don't leak across projects.
pub(super) fn project_ns(ship_dir: &Path) -> KvNamespace {
    KvNamespace::ProjectConfig(format!("runtime::{}", ship_dir.display()))
}

pub(super) fn normalize_git_config(mut git: GitConfig) -> GitConfig {
    if git.commit.iter().any(|entry| entry == "agents") {
        for entry in ["mcp", "permissions", "rules"] {
            if !git.commit.iter().any(|existing| existing == entry) {
                git.commit.push(entry.to_string());
            }
        }
        git.commit.retain(|entry| entry != "agents");
    }

    if git.ignore.iter().any(|entry| entry == "agents") {
        for entry in ["mcp", "permissions", "rules"] {
            if !git.ignore.iter().any(|existing| existing == entry) {
                git.ignore.push(entry.to_string());
            }
        }
    }

    git
}

#[allow(clippy::type_complexity)]
pub(super) fn get_runtime_settings(
    ship_dir: &Path,
) -> Result<
    Option<(
        Vec<String>,
        Option<Vec<StatusConfig>>,
        Option<AiConfig>,
        Option<GitConfig>,
    )>,
> {
    let providers: Vec<String> = kv::get(project_ns(ship_dir), "providers")?
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();

    // If providers key doesn't exist at all, return None (fresh project).
    if providers.is_empty() && kv::get(project_ns(ship_dir), "providers")?.is_none() {
        return Ok(None);
    }

    let statuses: Vec<StatusConfig> = kv::get(project_ns(ship_dir), "statuses")?
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();
    let statuses = if statuses.is_empty() { None } else { Some(statuses) };

    let ai: Option<AiConfig> = kv::get(project_ns(ship_dir), "ai")?
        .and_then(|v| serde_json::from_value(v).ok());

    let git_val = kv::get(project_ns(ship_dir), "git")?;
    let git = git_val
        .and_then(|v| serde_json::from_value::<GitConfig>(v).ok())
        .map(normalize_git_config);

    Ok(Some((providers, statuses, ai, git)))
}

pub(super) fn save_runtime_settings(ship_dir: &Path, config: &ProjectConfig) -> Result<()> {
    kv::set(project_ns(ship_dir), "providers", &serde_json::to_value(&config.providers)?)?;
    kv::set(project_ns(ship_dir), "statuses", &serde_json::to_value(&config.statuses)?)?;
    kv::set(project_ns(ship_dir), "ai", &serde_json::to_value(&config.ai)?)?;
    kv::set(project_ns(ship_dir), "git", &serde_json::to_value(normalize_git_config(config.git.clone()))?)?;
    Ok(())
}

pub(super) fn migrate_json_config(path: &Path) -> Result<ProjectConfig> {
    use std::fs;
    #[derive(serde::Deserialize, Default)]
    struct LegacyConfig {
        statuses: Option<Vec<String>>,
    }

    let content = fs::read_to_string(path)?;
    let legacy: LegacyConfig = serde_json::from_str(&content).unwrap_or_default();

    let statuses = legacy
        .statuses
        .unwrap_or_default()
        .into_iter()
        .map(|id| StatusConfig {
            name: id_to_name(&id),
            color: default_color_for(&id),
            id,
        })
        .collect();
    Ok(ProjectConfig {
        statuses,
        ..Default::default()
    })
}

pub(super) fn id_to_name(id: &str) -> String {
    id.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn default_color_for(id: &str) -> String {
    match id {
        "backlog" => "gray".into(),
        "in-progress" => "blue".into(),
        "review" => "yellow".into(),
        "done" => "green".into(),
        "blocked" => "red".into(),
        _ => "gray".into(),
    }
}
