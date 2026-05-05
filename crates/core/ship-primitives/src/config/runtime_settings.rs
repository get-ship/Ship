use super::project::ProjectConfig;
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

/// Returns the project's persisted providers list, if any has been written.
/// None when the project is fresh (no `providers` key in KV yet).
pub(super) fn get_runtime_settings(ship_dir: &Path) -> Result<Option<Vec<String>>> {
    let raw = kv::get(project_ns(ship_dir), "providers")?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let providers: Vec<String> = serde_json::from_value(raw).unwrap_or_default();
    Ok(Some(providers))
}

pub(super) fn save_runtime_settings(ship_dir: &Path, config: &ProjectConfig) -> Result<()> {
    kv::set(project_ns(ship_dir), "providers", &serde_json::to_value(&config.providers)?)?;
    Ok(())
}

pub(super) fn migrate_json_config(_path: &std::path::Path) -> Result<ProjectConfig> {
    // Legacy JSON config used to carry `statuses` — that field is gone.
    // Fall back to a fresh default config; the user can repopulate via
    // Studio if they want their old workflow back.
    Ok(ProjectConfig::default())
}

