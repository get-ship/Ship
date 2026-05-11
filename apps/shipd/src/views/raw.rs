//! `/api/v1/view/{name}/raw` — full HTML document for Studio and
//! direct-browser mounting.
//!
//! For SDK-backed first-party views (settings, agents, chat, events)
//! this renders the canonical shell template inline and wraps it in
//! `_raw_shell.html.j2`. Asset URLs in the wrapper are absolute so the
//! document works identically when opened directly in a browser or when
//! embedded by Studio as an `<iframe>` source.
//!
//! For user-defined views under `.ship/views/{name}/index.html` the
//! handler falls back to reading that file verbatim (legacy path).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};
use serde::Serialize;

use crate::rest_api::ApiState;
use crate::views::templates;

#[allow(dead_code)] // used by wrap() which is called from the raw view handler below
#[derive(Serialize)]
struct RawCtx<'a> {
    title: &'a str,
    inner: &'a str,
}

fn err(status: StatusCode, msg: String) -> Response {
    Response::builder()
        .status(status)
        .body(axum::body::Body::from(msg))
        .unwrap()
}

async fn mint_cookie(state: &ApiState) -> String {
    let session_id =
        crate::auth::session::issue_browser_session(state.session_registry()).await;
    format!(
        "{name}={id}; HttpOnly; SameSite=Strict; Path=/api; Max-Age={ttl}",
        name = crate::auth::session::COOKIE_NAME,
        id = session_id,
        ttl = crate::auth::session::BROWSER_SESSION_TTL_SECS,
    )
}

fn html_ok(body: String, cookie: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "no-cache")
        .header(axum::http::header::SET_COOKIE, cookie)
        .body(axum::body::Body::from(body))
        .unwrap()
}

#[allow(dead_code)] // used via serve_view_raw handler (route registered in router.rs)
fn wrap(title: &str, inner: &str) -> Option<String> {
    templates::render("_raw_shell.html.j2", &RawCtx { title, inner }).ok()
}

/// GET `/api/v1/view/{name}/raw`
///
/// Cookie-first auth: mints a `ship_session` cookie scoped to
/// `/api` so subsequent fetches from the served document
/// authenticate without needing the query token. The auth middleware
/// has already validated the caller's token/session before we run.
#[utoipa::path(
    operation_id = "serveViewRaw",
    get,
    path = "/api/v1/view/{name}/raw",
    params(("name" = String, Path, description = "View name")),
    responses(
        (status = 200, description = "Raw view shell HTML with session cookie", content_type = "text/html"),
        (status = 400, description = "Invalid view name", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "View not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn serve_view_raw(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Response {
    if !crate::views::is_valid_view_name(&name) {
        return err(StatusCode::BAD_REQUEST, "invalid view name".into());
    }

    // Static `index.html` fallback for third-party views that ship a
    // plain HTML doc. File-routed (`.j2`) views use `/api/v1/v/{*path}`.
    let (view_dir, _) = match crate::views::resolve_view_dir(&name) {
        Some(v) => v,
        None => return err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    let html = match std::fs::read_to_string(view_dir.join("index.html")) {
        Ok(h) => h.replace("{{SHIP_TOKEN}}", ""),
        Err(_) => return err(StatusCode::NOT_FOUND, format!("no index.html for '{name}'")),
    };
    let cookie = mint_cookie(&state).await;
    html_ok(html, cookie)
}

#[cfg(test)]
#[path = "raw_tests.rs"]
mod raw_tests;
