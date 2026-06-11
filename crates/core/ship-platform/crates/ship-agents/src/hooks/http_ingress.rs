//! HTTP hook ingress domain — types, helpers, and handler logic for the five
//! daemon hook endpoints (session-start/end, before/after-tool, permission-denied).
//!
//! Moved from `apps/shipd/src/routes/hooks_api.rs` (Batch A routes-cull).
//!
//! The `PtyBroadcastPort` trait abstracts the shipd `SessionRegistry` so the
//! handler logic has no compile-time shipd dependency. The shipd handlers become
//! thin axum shims that delegate here.

use serde::{Deserialize, Serialize};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::hooks::{
    event_types as hook_event_types, HookPermissionDeniedPayload, HookSessionEndedPayload,
    HookSessionStartedPayload, HookToolUseCompletedPayload, HookToolUseStartedPayload, ToolStatus,
};
use ship_substrate::events::validator::EmitContext;
use ship_substrate::ports::KernelLog;

// ── Port trait: pty session broadcast ────────────────────────────────────────

/// Abstracts the shipd `SessionRegistry` for tool-use broadcast.
/// Implementations look up the session by `harness_session_id` and send a
/// `HarnessEvent::ToolUse` on its event channel.
pub trait PtyBroadcastPort: Send + Sync {
    /// Best-effort: silently no-ops if no session matches `harness_session_id`.
    fn broadcast_tool_use(
        &self,
        harness_session_id: &str,
        tool: &str,
        input: Option<&serde_json::Value>,
    );
}

// ── Wire types ───────────────────────────────────────────────────────────────

/// Normalized hook event sent by Claude HTTP hooks or the `ship hook` adapter.
///
/// Aliases let a RAW harness hook payload deserialize directly — Claude is
/// already snake_case; codex/gemini/opencode use camelCase / short variants.
#[derive(Debug, Deserialize)]
pub struct ShipHookEvent {
    #[serde(default, alias = "toolName", alias = "name")]
    pub tool_name: Option<String>,
    #[serde(default, alias = "toolInput", alias = "input")]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default, alias = "toolResponse", alias = "output")]
    pub tool_response: Option<serde_json::Value>,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub tool_status: Option<String>,
    #[serde(default)]
    pub tool_error: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub denied_by: Option<String>,
}

