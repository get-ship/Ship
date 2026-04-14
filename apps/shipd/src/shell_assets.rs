//! `/shell/*` static-asset router for the Ship dev shell.
//!
//! Two modes:
//!   1. **Embedded** — assets baked into the binary via `include_dir!`.
//!      Used in release builds and when `SHIP_SHELL_DEV_URL` is unset.
//!   2. **Dev proxy** — when `SHIP_SHELL_DEV_URL=http://127.0.0.1:3100` is
//!      set, reverse-proxies `/shell/*` to the Vite dev server. Preserves
//!      HMR while letting the daemon own the origin (so the session cookie
//!      works uniformly for both app and API).
//!
//! On the initial HTML GET (`/shell/` or `/shell/devtools/`), the handler
//! issues a `ship_session` browser cookie if the caller doesn't already
//! have one. This is the same session type used elsewhere in the daemon
//! (see `session.rs`) — we are not inventing new auth, only reusing the
//! existing mechanism on a new path.

use crate::session::{
    self, BROWSER_SESSION_TTL_SECS, COOKIE_NAME, issue_browser_session, parse_session_cookie,
};
use crate::auth::SessionRegistry;
use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use include_dir::{Dir, include_dir};
use std::sync::Arc;

/// Baked-in shell bundle. Empty when `apps/shell/dist/` was empty at
/// compile time — `shell_response` detects this and returns a friendly
/// build-me page instead of a 404.
static SHELL_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../shell/dist");

/// State passed to the `/shell/*` handlers.
#[derive(Clone)]
pub struct ShellState {
    pub sessions: SessionRegistry,
    pub dev_proxy: Option<Arc<String>>,
    pub client: reqwest::Client,
}

/// Build the `/shell` router (two variants: `/shell` and `/shell/{*path}`).
pub fn router(sessions: SessionRegistry) -> Router {
    let dev_proxy = std::env::var("SHIP_SHELL_DEV_URL").ok().map(|s| {
        let trimmed = s.trim_end_matches('/').to_string();
        tracing::info!("shipd: shell proxying to {}", trimmed);
        Arc::new(trimmed)
    });
    let state = ShellState {
        sessions,
        dev_proxy,
        client: reqwest::Client::new(),
    };
    Router::new()
        .route("/shell", any(handle))
        .route("/shell/", any(handle))
        .route("/shell/{*path}", any(handle))
        .with_state(state)
}

/// Dispatcher — routes to dev proxy or embedded handler.
async fn handle(State(state): State<ShellState>, req: Request) -> Response {
    if let Some(upstream) = state.dev_proxy.as_ref() {
        return proxy_to_upstream(&state, upstream.as_str(), req).await;
    }
    embedded_response(&state, req).await
}

/// Serve from the embedded `dist/` bundle.
async fn embedded_response(state: &ShellState, req: Request) -> Response {
    if SHELL_DIST.entries().is_empty() {
        return missing_bundle_page();
    }

    let path = req.uri().path();
    // `/shell` and `/shell/` → index.html; `/shell/devtools` and
    // `/shell/devtools/` → devtools.html; everything else is a file lookup.
    let (rel, is_html_entry) = resolve_path(path);

    let file = match rel.as_deref().and_then(|p| SHELL_DIST.get_file(p)) {
        Some(f) => f,
        None if is_html_entry => {
            return missing_bundle_page();
        }
        None => {
            return (StatusCode::NOT_FOUND, format!("shell: {path} not found")).into_response();
        }
    };

    let content_type = guess_mime(file.path().to_str().unwrap_or(""));
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type);

    if is_html_entry {
        // Mint a session cookie if the request doesn't carry one.
        if !has_session_cookie(&req) {
            let id = issue_browser_session(&state.sessions).await;
            let cookie = format!(
                "{COOKIE_NAME}={id}; HttpOnly; SameSite=Strict; Path=/; Max-Age={BROWSER_SESSION_TTL_SECS}"
            );
            if let Ok(v) = HeaderValue::from_str(&cookie) {
                builder = builder.header(header::SET_COOKIE, v);
            }
        }
        builder = builder.header(header::CACHE_CONTROL, "no-cache");
    }

    builder
        .body(Body::from(file.contents().to_vec()))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "response build failed").into_response())
}

/// Compute (relative-path-inside-dist, is_html_entry).
fn resolve_path(path: &str) -> (Option<String>, bool) {
    // Strip the `/shell` prefix.
    let rest = path.strip_prefix("/shell").unwrap_or(path);
    match rest {
        "" | "/" => (Some("index.html".into()), true),
        "/devtools" | "/devtools/" => (Some("devtools.html".into()), true),
        other => {
            let trimmed = other.trim_start_matches('/').to_string();
            (Some(trimmed), false)
        }
    }
}

