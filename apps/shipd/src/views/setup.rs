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

/// `GET /` — on a fresh install redirect to the `/setup` wizard; once a
/// workspace is configured SERVE THE SHELL (the Ship platform app) at the
/// root. The shell's client router lands on the home surface (`<sui-home>`);
/// there is no daemon `home` view, so the old `redirect("/sui/home")` 404'd
/// ("no view named home"). The shell IS the platform root — no `/hub`.
pub async fn index_handler(
    mode: Option<axum::Extension<deck::DaemonMode>>,
    cfg: Option<axum::Extension<std::sync::Arc<deck::config::DeckConfig>>>,
) -> Response {
    if has_any_workspace() {
        deck::serve_shell(mode, cfg).await
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
    async fn index_handler_serves_shell_never_missing_home_view() {
        // Fresh install (no workspace) -> redirect to /setup; configured ->
        // serve the shell HTML at / (or a not-built fallback). It must NEVER
        // redirect to /sui/home, which is not a daemon view (the bug we fixed).
        let resp = index_handler(None, None).await;
        if resp.status() == StatusCode::SEE_OTHER {
            let loc = resp.headers().get(header::LOCATION).unwrap().to_str().unwrap();
            assert_eq!(loc, "/setup", "fresh-install redirect must be /setup, never /sui/home");
        }
    }

    #[test]
    fn placeholder_html_contains_expected_shell() {
        assert!(PLACEHOLDER_HTML.contains("<title>Ship — Setup</title>"));
        assert!(PLACEHOLDER_HTML.contains("data-ship-setup"));
    }
}
