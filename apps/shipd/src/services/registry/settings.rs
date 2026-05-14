//! Registry dispatcher for `settings.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::Value;

use super::RegistryError;
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;


pub(super) fn try_dispatch(
    name: &str,
    _args: &Value,
    _state: &ApiState,
    _files: &FilesService,
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
