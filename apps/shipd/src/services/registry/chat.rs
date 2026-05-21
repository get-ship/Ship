//! Single-source `chat.*` ops — descriptors + dispatch via `ship_ops!`.

use super::wire::ChatStateResponse;
use crate::services::schema::ArgDescriptor;

ship_ops! {
    CHAT_OPS;
    {
        name: "chat.state",
        desc: "Recent messages for the chat drawer, scoped by workspace.",
        perm: "ship.chat.read",
        args: &[ArgDescriptor { name: "workspace_id", kind: "string?", required: false }],
        ret: ChatStateResponse,
        sensitive: true,
        run: |args, state, _files| {
            let ws_arg = args
                .get("workspace_id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let messages = crate::views::chat_state::list_messages(
                state.ship_dir(),
                ws_arg,
                crate::views::chat_state::DEFAULT_LIMIT,
            );
            let total = messages.len();
            Ok(ChatStateResponse {
                workspace_id: ws_arg.unwrap_or("").to_string(),
                messages,
                total,
            })
        },
    },
}
