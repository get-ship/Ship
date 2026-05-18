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
use ship_substrate::events::CallerKind;

use crate::auth::AuthContext;

/// Gate every account route on user presence. MCP session tokens are
/// denied even when their role grants `Scope::Full`; the floor is the
/// caller kind, not the scope.
fn require_user_presence(auth: &AuthContext) -> Option<Response> {
    match auth.caller_kind {
        CallerKind::Cli => None,
        _ => Some(
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "account ops require user-present caller (daemon token / browser cookie); MCP sessions are denied",
                })),
            )
                .into_response(),
        ),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "redirect_uri": "http://127.0.0.1:54321/callback" }))]
pub struct StartLoginRequest {
    pub redirect_uri: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct StartLoginResponse {
    pub state_id: String,
    pub browser_url: String,
}

#[utoipa::path(
    operation_id = "startAccountLogin",
    post,
    path = "/api/v1/account/login",
    request_body = StartLoginRequest,
    responses(
        (status = 200, description = "Login flow started", body = StartLoginResponse),
        (status = 400, description = "Invalid redirect_uri"),
        (status = 403, description = "Account ops require user presence"),
    ),
    tag = "account"
)]
pub async fn start_login(
    Extension(svc): Extension<Arc<AccountService>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<StartLoginRequest>,
) -> Response {
    if let Some(r) = require_user_presence(&auth) {
        return r;
    }
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

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CompleteLoginRequest {
    pub state_id: String,
    pub code: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CompleteLoginResponse {
    pub user_id: String,
}

#[utoipa::path(
    operation_id = "completeAccountLogin",
    post,
    path = "/api/v1/account/login/complete",
    request_body = CompleteLoginRequest,
    responses(
        (status = 200, description = "Login complete; token stored", body = CompleteLoginResponse),
        (status = 401, description = "Invalid or expired state, or PKCE failed at gateway"),
        (status = 403, description = "Account ops require user presence"),
        (status = 502, description = "Gateway unreachable or rejected exchange"),
    ),
    tag = "account"
)]
pub async fn complete_login(
    Extension(svc): Extension<Arc<AccountService>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CompleteLoginRequest>,
) -> Response {
    if let Some(r) = require_user_presence(&auth) {
        return r;
    }
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

#[utoipa::path(
    operation_id = "getAccountState",
    get,
    path = "/api/v1/account/state",
    responses(
        (status = 200, description = "Account state snapshot"),
        (status = 403, description = "Account ops require user presence"),
    ),
    tag = "account"
)]
pub async fn get_state(
    Extension(svc): Extension<Arc<AccountService>>,
    Extension(auth): Extension<AuthContext>,
) -> Response {
    if let Some(r) = require_user_presence(&auth) {
        return r;
    }
    match svc.state().await {
        Ok(state) => (StatusCode::OK, Json(state)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    operation_id = "logoutAccount",
    post,
    path = "/api/v1/account/logout",
    responses(
        (status = 204, description = "Token cleared (idempotent)"),
        (status = 403, description = "Account ops require user presence"),
    ),
    tag = "account"
)]
pub async fn logout(
    Extension(svc): Extension<Arc<AccountService>>,
    Extension(auth): Extension<AuthContext>,
) -> Response {
    if let Some(r) = require_user_presence(&auth) {
        return r;
    }
    match svc.logout().await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GetKeyResponse {
    pub key: String,
}

/// Transitional bridge — returns the stored API key so the CLI can
/// dual-write to its legacy `credentials.jsonc` for `ship publish` and
/// `ship convert <getship-url>` which still call the gateway directly.
/// REMOVE once those call sites route through the daemon.
#[utoipa::path(
    operation_id = "getAccountKey",
    get,
    path = "/api/v1/account/key",
    responses(
        (status = 200, description = "Stored API key", body = GetKeyResponse),
        (status = 401, description = "Not logged in"),
        (status = 403, description = "Account ops require user presence"),
    ),
    tag = "account"
)]
pub async fn get_key(
    Extension(svc): Extension<Arc<AccountService>>,
    Extension(auth): Extension<AuthContext>,
) -> Response {
    if let Some(r) = require_user_presence(&auth) {
        return r;
    }
    match svc.get_key().await {
        Ok(key) => (StatusCode::OK, Json(GetKeyResponse { key })).into_response(),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::scopes::Scope;

    fn cli_auth() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        }
    }

    fn mcp_auth_with_full_scope() -> AuthContext {
        // Worst case for the gate: an MCP caller that somehow holds
        // Scope::Full (a contributor-role session). Categorical denial
        // by CallerKind, regardless of scope.
        AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("attacker-agent".into()),
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        }
    }

    fn sdk_auth() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Sdk,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        }
    }

    #[test]
    fn cli_passes_user_presence_check() {
        assert!(require_user_presence(&cli_auth()).is_none());
    }

    #[test]
    fn mcp_is_denied_even_with_full_scope() {
        let r = require_user_presence(&mcp_auth_with_full_scope());
        assert!(r.is_some(), "MCP must be categorically denied");
    }

    #[test]
    fn sdk_is_denied() {
        // SDK is programmatic; account self-service should require an
        // interactive user. Keep this strict — relax only with intent.
        assert!(require_user_presence(&sdk_auth()).is_some());
    }
}
