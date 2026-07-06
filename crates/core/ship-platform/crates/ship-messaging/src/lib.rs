//! Outbound human-gateway service — adapter-only for now.
//!
//! When a `pipeline.run.blocked` event fires with `needs_human: true`, the
//! gateway forwards the block reason to a human via the configured adapter
//! (Telegram today; Slack/WhatsApp etc. plug in as new
//! [`NotificationAdapter`] implementations). The outbound notifier is the
//! only half wired today.
//!
//! Inbound (webhook → kernel re-emit) is a messaging fast-follow. The
//! `human.response` event registration that previously lived here has been
//! removed (theme-A contain A8): no emitter ever published it, and the
//! adapter-only `HumanGatewayService` has no use for an outbox. When the
//! inbound path lands it will own its own registration, mirroring this
//! crate's outbound shape.
//!
//! Lives at L1 (`ship-messaging`) because the adapters make outbound HTTP
//! calls — egress is forbidden in the L0 substrate. The generic
//! [`ServiceHandler`] plumbing it implements stays in `ship-substrate`.
//!
//! # Topology exception
//!
//! `ship-messaging` is an **intended-extension-point** crate (documented
//! operator exception to Rule C2 in `.artifacts/topology-proposal.md`).
//! It currently has one intra-workspace consumer (`shipd`) but is deliberately
//! kept as a standalone crate because new consumers are expected:
//! - `ship-triggers` (event-triggered notifications)
//! - future Slack adapter
//! - future WhatsApp adapter
//!
//! Re-reviewed at the next topology audit; the exception lapses if no second
//! consumer has arrived by then.

pub mod event_registrations;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use ship_substrate::events::pipeline::event_types::PIPELINE_RUN_BLOCKED;
use ship_substrate::events::pipeline::PipelineRunBlockedPayload;
use ship_substrate::events::{EventEnvelope, SqliteEventStore};
use ship_substrate::services::ServiceHandler;

// ── Adapter trait ─────────────────────────────────────────────────────────────

/// Outbound message sent to a human via a notification adapter.
pub struct OutboundMessage {
    pub text: String,
    pub pipeline_run_id: Option<String>,
    pub metadata: serde_json::Value,
}

/// Pluggable notification adapter. Implement this to add Slack, WhatsApp, etc.
#[async_trait]
pub trait NotificationAdapter: Send + Sync {
    /// Send a message to the human. Returns a harness message ID on success.
    async fn send(&self, msg: &OutboundMessage) -> Result<String>;
    /// Stable adapter name used in log output and config.
    fn name(&self) -> &'static str;
}

// ── TelegramAdapter ───────────────────────────────────────────────────────────

/// Telegram Bot API adapter.
///
/// Configuration via environment variables:
/// - `SHIP_TELEGRAM_TOKEN` — bot token from BotFather
/// - `SHIP_TELEGRAM_CHAT_ID` — target chat ID (user or group)
pub struct TelegramAdapter {
    token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramAdapter {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id, client: reqwest::Client::new() }
    }

    /// Build from environment variables. Returns `None` if either var is absent.
    pub fn from_env() -> Option<Self> {
        let token = ship_substrate::env::read(ship_substrate::env::Var::TelegramToken)?;
        let chat_id = ship_substrate::env::read(ship_substrate::env::Var::TelegramChatId)?;
        Some(Self::new(token, chat_id))
    }
}

#[async_trait]
impl NotificationAdapter for TelegramAdapter {
    fn name(&self) -> &'static str {
        "telegram"
    }

    async fn send(&self, msg: &OutboundMessage) -> Result<String> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.token
        );
        let body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": msg.text,
        });
        let resp: serde_json::Value = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if resp["ok"].as_bool() != Some(true) {
            anyhow::bail!(
                "telegram sendMessage failed: {}",
                resp["description"].as_str().unwrap_or("unknown error")
            );
        }
        let msg_id = resp["result"]["message_id"]
            .as_i64()
            .map(|id| id.to_string())
            .unwrap_or_default();
        Ok(msg_id)
    }
}

// ── HumanGatewayService ───────────────────────────────────────────────────────

