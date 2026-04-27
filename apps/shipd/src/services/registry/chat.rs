//! Registry dispatcher for `chat.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::rest_api::ApiState;
use crate::services::files::{FilesService, Scope};

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
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
