//! Registry dispatcher for `workspaces.*` ops. Surfaces the project
//! registry through the `_sui/data` SSE plane so `views/workspaces/`
//! and `views/sessions/` don't need to one-shot `GET /api/v1/projects`
//! out of band. Active-project switches emit `config.changed`
//! (`shell.project.active`), which already wakes the resolver.

use serde_json::{Value, json};

use super::RegistryError;
use crate::routes::rest_api::ApiState;
use crate::services::files::FilesService;

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "workspaces.list" => {
            let active = active_id();
            let records = ship_platform::projects::list()
                .map_err(|e| RegistryError::service(e.to_string()))?;
            let items: Vec<Value> = records
                .into_iter()
                .map(|r| {
                    let is_active = active.as_deref() == Some(r.id.as_str());
                    json!({
                        "id": r.id,
                        "name": r.name,
                        "path": r.path,
                        "created_at": r.created_at,
                        "last_seen_at": r.last_seen_at,
                        "active": is_active,
                    })
                })
                .collect();
            Ok(json!({
                "items": items,
                "active": active,
            }))
        }
        "workspaces.active" => Ok(json!({ "id": active_id() })),

        _ => return Ok(None),
    };
    Ok(Some(value?))
}

fn active_id() -> Option<String> {
    ship_platform::config::get_setting("shell.project.active")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
}
