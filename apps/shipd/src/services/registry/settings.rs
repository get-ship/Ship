//! Registry dispatcher for `settings.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::Value;

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
        "settings.state" => {
            let ctx = crate::services::settings::view_state::build_ctx();
            serde_json::to_value(&ctx)
                .map_err(|e| RegistryError::service(e.to_string()))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