fn has_session_cookie(req: &Request) -> bool {
    req.headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_session_cookie)
        .map(|id| session::is_browser_session_id(&id))
        .unwrap_or(false)
}

fn guess_mime(path: &str) -> &'static str {
    if path.ends_with(".html") { "text/html; charset=utf-8" }
    else if path.ends_with(".js") || path.ends_with(".mjs") { "text/javascript; charset=utf-8" }
    else if path.ends_with(".css") { "text/css; charset=utf-8" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".woff2") { "font/woff2" }
    else if path.ends_with(".json") { "application/json" }
    else { "application/octet-stream" }
}

/// Reverse-proxy the request to the Vite dev server.
async fn proxy_to_upstream(state: &ShellState, upstream: &str, req: Request) -> Response {
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let url = format!("{upstream}{path_and_query}");

    let method = req.method().clone();
    let mut builder = state.client.request(method, &url);
    // Forward a minimal, safe header set (skip hop-by-hop + Host).
    for (name, value) in req.headers() {
        let n = name.as_str().to_ascii_lowercase();
        if matches!(
            n.as_str(),
            "host" | "connection" | "content-length" | "transfer-encoding" | "upgrade"
        ) {
            continue;
        }
        builder = builder.header(name.as_str(), value.as_bytes());
    }

    let is_html_entry = matches!(
        req.uri().path(),
        "/shell" | "/shell/" | "/shell/devtools" | "/shell/devtools/"
    );
    let has_cookie = has_session_cookie(&req);

    let body_bytes = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("read body: {e}")).into_response(),
    };
    if !body_bytes.is_empty() {
        builder = builder.body(body_bytes.to_vec());
    }

    let upstream_res = match builder.send().await {
        Ok(r) => r,
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("proxy to {url}: {e}")).into_response(),
    };

    let status = StatusCode::from_u16(upstream_res.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let mut resp_builder = Response::builder().status(status);
    for (name, value) in upstream_res.headers() {
        let n = name.as_str().to_ascii_lowercase();
        if matches!(n.as_str(), "transfer-encoding" | "connection" | "content-length") {
            continue;
        }
        if let Ok(v) = HeaderValue::from_bytes(value.as_bytes()) {
            resp_builder = resp_builder.header(name.as_str(), v);
        }
    }

    if is_html_entry && !has_cookie {
        let id = issue_browser_session(&state.sessions).await;
        let cookie = format!(
            "{COOKIE_NAME}={id}; HttpOnly; SameSite=Strict; Path=/; Max-Age={BROWSER_SESSION_TTL_SECS}"
        );
        if let Ok(v) = HeaderValue::from_str(&cookie) {
            resp_builder = resp_builder.header(header::SET_COOKIE, v);
        }
    }

    let bytes = match upstream_res.bytes().await {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("read upstream: {e}")).into_response(),
    };
    resp_builder
        .body(Body::from(bytes))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "response build").into_response())
}

fn missing_bundle_page() -> Response {
    let html = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>Ship shell — not built</title>
<style>body{font-family:system-ui;padding:40px;max-width:640px;margin:auto;color:#333}
code{background:#f5f5f5;padding:2px 6px;border-radius:3px;font-family:ui-monospace,Menlo,monospace}
h1{font-size:18px}</style></head><body>
<h1>Ship shell bundle not found</h1>
<p>The daemon was built without a populated <code>apps/shell/dist/</code>.
Run:</p>
<pre><code>pnpm -F shell build</code></pre>
<p>then rebuild the daemon:</p>
<pre><code>cargo build -p shipd --bin shipd</code></pre>
<p>Or use the dev proxy — in one terminal start Vite:</p>
<pre><code>pnpm -F shell dev</code></pre>
<p>then run the daemon with <code>SHIP_SHELL_DEV_URL=http://127.0.0.1:3100</code>.</p>
</body></html>"#;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_maps_entries() {
        assert_eq!(resolve_path("/shell"), (Some("index.html".into()), true));
        assert_eq!(resolve_path("/shell/"), (Some("index.html".into()), true));
        assert_eq!(
            resolve_path("/shell/devtools"),
            (Some("devtools.html".into()), true)
        );
        assert_eq!(
            resolve_path("/shell/devtools/"),
            (Some("devtools.html".into()), true)
        );
        assert_eq!(
            resolve_path("/shell/assets/main.js"),
            (Some("assets/main.js".into()), false)
        );
    }

    #[test]
    fn mime_guess_covers_common_types() {
        assert_eq!(guess_mime("x.html"), "text/html; charset=utf-8");
        assert_eq!(guess_mime("x.js"), "text/javascript; charset=utf-8");
        assert_eq!(guess_mime("x.css"), "text/css; charset=utf-8");
        assert_eq!(guess_mime("x.bin"), "application/octet-stream");
    }
}
