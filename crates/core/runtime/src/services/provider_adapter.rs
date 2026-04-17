//! Provider adapter trait — single interface between executors and provider CLIs.
//!
//! Each provider implements this trait once. Executors use the trait methods
//! without knowing which provider they're running.

use std::path::Path;

use super::dispatch_ports::{LaunchCommand, LaunchProfile};

/// Events extracted from provider process output.
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderEvent {
    ToolUse { tool: String, input_preview: String },
    Message { text: String },
    Usage { input_tokens: u64, output_tokens: u64 },
    SessionStarted { session_id: String },
    TurnComplete { stop_reason: String },
    Custom { kind: String, data: serde_json::Value },
}

/// Complete lifecycle adapter for a provider CLI.
pub trait ProviderAdapter: Send + Sync {
    /// Provider identifier (e.g. "claude", "codex").
    fn id(&self) -> &str;

    /// Build the CLI command for an interactive (terminal) session.
    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand;

    /// Build the CLI command for a headless (non-interactive) session.
    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand;

    /// Pattern that indicates the CLI is ready for input.
    fn ready_pattern(&self) -> &str;

    /// Text to send after the CLI is ready to trigger job autostart.
    fn autostart_input(&self) -> &str;

    /// Command to send for graceful shutdown.
    fn exit_command(&self) -> &str;

    /// Parse a line of stdout from the provider process.
    fn parse_output(&self, _line: &str) -> Option<ProviderEvent> { None }

    /// Extract session metadata from the work directory after launch.
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

/// Resolve a provider name to its adapter.
pub fn adapter_for(provider: Option<&str>) -> Box<dyn ProviderAdapter> {
    match provider {
        Some("codex") => Box::new(super::providers::codex::CodexAdapter),
        Some("cursor") => Box::new(super::providers::cursor::CursorAdapter),
        Some("gemini") => Box::new(super::providers::gemini::GeminiAdapter),
        Some("opencode") => Box::new(super::providers::opencode::OpenCodeAdapter),
        _ => Box::new(super::providers::claude::ClaudeAdapter),
    }
}

#[cfg(test)]
mod adapter_for_tests {
    use super::*;

    #[test]
    fn adapter_for_cursor_returns_cursor_adapter() {
        assert_eq!(adapter_for(Some("cursor")).id(), "cursor");
    }

    #[test]
    fn adapter_for_gemini_returns_gemini_adapter() {
        assert_eq!(adapter_for(Some("gemini")).id(), "gemini");
    }

    #[test]
    fn adapter_for_codex_returns_codex_adapter() {
        assert_eq!(adapter_for(Some("codex")).id(), "codex");
    }

    #[test]
    fn adapter_for_opencode_returns_opencode_adapter() {
        assert_eq!(adapter_for(Some("opencode")).id(), "opencode");
    }

    #[test]
    fn adapter_for_none_returns_claude_adapter() {
        assert_eq!(adapter_for(None).id(), "claude");
    }

    #[test]
    fn adapter_for_unknown_returns_claude_adapter() {
        assert_eq!(adapter_for(Some("unknown")).id(), "claude");
    }
}
