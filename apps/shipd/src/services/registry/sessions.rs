//! Registry dispatcher for `sessions.*` ops. Split off from
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
        "sessions.list" => {
            Ok(json!({
                "items": state.pty_sessions().list(),
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
