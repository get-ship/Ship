//! First-run `/setup` and root `/` handlers.
//!
//! Both routes are unauthenticated — a freshly installed daemon has not
//! yet paired with any browser or CLI session, so the setup wizard must
//! be reachable without a token. Once a workspace exists the daemon
//! redirects away from `/setup` so the wizard can't accidentally wipe
//! state on an existing install.
//!
//! The real wizard UI is filled in by a `.ship/views/setup/` template
//! authored by the web lane. This handler only owns the routing +
//! first-run detection and a minimal placeholder served when no user
//! view has been authored yet.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

/// Placeholder HTML returned when no workspaces exist and the web lane
/// has not yet dropped a `.ship/views/setup/index.html.j2` on disk.
const PLACEHOLDER_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Ship — Setup</title>
  <style>
    body{font-family:system-ui,-apple-system,sans-serif;background:#0b0d10;color:#e6e8eb;margin:0;min-height:100vh;display:grid;place-items:center;padding:24px}
    main{max-width:560px;width:100%;text-align:center}
    h1{font-size:28px;margin:0 0 12px;letter-spacing:-0.01em}
    p{color:#9aa1a9;line-height:1.5;margin:0 0 20px}
    code{background:#1a1d21;padding:2px 6px;border-radius:4px;font-family:ui-monospace,Menlo,monospace;font-size:13px}
  </style>
</head>
<body>
  <main data-ship-setup>
    <h1>Welcome to Ship</h1>
    <p>The setup wizard is on its way. In the meantime, point a CLI or MCP client at <code>http://localhost:9315</code> to start configuring this daemon.</p>
    <p>Run <code>ship init</code> in a project to scaffold your first workspace.</p>
  </main>
</body>
</html>
"#;

/// `GET /setup` — serve the setup wizard if no workspaces exist, otherwise
/// redirect to `/` so the user lands on the dashboard.
#[utoipa::path(
    operation_id = "setupHandler",
    get,
    path = "/setup",
    responses(
        (status = 200, description = "Setup wizard HTML", content_type = "text/html"),
        (status = 303, description = "Redirect to / when a workspace already exists"),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "views"
)]
pub async fn setup_handler() -> Response {
    if has_any_workspace() {
        return redirect("/");
    }
    match render_user_view() {
        Some(html) => html_response(html),
        None => html_response(PLACEHOLDER_HTML.to_string()),
    }
}

/// `GET /` — redirect to `/setup` on a fresh install, or to `/shell/` once
/// at least one workspace is configured.
#[utoipa::path(
    operation_id = "indexHandler",
    get,
    path = "/",
    responses(
        (status = 303, description = "Redirect to /setup or /shell/ depending on first-run state"),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "views"
)]
pub async fn index_handler() -> Response {
    if has_any_workspace() {
        redirect("/shell/")
    } else {
        redirect("/setup")
    }
}

fn has_any_workspace() -> bool {
    runtime::projects::list()
        .map(|projects| !projects.is_empty())
        .unwrap_or(false)
}

/// Render the web-lane-authored `.ship/views/setup/index.html.j2` if it
/// exists. Returns `None` when the view has not been created yet (or fails
/// to parse) so the handler can fall back to the built-in placeholder.
fn render_user_view() -> Option<String> {
    let source = read_setup_view_source()?;
    match crate::views::templates::render_view_source(
        "setup/index.html.j2",
        &source,
        &serde_json::json!({}),
    ) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::warn!(error = %e, "setup view render failed; using placeholder");
            None
        }
    }
}

fn read_setup_view_source() -> Option<String> {
    for dir in [
        runtime::project::get_project_dir(None).ok(),
        runtime::project::get_global_dir().ok(),
    ]
    .into_iter()
    .flatten()
    {
        let path = dir.join("views").join("setup").join("index.html.j2");
        if let Ok(s) = std::fs::read_to_string(&path) {
            return Some(s);
        }
    }
    None
}

fn html_response(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("setup render failed")))
}

fn redirect(location: &str) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, location)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn setup_handler_returns_html() {
        let resp = setup_handler().await;
        // Either a 200 with HTML (no workspaces in the test harness
        // scratch dir) or a redirect (if one happens to exist). Both
        // are legal states — the handler's contract is "never panics".
        let status = resp.status();
        assert!(
            status == StatusCode::OK || status == StatusCode::SEE_OTHER,
            "unexpected status: {status}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn index_handler_redirects() {
        let resp = index_handler().await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let location = resp.headers().get(header::LOCATION).unwrap();
        let loc_str = location.to_str().unwrap();
        assert!(loc_str == "/setup" || loc_str == "/shell/");
    }

    #[test]
    fn placeholder_html_contains_expected_shell() {
        assert!(PLACEHOLDER_HTML.contains("<title>Ship — Setup</title>"));
        assert!(PLACEHOLDER_HTML.contains("data-ship-setup"));
    }
}
