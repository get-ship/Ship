//! Registry dispatcher for `actors.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

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
        "actors.list" => {
            let filter = ship_substrate::projections::actor::ListActorsFilter {
                workspace_id: args.get("workspace_id").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(String::from),
                kind: args.get("kind").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(String::from),
                status: args.get("status").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(String::from),
                limit: args.get("limit").and_then(|v| v.as_i64()).unwrap_or(0),
            };
            let actors = ship_substrate::projections::actor::list_actors(&filter)
                .map_err(|e| RegistryError::service(format!("actors.list: {e}")))?;
            Ok(json!({ "items": actors }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
