//! First-run `/setup` and root `/` handlers.
//!
//! Both routes are unauthenticated — a freshly installed daemon has not
//! yet paired with any browser or CLI session, so the setup wizard must
//! be reachable without a token. Once a workspace exists the daemon
//! redirects away from `/setup` so the wizard can't accidentally wipe
//! state on an existing install.
//!
//! The setup wizard UI now lives at the SUI view `setup`; the legacy
//! `.ship/views/setup/index.html.j2` render path was retired with the
//! j2 view shell. PLACEHOLDER_HTML remains as a fallback for the brief
//! window between daemon boot and the SUI runtime mounting.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

/// Placeholder HTML served when the SUI runtime hasn't mounted yet on a
/// fresh install. Real wizard UI lives at `/sui/setup`.
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
/// redirect to `/` so the user lands on the home view.
pub async fn setup_handler() -> Response {
    if has_any_workspace() {
        return redirect("/");
    }
    html_response(PLACEHOLDER_HTML.to_string())
}

/// `GET /` — redirect to `/setup` on a fresh install, or to the SUI home
/// view once at least one workspace is configured.
pub async fn index_handler() -> Response {
    if has_any_workspace() {
        redirect("/sui/home")
    } else {
        redirect("/setup")
    }
}

fn has_any_workspace() -> bool {
    ship_platform::projects::list()
        .map(|projects| !projects.is_empty())
        .unwrap_or(false)
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
        assert!(loc_str == "/setup" || loc_str == "/sui/home");
    }

    #[test]
    fn placeholder_html_contains_expected_shell() {
        assert!(PLACEHOLDER_HTML.contains("<title>Ship — Setup</title>"));
        assert!(PLACEHOLDER_HTML.contains("data-ship-setup"));
    }
}
