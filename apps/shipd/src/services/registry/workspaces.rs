//! Single-source `workspaces.*` ops — descriptors + dispatch via `ship_ops!`.
//!
//! Surfaces the project registry through the `_sui/data` SSE plane. Two
//! registries feed the list: the event-sourced `ship_platform::projects`
//! (preferred) and the legacy `~/.ship/projects.json` (fallback for installs
//! where the projection is empty). Legacy entries get `path_hash`-derived ids
//! so they line up with the event-sourced keying scheme.

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::Value;

use ship_substrate::home::path_hash;
use ship_substrate::project::{get_registry_path, load_registry};

use super::wire::{WorkspaceActiveResponse, WorkspaceRow, WorkspacesListResponse};
use super::RegistryError;

ship_ops! {
    WORKSPACES_OPS;
    {
        name: "workspaces.list",
        desc: "Known project workspaces with active flag per row.",
        perm: "ship.space.read",
        args: &[],
        ret: WorkspacesListResponse,
        run: |_args, _state, _files| {
            let active = active_id();
            let records = ship_platform::projects::list()
                .map_err(|e| RegistryError::service(e.to_string()))?;
            let mut seen_paths: HashSet<String> = HashSet::new();
            let mut items: Vec<WorkspaceRow> = records
                .into_iter()
                .map(|r| {
                    let is_active = active.as_deref() == Some(r.id.as_str());
                    seen_paths.insert(r.path.clone());
                    WorkspaceRow {
                        id: r.id,
                        name: r.name,
                        path: r.path,
                        created_at: serde_json::to_value(r.created_at).unwrap_or(Value::Null),
                        last_seen_at: serde_json::to_value(r.last_seen_at).unwrap_or(Value::Null),
                        active: is_active,
                    }
                })
                .collect();

            // Merge in any `~/.ship/projects.json` entries the projection
            // hasn't materialised (keeps the view populated when startup
            // registration failed or for legacy-only installs).
            for entry in load_legacy_entries() {
                let path_str = entry.path.to_string_lossy().to_string();
                if seen_paths.contains(&path_str) {
                    continue;
                }
                let id = path_hash(&entry.path);
                let is_active = active.as_deref() == Some(id.as_str());
                seen_paths.insert(path_str.clone());
                items.push(WorkspaceRow {
                    id,
                    name: entry.name,
                    path: path_str,
                    created_at: Value::Null,
                    last_seen_at: Value::Null,
                    active: is_active,
                });
            }

            Ok(WorkspacesListResponse { items, active })
        },
    },
    {
        name: "workspaces.active",
        desc: "Currently-active workspace id (null if none).",
        perm: "ship.space.read",
        args: &[],
        ret: WorkspaceActiveResponse,
        run: |_args, _state, _files| Ok(WorkspaceActiveResponse { id: active_id() }),
    },
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

/// Tolerant reader for `~/.ship/projects.json`. Accepts the canonical
/// `{"projects": [...]}` shape and a bare `[...]` array (older CLI builds).
fn load_legacy_entries() -> Vec<LegacyEntry> {
    if let Ok(reg) = load_registry() {
        return reg
            .projects
            .into_iter()
            .map(|e| LegacyEntry { name: e.name, path: e.path })
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
            Some(LegacyEntry { name, path: PathBuf::from(path) })
        })
        .collect()
}
