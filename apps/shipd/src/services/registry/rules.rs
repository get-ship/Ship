//! Registry dispatcher for `rules.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, parse_scope, required_str};
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
        "rules.list" => Ok(json!({
            "items": crate::services::rules::state::list(&files),
        })),
        "rules.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, body) = crate::services::rules::state::load_body(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::rules::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::rules::state::ScopeTag::from(resolved),
                "body": body,
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
