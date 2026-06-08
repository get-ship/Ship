//! deck-demo — mounting deck on axum, in one place.
//!
//! Two entry points share one definition of "how to mount deck":
//! - [`deck_app`] — a ready router, NO filesystem watcher (read-only host /
//!   tests). On-demand bundling still works; nothing watches for edits.
//! - [`deck_dev_server`] — the same, plus the bundler's FS watcher running, so
//!   editing a `.sui` rebuilds the affected target and pushes an HMR event.
//!
//! The host contract:
//! 1. Point the resolver at an app dir via `DECK_PROJECT_DIR` (leave
//!    `DECK_FIRST_PARTY_VIEWS` unset — non-ship host, no first-party bucket).
//!    The resolver also reads built bundles from `<app_dir>/.deck/`.
//! 2. Mount `deck::router()`.
//! 3. Layer the extensions deck's handlers read — `DaemonMode` and a
//!    `Bundler` (so a controller view gets its Rolldown bundle on demand).
//! 4. Add the HMR SSE route the view-host's live-reload client subscribes to.
//!    `deck::router()` does NOT provide `/_sui/dev/hmr` — it's host-wired (the
//!    e2e's 404). We bridge the bundler's event firehose to it directly.
//! 5. Mount the data port (`data_port::router`) for live-data + mutation
//!    endpoints backed by the in-memory [`DemoStore`].
//!
//! Decoupling gaps surfaced (see README): env config is still process-global
//! (now `DECK_*`-named with a `SHIP_*` legacy fallback — Fix C);
//! `Bundler::start` needs `ship_substrate::ShipHome`; `router()` carries no
//! bundler and no HMR (both host-wired); the bundler out-dir must equal the
//! resolver's project root.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::get;
use axum::{Extension, Router};
use deck::bundler::Bundler;
use deck::{DaemonMode, router, serve_bundle_hmr};

pub mod data_port;
pub use data_port::DemoStore;

/// Resolve the daemon mode from the `DECK_DEMO_MODE` environment variable.
/// `"prod"` or `"production"` → [`DaemonMode::Production`]; anything else
/// (including absent) → [`DaemonMode::Development`].
pub fn demo_mode() -> DaemonMode {
    match std::env::var("DECK_DEMO_MODE").as_deref() {
        Ok("prod") | Ok("production") => DaemonMode::Production,
        _ => DaemonMode::Development,
    }
}

/// This crate's example app: `app/views/<name>/page.sui`.
pub fn default_app_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app")
}

/// Construct the bundler for `app_dir`, pointing deck's resolver at it.
fn make_bundler(app_dir: &Path) -> Bundler {
    // Authoritative project-root override for deck-core's resolver. Set
    // ONCE at startup — deck serves one root per process, so a single
    // global is sound (see `deck_core::resolver::set_project_root` doc).
    // repo_root == app_dir so bundle output (`<root>/.deck`) lands where
    // the resolver reads it (`<root>/.deck`). Zero-config host: no
    // deck.jsonc — use defaults (no consumer targets, .deck output root).
    deck::resolver::set_project_root(app_dir.to_path_buf());
    let home = ship_substrate::ShipHome::from_env();
    let cfg = deck::config::DeckConfig::load(app_dir);
    Bundler::start(&home, app_dir.to_path_buf(), None, cfg)
}

/// Design tokens + all six themes, concatenated — matches what shipd serves at
/// /api/assets/view-tokens.css so view-host.html's existing `<link>` resolves.
///
/// The view-host requests this URL for every SUI view it mounts. Deck-axum has
/// no `/api/assets/*` handler (tokens are a Ship concern there), so we wire it
/// directly in the deck-demo host rather than patching deck-core.
const VIEW_TOKENS_CSS: &str = concat!(
    include_str!("../../../packages/sui/tokens/src/tokens.css"),
    "\n",
    include_str!("../../../packages/sui/tokens/src/themes.css"),
);

async fn serve_view_tokens() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(VIEW_TOKENS_CSS))
        .unwrap()
}

/// deck's hosting surface + the two extensions its handlers read + the HMR SSE
/// route + the in-memory data port. Shared by both entry points.
fn build_router(
    mode: DaemonMode,
    bundler: Bundler,
    cfg: Arc<deck::config::DeckConfig>,
    store: Arc<DemoStore>,
) -> Router {
    router()
        // Tokens + themes — view-host.html requests /api/assets/view-tokens.css.
        // Deck has no /api/assets/* route; the deck-demo host wires it directly.
        .route("/api/assets/view-tokens.css", get(serve_view_tokens))
        // The canonical deck-axum HMR handler — no hand-rolled variant here.
        .route("/_sui/dev/hmr", get(serve_bundle_hmr))
        // Data port: live-data SSE + mutation + data endpoints (ship-free,
        // in-memory, mirrors shipd handlers with the kernel removed).
        .merge(data_port::router(store))
        .layer(Extension(mode))
        // DeckConfig Extension — scaffold handlers read scaffold declarations
        // and inject_globals from this. Zero-config hosts (no deck.jsonc) get
        // built-in defaults and degrade gracefully.
        .layer(Extension(cfg))
        .layer(Extension(bundler))
}

/// A ready router with NO filesystem watcher (read-only host / tests).
/// Reads the daemon mode from [`demo_mode`] (`DECK_DEMO_MODE` env var).
pub fn deck_app(app_dir: PathBuf) -> Router {
    deck_app_with_mode(app_dir, demo_mode())
}

/// Like [`deck_app`] but with an explicit mode — for tests that need to
/// verify production vs development behavior without reaching into the env.
pub fn deck_app_with_mode(app_dir: PathBuf, mode: DaemonMode) -> Router {
    let cfg = Arc::new(deck::config::DeckConfig::load(&app_dir));
    let store = DemoStore::new();
    build_router(mode, make_bundler(&app_dir), cfg, store)
}

/// A dev server with the FS watcher running: edit a `.sui` → rebuild the
/// affected target → push an HMR event. Async because `start_watching` spawns
/// the watch task. Reads the daemon mode from [`demo_mode`].
pub async fn deck_dev_server(app_dir: PathBuf) -> Router {
    let mode = demo_mode();
    let cfg = Arc::new(deck::config::DeckConfig::load(&app_dir));
    let bundler = make_bundler(&app_dir);
    if let Err(e) = bundler.start_watching().await {
        eprintln!("deck-demo: watcher failed to start: {e}");
    }
    let store = DemoStore::new();
    build_router(mode, bundler, cfg, store)
}
