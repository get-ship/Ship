//! Per-app serving — the host-driven replacement for the old "scaffold by name"
//! model. The previous API read a named scaffold from
//! `Extension<Arc<DeckConfig>>::scaffolds[]` keyed on a hardcoded name
//! (`"shell"`, `"space"`). The new per-app API takes a list of
//! [`AppServeConfig`] (host-injected) and serves each at the route the app
//! declared. The framework is per-app; the host owns the list.
//!
//! ## Router shape
//!
//! For each app, [`serve_apps`] mounts three routes:
//!
//! - `GET <app.route>` → [`serve_app_index`]
//!   Serve the app's `index.html` with `inject_globals` substituted from
//!   the URL path captures. Path captures are extracted as a
//!   `HashMap<String, String>` so any `{name}` segment in the route is
//!   available for substitution.
//! - `GET <app.asset_prefix>{file}` → [`serve_app_asset`]
//!   Serve a per-app static asset (`<app.source>/assets/{file}`). The
//!   asset prefix is the collision-avoidance path the bundler can't do
//!   on its own (each app's `index.html` references `/assets/...`; the
//!   host re-roots those under a unique prefix per app).
//! - `GET /{file}` (the app's slice) → [`serve_app_public`]
//!   Serve a per-app public asset. The bundler copies `<project>/public/`
//!   to `<project>/<source>/` at build time, so single-segment root
//!   paths land here.
//!
//! All three handlers read their app config from
//! `Extension<Arc<AppServeConfig>>`. The host layers each app's routes
//! on top of the others via the axum router, so each handler sees only
//! the app that owns the route it serves.
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Extension,
    extract::Path,
    response::Response,
    routing::get,
    Router,
};

use deck_config::AppServeConfig;
use deck_core::DaemonMode;

use crate::{into_axum, mode_from_ext};

// ── Handlers ───────────────────────────────────────────────────────────────────

/// `GET <app.route>` — serve the app's `index.html` with `inject_globals`
/// substituted from the URL path captures.
///
/// The axum `Path` extractor is `HashMap<String, String>` because axum
/// supports it natively: every `{name}` segment in the route is captured
/// into the map. The framework's `serve_scaffold` substitutes `$<key>`
/// tokens in `inject_globals` values from the map's entries.
pub async fn serve_app_index(
    mode: Option<Extension<DaemonMode>>,
    app: Option<Extension<Arc<AppServeConfig>>>,
    Path(params): Path<HashMap<String, String>>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    let Some(app) = app.as_ref().map(|e| e.0.as_ref()) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no app configured for this route",
            mode_val,
        ));
    };
    let params_refs: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    into_axum(deck_core::serve_scaffold(mode_val, app, &params_refs))
}

/// `GET <app.asset_prefix>{file}` — serve a per-app static asset
/// (`<app.source>/assets/{file}`). The asset prefix is the per-app
/// re-root path the bundler emits in the dev's `index.html` (e.g. each
/// app's HTML references `/assets/...` and the host serves them under
/// `/<app_name>/_assets/...`).
pub async fn serve_app_asset(
    mode: Option<Extension<DaemonMode>>,
    app: Option<Extension<Arc<AppServeConfig>>>,
    Path(file): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    let Some(app) = app.as_ref().map(|e| e.0.as_ref()) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no app configured for this asset path",
            mode_val,
        ));
    };
    into_axum(deck_core::serve_scaffold_asset(mode_val, app, &file))
}

/// `GET /{file}` (the app's slice) — serve a per-app public asset.
/// The bundler copies `<project>/public/` to `<project>/<source>/` at
/// build time, so a `favicon.ico` lives at
/// `<project>/<source>/favicon.ico`. Reads from the per-app source dir
/// (NOT `<source>/assets/`, which is the per-app bundle asset path —
/// public files are siblings, not children, of `assets/`).
pub async fn serve_app_public(
    mode: Option<Extension<DaemonMode>>,
    app: Option<Extension<Arc<AppServeConfig>>>,
    Path(file): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode);
    let Some(app) = app.as_ref().map(|e| e.0.as_ref()) else {
        return into_axum(deck_core::DeckResponse::not_found(
            "no app configured for this public asset",
            mode_val,
        ));
    };
    into_axum(deck_core::serve_public_asset(mode_val, &app.source, &file))
}

// ── Router ─────────────────────────────────────────────────────────────────────

/// Build the per-app router for a single app. The host calls this once
/// per app and merges the resulting routers (or calls [`serve_apps`]
/// which does the loop).
///
/// Each app gets two routes:
/// - `GET <route>` (with `{name}` captures extracted as a HashMap)
/// - `GET <asset_prefix>{file}`
///
/// The per-app public-asset catch-all is NOT per-app here — the host's
/// existing `/{file}` catch-all routes to the FIRST app in its list
/// (the asset_prefix is the per-app namespace for `assets/`; public
/// files are app-specific so a single-segment root path needs a
/// per-app source lookup). The host wires `serve_app_public` directly
/// for each app at the `/{file}` route, layering them so the first
/// app's public wins.
pub fn serve_app(app: &AppServeConfig) -> Router {
    let app_ext = Arc::new(app.clone());
    Router::new()
        .route(&app.route, get(serve_app_index))
        .route(
            &format!(
                "/{}/{{file}}",
                app.asset_prefix.trim_end_matches('/').trim_start_matches('/')
            ),
            get(serve_app_asset),
        )
        .layer(Extension(app_ext))
}

/// Build the per-app router for a list of apps. The host calls this once
/// at startup; the resulting router merges into the host's root alongside
/// the Connect fallback + the deck framework surface (`/_deck/*`).
///
/// The order matters: the FIRST app in the list catches single-segment
/// root paths (`/{file}`) for public assets. Hosts that have multiple
/// apps with public assets should give each app a unique asset_prefix
/// (the bundler emits this automatically from the project's name) so
/// the per-app asset namespace is collision-free.
pub fn serve_apps(apps: &[AppServeConfig]) -> Router {
    let mut router = Router::new();
    for app in apps {
        router = router.merge(serve_app(app));
    }
    // Public-asset catch-all: the first app's `serve_app_public` handles
    // single-segment root paths. The Extension is per-app, so each app
    // would need its own route — but axum only allows ONE `/{file}` route
    // per router. Hosts with multiple apps use the asset_prefix to
    // namespace their public assets; the bare `/{file}` is reserved for
    // the primary app (the one at the root route).
    if let Some(primary) = apps.first() {
        let primary_ext = Arc::new(primary.clone());
        router = router
            .route(
                "/{file}",
                get(serve_app_public).layer(Extension(primary_ext)),
            );
    }
    router
}
