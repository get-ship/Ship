//! Space registry over substrate ports — list / active / add / activate.
//!
//! The `projects` table + the `active_space` pointer are Runtime-scoped
//! (runtime.db): the machine-wide record of known space paths, read daemon-wide
//! with no space in hand (the source the space-id resolver itself consults).
//!
//! ## Single-table design (keystone-A dual-read-model dissolution, 2026-06-10)
//!
//! The former `space_registry` DataStore table is **dissolved**. ONE table
//! survives: the event-sourced `projects` SQL table (schema in
//! `ship-substrate/migrations/0001_schema.sql`), materialized by
//! `ProjectsProjection` (owned here). Record reads go through
//! `ship_substrate::db::projects::*_at(ship_dir)` helpers; the kernel provides
//! the `ship_dir` via `KernelLog::try_ship_dir()`. The DataStore port is kept
//! only for the `space_active` single-row pointer.
//!
//! The write path emits events only; `ProjectsProjection` folds them into
//! the `projects` SQL table inside the same kernel transaction (same-TX
//! guarantee identical to the former direct-write path).
//!
//! The sync SQL READ HELPERS (`list_active` / `get` / `find_by_path`) STAY in
//! `ship_substrate::db::projects` so the space-id resolver and `SpaceRegistry`
//! impl can keep working unchanged without any async overhead.

use std::path::{Path, PathBuf};

use ship_substrate::home::path_hash;
use ship_substrate::ports::{DataStore, KernelLog};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{PROJECT_REGISTERED, PROJECT_SEEN, PROJECT_SWITCHED};
use ship_types::payload::{ProjectRegistered, ProjectSeen, ProjectSwitched};

use crate::error::SpaceError;
use crate::helpers::{now_rfc3339, runtime_emit_ctx};
use crate::projection;
use crate::types::SpaceRecord;

// ── Conversion from L0 read type ─────────────────────────────────────────────

impl From<ship_substrate::db::projects::SpaceRecord> for SpaceRecord {
    fn from(r: ship_substrate::db::projects::SpaceRecord) -> Self {
        SpaceRecord {
            id: r.id,
            path: r.path,
            name: r.name,
            created_at: r.created_at,
            last_seen_at: r.last_seen_at,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Derive the human label from a path's final segment.
fn derive_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("space")
        .to_string()
}

/// Obtain the ship_dir from the kernel's router. Returns `None` if the router
/// is momentarily locked — callers degrade gracefully (empty list / not-found).
fn ship_dir(kernel: &KernelLog) -> Option<PathBuf> {
    kernel.try_ship_dir()
}

// ── Public registry API ───────────────────────────────────────────────────────

/// List all live (non-archived) spaces, newest `last_seen_at` first.
pub async fn list(kernel: &KernelLog) -> Result<Vec<SpaceRecord>, SpaceError> {
    let records = match ship_dir(kernel) {
        Some(dir) => ship_substrate::db::projects::list_active_at(&dir)
            .map_err(|e| SpaceError::Internal(e.to_string()))?
            .into_iter()
            .map(SpaceRecord::from)
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };
    Ok(records)
}

/// The currently-active space record, or `None` on cold boot (or when the stored
/// id no longer resolves to a live row).
pub async fn active(store: &dyn DataStore, kernel: &KernelLog) -> Result<Option<SpaceRecord>, SpaceError> {
    let Some(id) = projection::get_active_id(store).await? else {
        return Ok(None);
    };
    Ok(get(kernel, &id).await?)
}

/// Look up a registry record by id. Reads from the `projects` SQL table via
/// the kernel's ship_dir.
pub async fn get(kernel: &KernelLog, id: &str) -> Result<Option<SpaceRecord>, SpaceError> {
    let Some(dir) = ship_dir(kernel) else {
        return Ok(None);
    };
    ship_substrate::db::projects::get_at(&dir, id)
        .map(|opt| opt.map(SpaceRecord::from))
        .map_err(|e| SpaceError::Internal(e.to_string()))
}

/// Idempotent register. Re-adding a known path returns the SAME id with its
/// `last_seen_at` bumped and the original name preserved; a new path mints a
/// `project.registered` event. The id is the path hash (stable across renames).
///
/// `path` is taken as the caller supplies it — canonicalization is a host
/// concern the daemon performs before calling (the crate touches no fs).
///
/// The write path is event-only: `ProjectsProjection` folds the emitted event
/// into the `projects` SQL table inside the kernel transaction. The DataStore
/// port is no longer used for record writes.
pub async fn add(
    store: &dyn DataStore,
    kernel: &KernelLog,
    path: &Path,
    name: Option<&str>,
) -> Result<SpaceRecord, SpaceError> {
    let id = path_hash(path);
    let now = now_rfc3339();

    // Check for an existing registration by path via the SQL table.
    let existing = match ship_dir(kernel) {
        Some(dir) => ship_substrate::db::projects::get_at(&dir, &id)
            .map(|opt| opt.map(SpaceRecord::from))
            .map_err(|e| SpaceError::Internal(e.to_string()))?,
        None => None,
    };

    if let Some(existing) = existing {
        // Known path → cheap `project.seen` (bump last_seen_at, keep name).
        // The fold writes to `projects` SQL; no DataStore write needed.
        let envelope =
            EventEnvelope::new(PROJECT_SEEN, &id, &ProjectSeen { seen_at: now.clone() })
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

    let payload = ProjectRegistered {
        path: path_str,
        name: resolved_name,
        created_at: now.clone(),
    };
    let envelope = EventEnvelope::new(PROJECT_REGISTERED, &id, &payload)
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
    kernel.emit(envelope, &runtime_emit_ctx()).await?;

    // The `_store` parameter is retained (but unused for records) to keep the
    // caller signature stable; it is still used by `activate` for the active
    // pointer.
    let _ = store;

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
    let record = get(kernel, id)
        .await?
        .ok_or_else(|| SpaceError::BadArgs(format!("space '{id}' not known")))?;
    let previous = projection::get_active_id(store).await?;
    let now = now_rfc3339();

    projection::set_active_id(store, &record.id, &now).await?;

    // Emit `project.seen` to bump `last_seen_at` in the `projects` SQL table
    // (previously done via `projection::touch`; now handled by ProjectsProjection).
    let seen_env =
        EventEnvelope::new(PROJECT_SEEN, &record.id, &ProjectSeen { seen_at: now.clone() })
            .map_err(|e| SpaceError::Internal(e.to_string()))?;
    kernel.emit(seen_env, &runtime_emit_ctx()).await?;

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
