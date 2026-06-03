//! Space info composition — identity + effective settings overlay.
//!
//! `info` is the single-space introspection read: a space's identity (id / root
//! / name from the `projects` projection) plus its effective settings. The
//! settings are layered **global ⟵ space**: the daemon-global settings object is
//! the base, overlaid per-key by the space's event-sourced overrides
//! (`space_settings::get_all`). Space overrides win per-key. The base global
//! object is untouched — the overlay is read-time.
//!
//! The base global settings object is supplied by the caller (the daemon reads
//! its `settings.state` snapshot and passes it in). This keeps the daemon thin
//! — it marshals the global snapshot — while the *overlay* (the product logic)
//! lives here.

use serde_json::Value;

/// Identity + effective settings for a space.
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    /// Path-hash space identity, if the daemon resolved one.
    pub space_id: Option<String>,
    /// Filesystem root of the space (from the projects projection).
    pub root: Option<String>,
    /// Human label of the space (from the projects projection).
    pub name: Option<String>,
    /// Effective settings = global base overlaid per-key by space overrides.
    pub settings: Value,
}

/// Compose [`SpaceInfo`] for `space_id` over a global-settings base.
///
/// `global_settings` is the daemon-global settings object (the overlay base).
/// `space_id` is the space whose identity + overrides are composed; `None`
/// means the daemon resolved no space, so identity is empty and the settings
/// pass through unoverlaid.
pub fn compose(space_id: Option<&str>, global_settings: Value) -> SpaceInfo {
    let (root, name) = match space_id {
        Some(id) => match crate::projections::projects::get(id) {
            Ok(Some(row)) => (Some(row.path), Some(row.name)),
            _ => (None, None),
        },
        None => (None, None),
    };

    let mut settings = global_settings;
    if let (Some(id), Some(obj)) = (space_id, settings.as_object_mut())
        && let Ok(overrides) = crate::projections::space_settings::get_all(id)
    {
        for (key, value) in overrides {
            obj.insert(key, value);
        }
    }

    SpaceInfo {
        space_id: space_id.map(str::to_string),
        root,
        name,
        settings,
    }
}
