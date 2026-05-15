//! Registry dispatcher for `workspaces.*` ops. Surfaces the project
//! registry through the `_sui/data` SSE plane so `views/workspaces/`
//! and `views/sessions/` don't need to one-shot `GET /api/v1/projects`
//! out of band. Active-project switches emit `config.changed`
//! (`shell.project.active`), which already wakes the resolver.
//!
//! Two registries feed the list:
//! * Event-sourced `ship_platform::projects` (preferred — drives switching,
//!   `last_seen_at` recency, archive lifecycle).
//! * Legacy `~/.ship/projects.json` via `ship_substrate::project` (fallback
//!   for installs where the event projection is empty / startup
//!   registration failed). Entries from the legacy file are synthesised
//!   with `path_hash`-derived ids so they line up with the event-sourced
//!   keying scheme.

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::{Value, json};

use ship_substrate::home::path_hash;
use ship_substrate::project::{get_registry_path, load_registry};

use super::RegistryError;
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "workspaces.list" => {
            let active = active_id();
            let records = ship_platform::projects::list()
                .map_err(|e| RegistryError::service(e.to_string()))?;
            let mut seen_paths: HashSet<String> = HashSet::new();
            let mut items: Vec<Value> = records
                .into_iter()
                .map(|r| {
                    let is_active = active.as_deref() == Some(r.id.as_str());
                    seen_paths.insert(r.path.clone());
                    json!({
                        "id": r.id,
                        "name": r.name,
                        "path": r.path,
                        "created_at": r.created_at,
                        "last_seen_at": r.last_seen_at,
                        "active": is_active,
                    })
                })
                .collect();

            // Merge in any entries from `~/.ship/projects.json` that the
            // event projection hasn't materialised. Keeps the workspaces
            // view populated when startup registration failed (e.g. a
            // pending migration) or for installs that only ever used the
            // legacy CLI registry. We bypass `list_registered_projects`'s
            // worktree filter — Ship-feat-sail itself is a linked worktree
            // (the only entry the user has registered), and the view's
            // whole purpose is to surface those.
            for entry in load_legacy_entries() {
                let path_str = entry.path.to_string_lossy().to_string();
                if seen_paths.contains(&path_str) {
                    continue;
                }
                let id = path_hash(&entry.path);
                let is_active = active.as_deref() == Some(id.as_str());
                seen_paths.insert(path_str.clone());
                items.push(json!({
                    "id": id,
                    "name": entry.name,
                    "path": path_str,
                    "created_at": Value::Null,
                    "last_seen_at": Value::Null,
                    "active": is_active,
                }));
            }

            Ok(json!({
                "items": items,
                "active": active,
            }))
        }
        "workspaces.active" => Ok(json!({ "id": active_id() })),

        _ => return Ok(None),
    };
    Ok(Some(value?))
}

fn active_id() -> Option<String> {
    ship_platform::config::get_setting("shell.project.active")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
}

struct LegacyEntry {
    name: String,
    path: PathBuf,
}

/// Tolerant reader for `~/.ship/projects.json`. Accepts two on-disk shapes:
///
/// 1. The canonical `{"projects": [{"name", "path"}, ...]}` written by
///    `ship_substrate::project::save_registry`.
/// 2. A bare `[{"name", "path", ...}, ...]` array — the shape produced by
///    older CLI builds and still present in many active installs. Without
///    this fallback the strict `serde_json::from_str::<ProjectRegistry>`
///    deserialize errors out and the workspaces view goes blank even
///    though `projects.json` clearly has entries.
fn load_legacy_entries() -> Vec<LegacyEntry> {
    if let Ok(reg) = load_registry() {
        return reg
            .projects
            .into_iter()
            .map(|e| LegacyEntry {
                name: e.name,
                path: e.path,
            })
            .collect();
    }
    let Ok(path) = get_registry_path() else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return Vec::new();
    };
    let array = value.as_array().cloned().unwrap_or_default();
    array
        .into_iter()
        .filter_map(|entry| {
            let obj = entry.as_object()?;
            let path = obj.get("path").and_then(Value::as_str)?;
            let name = obj
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    PathBuf::from(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| path.to_string());
            Some(LegacyEntry {
                name,
                path: PathBuf::from(path),
            })
        })
        .collect()
}
