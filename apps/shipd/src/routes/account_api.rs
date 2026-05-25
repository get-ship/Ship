//! Daemon HTTP routes for Ship Cloud account ops.
//!
//! All routes here require **user presence** — `CallerKind::Cli` or browser
//! cookie. MCP session tokens are categorically denied regardless of scope.
//! See SECURITY.md for the rationale (account ops shouldn't be reachable
//! through an agent's daemon session, even when the agent role is
//! `contributor` and would otherwise have `Scope::Full`).
//!
//! Routes:
//!
//! - `POST /api/v1/account/login` → start a PKCE login. CLI passes its
//!   local-callback `redirect_uri`; daemon returns the browser URL to open.
//! - `POST /api/v1/account/login/complete` → finish a PKCE login. CLI passes
//!   the `state_id` from the start call + the `code` it received via its
//!   own callback server.
//! - `GET  /api/v1/account/state` → snapshot for Studio / `ship whoami`.
//! - `POST /api/v1/account/logout` → clear the daemon's stored token.

use std::sync::Arc;

use axum::{
    Extension, Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ship_platform::account::AccountService;



#[derive(serde::Deserialize)]
pub struct StartLoginRequest {
    pub redirect_uri: String,
}

#[derive(serde::Serialize)]
pub struct StartLoginResponse {
    pub state_id: String,
    pub browser_url: String,
}

pub async fn start_login(
    Extension(svc): Extension<Arc<AccountService>>,
    Json(req): Json<StartLoginRequest>,
) -> Response {
    if !req.redirect_uri.starts_with("http://127.0.0.1:")
        && !req.redirect_uri.starts_with("http://localhost:")
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "redirect_uri must be a localhost http URL"
            })),
        )
            .into_response();
    }
    match svc.start_login(&req.redirect_uri).await {
        Ok(resp) => (
            StatusCode::OK,
            Json(StartLoginResponse {
                state_id: resp.state_id,
                browser_url: resp.browser_url,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct CompleteLoginRequest {
    pub state_id: String,
    pub code: String,
}

#[derive(serde::Serialize)]
pub struct CompleteLoginResponse {
    pub user_id: String,
}

pub async fn complete_login(
    Extension(svc): Extension<Arc<AccountService>>,
    Json(req): Json<CompleteLoginRequest>,
) -> Response {
    match svc.complete_login(&req.state_id, &req.code).await {
        Ok(me) => (
            StatusCode::OK,
            Json(CompleteLoginResponse { user_id: me.id }),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("invalid or expired") {
                StatusCode::UNAUTHORIZED
            } else if msg.contains("gateway") {
                StatusCode::BAD_GATEWAY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!({ "error": msg }))).into_response()
        }
    }
}

pub async fn get_state(
    Extension(svc): Extension<Arc<AccountService>>,
) -> Response {
    match svc.state().await {
        Ok(state) => (StatusCode::OK, Json(state)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn logout(
    Extension(svc): Extension<Arc<AccountService>>,
) -> Response {
    match svc.logout().await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// User-presence enforcement (CallerKind::Cli) is declared via `/// @auth cli`
// on each `Route::*AccountLogin*` / `Route::Logout*` variant. The router
// enforces at registration time; the comprehensive denial matrix (CLI passes,
// MCP denied even with Scope::Full, SDK denied) is covered in
// `apps/shipd/src/routes/scope_enforcement_tests.rs`. The Connect AccountService
// (account_rpc.rs) re-enforces the same presence check per-op, since the
// ops_rpc router doesn't run the per-route middleware.
