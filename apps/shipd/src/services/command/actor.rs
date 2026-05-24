//! Actor control commands — kernel-level operations (operational/internal: an
//! operator/debug surface, not agent-facing capability — see #95). The kernel
//! logic lives in `services::actor_control`; this layer does identity binding +
//! arg parsing only. C1 (behaviour-preserving).

use serde_json::Value;

use ship_substrate::events::CallerKind;

use crate::services::actor_control::{deliver_actor_input, ActorInputError};

use super::{req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "actor.input" => {
            let id = req_str(args, "actor_id")?;
            // An MCP agent caller may only address its own actor (parity with the
            // REST handler's enforce_mcp_identity).
            if ctx.auth.caller_kind == CallerKind::Mcp
                && let Some(bound) = ctx.auth.agent_id.as_deref()
                && bound != id
            {
                return Err(CommandError::forbidden(format!(
                    "caller '{bound}' may not input to actor '{id}'"
                )));
            }
            let text = req_str(args, "text")?;
            deliver_actor_input(ctx.state, ctx.auth, &id, &text)
                .await
                .map_err(|e| match e {
                    ActorInputError::Empty => CommandError::bad_args("empty text"),
                    ActorInputError::NotOnMesh => {
                        CommandError::not_found(format!("actor '{id}' is not on the mesh"))
                    }
                    ActorInputError::Service(m) => CommandError::service(m),
                })?;
            Ok(Some(serde_json::json!({ "ok": true })))
        }
        _ => Ok(None),
    }
}
