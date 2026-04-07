//! Claude CLI provider adapter.

use crate::services::provider_adapter::ProviderAdapter;

pub struct ClaudeProviderAdapter;

impl ProviderAdapter for ClaudeProviderAdapter {
    fn ready_pattern(&self) -> &str {
        "❯"
    }

    fn autostart_input(&self) -> &str {
        "go"
    }

    fn exit_command(&self) -> &str {
        "/exit"
    }

    fn cli_command(&self) -> &str {
        "claude --dangerously-skip-permissions \
         --dangerously-load-development-channels server:ship"
    }
}
