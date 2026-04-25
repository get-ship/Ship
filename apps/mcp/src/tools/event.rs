//! Transport for the `emit_event` MCP tool.
//!
//! The agent calls `event(event_type, payload, elevated)`. The handler injects
//! `actor_id` (MCP connection context) and `workspace_id` (current branch),
//! then POSTs `/api/events/emit` with `validate_domain=true`. The daemon
//! enforces the reserved-namespace and dotted-event-type rules and routes the
//! envelope through its kernel — so projection updates, audit, and subscriber
//! delivery happen on the same single rail every other event uses.

use crate::network_client;

/// Forward an agent-emitted domain event to the daemon. Returns the daemon's
/// JSON response on success or a human-readable error string on failure.
///
/// `actor_id` and `workspace_id` are injected by the caller from MCP
/// connection context — they cannot be supplied by the agent payload.
pub async fn emit_domain_event(
    actor_id: &str,
    workspace_id: &str,
    event_type: &str,
    payload: serde_json::Value,
    elevated: bool,
) -> Result<String, String> {
    let body = serde_json::json!({
        "event_type": event_type,
        "entity_id": workspace_id,
        "workspace_id": workspace_id,
        "actor_id": actor_id,
        "payload": payload,
        "elevated": elevated,
        "validate_domain": true,
    });
    network_client::emit_event_via_daemon(body)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod event_tests;
