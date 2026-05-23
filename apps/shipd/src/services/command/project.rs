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
        "project.add" => {
            // Register a directory as a project. Mirrors routes/projects_api: the
            // path must be a dir, and we ensure its `.ship/` exists so the daemon
            // has somewhere to store project-scoped config.
            let path = req_str(args, "path")?;
            let path = std::path::PathBuf::from(path.trim());
            if !path.is_dir() {
                return Err(CommandError::bad_args(format!(
                    "not a directory: {}",
                    path.display()
                )));
            }
            let ship_dir = ship_substrate::ShipHome::project_ship_dir(&path);
            if !ship_dir.exists() {
                std::fs::create_dir_all(&ship_dir)
                    .map_err(|e| CommandError::service(e.to_string()))?;
            }
            let name = args.get("name").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
            let record = ship_platform::projects::register(&path, name)
                .map_err(|e| CommandError::service(e.to_string()))?;
            // Flat ProjectRecord shape — decodes into the typed message.
            Ok(Some(serde_json::json!({
                "id": record.id,
                "path": record.path,
                "name": record.name,
                "created_at": record.created_at,
                "last_seen_at": record.last_seen_at,
            })))
        }
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
