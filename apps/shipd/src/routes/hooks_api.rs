//! Unified hook endpoints for the shipd daemon.
//!
//! Four endpoints receive normalized `ShipHookEvent` payloads from either
//! Claude HTTP hooks (direct) or the `ship hook` CLI adapter (which
//! normalizes harness-specific stdin before POSTing here).
//!
//! Phase 1: the hook pipeline always returns "allow". Security scanning
//! and custom rules are Phase 2.

use axum::{extract::State, Extension, Json, http::StatusCode, response::{IntoResponse, Response}};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::hooks::{
    event_types as hook_event_types, HookPermissionDeniedPayload, HookSessionEndedPayload,
    HookSessionStartedPayload, HookToolUseCompletedPayload, HookToolUseStartedPayload, ToolStatus,
};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;

use crate::auth::{AuthContext, Scope, enforce_mcp_identity, enforce_mcp_workspace, require_scope};
use crate::routes::rest_api::ApiState;

/// Truncate a JSON value to a printable preview ≤400 chars. Legacy
/// consumers of `input_preview` / `response_preview` expect a string.
fn preview(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 400 {
        s.chars().take(400).collect()
    } else {
        s
    }
}

/// Parse the optional `tool_status` body field into the typed enum.
/// Defaults to `Unknown` when absent or unrecognized — harnesses
/// without status reporting still emit valid completion events.
fn parse_tool_status(s: Option<&str>) -> ToolStatus {
    match s {
        Some("ok") => ToolStatus::Ok,
        Some("error") | Some("err") => ToolStatus::Error,
        _ => ToolStatus::Unknown,
    }
}

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

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
    // Aliases let a RAW harness hook payload deserialize directly — so a
    // sandboxed agent can `curl` its harness's native payload at the daemon
    // with no `ship hook` CLI in the container to normalize it first. Claude
    // is already snake_case; codex/gemini/opencode use the camelCase / short
    // variants. Mirrors the field-aliasing the CLI's normalize_stdin did.
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
    /// Harness identifier (claude/codex/cursor/gemini/opencode) the
    /// hook originates from. `ship hook` CLI populates this via
    /// `detect_harness()`; direct HTTP callers can pass it explicitly.
    #[serde(default)]
    pub harness: Option<String>,
    /// Model name the harness reports for this session. Populated by
    /// adapters that surface model info on session-start.
    #[serde(default)]
    pub model: Option<String>,
    /// Tool-completion status. Convention: "ok" | "error" | "unknown".
    /// Defaults to "unknown" when the harness doesn't expose it on
    /// PostToolUse.
    #[serde(default)]
    pub tool_status: Option<String>,
    /// Error message when tool_status = "error".
    #[serde(default)]
    pub tool_error: Option<String>,
    /// Reason a session ended ("operator-cancelled", "stop-hook",
    /// "process-exit", etc.). Carried on session-end payloads.
    /// Also carries the deny reason on permission-denied payloads.
    #[serde(default)]
    pub reason: Option<String>,
    /// Origin of a permission denial: "ship-gate" (today, always),
    /// "operator" (future approval UI), "harness" (provider-side
    /// permission check). Only meaningful on /hooks/permission-denied.
    #[serde(default)]
    pub denied_by: Option<String>,
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

/// Resolve actor_id from the event body or auth context. For MCP callers
/// the body's `actor_id` is bound to `auth.agent_id` — workers can't
/// claim another agent's identity. Daemon-token callers keep their
/// trusted preference for the body value.
///
/// Also enforces cross-project scope: a workspace-bound scoped token
/// (e.g. a sandboxed agent's `ship_sess_*`) cannot post a hook tagged
/// for a different project — the same `enforce_mcp_workspace` guard
/// dispatch + MCP use. Latent until sandbox tokens carry a workspace id
/// (mints with `None` today → passes), but closes the forge here so it's
/// covered the moment workspace-binding lands. Bites only on a definite
/// cross-project mismatch.
#[allow(clippy::result_large_err)] // axum Response is the contract here
fn resolve_actor(event: &ShipHookEvent, auth: &AuthContext) -> Result<String, Response> {
    enforce_mcp_identity(auth, event.actor_id.as_deref())?;
    enforce_mcp_workspace(auth, event.workspace.as_deref())?;
    Ok(event
        .actor_id
        .clone()
        .or_else(|| auth.agent_id.clone())
        .unwrap_or_else(|| "unknown".to_string()))
}

