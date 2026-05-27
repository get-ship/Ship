//! Space commands — per-space settings overrides (#129).
//!
//! `space.setting.set` is the event-sourced per-space override write: it emits
//! `space.setting.changed` with the caller's space riding the envelope
//! (`with_context`), NOT the payload — mirror `ConfigFileChanged`. The
//! `space_settings` projection materializes the overlay; `SpaceService.Info`
//! layers it over the global daemon settings (`settings.state`).
//!
//! C1 like `settings.set`. The space is resolved from the caller's emit
//! context (the #128 per-connection space, falling back to the daemon's home
//! space); a request with no resolvable space is rejected — there is no overlay
//! to attach the override to.

use serde_json::Value;

use ship_substrate::events::EventEnvelope;
use ship_types::event_types::SPACE_SETTING_CHANGED;
use ship_types::payload::SpaceSettingChanged;

use super::{req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "space.setting.set" => {
            let key = req_str(args, "key")?;
            let raw = req_str(args, "value_json")?;
            // Validate the value is real JSON before persisting (same posture as
            // settings.set). The stored override re-serializes the parsed value.
            let value: Value = serde_json::from_str(&raw).map_err(|e| {
                CommandError::bad_args(format!("`value_json` is not valid JSON: {e}"))
            })?;

            let emit_ctx = ctx.auth.to_emit_context();
            let space_id = emit_ctx.space_id.clone().ok_or_else(|| {
                CommandError::bad_args(
                    "no space resolved for this request — a per-space setting needs a space",
                )
            })?;

            let payload = SpaceSettingChanged { key: key.clone(), value_json: value.to_string() };
            let envelope = EventEnvelope::new(SPACE_SETTING_CHANGED, &key, &payload)
                .map_err(|e| CommandError::service(e.to_string()))?
                .with_context(Some(&space_id), emit_ctx.session_id.as_deref());

            ctx.state
                .kernel()
                .lock()
                .await
                .emit(envelope, &emit_ctx)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;

            // Echo the typed SettingValue shape (value re-serialized as a string).
            Ok(Some(serde_json::json!({ "key": key, "value_json": value.to_string() })))
        }
        _ => Ok(None),
    }
}
