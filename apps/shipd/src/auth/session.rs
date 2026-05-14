//! HTTP-coupled session bits — `POST /api/v1/runtime/session` issuance.
//!
//! Pure session helpers (generation/parsing/lookup/registry) moved to
//! `ship_platform::auth::session`. This file holds only the axum-typed
//! handler + its response shape.

use axum::{
    body::Body,
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
};
// Re-export the platform's pure helpers so existing
// `crate::auth::session::*` import paths continue to resolve.
pub use ship_platform::auth::session::{
    BROWSER_SESSION_TTL_SECS, COOKIE_NAME, generate_browser_session_id, is_browser_session_id,
    issue_browser_session, issue_browser_session_with_ttl, lookup_browser_session,
    parse_session_cookie,
};

use crate::auth::AuthContext;
use crate::auth::scopes::{Scope, SessionRegistry};

/// Response body for `POST /api/v1/runtime/session`.
#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "ok": true, "expires_in": 3600 }))]
pub struct SessionIssueResponse {
    pub ok: bool,
    pub expires_in: u64,
}

/// `POST /api/v1/runtime/session` handler. Requires the caller to already
/// hold a daemon-token credential (the auth middleware enforces this).
/// Issues a browser-session cookie scoped to `/api/v1/runtime` and
/// returns `{ok, expires_in}`.
#[utoipa::path(
    operation_id = "issueSessionHandler",
    post,
    path = "/api/v1/session",
    responses(
        (status = 200, description = "Browser session cookie issued", body = SessionIssueResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 403, description = "Session issuance requires Full scope"),
    ),
    tag = "auth"
)]
pub async fn issue_session_handler(
    registry: SessionRegistry,
    auth: AuthContext,
) -> Response<Body> {
    if !auth.has_scope(Scope::Full) {
        return (StatusCode::FORBIDDEN, "session issuance requires Full scope")
            .into_response();
    }

    let id = issue_browser_session(&registry).await;
    let cookie = format!(
        "{COOKIE_NAME}={id}; HttpOnly; SameSite=Strict; Path=/api/v1/runtime; Max-Age={BROWSER_SESSION_TTL_SECS}"
    );

    let body = serde_json::to_vec(&SessionIssueResponse {
        ok: true,
        expires_in: BROWSER_SESSION_TTL_SECS,
    })
    .expect("serialize SessionIssueResponse");

    let mut resp = Response::new(Body::from(body));
    *resp.status_mut() = StatusCode::OK;
    let headers = resp.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("valid cookie header"),
    );
    resp
}
