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

            let emit_ctx = ctx.auth.to_emit_context(ctx.auth.caller_kind.clone());
            // Intended P2 fail-soft (mandatory fix #2): with the
            // `daemon_space_id` emit-path fallback gone, an operator/daemon-token
            // caller (binding has no space) calling `space.set` hard-errors
            // instead of silently writing the daemon home space's config. An
            // operator with no space genuinely cannot set a space's setting.
            let space_id = emit_ctx.space_key().ok_or_else(|| {
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
        "space.window.close" => {
            // The daemon is the brain: it owns the window's pid-bound token and
            // lifecycle. Closing = revoke the token (instant authority cut) then
            // close the actor lifecycle. The OS frame is the shell's to destroy;
            // revocation neuters the webview regardless. C3 — operator-only (the
            // gate already enforced is_operator before we get here).
            let actor_id = req_str(args, "actor_id")?;

            let Some(window) = ship_substrate::projections::actor::get_actor(&actor_id)
                .map_err(|e| CommandError::service(e.to_string()))?
            else {
                return Err(CommandError::bad_args(format!("no window actor `{actor_id}`")));
            };
            if window.kind != "window" {
                return Err(CommandError::bad_args(format!(
                    "actor `{actor_id}` is not a window (kind={})",
                    window.kind
                )));
            }

            // Revoke: drop every session whose pid-bound identity matches this
            // window (same space + pid). A window is single-instance per space,
            // so this is the window's token.
            let token_revoked = {
                let mut reg = ctx.state.session_registry().lock().await;
                let before = reg.len();
                reg.retain(|_, s| {
                    !(s.space_id == window.space_id && s.pid == window.pid.map(|p| p as u32))
                });
                before != reg.len()
            };

            // Close the lifecycle (off-thread: durable event append uses block_on).
            if let Some(space_id) = window.space_id.clone() {
                let aid = actor_id.clone();
                tokio::task::spawn_blocking(move || {
                    ship_substrate::db::actor_events::deregister_window(&aid, &space_id)
                })
                .await
                .map_err(|e| CommandError::service(format!("deregister join: {e}")))?
                .map_err(|e| CommandError::service(e.to_string()))?;
            }

            Ok(Some(serde_json::json!({ "closed": true, "token_revoked": token_revoked })))
        }
        _ => Ok(None),
    }
}