/// Hook response: allow or block with reason.
#[derive(Debug, Serialize)]
pub struct HookResponse {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HookResponse {
    pub fn allow() -> Self {
        Self { action: "allow".to_string(), reason: None }
    }
}

// ── Pure helpers ─────────────────────────────────────────────────────────────

/// Truncate a JSON value to a printable preview ≤400 chars.
pub fn preview(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 400 { s.chars().take(400).collect() } else { s }
}

/// Parse the optional `tool_status` body field into the typed enum.
pub fn parse_tool_status(s: Option<&str>) -> ToolStatus {
    match s {
        Some("ok") => ToolStatus::Ok,
        Some("error") | Some("err") => ToolStatus::Error,
        _ => ToolStatus::Unknown,
    }
}

pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ── Handler logic ─────────────────────────────────────────────────────────────
//
// Each `handle_*` function is the domain logic extracted from the axum handler.
// The thin shipd handler does: auth check → resolve_actor/session → call here.

/// Domain logic for POST /hooks/session-start.
pub async fn handle_session_start(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    event: &ShipHookEvent,
    actor: &str,
    session: Option<&str>,
) {
    let target = session.unwrap_or(actor);
    tracing::info!(target, "hook.session_started: about to emit");
    if target == "unknown" {
        return;
    }
    let payload = HookSessionStartedPayload {
        actor_id: actor.to_string(),
        session_id: session.map(str::to_string),
        harness: event.harness.clone(),
        model: event.model.clone(),
        started_at: now_iso8601(),
    };
    if let Ok(env) = EventEnvelope::new(hook_event_types::HOOK_SESSION_STARTED, target, &payload) {
        let env = env.with_target(target).with_actor_id(target);
        if let Err(e) = kernel.emit(env, &emit_ctx).await {
            tracing::warn!(target, "hook.session_started emit failed: {e}");
        }
    }
}

/// Domain logic for POST /hooks/before-tool.
pub async fn handle_before_tool(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    _pty_broadcast: &dyn PtyBroadcastPort,
    event: &ShipHookEvent,
    actor: &str,
    _session: Option<&str>,
) {
    let tool = event.tool_name.as_deref().unwrap_or("unknown");

    if actor == "unknown" {
        return;
    }
    let payload = HookToolUseStartedPayload {
        actor_id: actor.to_string(),
        tool: tool.to_string(),
        input: event.tool_input.clone(),
        input_preview: event.tool_input.as_ref().map(preview),
        started_at: now_iso8601(),
    };
    if let Ok(env) = EventEnvelope::new(hook_event_types::HOOK_TOOL_USE_STARTED, actor, &payload) {
        let env = env.with_target(actor).with_actor_id(actor);
        if let Err(e) = kernel.emit(env, &emit_ctx).await {
            tracing::warn!(actor, "hook.tool_use_started emit failed: {e}");
        }
    }
}

/// Domain logic for POST /hooks/after-tool.
pub async fn handle_after_tool(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    _pty_broadcast: &dyn PtyBroadcastPort,
    event: &ShipHookEvent,
    actor: &str,
    _session: Option<&str>,
) {
    let tool = event.tool_name.as_deref().unwrap_or("unknown");
    if actor == "unknown" {
        return;
    }
    let payload = HookToolUseCompletedPayload {
        actor_id: actor.to_string(),
        tool: tool.to_string(),
        status: parse_tool_status(event.tool_status.as_deref()),
        error: event.tool_error.clone(),
        response: event.tool_response.clone(),
        response_preview: event.tool_response.as_ref().map(preview),
        ended_at: now_iso8601(),
    };
    if let Ok(env) =
        EventEnvelope::new(hook_event_types::HOOK_TOOL_USE_COMPLETED, actor, &payload)
    {
        let env = env.with_target(actor).with_actor_id(actor);
        if let Err(e) = kernel.emit(env, &emit_ctx).await {
            tracing::warn!(actor, "hook.tool_use_completed emit failed: {e}");
        }
    }
}

/// Domain logic for POST /hooks/session-end.
pub async fn handle_session_end(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    event: &ShipHookEvent,
    actor: &str,
    session: Option<&str>,
) {
    let target = session.unwrap_or(actor);
    if target == "unknown" {
        return;
    }
    let payload = HookSessionEndedPayload {
        actor_id: actor.to_string(),
        session_id: session.map(str::to_string),
        ended_at: now_iso8601(),
        reason: event.reason.clone(),
    };
    if let Ok(env) = EventEnvelope::new(hook_event_types::HOOK_SESSION_ENDED, target, &payload) {
        let env = env.with_target(target).with_actor_id(target);
        if let Err(e) = kernel.emit(env, &emit_ctx).await {
            tracing::warn!(target, "hook.session_ended emit failed: {e}");
        }
    }
}

/// Domain logic for POST /hooks/permission-denied.
pub async fn handle_permission_denied(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    event: &ShipHookEvent,
    actor: &str,
    session: Option<&str>,
) {
    if actor == "unknown" {
        return;
    }
    let tool = event.tool_name.as_deref().unwrap_or("unknown");
    let reason = event
        .reason
        .clone()
        .unwrap_or_else(|| "denied (no reason)".to_string());
    let payload = HookPermissionDeniedPayload {
        actor_id: actor.to_string(),
        session_id: session.map(str::to_string),
        harness: event.harness.clone(),
        tool: tool.to_string(),
        input: event.tool_input.clone(),
        input_preview: event.tool_input.as_ref().map(preview),
        reason,
        denied_by: event.denied_by.clone().unwrap_or_else(|| "ship-gate".to_string()),
        denied_at: now_iso8601(),
    };
    if let Ok(env) =
        EventEnvelope::new(hook_event_types::HOOK_PERMISSION_DENIED, actor, &payload)
    {
        let env = env.with_target(actor).with_actor_id(actor);
        if let Err(e) = kernel.emit(env, &emit_ctx).await {
            tracing::warn!(actor, "hook.permission_denied emit failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_response_allow_serializes() {
        let resp = HookResponse::allow();
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["action"], "allow");
        assert!(json.get("reason").is_none());
    }

    #[test]
    fn hook_event_deserializes_minimal() {
        let json = r#"{}"#;
        let event: ShipHookEvent = serde_json::from_str(json).unwrap();
        assert!(event.tool_name.is_none());
        assert!(event.actor_id.is_none());
    }

    #[test]
    fn hook_event_deserializes_raw_harness_payloads_via_aliases() {
        let camel = r#"{ "toolName": "Bash", "toolInput": {"command": "ls"} }"#;
        let e: ShipHookEvent = serde_json::from_str(camel).unwrap();
        assert_eq!(e.tool_name.as_deref(), Some("Bash"));
        assert_eq!(e.tool_input.as_ref().unwrap()["command"], "ls");

        let short = r#"{ "name": "Edit", "input": {"path": "/x"}, "output": {"ok": true} }"#;
        let e: ShipHookEvent = serde_json::from_str(short).unwrap();
        assert_eq!(e.tool_name.as_deref(), Some("Edit"));
        assert_eq!(e.tool_input.as_ref().unwrap()["path"], "/x");
        assert_eq!(e.tool_response.as_ref().unwrap()["ok"], true);
    }

    #[test]
    fn hook_event_deserializes_full() {
        let json = r#"{
            "tool_name": "Bash",
            "tool_input": {"command": "cargo test"},
            "actor_id": "agent.rust-lane",
            "workspace": "auth-feature",
            "session_id": "sess-123"
        }"#;
        let event: ShipHookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.tool_name.as_deref(), Some("Bash"));
        assert_eq!(event.actor_id.as_deref(), Some("agent.rust-lane"));
        assert_eq!(event.workspace.as_deref(), Some("auth-feature"));
        assert_eq!(event.session_id.as_deref(), Some("sess-123"));
    }
}
