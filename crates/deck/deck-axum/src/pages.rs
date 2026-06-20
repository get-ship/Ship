//! deck-owned root + shared-asset handlers: design tokens, theme CSS/list,
//! first-run `/` + `/setup`, the single-segment public-asset-or-shell
//! dispatcher, and the SPA fallback. All thin axum shims over `deck-core`.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};

use deck_config::DeckConfig;
use deck_core::{DaemonMode, DataPort};

use crate::{handlers::serve_shell, into_axum, mode_from_ext};

// ── Shared CSS / theme assets ──────────────────────────────────────────────────

/// `GET /api/assets/view-tokens.css`
pub async fn serve_view_tokens(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_view_tokens(mode_from_ext(mode)))
}

/// `GET /api/assets/theme.css`
pub async fn serve_theme_css(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_theme_css(mode_from_ext(mode)))
}

/// `GET /api/v1/themes`
pub async fn serve_themes_list(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_themes_list(mode_from_ext(mode)))
}

// ── First-run flow ─────────────────────────────────────────────────────────────

/// Read the host's provisioning state through the `DataPort`. When no port is
/// wired the deck default is "not provisioned" (the fresh-install posture).
fn is_provisioned(port: &Option<Extension<Arc<dyn DataPort>>>) -> bool {
    port.as_ref().map(|e| e.0.is_provisioned()).unwrap_or(false)
}

/// `GET /` — serve the shell once provisioned; on a fresh install redirect to
/// `/setup`. The shell's client router lands on the home surface.
pub async fn serve_index(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    port: Option<Extension<Arc<dyn DataPort>>>,
) -> Response {
    if is_provisioned(&port) {
        serve_shell(mode, cfg).await
    } else {
        into_axum(deck_core::redirect("/setup", mode_from_ext(mode)))
    }
}

/// `GET /setup` — serve the setup placeholder on a fresh install, otherwise
/// redirect to `/` so the wizard can't wipe an existing install.
pub async fn serve_setup(
    mode: Option<Extension<DaemonMode>>,
    port: Option<Extension<Arc<dyn DataPort>>>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    if is_provisioned(&port) {
        into_axum(deck_core::redirect("/", mode_val))
    } else {
        into_axum(deck_core::serve_setup_placeholder(mode_val))
    }
}

// ── SPA / public-asset dispatch ─────────────────────────────────────────────────

/// `GET /{file}` — a single-segment path. A dotted name (`favicon.ico`,
/// `ship-logo.svg`) is a real public asset, served from the host's `public_dir`
/// (404 if absent). A bare segment (`/git`, `/agents`, `/settings`) is a
/// client-side route, so serve the shell scaffold and let client-side routing
/// resolve it.
pub async fn spa_or_public_asset(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(file): Path<String>,
) -> Response {
    if file.contains('.') {
        return crate::handlers::serve_public_asset(mode, cfg, Path(file)).await;
    }
    serve_shell(mode, cfg).await
}

/// Router-level fallback for unmatched requests. A GET navigation to a
/// multi-segment client route (`/studio/themes`, …) has no server route, so
/// serve the shell scaffold's index.html and let client-side routing take over.
/// Every other unmatched request keeps an honest 404 — the fallback never
/// shadows a real backend path or static asset.
pub async fn spa_fallback(
    method: Method,
    uri: Uri,
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
) -> Response {
    let path = uri.path();
    let is_backend_or_asset = path.starts_with("/api")
        || path.starts_with("/_deck")
        || path.starts_with("/hooks")
        || path.starts_with("/auth")
        || path.starts_with("/daemon")
        || path.starts_with("/space")
        || path.starts_with("/shell")
        // Connect-RPC paths are `/ship.ops.v1.*` and assets carry an extension —
        // both contain a `.`, so this single check covers them.
        || path.contains('.');
    if method == Method::GET && !is_backend_or_asset {
        return serve_shell(mode, cfg).await;
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}
