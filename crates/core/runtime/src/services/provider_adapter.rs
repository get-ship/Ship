//! Provider adapter trait — single interface between executors and provider CLIs.
//!
//! `ProviderAdapter` is the contract between job executors and the underlying
//! provider CLI. It replaces the defunct `provider_launchers` module.
//!
//! To add a new provider, implement `ProviderAdapter` in `providers/` and add
//! a match arm to `adapter_for`.

/// Behaviour that every provider CLI must implement for executor integration.
pub trait ProviderAdapter: Send + Sync {
    /// Pattern that appears in the terminal pane when the agent is ready for input.
    fn ready_pattern(&self) -> &str;

    /// Text sent to the agent after it becomes ready (triggers job autostart).
    fn autostart_input(&self) -> &str;

    /// Command sent to the agent to request a graceful exit.
    fn exit_command(&self) -> &str;

    /// Base CLI invocation string (without slug, model, or flag overrides).
    fn cli_command(&self) -> &str;
}

/// Return the adapter for a given provider name. Defaults to Claude.
pub fn adapter_for(provider: Option<&str>) -> Box<dyn ProviderAdapter> {
    match provider {
        Some("codex") => Box::new(super::providers::codex::CodexProviderAdapter),
        _ => Box::new(super::providers::claude::ClaudeProviderAdapter),
    }
}
