//! `/api/runtime/view/{name}/raw` — full HTML document for Studio and
//! direct-browser mounting.
//!
//! For SDK-backed first-party views (pipelines, git, specs, settings)
//! this renders the canonical shell template inline and wraps it in
//! `_raw_shell.html.j2`. Asset URLs in the wrapper are absolute so the
//! document works identically when opened directly in a browser or when
//! embedded by Studio as an `<iframe>` source.
//!
//! For user-defined views under `.ship/views/{name}/index.html` the
//! handler falls back to reading that file verbatim (legacy path).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
};
use serde::Serialize;

use crate::rest_api::ApiState;
use crate::views::templates;

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
        crate::session::issue_browser_session(state.session_registry()).await;
    format!(
        "{name}={id}; HttpOnly; SameSite=Strict; Path=/api/runtime; Max-Age={ttl}",
        name = crate::session::COOKIE_NAME,
        id = session_id,
        ttl = crate::session::BROWSER_SESSION_TTL_SECS,
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

fn wrap(title: &str, inner: &str) -> Option<String> {
    templates::render("_raw_shell.html.j2", &RawCtx { title, inner }).ok()
}

/// Render the SDK shell for a known first-party view name.
/// Returns `None` for unknown names, leaving the caller to fall back.
async fn render_sdk_shell(state: &ApiState, name: &str) -> Option<String> {
    let inner = match name {
        "pipelines" => {
            crate::views_pipelines::pipelines_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        "git" => {
            crate::views_git::git_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        "specs" => {
            crate::specs_view::specs_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        "settings" => {
            crate::views_settings::settings_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        "agents" => {
            crate::views_agents::agents_shell(State(state.clone()))
                .await
                .ok()?
                .0
        }
        "skills" => {
            crate::views_skills::skills_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        "chat" => {
            crate::views_chat::chat_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        "events" => {
            crate::views_events::events_shell(
                State(state.clone()),
                Query(Default::default()),
            )
            .await
            .ok()?
            .0
        }
        _ => return None,
    };
    let title = match name {
        "pipelines" => "Pipelines",
        "git" => "Git",
        "specs" => "Specs",
        "settings" => "Settings",
        "agents" => "Agents",
        "skills" => "Skills",
        "chat" => "Chat",
        "events" => "Events",
        _ => "Ship",
    };
    wrap(title, &inner)
}

/// GET `/api/runtime/view/{name}/raw`
///
/// Cookie-first auth: mints a `ship_session` cookie scoped to
/// `/api/runtime` so subsequent fetches from the served document
/// authenticate without needing the query token. The auth middleware
/// has already validated the caller's token/session before we run.
#[utoipa::path(
    operation_id = "serveViewRaw",
    get,
    path = "/api/runtime/view/{name}/raw",
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

    if let Some(html) = render_sdk_shell(&state, &name).await {
        let cookie = mint_cookie(&state).await;
        return html_ok(html, cookie);
    }

    // Legacy fallback for user-defined views under `.ship/views/{name}/`.
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
#[path = "views_raw_tests.rs"]
mod views_raw_tests;
