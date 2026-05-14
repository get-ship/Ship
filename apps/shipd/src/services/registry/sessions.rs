//! Registry dispatcher for `sessions.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::RegistryError;
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;


pub(super) fn try_dispatch(
    name: &str,
    _args: &Value,
    state: &ApiState,
    _files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "sessions.list" => {
            Ok(json!({
                "items": state.pty_sessions().list(),
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
