//! Registry dispatcher for `specs.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::access::RuntimeAccess;
use crate::rest_api::ApiState;
use crate::services::files::{FilesService, Scope};

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let svc = state.services().specs();
    let value: Result<Value, RegistryError> = match name {
        "specs.list" => {
            let specs = svc.list().map_err(|e| RegistryError::service(e.to_string()))?;
            Ok(json!({ "items": specs }))
        }
        "specs.get" => {
            let id = required_str(args, "id")?;
            let (spec, body) = svc.get_with_body(id)
                .map_err(|e| match e {
                    crate::services::specs::service::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({ "spec": spec, "body": body }))
        }
        "specs.summaries" => {
            let q = args.get("q").and_then(|v| v.as_str()).unwrap_or("");
            let selected = args
                .get("selected")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let ctx = crate::services::specs::view_state::build_summaries(svc, q, selected);
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "specs.detail" => {
            let id = required_str(args, "id")?;
            let detail = crate::services::specs::view_state::build_detail(svc, id)
                .ok_or_else(|| RegistryError::not_found(format!("spec `{id}`")))?;
            serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
