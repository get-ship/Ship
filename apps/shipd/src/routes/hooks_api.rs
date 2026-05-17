//! Unified hook endpoints for the shipd daemon.
//!
//! Four endpoints receive normalized `ShipHookEvent` payloads from either
//! Claude HTTP hooks (direct) or the `ship hook` CLI adapter (which
//! normalizes harness-specific stdin before POSTing here).
//!
//! Phase 1: the hook pipeline always returns "allow". Security scanning
//! and custom rules are Phase 2.

use axum::{extract::State, Extension, Json, http::StatusCode, response::IntoResponse};
use ship_substrate::events::EventEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;

/// Normalized hook event sent by Claude HTTP hooks or the `ship hook` adapter.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "tool_name": "Bash",
    "tool_input": { "command": "ls" },
    "actor_id": "agent-writer",
    "workspace": "default",
    "session_id": "sess-2f9c"
}))]
pub struct ShipHookEvent {
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_response: Option<serde_json::Value>,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Hook response: allow or block with reason.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "action": "allow" }))]
pub struct HookResponse {
    #[schema(example = "allow")]
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HookResponse {
    fn allow() -> Self {
        Self { action: "allow".to_string(), reason: None }
    }
}

/// Resolve actor_id from the event body or auth context.
fn resolve_actor(event: &ShipHookEvent, auth: &AuthContext) -> String {
    event
        .actor_id
        .clone()
        .or_else(|| auth.agent_id.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Resolve session_id from the event body or auth context.
fn resolve_session(event: &ShipHookEvent, auth: &AuthContext) -> Option<String> {
    event.session_id.clone().or_else(|| auth.session_id.clone())
}

/// POST /api/v1/hooks/session-start
#[utoipa::path(
    operation_id = "hookSessionStart",
    post,
    path = "/api/v1/hooks/session-start",
    request_body = ShipHookEvent,
    responses(
        (status = 200, description = "Hook processed", body = HookResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "hooks"
)]
pub async fn hook_session_start(
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> impl IntoResponse {
    let actor = resolve_actor(&event, &auth);
    let session = resolve_session(&event, &auth);

    tracing::debug!(actor = %actor, session = ?session, "hook: session-start");

    // Update heartbeat if we have a session.
    if let Some(ref sid) = session {
        let _ = ship_substrate::db::session_drain::update_heartbeat(sid);
    }

    (StatusCode::OK, Json(HookResponse::allow()))
}

/// POST /api/v1/hooks/before-tool
#[utoipa::path(
    operation_id = "hookBeforeTool",
    post,
    path = "/api/v1/hooks/before-tool",
    request_body = ShipHookEvent,
    responses(
        (status = 200, description = "Hook allow or block decision", body = HookResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "hooks"
)]
pub async fn hook_before_tool(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> impl IntoResponse {
    let actor = resolve_actor(&event, &auth);
    let session = resolve_session(&event, &auth);
    let tool = event.tool_name.as_deref().unwrap_or("unknown");

    tracing::debug!(actor = %actor, tool = %tool, session = ?session, "hook: before-tool");

    // Existing session-narrative bookkeeping (heartbeat / tool count)
    // stays — it's the workspace-session vocabulary, separate from the
    // actor event log.
    if let Some(ref sid) = session {
        let _ = ship_substrate::db::session_drain::increment_tool_count(sid);
        let _ = ship_substrate::db::session_drain::update_heartbeat(sid);
        broadcast_tool_use(&state, sid, tool, event.tool_input.as_ref());
    }

    // Emit a kernel event correlated by actor_id. Lands in the actor's
    // event stream (queryable via /api/actors/{id}/events) so chat
    // threads and dashboards can render tool cards regardless of
    // whether the actor is in the legacy PtySession registry. No more
    // broadcast_tool_use silently dropping when the registry doesn't
    // have a matching harness_session_id.
    if actor != "unknown" {
        let payload = json!({
            "tool": tool,
            "input_preview": event.tool_input.as_ref().map(|v| {
                let s = v.to_string();
                if s.len() > 400 { s.chars().take(400).collect::<String>() } else { s }
            }),
        });
        if let Ok(env) = EventEnvelope::new("hook.tool_use_started", &actor, &payload) {
            let env = env.with_target(&actor).with_actor_id(&actor);
            let ctx = auth.to_daemon_emit_context();
            if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                tracing::warn!(actor, "hook.tool_use_started emit failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow()))
}

/// Route a tool-use event into the matching Ship chat session so the
/// command bar renders it as an inline tool card. The hook carries the
/// harness's own session id — we reverse-lookup our Ship session via
/// `harness_session_id` and broadcast a `HarnessEvent::ToolUse` on
/// its existing event channel (the UI already consumes this variant).
fn broadcast_tool_use(
    state: &ApiState,
    harness_session_id: &str,
    tool: &str,
    input: Option<&serde_json::Value>,
) {
    let Some(session) = state
        .pty_sessions()
        .find_by_harness_session(harness_session_id)
    else {
        return;
    };
    let input_preview = input
        .map(|v| {
            let s = v.to_string();
            if s.len() > 400 { s.chars().take(400).collect() } else { s }
        })
        .unwrap_or_default();
    let ev = ship_platform::services::harness_adapter::HarnessEvent::ToolUse {
        tool: tool.to_string(),
        input_preview,
    };
    let _ = session.0.event_tx.send(ev);
}

/// POST /api/v1/hooks/after-tool
#[utoipa::path(
    operation_id = "hookAfterTool",
    post,
    path = "/api/v1/hooks/after-tool",
    request_body = ShipHookEvent,
    responses(
        (status = 200, description = "Hook processed", body = HookResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "hooks"
)]
pub async fn hook_after_tool(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> impl IntoResponse {
    let actor = resolve_actor(&event, &auth);
    let session = resolve_session(&event, &auth);
    let tool = event.tool_name.as_deref().unwrap_or("unknown");

    tracing::debug!(actor = %actor, tool = %tool, session = ?session, "hook: after-tool");

    if let Some(ref sid) = session {
        let _ = ship_substrate::db::session_drain::update_heartbeat(sid);
        // The UI distinguishes start from finish by arrival order, not a
        // dedicated event variant. Emitting a second ToolUse with the
        // response preview lets the card flip from "running" to "done"
        // without a schema change.
        broadcast_tool_use(&state, sid, tool, event.tool_response.as_ref());
    }

    // Actor-correlated tool-completed event — symmetric with the
    // before-tool emission. Threads / dashboards transition the tool
    // card from "running" to "done" by arrival order.
    if actor != "unknown" {
        let payload = json!({
            "tool": tool,
            "response_preview": event.tool_response.as_ref().map(|v| {
                let s = v.to_string();
                if s.len() > 400 { s.chars().take(400).collect::<String>() } else { s }
            }),
        });
        if let Ok(env) = EventEnvelope::new("hook.tool_use_completed", &actor, &payload) {
            let env = env.with_target(&actor).with_actor_id(&actor);
            let ctx = auth.to_daemon_emit_context();
            if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                tracing::warn!(actor, "hook.tool_use_completed emit failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow()))
}

/// POST /api/v1/hooks/session-end
#[utoipa::path(
    operation_id = "hookSessionEnd",
    post,
    path = "/api/v1/hooks/session-end",
    request_body = ShipHookEvent,
    responses(
        (status = 200, description = "Hook processed", body = HookResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "hooks"
)]
pub async fn hook_session_end(
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> impl IntoResponse {
    let actor = resolve_actor(&event, &auth);
    let session = resolve_session(&event, &auth);

    tracing::debug!(actor = %actor, session = ?session, "hook: session-end");

    if let Some(ref sid) = session {
        let _ = ship_substrate::db::session_drain::update_heartbeat(sid);
    }

    (StatusCode::OK, Json(HookResponse::allow()))
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

    #[test]
    fn resolve_actor_prefers_event() {
        let event = ShipHookEvent {
            tool_name: None,
            tool_input: None,
            tool_response: None,
            actor_id: Some("from-event".into()),
            workspace: None,
            session_id: None,
        };
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: Some("from-auth".into()),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
        };
        assert_eq!(resolve_actor(&event, &auth), "from-event");
    }

    #[test]
    fn resolve_actor_falls_back_to_auth() {
        let event = ShipHookEvent {
            tool_name: None,
            tool_input: None,
            tool_response: None,
            actor_id: None,
            workspace: None,
            session_id: None,
        };
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: Some("from-auth".into()),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
        };
        assert_eq!(resolve_actor(&event, &auth), "from-auth");
    }
}
