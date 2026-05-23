//! Mesh commands — the emit-with-identity shape: validate caller identity, then
//! build a registered event and emit it through the kernel with the caller's
//! emit context. (The #78 lift can move the envelope assembly into a mesh
//! service; the command layer stays the same shape either way.)

use serde_json::Value;

use super::{opt_str, req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "mesh.send" => {
            let from = req_str(args, "from")?;
            // An agent caller may only send as itself.
            if let Some(agent_id) = ctx.auth.agent_id.as_deref() {
                if agent_id != from {
                    return Err(CommandError::forbidden(format!(
                        "from '{from}' does not match caller '{agent_id}'"
                    )));
                }
            }
            let to = req_str(args, "to")?;
            let body: Value = match opt_str(args, "body_json") {
                Some(s) => serde_json::from_str(&s)
                    .map_err(|e| CommandError::bad_args(format!("body_json: {e}")))?,
                None => Value::Null,
            };
            let envelope = ship_substrate::events::EventEnvelope::new(
                "mesh.send",
                &to,
                &serde_json::json!({ "from": from, "to": to, "body": body }),
            )
            .map(|e| e.with_actor_id(&from))
            .map_err(|e| CommandError::bad_args(e.to_string()))?;
            let emit_ctx = ctx.auth.to_daemon_emit_context();
            ctx.state
                .kernel()
                .lock()
                .await
                .emit(envelope, &emit_ctx)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!("sent")))
        }
        "mesh.broadcast" => {
            let from = req_str(args, "from")?;
            if let Some(agent_id) = ctx.auth.agent_id.as_deref() {
                if agent_id != from {
                    return Err(CommandError::forbidden(format!(
                        "from '{from}' does not match caller '{agent_id}'"
                    )));
                }
            }
            let body: Value = match opt_str(args, "body_json") {
                Some(s) => serde_json::from_str(&s)
                    .map_err(|e| CommandError::bad_args(format!("body_json: {e}")))?,
                None => Value::Null,
            };
            let mut payload = serde_json::json!({ "from": from, "body": body });
            if let Some(cap) = opt_str(args, "capability_filter") {
                payload["capability_filter"] = serde_json::json!(cap);
            }
            let envelope =
                ship_substrate::events::EventEnvelope::new("mesh.broadcast", &from, &payload)
                    .map(|e| e.with_actor_id(&from))
                    .map_err(|e| CommandError::bad_args(e.to_string()))?;
            let emit_ctx = ctx.auth.to_daemon_emit_context();
            ctx.state
                .kernel()
                .lock()
                .await
                .emit(envelope, &emit_ctx)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!("broadcast")))
        }
        _ => Ok(None),
    }
}
