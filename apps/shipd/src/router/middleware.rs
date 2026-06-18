//! Router-level middlewares: the SPA-navigation shim and the build-skew header.
use axum::response::Response;
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
        if apps.as_ref().is_some_and(|v| v.iter().any(|a| a.route == "/")) {
            return deck::pages::serve_index(
                mode,
                apps.map(axum::Extension),
                None,
            )
            .await;
        }
    }
    next.run(req).await
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
    const BACKEND: [&str; 8] = [
        "/api", "/_deck", "/hooks", "/auth", "/webhook", "/daemon", "/space", "/shell",
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
