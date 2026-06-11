//! Thin HTTP ingress for daemon hook endpoints — session-start/end,
//! before/after-tool, permission-denied.
//!
//! All domain logic lives in `ship_agents::hooks::http_ingress`. These handlers
//! are responsible only for: auth scope enforcement, actor/session resolution
//! (which requires `AuthContext`), and building the `KernelLog` + `EmitContext`
//! for each request. The actual event-building and kernel emit are delegated.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ship_agents::hooks::http_ingress::{
    HookResponse, ShipHookEvent, handle_before_tool, handle_after_tool,
    handle_permission_denied, handle_session_end, handle_session_start,
};
use ship_substrate::events::CallerKind;
use ship_substrate::ports::KernelLog;
use std::sync::Arc;

use crate::auth::{AuthContext, Scope, enforce_mcp_identity, enforce_mcp_workspace, require_scope};
use crate::ship_runtime::ApiState;

// ── PtyBroadcastPort implementation ──────────────────────────────────────────

struct PtyBroadcastAdapter<'a>(&'a ApiState);

impl ship_agents::hooks::http_ingress::PtyBroadcastPort for PtyBroadcastAdapter<'_> {
    fn broadcast_tool_use(
        &self,
        harness_session_id: &str,
        tool: &str,
        input: Option<&serde_json::Value>,
    ) {
        let Some(session) = self.0.pty_sessions().find_by_harness_session(harness_session_id)
        else {
            return;
        };
        let input_preview = input
            .map(|v| {
                let s = v.to_string();
                if s.len() > 400 { s.chars().take(400).collect() } else { s }
            })
            .unwrap_or_default();
        let ev = ship_harnesses::harness_adapter::HarnessEvent::ToolUse {
            tool: tool.to_string(),
            input_preview,
        };
        let _ = session.0.event_tx.send(ev);
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

#[allow(clippy::result_large_err)]
fn resolve_actor(event: &ShipHookEvent, auth: &AuthContext) -> Result<String, Response> {
    enforce_mcp_identity(auth, event.actor_id.as_deref())?;
    enforce_mcp_workspace(auth, event.workspace.as_deref())?;
    Ok(event
        .actor_id
        .clone()
        .or_else(|| auth.agent_id.clone())
        .unwrap_or_else(|| "unknown".to_string()))
}

fn resolve_session(event: &ShipHookEvent, auth: &AuthContext) -> Option<String> {
    event.session_id.clone().or_else(|| auth.session_id.clone())
}

fn make_kernel(state: &ApiState) -> Arc<KernelLog> {
    Arc::new(KernelLog::new(state.kernel().clone()))
}

// ── Handlers ─────────────────────────────────────────────────────────────────

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
    let kernel = make_kernel(&state);
    let emit_ctx = auth.to_emit_context(CallerKind::Runtime);
    handle_session_start(&kernel, emit_ctx, &event, &actor, session.as_deref()).await;
    (StatusCode::OK, Json(HookResponse::allow())).into_response()
}

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
    tracing::debug!(actor = %actor, session = ?session, "hook: before-tool");
    let kernel = make_kernel(&state);
    let emit_ctx = auth.to_emit_context(CallerKind::Runtime);
    let pty = PtyBroadcastAdapter(&state);
    handle_before_tool(&kernel, emit_ctx, &pty, &event, &actor, session.as_deref()).await;
    (StatusCode::OK, Json(HookResponse::allow())).into_response()
}

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
    tracing::debug!(actor = %actor, session = ?session, "hook: after-tool");
    let kernel = make_kernel(&state);
    let emit_ctx = auth.to_emit_context(CallerKind::Runtime);
    let pty = PtyBroadcastAdapter(&state);
    handle_after_tool(&kernel, emit_ctx, &pty, &event, &actor, session.as_deref()).await;
    (StatusCode::OK, Json(HookResponse::allow())).into_response()
}

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
    let kernel = make_kernel(&state);
    let emit_ctx = auth.to_emit_context(CallerKind::Runtime);
    handle_session_end(&kernel, emit_ctx, &event, &actor, session.as_deref()).await;
    (StatusCode::OK, Json(HookResponse::allow())).into_response()
}

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
    tracing::debug!(actor = %actor, "hook: permission-denied");
    let kernel = make_kernel(&state);
    let emit_ctx = auth.to_emit_context(CallerKind::Runtime);
    handle_permission_denied(&kernel, emit_ctx, &event, &actor, session.as_deref()).await;
    (StatusCode::OK, Json(HookResponse::allow())).into_response()
}