/// Resolve session_id from the event body or auth context. session_id is
/// not a privilege claim (see `auth_middleware` — `x-ship-session-id` is
/// already freely settable), so no binding is enforced here.
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
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e.into_response();
    }
    let actor = match resolve_actor(&event, &auth) {
        Ok(a) => a,
        Err(e) => return e,
    };
    let session = resolve_session(&event, &auth);

    tracing::debug!(actor = %actor, session = ?session, "hook: session-start");

    if let Some(ref sid) = session {
        let _ = ship_substrate::db::session_drain::update_heartbeat(sid);
    }

    // Emit a kernel event correlated by the session id (the dispatched
    // slug for orchestrator sessions; the harness's own session_id for
    // legacy spawns). Mirrors the hook.tool_use_* pattern. Target +
    // entity_id + actor_id all carry the session id so observers can
    // filter to a specific dispatch.
    let target = session.as_deref().unwrap_or(actor.as_str());
    tracing::info!(target, "hook.session_started: about to emit");
    if target != "unknown" {
        let payload = HookSessionStartedPayload {
            actor_id: actor.clone(),
            session_id: session.clone(),
            harness: event.harness.clone(),
            model: event.model.clone(),
            started_at: now_iso8601(),
        };
        match EventEnvelope::new(hook_event_types::HOOK_SESSION_STARTED, target, &payload) {
            Ok(env) => {
                let env = env.with_target(target).with_actor_id(target);
                let ctx = auth.to_daemon_emit_context();
                if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                    tracing::warn!(target, "hook.session_started emit failed: {e}");
                }
            }
            Err(e) => {
                tracing::warn!(target, "hook.session_started envelope build failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow())).into_response()
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
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e.into_response();
    }
    let actor = match resolve_actor(&event, &auth) {
        Ok(a) => a,
        Err(e) => return e,
    };
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
        let payload = HookToolUseStartedPayload {
            actor_id: actor.clone(),
            tool: tool.to_string(),
            input: event.tool_input.clone(),
            input_preview: event.tool_input.as_ref().map(preview),
            started_at: now_iso8601(),
        };
        if let Ok(env) = EventEnvelope::new(hook_event_types::HOOK_TOOL_USE_STARTED, &actor, &payload) {
            let env = env.with_target(&actor).with_actor_id(&actor);
            let ctx = auth.to_daemon_emit_context();
            if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                tracing::warn!(actor, "hook.tool_use_started emit failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow())).into_response()
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
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e.into_response();
    }
    let actor = match resolve_actor(&event, &auth) {
        Ok(a) => a,
        Err(e) => return e,
    };
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
        let payload = HookToolUseCompletedPayload {
            actor_id: actor.clone(),
            tool: tool.to_string(),
            status: parse_tool_status(event.tool_status.as_deref()),
            error: event.tool_error.clone(),
            response: event.tool_response.clone(),
            response_preview: event.tool_response.as_ref().map(preview),
            ended_at: now_iso8601(),
        };
        if let Ok(env) = EventEnvelope::new(hook_event_types::HOOK_TOOL_USE_COMPLETED, &actor, &payload) {
            let env = env.with_target(&actor).with_actor_id(&actor);
            let ctx = auth.to_daemon_emit_context();
            if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                tracing::warn!(actor, "hook.tool_use_completed emit failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow())).into_response()
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
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e.into_response();
    }
    let actor = match resolve_actor(&event, &auth) {
        Ok(a) => a,
        Err(e) => return e,
    };
    let session = resolve_session(&event, &auth);

    tracing::debug!(actor = %actor, session = ?session, "hook: session-end");

    if let Some(ref sid) = session {
        let _ = ship_substrate::db::session_drain::update_heartbeat(sid);
    }

    // Emit a kernel event correlated by the session id. NOTE: today
    // the claude compiler wires the Stop hook → `ship hook session-end`,
    // which fires on every turn end (Claude's Stop semantics), not on
    // actual session boundary. Until the Stop → turn-end split lands
    // separately, observers should treat this as "Stop hook fired" not
    // "session truly ended."
    let target = session.as_deref().unwrap_or(actor.as_str());
    if target != "unknown" {
        let payload = HookSessionEndedPayload {
            actor_id: actor.clone(),
            session_id: session.clone(),
            ended_at: now_iso8601(),
            reason: event.reason.clone(),
        };
        if let Ok(env) = EventEnvelope::new(hook_event_types::HOOK_SESSION_ENDED, target, &payload) {
            let env = env.with_target(target).with_actor_id(target);
            let ctx = auth.to_daemon_emit_context();
            if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                tracing::warn!(target, "hook.session_ended emit failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow())).into_response()
}

/// POST /api/v1/hooks/permission-denied
///
/// Emitted by `ship hook before-tool` when its fail-closed integrity
/// gate (cli-framework::hooks::tool_gate) denies a call before the
/// daemon roundtrip. The CLI still returns the harness's expected
/// deny shape on stdout so the tool never runs; this endpoint just
/// logs the denial as a kernel event so analytics + dashboards can
/// show "denied" alongside "succeeded" / "errored".
#[utoipa::path(
    operation_id = "hookPermissionDenied",
    post,
    path = "/api/v1/hooks/permission-denied",
    request_body = ShipHookEvent,
    responses(
        (status = 200, description = "Hook processed", body = HookResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "hooks"
)]
pub async fn hook_permission_denied(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(event): Json<ShipHookEvent>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e.into_response();
    }
    let actor = match resolve_actor(&event, &auth) {
        Ok(a) => a,
        Err(e) => return e,
    };
    let session = resolve_session(&event, &auth);
    let tool = event.tool_name.as_deref().unwrap_or("unknown");
    let reason = event
        .reason
        .clone()
        .unwrap_or_else(|| "denied (no reason)".to_string());

    tracing::debug!(actor = %actor, tool = %tool, "hook: permission-denied");

    if actor != "unknown" {
        let payload = HookPermissionDeniedPayload {
            actor_id: actor.clone(),
            session_id: session.clone(),
            harness: event.harness.clone(),
            tool: tool.to_string(),
            input: event.tool_input.clone(),
            input_preview: event.tool_input.as_ref().map(preview),
            reason,
            denied_by: event.denied_by.clone().unwrap_or_else(|| "ship-gate".to_string()),
            denied_at: now_iso8601(),
        };
        if let Ok(env) =
            EventEnvelope::new(hook_event_types::HOOK_PERMISSION_DENIED, &actor, &payload)
        {
            let env = env.with_target(&actor).with_actor_id(&actor);
            let ctx = auth.to_daemon_emit_context();
            if let Err(e) = state.kernel().lock().await.emit(env, &ctx).await {
                tracing::warn!(actor, "hook.permission_denied emit failed: {e}");
            }
        }
    }

    (StatusCode::OK, Json(HookResponse::allow())).into_response()
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
        // A sandboxed agent curls its harness-native payload straight at the
        // daemon (no `ship hook` CLI to normalize first). Claude is already
        // snake_case; the camelCase / short variants from codex/gemini/
        // opencode must deserialize via the field aliases.
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

    fn make_event(actor_id: Option<&str>) -> ShipHookEvent {
        ShipHookEvent {
            tool_name: None,
            tool_input: None,
            tool_response: None,
            actor_id: actor_id.map(str::to_string),
            workspace: None,
            session_id: None,
            harness: None,
            model: None,
            tool_status: None,
            tool_error: None,
            reason: None,
            denied_by: None,
        }
    }

    fn cli_auth(agent_id: Option<&str>) -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: agent_id.map(str::to_string),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec![],
        }
    }

    fn mcp_auth(agent_id: &str) -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some(agent_id.to_string()),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec![],
        }
    }

    #[test]
    fn resolve_actor_cli_prefers_event() {
        let resolved = resolve_actor(&make_event(Some("from-event")), &cli_auth(Some("from-auth")));
        assert_eq!(resolved.ok().as_deref(), Some("from-event"));
    }

    #[test]
    fn resolve_actor_cli_falls_back_to_auth() {
        let resolved = resolve_actor(&make_event(None), &cli_auth(Some("from-auth")));
        assert_eq!(resolved.ok().as_deref(), Some("from-auth"));
    }

    #[test]
    fn resolve_actor_mcp_matching_passes() {
        let resolved = resolve_actor(&make_event(Some("agent-self")), &mcp_auth("agent-self"));
        assert_eq!(resolved.ok().as_deref(), Some("agent-self"));
    }

    #[test]
    fn resolve_actor_mcp_mismatched_rejected() {
        assert!(resolve_actor(&make_event(Some("agent-other")), &mcp_auth("agent-self")).is_err());
    }
}
