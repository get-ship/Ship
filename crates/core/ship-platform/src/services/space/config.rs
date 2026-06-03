//! Per-space config read/write.
//!
//! Reads return a space's event-sourced override map (or a single key) from the
//! `space_settings` projection. The write is the event-sourced per-space
//! override: it validates the value is real JSON (same posture as
//! `settings.set`), then emits `space.setting.changed` through the injected
//! [`SpaceEvents`] port (the caller's space rides the envelope, not the
//! payload). The `space_settings` projection materializes the overlay; `info`
//! layers it over the global daemon settings.

use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use super::ports::SpaceEvents;
use super::SpaceServiceError;

/// All overrides for `space_id`, as a `key -> Value` map (empty when none).
pub fn get_all(space_id: &str) -> Result<HashMap<String, Value>> {
    crate::projections::space_settings::get_all(space_id)
}

/// A single override for `(space_id, key)`, or `None` when unset.
pub fn get(space_id: &str, key: &str) -> Result<Option<Value>> {
    crate::projections::space_settings::get(space_id, key)
}

/// The echoed shape of a per-space setting write: the key plus the re-serialized
/// JSON value (mirrors the typed `SettingValue` wire shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingValue {
    pub key: String,
    pub value_json: String,
}

/// Write a per-space override for `(space_id, key)`.
///
/// `raw_value_json` must parse as JSON (rejected with `bad_args` otherwise — the
/// same posture as `settings.set`). The stored override re-serializes the parsed
/// value (canonical form). Emits `space.setting.changed` through `events`; the
/// space rides the envelope inside the port impl. Returns the echoed value.
pub async fn set(
    events: &dyn SpaceEvents,
    space_id: &str,
    key: &str,
    raw_value_json: &str,
) -> Result<SettingValue, SpaceServiceError> {
    let value: Value = serde_json::from_str(raw_value_json)
        .map_err(|e| SpaceServiceError::bad_args(format!("`value_json` is not valid JSON: {e}")))?;
    let value_json = value.to_string();

    events
        .emit_setting_changed(space_id, key, &value_json)
        .await
        .map_err(|e| SpaceServiceError::service(e.to_string()))?;

    Ok(SettingValue {
        key: key.to_string(),
        value_json,
    })
}
