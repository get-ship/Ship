//! Space info — a space's identity (id / root / name from `projects`).
//!
//! The per-space settings overlay moved to the unified `settings_overlay` (S4):
//! `SettingsService.Get/List(scope=SPACE)` is the single read surface. `Info`
//! returns only the identity triple; consumers overlay settings via
//! `SettingsService`.

use ship_substrate::ports::KernelLog;

use crate::error::SpaceError;
use crate::types::SpaceInfo;
use crate::registry;

/// Compose [`SpaceInfo`] for `space_id`. Returns the identity triple
/// (id / root / name from the `projects` table) or an unset identity when
/// `space_id` is `None` (no space bound for this connection).
///
/// Per-space settings are NOT composed here — they live on the unified
/// `settings_overlay` table, read via `ship_config::overlay::get(store, key)`.
/// Callers fetch the effective settings via `SettingsService`.
pub async fn compose(
    kernel: &KernelLog,
    space_id: Option<&str>,
) -> Result<SpaceInfo, SpaceError> {
    let (root, name) = match space_id {
        Some(id) => match registry::get(kernel, id).await? {
            Some(rec) => (Some(rec.path), Some(rec.name)),
            None => return Err(SpaceError::NotFound(format!("space '{id}' not found"))),
        },
        None => (None, None),
    };

    Ok(SpaceInfo {
        space_id: space_id.map(str::to_string),
        root,
        name,
    })
}
