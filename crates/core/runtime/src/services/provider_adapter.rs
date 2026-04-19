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

/// Declarative capability flags per provider.
/// Used by executors + pipeline-step routing to branch on what's supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderCapabilities {
    /// Provider supports `-p`-style headless one-shot execution.
    pub headless: bool,
    /// Provider supports interactive TUI mode.
    pub interactive: bool,
    /// Provider has a long-lived server/daemon mode (codex app-server,
    /// opencode serve) suitable for persistent orchestrator connections.
    pub server_mode: bool,
    /// Provider can emit JSON-lines structured output (codex --json,
    /// claude stream-json, cursor stream-json).
    pub json_stream: bool,
    /// Provider supports MCP tool injection at the config level.
    pub mcp: bool,
    /// Provider has a hook mechanism we can wire `ship hook session-end` into.
    pub hooks: bool,
    /// Provider supports resuming a prior session in headless/server mode
    /// (codex exec resume, opencode REST sessions).
    pub session_resume: bool,
}

impl ProviderCapabilities {
    /// Conservative defaults — headless + interactive only.
    pub const MINIMAL: Self = Self {
        headless: true,
        interactive: true,
        server_mode: false,
        json_stream: false,
        mcp: false,
        hooks: false,
        session_resume: false,
    };
}

/// Complete lifecycle adapter for a provider CLI.
pub trait ProviderAdapter: Send + Sync {
    /// Provider identifier (e.g. "claude", "codex").
    fn id(&self) -> &str;

    /// Declarative capability flags — what this provider supports.
    fn caps(&self) -> ProviderCapabilities;

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
/// Returns Err for unknown providers (no silent default) so a `foo-typo`
/// config doesn't silently run as Claude with weird flags.
pub fn adapter_for(provider: &str) -> Result<Box<dyn ProviderAdapter>, String> {
    match provider {
        "claude" | "claude-code" => Ok(Box::new(super::providers::claude::ClaudeAdapter)),
        "codex" => Ok(Box::new(super::providers::codex::CodexAdapter)),
        "cursor" => Ok(Box::new(super::providers::cursor::CursorAdapter)),
        "gemini" => Ok(Box::new(super::providers::gemini::GeminiAdapter)),
        "opencode" => Ok(Box::new(super::providers::opencode::OpenCodeAdapter)),
        unknown => Err(format!(
            "unknown provider '{unknown}'. Known: claude, codex, cursor, gemini, opencode."
        )),
    }
}

/// Convenience: resolve an Option<&str>, falling back to "claude" when None.
/// Used by executors that want a default for unconfigured pipelines.
pub fn adapter_for_or_default(
    provider: Option<&str>,
) -> Result<Box<dyn ProviderAdapter>, String> {
    adapter_for(provider.unwrap_or("claude"))
}

/// Enumerate all known providers and their capabilities.
/// Used by `ship status --providers` and Studio's provider picker.
pub fn capabilities() -> Vec<(&'static str, ProviderCapabilities)> {
    use super::providers::{
        claude::ClaudeAdapter, codex::CodexAdapter, cursor::CursorAdapter,
        gemini::GeminiAdapter, opencode::OpenCodeAdapter,
    };
    vec![
        ("claude", ClaudeAdapter.caps()),
        ("codex", CodexAdapter.caps()),
        ("cursor", CursorAdapter.caps()),
        ("gemini", GeminiAdapter.caps()),
        ("opencode", OpenCodeAdapter.caps()),
    ]
}

#[cfg(test)]
mod adapter_for_tests {
    use super::*;

    #[test]
    fn adapter_for_cursor_returns_cursor_adapter() {
        assert_eq!(adapter_for("cursor").unwrap().id(), "cursor");
    }

    #[test]
    fn adapter_for_gemini_returns_gemini_adapter() {
        assert_eq!(adapter_for("gemini").unwrap().id(), "gemini");
    }

    #[test]
    fn adapter_for_codex_returns_codex_adapter() {
        assert_eq!(adapter_for("codex").unwrap().id(), "codex");
    }

    #[test]
    fn adapter_for_opencode_returns_opencode_adapter() {
        assert_eq!(adapter_for("opencode").unwrap().id(), "opencode");
    }

    #[test]
    fn adapter_for_or_default_none_returns_claude_adapter() {
        assert_eq!(adapter_for_or_default(None).unwrap().id(), "claude");
    }

    #[test]
    fn adapter_for_unknown_returns_err() {
        let result = adapter_for("nonexistent");
        assert!(result.is_err());
        let msg = result.err().unwrap();
        assert!(msg.contains("unknown provider"), "msg was: {msg}");
        assert!(msg.contains("Known: claude, codex"), "msg was: {msg}");
    }

    #[test]
    fn adapter_for_or_default_propagates_unknown_err() {
        let result = adapter_for_or_default(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn capabilities_lists_all_five_providers() {
        let caps = capabilities();
        let ids: Vec<&str> = caps.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec!["claude", "codex", "cursor", "gemini", "opencode"]);
    }

    #[test]
    fn codex_declares_server_mode() {
        // Opencode's server_mode is currently off pending OpenCodeSession
        // rewrite to match the new `{parts:[...]}` / prompt_async API.
        // See `opencode.rs::caps` comment.
        let caps = capabilities();
        let server_capable: Vec<&str> = caps
            .iter()
            .filter(|(_, c)| c.server_mode)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(server_capable, vec!["codex"]);
    }

    #[test]
    fn claude_does_not_declare_server_mode() {
        let claude = adapter_for("claude").unwrap();
        assert!(!claude.caps().server_mode);
    }

    #[test]
    fn minimal_caps_are_conservative() {
        let m = ProviderCapabilities::MINIMAL;
        assert!(m.headless);
        assert!(m.interactive);
        assert!(!m.server_mode);
        assert!(!m.json_stream);
        assert!(!m.mcp);
        assert!(!m.hooks);
        assert!(!m.session_resume);
    }
}
