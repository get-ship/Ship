//! Space commands — thin marshal over the L1 space service.
//!
//! `space.setting.set` and `space.window.close` are relocated into
//! `ship_platform::services::space`; this handler resolves the caller's emit
//! context / space, supplies the injected ports (kernel emit, session
//! registry), and serializes the typed result. The product logic — JSON
//! validation, the event-sourced override write, the revoke+deregister
//! sequence — lives in the service.
//!
//! `space.setting.set` is C1 (a settings write); the space is resolved from the
//! caller's emit context — a request with no resolvable space is rejected (no
//! overlay to attach the override to). `space.window.close` is C3 (the gate
//! enforces operator-only before this runs).

use serde_json::Value;

use ship_platform::services::space::{self, SpaceErrorKind, SpaceServiceError};

use crate::space_ports::{KernelSpaceEvents, RegistryWindowSessions};

use super::{req_str, CommandCtx, CommandError};

/// Map an L1 `SpaceServiceError` onto the daemon's `CommandError` wire shape.
fn to_command_error(e: SpaceServiceError) -> CommandError {
    match e.kind {
        SpaceErrorKind::BadArgs => CommandError::bad_args(e.message),
        SpaceErrorKind::NotFound => CommandError::not_found(e.message),
        SpaceErrorKind::Forbidden => CommandError::forbidden(e.message),
        // The daemon has no dedicated conflict constructor; surface as a service
        // error (500) — `Conflict` only arises on the window `Home` path with no
        // daemon space identity, which `space.setting.set`/`close` never hit.
        SpaceErrorKind::Conflict => CommandError::service(e.message),
        SpaceErrorKind::Service => CommandError::service(e.message),
    }
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "space.setting.set" => {
            let key = req_str(args, "key")?;
            let raw = req_str(args, "value_json")?;

            let emit_ctx = ctx.auth.to_emit_context(ctx.auth.caller_kind.clone());
            // Intended P2 fail-soft: with the `daemon_space_id` emit-path
            // fallback gone, an operator/daemon-token caller (binding has no
            // space) hard-errors instead of silently writing the daemon home
            // space's config — an operator with no space cannot set a space's
            // setting.
            let space_id = emit_ctx.space_key().ok_or_else(|| {
                CommandError::bad_args(
                    "no space resolved for this request — a per-space setting needs a space",
                )
            })?;

            let events = KernelSpaceEvents { kernel: ctx.state.kernel(), emit_ctx };
            let out = space::config::set(&events, &space_id, &key, &raw)
                .await
                .map_err(to_command_error)?;
            Ok(Some(serde_json::json!({ "key": out.key, "value_json": out.value_json })))
        }
        "space.window.close" => {
            // C3 — operator-only (the gate enforced is_operator before here).
            // The service revokes the window's pid-bound token then closes its
            // lifecycle; the registry-backed port performs the revoke.
            let actor_id = req_str(args, "actor_id")?;
            let sessions = RegistryWindowSessions { registry: ctx.state.session_registry() };
            let closed = space::windows::close(&sessions, &actor_id)
                .await
                .map_err(to_command_error)?;
            Ok(Some(serde_json::json!({
                "closed": closed.closed,
                "token_revoked": closed.token_revoked,
            })))
        }
        _ => Ok(None),
    }
}
