//! Registry dispatcher for `mcp.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::{FilesService, Scope};

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "mcp.list" => Ok(json!({
            "items": ship_platform::services::mcp::state::list(&files),
        })),
        "mcp.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = ship_platform::services::mcp::state::load_entry(&files, id, scope)
                .map_err(|e| match e {
                    ship_platform::services::mcp::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": ship_platform::services::mcp::state::ScopeTag::from(resolved),
                "entry": entry,
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
