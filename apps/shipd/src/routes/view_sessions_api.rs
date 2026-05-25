//! REST handlers for agent-owned view sessions.
//!
//! Six routes (spec §Daemon REST endpoints):
//!
//! - `POST   /api/v1/view-sessions`              — open
//! - `POST   /api/v1/view-sessions/{id}/patch`   — patch
//! - `POST   /api/v1/view-sessions/{id}/append`  — append
//! - `POST   /api/v1/view-sessions/{id}/signal`  — signal
//! - `DELETE /api/v1/view-sessions/{id}`         — close
//! - `GET    /api/v1/view-sessions/{id}`         — subscribe (SSE)
//!
//! Authentication/ownership: the caller's `AuthContext.agent_id` must be
//! present; mutating routes require the caller to own the session. The
//! iframe HTML shell endpoint (`/raw`) was retired with the j2 view shell;
//! agents that want to render their own canvas open a SUI view instead.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;

use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;
use crate::views::session::SessionError;

#[derive(Deserialize)]
pub struct OpenReq {
    pub template: String,
    pub title: Option<String>,
    pub initial_context: Option<Value>,
}

#[derive(Deserialize)]
pub struct PatchReq {
    pub selector: String,
    pub html: String,
}

#[derive(Deserialize)]
pub struct SignalReq {
    pub name: String,
    pub value: Value,
}

#[allow(clippy::result_large_err)]
fn require_agent(auth: &AuthContext) -> Result<String, Response> {
    auth.agent_id.clone().ok_or_else(|| {
        err_json(
            StatusCode::UNAUTHORIZED,
            serde_json::json!({"code": "unauthenticated",
                "message": "view-session routes require an authenticated agent id"}),
        )
    })
}

fn err_json(status: StatusCode, body: Value) -> Response {
    (status, Json(serde_json::json!({"ok": false, "error": body}))).into_response()
}

fn session_err(e: SessionError) -> Response {
    let status = match &e {
        SessionError::NotFound { .. } => StatusCode::NOT_FOUND,
        SessionError::NotOwner { .. } => StatusCode::FORBIDDEN,
        SessionError::QuotaExceeded { .. } => StatusCode::CONFLICT,
        SessionError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        SessionError::UnknownTemplate { .. } => StatusCode::BAD_REQUEST,
    };
    // `SessionError` serializes to `{code, ...fields}` via `#[serde(tag="code")]`.
    // `to_value` on a plain serde-derived enum with owned `String` fields is
    // infallible; the `unwrap_or_else` guards against future variants that
    // introduce custom `Serialize` impls which could return an error.
    err_json(
        status,
        serde_json::to_value(&e).unwrap_or_else(|_| serde_json::json!({"code": "serialize_failed"})),
    )
}

fn ok_json(body: Value) -> Response {
    (StatusCode::OK, Json(serde_json::json!({"ok": true, "data": body}))).into_response()
}

// ───────────────────────── Handlers ─────────────────────────

pub async fn open_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<OpenReq>,
) -> Response {
    let agent = match require_agent(&auth) { Ok(a) => a, Err(r) => return r };
    match state
        .view_sessions()
        .open(
            agent,
            req.template,
            req.title,
            req.initial_context.unwrap_or_else(|| serde_json::json!({})),
        )
        .await
    {
        Ok(view_id) => {
            let url = format!("/api/v1/view-sessions/{view_id}/raw");
            let meta = crate::views::mcp_apps_embed::view_session_meta(&view_id);
            ok_json(serde_json::json!({
                "view_id": view_id,
                "url": url,
                "_meta": meta,
            }))
        }
        Err(e) => session_err(e),
    }
}

pub async fn patch_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(req): Json<PatchReq>,
) -> Response {
    let agent = match require_agent(&auth) { Ok(a) => a, Err(r) => return r };
    match state.view_sessions().patch(&id, &agent, &req.selector, req.html).await {
        Ok(()) => ok_json(serde_json::json!({"ok": true})),
        Err(e) => session_err(e),
    }
}

pub async fn append_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(req): Json<PatchReq>,
) -> Response {
    let agent = match require_agent(&auth) { Ok(a) => a, Err(r) => return r };
    match state.view_sessions().append(&id, &agent, &req.selector, req.html).await {
        Ok(()) => ok_json(serde_json::json!({"ok": true})),
        Err(e) => session_err(e),
    }
}

pub async fn signal_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(req): Json<SignalReq>,
) -> Response {
    let agent = match require_agent(&auth) { Ok(a) => a, Err(r) => return r };
    match state.view_sessions().signal(&id, &agent, &req.name, req.value).await {
        Ok(()) => ok_json(serde_json::json!({"ok": true})),
        Err(e) => session_err(e),
    }
}

pub async fn close_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    let agent = match require_agent(&auth) { Ok(a) => a, Err(r) => return r };
    match state.view_sessions().close(&id, &agent).await {
        Ok(()) => ok_json(serde_json::json!({"ok": true})),
        Err(e) => session_err(e),
    }
}

pub async fn subscribe_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    // Subscribe is read-only but still agent-scoped: anonymous MCP
    // callers have no business tailing another agent's canvas. The
    // daemon's bearer auth has already run; this rejects connections
    // that lack an agent id.
    if let Err(r) = require_agent(&auth) {
        return r;
    }
    match state.view_sessions().subscribe(&id).await {
        Ok(sse) => sse.into_response(),
        Err(e) => session_err(e),
    }
}

#[cfg(test)]
#[path = "view_sessions_api_tests.rs"]
mod tests;
