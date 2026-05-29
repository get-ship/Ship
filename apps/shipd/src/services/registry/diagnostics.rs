//! Registry dispatcher for `diagnostics.*` ops.

use serde_json::Value;

use super::RegistryError;
use crate::ship_runtime::ApiState;

pub fn try_dispatch(
    name: &str,
    _args: &Value,
    state: &ApiState,
    _files: &ship_platform::services::files::FilesService,
) -> Result<Option<Value>, RegistryError> {
    match name {
        "diagnostics.snapshot" => {
            let reg = state
                .diag_registry()
                .ok_or_else(|| RegistryError::service("diagnostic registry not yet initialized"))?;
            Ok(Some(crate::diag_rpc::handle_snapshot(reg)))
        }
        _ => Ok(None),
    }
}
