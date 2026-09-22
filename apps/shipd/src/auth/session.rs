//! HTTP-coupled session bits — `POST /api/v1/session` issuance (Route::IssueSession).
//!
//! Pure session helpers (generation/parsing/lookup/registry) moved to
//! `ship_auth::session`. This file holds only the axum-typed
//! handler + its response shape.

use axum::{
    body::Body,
    http::{HeaderValue, Response, StatusCode, header},
};
// Re-export the platform's pure helpers so existing
// `crate::auth::session::*` import paths continue to resolve.
pub use ship_auth::session::{
    BROWSER_SESSION_TTL_SECS, COOKIE_NAME, generate_browser_session_id, is_browser_session_id,
    issue_browser_session, issue_browser_session_with_ttl, lookup_browser_session,
    parse_session_cookie,
};

use crate::auth::AuthContext;
use crate::auth::scopes::SessionRegistry;

/// Response body for `POST /api/v1/session`.
#[derive(serde::Serialize)]
pub struct SessionIssueResponse {
    pub ok: bool,
    pub expires_in: u64,
}

/// `POST /api/v1/session` handler. Requires the caller to already
/// hold a verified session credential (the auth middleware enforces this).
/// Issues a browser-session cookie scoped to `/api/v1/runtime` and
/// returns `{ok, expires_in}`.
pub async fn issue_session_handler(
    registry: SessionRegistry,
    _auth: AuthContext,
) -> Response<Body> {
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
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("valid cookie header"),
    );
    resp
}
