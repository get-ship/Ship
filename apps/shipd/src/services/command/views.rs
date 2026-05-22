//! View-authoring commands — session edits, edit-mode grant/revoke, and the
//! authoring-tool dispatch. Identity-bound (the agent's own SUI session).

use serde_json::Value;

use super::{opt_str, req_str, response_to_value, require_agent, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "view.patch_session" => {
            let agent = require_agent(ctx)?;
            ctx.state
                .view_sessions()
                .patch(&req_str(args, "id")?, &agent, &req_str(args, "selector")?, req_str(args, "html")?)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true })))
        }
        "view.append_session" => {
            let agent = require_agent(ctx)?;
            ctx.state
                .view_sessions()
                .append(&req_str(args, "id")?, &agent, &req_str(args, "selector")?, req_str(args, "html")?)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true })))
        }
        "view.signal_session" => {
            let agent = require_agent(ctx)?;
            let value: Value = match opt_str(args, "value_json") {
                Some(s) => serde_json::from_str(&s)
                    .map_err(|e| CommandError::bad_args(format!("value_json: {e}")))?,
                None => Value::Null,
            };
            ctx.state
                .view_sessions()
                .signal(&req_str(args, "id")?, &agent, &req_str(args, "name")?, value)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true })))
        }
        "view.enter_edit_mode" | "view.exit_edit_mode" => {
            let view = req_str(args, "view")?;
            if !crate::routes::view_edit_mode_api::is_valid_view_name_pub(&view) {
                return Err(CommandError::bad_args("invalid view name"));
            }
            let session_id = req_str(args, "session_id")?;
            let granted = name == "view.enter_edit_mode";
            if granted {
                ship_platform::view_authoring::grant(&session_id, &view);
            } else {
                ship_platform::view_authoring::revoke(&session_id, &view);
            }
            Ok(Some(serde_json::json!({ "view": view, "session_id": session_id, "granted": granted })))
        }
        "view.tool" => {
            let view = req_str(args, "view")?;
            let body_args: Value = match opt_str(args, "args_json") {
                Some(s) => serde_json::from_str(&s)
                    .map_err(|e| CommandError::bad_args(format!("args_json: {e}")))?,
                None => Value::Null,
            };
            let body = crate::routes::view_edit_tool_api::ToolBody {
                session_id: req_str(args, "session_id")?,
                op: req_str(args, "op")?,
                args: body_args,
            };
            let resp =
                crate::routes::view_edit_tool_api::dispatch_tool_core(ctx.state, view, body).await;
            Ok(Some(response_to_value(resp).await?))
        }
        _ => Ok(None),
    }
}
