//! Harness-agnostic push adapter layer.
//!
//! The runtime produces `EventEnvelope`s. Push adapters translate them into
//! harness-specific notification formats and deliver them to connected agents.
//!
//! Each harness has its own push mechanism:
//! - Claude Code: `notifications/claude/channel` (channels API)
//! - Cursor, Codex, OpenCode, Gemini: TBD — fall back to generic MCP notification
//!
//! The adapter is selected at connection time based on the detected MCP harness.

pub mod claude_channel;
pub mod mcp_notification;

use async_trait::async_trait;
use rmcp::{Peer, RoleServer};
use ship_substrate::events::EventEnvelope;

/// Push adapter trait — harness-specific event delivery.
///
/// Implementations translate runtime events into the format expected by
/// each harness's push mechanism. The relay calls `push_event` for every
/// event the agent should receive.
#[async_trait]
pub trait PushAdapter: Send + Sync + 'static {
    /// Deliver an event to the connected agent.
    async fn push_event(&self, event: &EventEnvelope);

    /// Human-readable adapter name for logging.
    fn adapter_name(&self) -> &'static str;
}

/// Select the appropriate push adapter based on the detected MCP harness.
///
/// Harness names come from the MCP handshake `clientInfo.name` field.
/// Unknown harnesses get the generic MCP notification fallback.
pub fn select_adapter(harness: Option<&str>, peer: Peer<RoleServer>) -> Box<dyn PushAdapter> {
    match harness {
        Some(name) if is_claude_harness(name) => {
            Box::new(claude_channel::ClaudeChannelAdapter::new(peer))
        }
        _ => Box::new(mcp_notification::McpNotificationAdapter::new(peer)),
    }
}

fn is_claude_harness(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("claude") || lower.contains("claude-code")
}

#[cfg(test)]
#[path = "push_integration_tests.rs"]
mod tests;
