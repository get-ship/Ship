//! Persistent connection to a provider's server-mode endpoint.
//!
//! Codex exposes a JSON-RPC app-server over stdio; OpenCode exposes an
//! HTTP+SSE server. Both are designed for long-lived orchestrator
//! connections rather than one-shot CLI spawns. `ServerSession`
//! abstracts over their differences so `ServerExecutor` can delegate
//! turns + cancellation without knowing the wire protocol.
//!
//! Per-provider implementations land in follow-on slices
//! (`provider-codex-02`, `provider-opencode-02`). This module provides
//! the trait, the event vocabulary, and a `StubSession` used for tests
//! and as the placeholder until real sessions are bound.
//!
//! Events flow from provider -> `SessionEvent` -> (executor) -> ship's
//! `process.*` / `executor.*` envelopes. Keeping the vocabulary
//! provider-agnostic means the executor's event-mapping lives in one
//! place instead of every adapter.

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

/// Stream event from a provider's server-mode connection.
///
/// Kept provider-agnostic so the executor's mapping layer (to ship's
/// `process.*` / `executor.*` envelopes) can be a single match. Codex
/// emits `ToolCall*`, `AgentMessage*`, `ExecCommand*`, etc. over JSON-RPC;
/// OpenCode emits analogous SSE events. Both funnel through this enum.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    TurnStarted { turn_id: String },
    AgentMessage { text: String },
    ToolUse { tool: String, input_preview: String },
    CommandStarted { cmd: String },
    CommandOutput { stream: String, line: String },
    TurnCompleted { stop_reason: String },
    Error { message: String },
}

/// Long-lived session with a provider's server-mode endpoint.
///
/// One `ServerSession` per live provider connection. A turn is the unit
/// of work: submit a prompt, stream events until `TurnCompleted` (or
/// `Error`), optionally cancel. `close` tears down the underlying
/// transport. All methods are idempotent-safe — `cancel` on an unknown
/// or already-completed turn is a no-op, as is `close` on an already-
/// closed session.
#[async_trait]
pub trait ServerSession: Send + Sync {
    /// Provider id (matches `ProviderAdapter::id`).
    fn provider(&self) -> &str;

    /// Submit a turn; returns the turn id immediately. Streamed events
    /// arrive via `subscribe`.
    async fn start_turn(&self, prompt: &str) -> Result<String>;

    /// Subscribe to streaming events for the active turn. The receiver
    /// closes once the turn produces `TurnCompleted` or `Error`.
    async fn subscribe(&self, turn_id: &str) -> Result<UnboundedReceiver<SessionEvent>>;

    /// Cancel an in-flight turn. Idempotent.
    async fn cancel(&self, turn_id: &str) -> Result<()>;

    /// Close the session cleanly. Idempotent.
    async fn close(&self) -> Result<()>;
}

/// Stub session used by tests + as the placeholder until per-provider
/// implementations (`provider-codex-02`, `provider-opencode-02`) land.
/// Errors loudly when `start_turn`/`subscribe` are called so a
/// misrouted job doesn't silently appear to succeed.
pub struct StubSession {
    pub provider: String,
}

impl StubSession {
    pub fn new(provider: impl Into<String>) -> Self {
        Self { provider: provider.into() }
    }
}

#[async_trait]
impl ServerSession for StubSession {
    fn provider(&self) -> &str {
        &self.provider
    }

    async fn start_turn(&self, _prompt: &str) -> Result<String> {
        anyhow::bail!(
            "StubSession::start_turn called — no real ServerSession bound for '{}'",
            self.provider
        )
    }

    async fn subscribe(&self, _turn_id: &str) -> Result<UnboundedReceiver<SessionEvent>> {
        anyhow::bail!(
            "StubSession::subscribe called — no real ServerSession bound for '{}'",
            self.provider
        )
    }

    async fn cancel(&self, _turn_id: &str) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_session_start_turn_errors_with_provider_name() {
        let s = StubSession::new("codex");
        let err = s.start_turn("hi").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("StubSession"), "msg was: {msg}");
        assert!(msg.contains("codex"), "msg was: {msg}");
    }

    #[tokio::test]
    async fn stub_session_subscribe_errors_with_provider_name() {
        let s = StubSession::new("opencode");
        let err = s.subscribe("t-1").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("StubSession"), "msg was: {msg}");
        assert!(msg.contains("opencode"), "msg was: {msg}");
    }

    #[tokio::test]
    async fn stub_session_cancel_and_close_are_ok() {
        let s = StubSession::new("codex");
        assert!(s.cancel("t-1").await.is_ok());
        assert!(s.close().await.is_ok());
    }

    #[tokio::test]
    async fn stub_session_provider_reflects_construction() {
        let s = StubSession::new("opencode");
        assert_eq!(s.provider(), "opencode");
    }
}
