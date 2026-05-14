//! Registry dispatcher for `rules.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, parse_scope, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;


pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    _state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "rules.list" => Ok(json!({
            "items": ship_platform::services::rules::state::list(files),
        })),
        "rules.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, body) = ship_platform::services::rules::state::load_body(files, id, scope)
                .map_err(|e| match e {
                    ship_platform::services::rules::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": ship_platform::services::rules::state::ScopeTag::from(resolved),
                "body": body,
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