/// Headless service that listens for `pipeline.run.blocked` events and notifies a human.
pub struct HumanGatewayService {
    adapter: Arc<dyn NotificationAdapter>,
}

impl HumanGatewayService {
    pub fn new(adapter: Arc<dyn NotificationAdapter>) -> Self {
        Self { adapter }
    }

    fn handle_job_blocked(&self, event: &EventEnvelope) {
        let payload: PipelineRunBlockedPayload = match serde_json::from_str(&event.payload_json) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("human-gateway: failed to parse pipeline.run.blocked payload: {e}");
                return;
            }
        };

        if !payload.needs_human {
            return;
        }

        let pipeline_run_id = Some(payload.pipeline_run_id.clone());
        // Render without a slug: the canonical `PipelineRunBlockedPayload`
        // carries no `slug` field (it belongs on the run-dispatched event,
        // not the blocked one), and the local shadow that used to mint a
        // phantom "unknown" slug was deleted (theme-A contain A8).
        let text = format!(
            "\u{1f6a7} Blocked: {} ({})",
            payload.blocker,
            payload.pipeline_run_id,
        );

        let msg = OutboundMessage {
            text,
            pipeline_run_id,
            metadata: serde_json::json!({}),
        };

        let adapter = self.adapter.clone();
        tokio::spawn(async move {
            match adapter.send(&msg).await {
                Ok(id) => tracing::info!("human-gateway: message sent, id={id}"),
                Err(e) => tracing::error!("human-gateway: adapter send failed: {e}"),
            }
        });
    }
}

impl ServiceHandler for HumanGatewayService {
    fn name(&self) -> &str {
        "human-gateway"
    }

    fn handle(&mut self, event: &EventEnvelope, _store: &SqliteEventStore) -> Result<()> {
        if event.event_type == PIPELINE_RUN_BLOCKED {
            self.handle_job_blocked(event);
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pipeline_run_blocked_payload_needs_human_false() {
        let json = r#"{"pipeline_run_id":"j1","blocker":"waiting","needs_human":false}"#;
        let p: PipelineRunBlockedPayload = serde_json::from_str(json).unwrap();
        assert!(!p.needs_human);
        assert_eq!(p.pipeline_run_id, "j1");
    }

    #[test]
    fn pipeline_run_blocked_payload_needs_human_true() {
        let json = r#"{"pipeline_run_id":"j2","blocker":"needs approval","needs_human":true}"#;
        let p: PipelineRunBlockedPayload = serde_json::from_str(json).unwrap();
        assert!(p.needs_human);
        assert_eq!(p.blocker, "needs approval");
    }

    #[test]
    fn render_omits_phantom_slug() {
        // Regression: the local-mirror payload had a `slug` field the
        // canonical one does not. Rendering it produced a literal
        // "unknown" line on every block, which is user-facing noise.
        // The canonical payload has no slug; the render must not pretend.
        let payload = PipelineRunBlockedPayload {
            pipeline_run_id: "abc".to_string(),
            blocker: "manual review required".to_string(),
            needs_human: true,
        };
        let text = format!(
            "\u{1f6a7} Blocked: {} ({})",
            payload.blocker, payload.pipeline_run_id,
        );
        assert!(text.contains("manual review required"));
        assert!(text.contains("abc"));
        assert!(!text.contains("unknown"));
        assert!(!text.contains("Job:"), "phantom Job: line must be gone");
    }

    #[test]
    fn payload_alias_job_id() {
        // Legacy producers emit `job_id`; the canonical struct accepts it
        // via `#[serde(alias = "job_id")]` so deserialization must not
        // choke on historical envelopes.
        let legacy = json!({"job_id": "j-1", "blocker": "x", "needs_human": true});
        let p: PipelineRunBlockedPayload = serde_json::from_value(legacy).unwrap();
        assert_eq!(p.pipeline_run_id, "j-1");
    }

    #[test]
    fn test_telegram_adapter_from_env_missing() {
        // Without env vars set, from_env returns None.
        // SAFETY: test-only, single-threaded context
        unsafe {
            std::env::remove_var("SHIP_TELEGRAM_TOKEN");
            std::env::remove_var("SHIP_TELEGRAM_CHAT_ID");
        }
        assert!(TelegramAdapter::from_env().is_none());
    }
}
