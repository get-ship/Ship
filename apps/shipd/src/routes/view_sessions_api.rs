//! REST handlers for agent-owned view sessions.
//!
//! Seven routes (spec §Daemon REST endpoints):
//!
//! - `POST   /api/v1/view-sessions`              — open
//! - `POST   /api/v1/view-sessions/{id}/patch`   — patch
//! - `POST   /api/v1/view-sessions/{id}/append`  — append
//! - `POST   /api/v1/view-sessions/{id}/signal`  — signal
//! - `DELETE /api/v1/view-sessions/{id}`         — close
//! - `GET    /api/v1/view-sessions/{id}`         — subscribe (SSE)
//! - `GET    /api/v1/view-sessions/{id}/raw`     — iframe HTML shell
//!
//! Authentication/ownership: the caller's `AuthContext.agent_id` must be
//! present; mutating routes require the caller to own the session.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::AuthContext;
use crate::rest_api::ApiState;
use crate::views::session::SessionError;

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "template": "chat",
    "title": "Planner chat",
    "initial_context": { "topic": "ship" }
}))]
pub struct OpenReq {
    pub template: String,
    pub title: Option<String>,
    pub initial_context: Option<Value>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "selector": "#status",
    "html": "<span class=\"ok\">running</span>"
}))]
pub struct PatchReq {
    pub selector: String,
    pub html: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "name": "approve",
    "value": { "by": "me", "at": "2026-04-22T10:15:00Z" }
}))]
pub struct SignalReq {
    pub name: String,
    pub value: Value,
}

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

#[utoipa::path(
    operation_id = "openSession",
    post,
    path = "/api/v1/view-sessions",
    request_body = OpenReq,
    responses(
        (status = 200, description = "View session created"),
        (status = 401, description = "Unauthenticated / missing agent id", body = crate::openapi_types::ErrorResponse),
        (status = 400, description = "Unknown template", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "Agent quota exceeded", body = crate::openapi_types::ErrorResponse),
        (status = 429, description = "Rate limited", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
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
            let meta = crate::mcp::apps_bridge::view_session_meta(&view_id);
            ok_json(serde_json::json!({
                "view_id": view_id,
                "url": url,
                "_meta": meta,
            }))
        }
        Err(e) => session_err(e),
    }
}

#[utoipa::path(
    operation_id = "patchSession",
    post,
    path = "/api/v1/view-sessions/{id}/patch",
    params(("id" = String, Path, description = "View session id")),
    request_body = PatchReq,
    responses(
        (status = 200, description = "Patch applied"),
        (status = 401, description = "Unauthenticated / missing agent id", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Not session owner", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
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

#[utoipa::path(
    operation_id = "appendSession",
    post,
    path = "/api/v1/view-sessions/{id}/append",
    params(("id" = String, Path, description = "View session id")),
    request_body = PatchReq,
    responses(
        (status = 200, description = "Fragment appended"),
        (status = 401, description = "Unauthenticated / missing agent id", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Not session owner", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
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

#[utoipa::path(
    operation_id = "signalSession",
    post,
    path = "/api/v1/view-sessions/{id}/signal",
    params(("id" = String, Path, description = "View session id")),
    request_body = SignalReq,
    responses(
        (status = 200, description = "Signal delivered"),
        (status = 401, description = "Unauthenticated / missing agent id", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Not session owner", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
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

#[utoipa::path(
    operation_id = "closeSession",
    delete,
    path = "/api/v1/view-sessions/{id}",
    params(("id" = String, Path, description = "View session id")),
    responses(
        (status = 200, description = "Session closed"),
        (status = 401, description = "Unauthenticated / missing agent id", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Not session owner", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
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

#[utoipa::path(
    operation_id = "subscribeSession",
    get,
    path = "/api/v1/view-sessions/{id}",
    params(("id" = String, Path, description = "View session id")),
    responses(
        (status = 200, description = "SSE stream of session patches", content_type = "text/event-stream"),
        (status = 401, description = "Unauthenticated / missing agent id", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
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

/// Iframe shell HTML. Renders the session's template via the static minijinja
/// env, exposing `{view_id, title, context}` plus `{{SHIP_TOKEN}}` scrubbed
/// for cookie-auth parity with `views::serve_view_raw`.
#[utoipa::path(
    operation_id = "rawSession",
    get,
    path = "/api/v1/view-sessions/{id}/raw",
    params(("id" = String, Path, description = "View session id")),
    responses(
        (status = 200, description = "Rendered iframe shell HTML", content_type = "text/html"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Session not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn raw_session(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Response {
    let (template, title, context) = match state.view_sessions().shell_context(&id).await {
        Ok(v) => v,
        Err(e) => return session_err(e),
    };
    #[derive(serde::Serialize)]
    struct Ctx {
        view_id: String,
        title: Option<String>,
        context: Value,
    }
    let ctx = Ctx { view_id: id, title, context };
    match crate::views::templates::render(&template, &ctx) {
        Ok(html) => {
            let html = html.replace("{{SHIP_TOKEN}}", "");
            Response::builder()
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(axum::http::header::CACHE_CONTROL, "no-cache")
                // CSP shared with `_meta.ui.csp` in the MCP Apps bridge so
                // local iframe and external host renderings align.
                .header("content-security-policy", crate::mcp::apps_bridge::IFRAME_CSP)
                .body(axum::body::Body::from(html))
                // Infallible: status + static header names/values above are
                // all valid; `Response::builder` only errors on malformed
                // header construction, none of which applies here.
                .unwrap()
        }
        Err(msg) => err_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"code": "render_failed", "message": msg}),
        ),
    }
}

#[cfg(test)]
#[path = "view_sessions_api_tests.rs"]
mod tests;
