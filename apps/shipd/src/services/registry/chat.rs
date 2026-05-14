//! Registry dispatcher for `chat.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::RegistryError;
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;


pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    _files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "chat.state" => {
            let ship_dir = state.ship_dir();
            let ws_arg = args.get("workspace_id").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let messages = crate::views::chat_state::list_messages(
                ship_dir, ws_arg, crate::views::chat_state::DEFAULT_LIMIT,
            );
            Ok(json!({
                "workspace_id": ws_arg.unwrap_or(""),
                "messages": messages.clone(),
                "total": messages.len(),
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
