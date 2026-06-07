//! Space registry over substrate ports — list / active / add / activate.
//!
//! The `projects` table + the `active_space` pointer are Runtime-scoped
//! (runtime.db): the machine-wide record of known space paths, read daemon-wide
//! with no space in hand (the source the space-id resolver itself consults). The
//! crate is the sole writer — it persists rows through the [`DataStore`] port
//! and emits the matching `project.*` events through [`KernelLog`] (the kernel
//! routes them to runtime.db). No sqlite/sqlx and no raw fs here: path → id is
//! the substrate `path_hash` helper (a pure hash, not a filesystem touch).
//!
//! [`list`] also merges entries from the legacy `~/.ship/projects.json` file
//! (kept from the old `workspaces.list` registry op) so that installs whose
//! event-sourced projection is empty still return a populated list. Legacy
//! entries use the `path_hash`-derived id scheme identical to the event-sourced
//! keying, and are filtered if the path already appears in the DataStore table.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use ship_substrate::home::path_hash;
use ship_substrate::ports::{DataStore, KernelLog};
use ship_substrate::project::{get_registry_path, load_registry};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{PROJECT_REGISTERED, PROJECT_SEEN, PROJECT_SWITCHED};
use ship_types::payload::{ProjectRegistered, ProjectSeen, ProjectSwitched};

use crate::error::SpaceError;
use crate::helpers::{now_rfc3339, runtime_emit_ctx};
use crate::projection;
use crate::types::SpaceRecord;

/// Derive the human label from a path's final segment.
fn derive_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("space")
        .to_string()
}

/// List all live (non-archived) spaces, newest `last_seen_at` first.
///
/// Also merges any `~/.ship/projects.json` entries whose path has not yet been
/// materialised in the DataStore projection. Legacy entries receive a
/// `path_hash`-derived id (identical to the scheme used by [`add`]) so they
/// are stable and comparable with event-sourced rows. Any path that IS already
/// in the DataStore table is skipped.
pub async fn list(store: &dyn DataStore) -> Result<Vec<SpaceRecord>, SpaceError> {
    let mut records = projection::list_active(store).await?;
    let seen_paths: HashSet<String> = records.iter().map(|r| r.path.clone()).collect();
    for entry in load_legacy_entries() {
        let path_str = entry.path.to_string_lossy().to_string();
        if seen_paths.contains(&path_str) {
            continue;
        }
        let id = path_hash(&entry.path);
        records.push(SpaceRecord {
            id,
            name: entry.name,
            path: path_str,
            created_at: String::new(),
            last_seen_at: String::new(),
        });
    }
    Ok(records)
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

/// The currently-active space record, or `None` on cold boot (or when the stored
/// id no longer resolves to a live row).
pub async fn active(store: &dyn DataStore) -> Result<Option<SpaceRecord>, SpaceError> {
    let Some(id) = projection::get_active_id(store).await? else {
        return Ok(None);
    };
    Ok(get(store, &id).await?)
}

/// Look up a registry record by id.
pub async fn get(store: &dyn DataStore, id: &str) -> Result<Option<SpaceRecord>, SpaceError> {
    Ok(projection::get(store, id)
        .await?
        .map(|row| projection::row_to_record(&row)))
}

/// Idempotent register. Re-adding a known path returns the SAME id with its
/// `last_seen_at` bumped and the original name preserved; a new path mints a
/// `project.registered` event. The id is the path hash (stable across renames).
///
/// `path` is taken as the caller supplies it — canonicalization is a host
/// concern the daemon performs before calling (the crate touches no fs).
pub async fn add(
    store: &dyn DataStore,
    kernel: &KernelLog,
    path: &Path,
    name: Option<&str>,
) -> Result<SpaceRecord, SpaceError> {
    let id = path_hash(path);
    let now = now_rfc3339();

    if let Some(existing) = get(store, &id).await? {
        // Known path → cheap `project.seen` (bump last_seen_at, keep name).
        projection::touch(store, &id, &now).await?;
        let envelope = EventEnvelope::new(PROJECT_SEEN, &id, &ProjectSeen { seen_at: now.clone() })
            .map_err(|e| SpaceError::Internal(e.to_string()))?;
        kernel.emit(envelope, &runtime_emit_ctx()).await?;
        return Ok(SpaceRecord { last_seen_at: now, ..existing });
    }

    let resolved_name = name
        .map(str::to_string)
        .unwrap_or_else(|| derive_name(path));
    let path_str = path.to_string_lossy().to_string();
    let record = SpaceRecord {
        id: id.clone(),
        path: path_str.clone(),
        name: resolved_name.clone(),
        created_at: now.clone(),
        last_seen_at: now.clone(),
    };
    projection::upsert_record(store, &record).await?;

    let payload = ProjectRegistered {
        path: path_str,
        name: resolved_name,
        label: None,
        created_at: now.clone(),
    };
    let envelope = EventEnvelope::new(PROJECT_REGISTERED, &id, &payload)
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
    kernel.emit(envelope, &runtime_emit_ctx()).await?;

    Ok(record)
}

/// Switch the active space to `id`. Bumps `last_seen_at`, writes the active
/// pointer, and emits `project.switched`. Errors with `BadArgs` if `id` does not
/// resolve to a known space. Returns the new active record.
pub async fn activate(
    store: &dyn DataStore,
    kernel: &KernelLog,
    id: &str,
) -> Result<SpaceRecord, SpaceError> {
    let record = get(store, id)
        .await?
        .ok_or_else(|| SpaceError::BadArgs(format!("space '{id}' not known")))?;
    let previous = projection::get_active_id(store).await?;
    let now = now_rfc3339();

    projection::set_active_id(store, &record.id, &now).await?;
    let _ = projection::touch(store, &record.id, &now).await?;

    let payload = ProjectSwitched {
        space_id: record.id.clone(),
        name: record.name.clone(),
        path: record.path.clone(),
        previous,
        at: now.clone(),
    };
    let envelope = EventEnvelope::new(PROJECT_SWITCHED, &record.id, &payload)
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
    // The switched-to space's id IS its space — carry it through the binding so
    // the project.switched event is attributable, even though the registry
    // tables themselves are Runtime-scoped.
    kernel
        .emit(envelope, &crate::helpers::space_emit_ctx(&record.id))
        .await?;

    Ok(SpaceRecord { last_seen_at: now, ..record })
}
