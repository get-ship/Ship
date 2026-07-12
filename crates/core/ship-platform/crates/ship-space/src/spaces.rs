//! Read API over the `spaces` Runtime-scoped projection.
//!
//! The projection table is the device-local record of opened spaces (the
//! LaunchServices analogy). Identity lives here — `space_id` is the
//! mint-once nanoid assigned by [`identity::ensure_identity_for_root`] — and
//! `path` is just the location on this device.
//!
//! Reads are sync helpers; writes are event-only (the kernel routes them
//! into the append log and `spaces_projection::SpacesProjection` materialises
//! the resulting row in the same transaction). This module never imports
//! sqlx — it opens the runtime.db under an explicit `ship_dir` via L0's
//! `db::spaces::*_at` helpers.

use std::path::{Path, PathBuf};

use ship_substrate::db;
use ship_substrate::ports::KernelLog;

use crate::error::SpaceError;
use crate::types::SpaceRecord;

fn ship_dir(kernel: &KernelLog) -> Option<PathBuf> {
    kernel.try_ship_dir()
}

fn err(e: impl std::fmt::Display) -> SpaceError {
    SpaceError::Internal(e.to_string())
}

/// List every space currently open on this device, newest `last_seen_at` first.
pub fn list(kernel: &KernelLog) -> Result<Vec<SpaceRecord>, SpaceError> {
    let records = match ship_dir(kernel) {
        Some(dir) => db::spaces::list_open_at(&dir)
            .map_err(err)?
            .into_iter()
            .map(SpaceRecord::from)
            .collect(),
        None => Vec::new(),
    };
    Ok(records)
}

/// Look up an open space by its identity (`space_id`).
pub fn get(kernel: &KernelLog, space_id: &str) -> Result<Option<SpaceRecord>, SpaceError> {
    let Some(dir) = ship_dir(kernel) else {
        return Ok(None);
    };
    db::spaces::get_at(&dir, space_id)
        .map(|opt| opt.map(SpaceRecord::from))
        .map_err(err)
}

/// Look up an open space by its current filesystem path.
///
/// Returns `None` for paths the runtime has never opened. To turn a path into
/// identity (mint-once), reach for [`identity::ensure_identity_for_root`] —
/// this function never mints.
pub fn find_by_path(kernel: &KernelLog, path: &Path) -> Result<Option<SpaceRecord>, SpaceError> {
    let Some(dir) = ship_dir(kernel) else {
        return Ok(None);
    };
    db::spaces::find_by_path_at(&dir, &path.to_string_lossy())
        .map(|opt| opt.map(SpaceRecord::from))
        .map_err(err)
}

/// The stacks registered as members of `space_id`, in registration order.
pub fn list_stacks(
    kernel: &KernelLog,
    space_id: &str,
) -> Result<Vec<db::spaces::StackRow>, SpaceError> {
    let Some(dir) = ship_dir(kernel) else {
        return Ok(Vec::new());
    };
    db::spaces::list_stacks_at(&dir, space_id).map_err(err)
}
