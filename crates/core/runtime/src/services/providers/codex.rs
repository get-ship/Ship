//! Codex CLI provider adapter.

use crate::services::provider_adapter::ProviderAdapter;

pub struct CodexProviderAdapter;

impl ProviderAdapter for CodexProviderAdapter {
    fn ready_pattern(&self) -> &str {
        "$ "
    }

    fn autostart_input(&self) -> &str {
        "go"
    }

    fn exit_command(&self) -> &str {
        "q"
    }

    fn cli_command(&self) -> &str {
        "codex"
    }
}
