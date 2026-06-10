//! Space info composition — identity + effective settings overlay.
//!
//! `info` is the single-space introspection read: a space's identity (id / root
//! / name from the `projects` registry) plus its effective settings. Settings
//! are layered **global ⟵ space**: the daemon-global settings object is the base,
//! overlaid per-key by the space's event-sourced overrides (`config::get_all`).
//! Space overrides win per-key; non-overridden keys survive; the base global
//! object is untouched (the overlay is read-time).
//!
//! The base global settings object is supplied by the caller (the daemon reads
//! its `settings.state` snapshot and passes it in). This keeps the daemon thin —
//! it marshals the global snapshot — while the *overlay* (the product logic)
//! lives here.
//!
//! Identity reads come from the `projects` SQL table via `registry::get(kernel,
//! id)` (dissolution of the `space_registry` DataStore table, 2026-06-10); the
//! overlay reads come from the per-space `settings_store` (DataStore port).

use serde_json::Value;

use ship_substrate::ports::{DataStore, KernelLog};

use crate::error::SpaceError;
use crate::types::SpaceInfo;
use crate::{config, registry};

/// Compose [`SpaceInfo`] for `space_id` over a global-settings base.
///
/// `global_settings` is the daemon-global settings object (the overlay base).
/// `space_id` is the space whose identity + overrides are composed; `None` means
/// no space resolved, so identity is empty and the settings pass through
/// unoverlaid.
///
/// `kernel` provides the ship_dir for identity lookups (reads from `projects`
/// SQL table via `registry::get`); `settings_store` is the per-space store
/// (resolves the `space_settings` overlay). When both point at the same physical
/// db, the read still works — the tables are distinct.
pub async fn compose(
    kernel: &KernelLog,
    settings_store: &dyn DataStore,
    space_id: Option<&str>,
    global_settings: Value,
) -> Result<SpaceInfo, SpaceError> {
    let (root, name) = match space_id {
        Some(id) => match registry::get(kernel, id).await? {
            Some(rec) => (Some(rec.path), Some(rec.name)),
            None => (None, None),
        },
        None => (None, None),
    };

    let mut settings = global_settings;
    if let (Some(_id), Some(obj)) = (space_id, settings.as_object_mut()) {
        // `settings_store` is already bound to the target space's partition, so
        // the overlay read needs no space arg — the store IS the per-space view.
        let overrides = config::get_all(settings_store).await?;
        for (key, value) in overrides {
            obj.insert(key, value);
        }
    }

    Ok(SpaceInfo {
        space_id: space_id.map(str::to_string),
        root,
        name,
        settings,
    })
}
