//! Project commands. Delegates to ship-space for add (register) and activate.

use std::sync::Arc;

use serde_json::Value;

use ship_space::registry;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceId};

use super::{req_str, CommandCtx, CommandError};

const RUNTIME_PARTITION: &str = "runtime";

fn runtime_store(
    state: &crate::ship_runtime::ApiState,
) -> Arc<dyn ship_substrate::ports::DataStore> {
    use ship_substrate::data::DataService;
    let runtime_db = ship_substrate::db::runtime_db_path(state.ship_dir());
    let svc = DataService::open_at(&runtime_db).expect("data service over runtime.db");
    Arc::new(SingleBackendDataStore::new(svc, RUNTIME_PARTITION))
}

fn kernel_log(
    state: &crate::ship_runtime::ApiState,
) -> Arc<KernelLog> {
    Arc::new(KernelLog::new(state.kernel().clone()))
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "project.add" => {
            // Register a directory as a project. The path must be a dir;
            // we ensure its `.ship/` exists so the daemon has somewhere to
            // store project-scoped config.
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
            // Canonicalize before handing to ship-space (the crate touches no fs).
            let canonical = std::fs::canonicalize(&path).unwrap_or(path);
            let store = runtime_store(ctx.state);
            let kernel = kernel_log(ctx.state);
            let record = registry::add(&*store, &kernel, &canonical, name)
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
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
            let store = runtime_store(ctx.state);
            let kernel = kernel_log(ctx.state);
            let _ = SpaceId::new(id.to_string()); // assert non-empty
            let record = registry::activate(&*store, &kernel, &id)
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
