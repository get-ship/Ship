//! Project commands. Delegates straight to the platform service — the clean
//! shape: no event-building in the command layer, just a typed service call.

use serde_json::Value;

use super::{req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "project.activate" => {
            let id = req_str(args, "id")?;
            let record = ship_platform::projects::switch_to(ctx.state.kernel(), &id)
                .await
                .map_err(|e| {
                    let msg = e.to_string();
                    if msg.contains("not known") {
                        CommandError::bad_args(msg)
                    } else {
                        CommandError::service(msg)
                    }
                })?;
            Ok(Some(serde_json::json!({
                "ok": true,
                "active": { "id": record.id, "name": record.name, "path": record.path },
            })))
        }
        _ => Ok(None),
    }
}
