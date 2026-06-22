//! Router-level middlewares: the SPA-navigation shim and the build-skew header.
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use deck::AppServeConfig;

/// Pre-routing SPA-navigation shim. A GET to a client-route path — a bare in-app
/// route like `/git` or `/studio/themes` with no real server handler — is served
/// the primary app's `index.html` so a deep-link refresh hydrates client-side.
/// Everything else flows through to routing, crucially the Connect-ops
/// `fallback_service` (`/ship.ops.v1.*`, the root fallback) which a
/// `.fallback(spa)` handler would otherwise shadow into a dead data plane.
/// Runs INNER to the DaemonMode/AppServeConfig extension layers so it can
/// read them for the primary-app lookup.
pub(super) async fn spa_nav_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    if req.method() == axum::http::Method::GET && is_client_nav(req.uri().path()) {
        let mode = req
            .extensions()
            .get::<deck::DaemonMode>()
            .cloned()
            .map(axum::Extension);
        let apps = req.extensions().get::<Arc<Vec<AppServeConfig>>>().cloned();
        // Has a primary app? Serve its `index.html` (the SPA fallback
        // for client-nav paths). No primary app? Let routing continue
        // and the 404 handle it.
        //
        // Note: we read the file directly rather than going through
        // `deck::pages::serve_index`. serve_index does a
        // provisioning check (DataPort::is_provisioned → redirect to
        // /setup on fresh installs); for client-nav SPA hydration we
        // want the index regardless of provisioning state, so the
        // client-side router can boot. The first-run flow only fires
        // for the explicit `/` and `/setup` paths (their own routes).
        if let Some(apps) = apps {
            if let Some(primary) = apps.iter().find(|a| a.route == "/") {
                return read_index_html(
                    primary,
                    mode.map(|m| m.0).unwrap_or(deck_core::DaemonMode::Development),
                );
            }
        }
    }
    next.run(req).await
}

/// Read `<project>/<app.source>/index.html` and serve it as text/html.
/// Used by the SPA-nav middleware to hydrate a deep-link without going
/// through the first-run check; same file `serve_index` would return
/// once provisioned.
fn read_index_html(
    app: &AppServeConfig,
    mode: deck_core::DaemonMode,
) -> Response {
    // The per-app source is a repo-relative path (e.g. `apps/hub/.deck`).
    // We read from the project root; deck-core's `read_source_asset`
    // already does this. For test isolation the middleware keeps the
    let path = format!("{}/index.html", app.source);
    // The per-app source is repo-relative. The middleware reads from
    // disk directly (no deck-core coupling beyond the AppServeConfig
    // shape — the dev loop is shipd's concern, the per-app file
    // The per-app source is repo-relative. The dev daemon's cwd is the
    // worktree root; for production the daemon_dir contains the
    // resolved worktree hash and shipd resolves the workspace from
    // there. For now `current_dir` matches the dev loop's contract;
    // a future refactor will pass the workspace PathBuf as a
    // top-level Extension (like the apps + DataPort pattern).
    let Ok(body) = std::fs::read_to_string(
        std::env::current_dir().unwrap_or_default().join(&path),
    ) else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            format!("primary app `{}` index not found at {path}", app.name),
        )
            .into_response();
    };
    // Match `serve_scaffold`'s HTML response shape: a `<meta name="ship-mode">`
    // element injected into `<head>` so the client's `<sui-runtime>` can
    // know it's a dev server (the runtime reads the meta and dispatches
    // accordingly). For client-nav deep links, the runtime boots first
    // then the client router takes over.
    let meta = format!(r#"<meta name="ship-mode" content="{mode:?}" />"#);
    let html = body.replacen("<head>", &format!("<head>\n  {meta}"), 1);
    (
        axum::http::StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
        ],
        html,
    )
        .into_response()
}

/// Whether `path` is an in-app client route that should hydrate as the shell,
/// as opposed to a real server route or asset. Excludes every backend prefix,
/// the first-run handlers (`/`, `/setup`), and dotted asset/RPC paths
/// (`/favicon.ico`, `/ship.ops.v1.*`). This runs BEFORE routing, so the
/// exclusion set must cover every real GET route — a miss shadows it.
pub(super) fn is_client_nav(path: &str) -> bool {
    if path == "/" || path == "/setup" {
        return false;
    }
    const BACKEND: [&str; 7] = [
        "/api", "/_deck", "/hooks", "/auth", "/daemon", "/space", "/shell",
    ];
    if BACKEND.iter().any(|p| path.starts_with(p)) {
        return false;
    }
    // Dotted paths are assets (favicon.ico) or Connect RPC (`/ship.*`).
    !path.contains('.')
}

/// Stamp `X-Shipd-Build` on every response so clients can detect daemon
/// version skew. On the way in, also log a one-time WARN per distinct
/// `X-Shipd-Client-Build` value when the client's SHA diverges from
/// ours — daemon-side proof that a stale CLI is talking to us.
pub(super) async fn build_header_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let bi = build_info::build_info!();

    // Inbound: warn once per distinct client SHA on skew.
    if let Some(client_value) = req
        .headers()
        .get("x-shipd-client-build")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
    {
        let server_short = bi.short();
        let client_sha = client_value.split('+').next().unwrap_or("").to_string();
        if client_value != server_short && client_sha != bi.git_sha {
            static WARNED: std::sync::OnceLock<
                std::sync::Mutex<std::collections::HashSet<String>>,
            > = std::sync::OnceLock::new();
            let warned = WARNED.get_or_init(|| std::sync::Mutex::new(Default::default()));
            let mut set = warned.lock().unwrap();
            if set.insert(client_value.clone()) {
                tracing::warn!(
                    client = %client_value,
                    server = %server_short,
                    "client build differs from daemon — restart the client to avoid API skew"
                );
            }
        }
    }

    let mut response = next.run(req).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&bi.short()) {
        response.headers_mut().insert("X-Shipd-Build", value);
    }
    response
}
