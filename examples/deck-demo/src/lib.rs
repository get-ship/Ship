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
//!
//! Decoupling gaps surfaced (see README): env config is still process-global
//! (now `DECK_*`-named with a `SHIP_*` legacy fallback — Fix C);
//! `Bundler::start` needs `ship_substrate::ShipHome`; `router()` carries no
//! bundler and no HMR (both host-wired); the bundler out-dir must equal the
//! resolver's project root.

use std::path::{Path, PathBuf};

use axum::routing::get;
use axum::{Extension, Router};
use deck::bundler::Bundler;
use deck::{DaemonMode, router, serve_bundle_hmr};

/// This crate's example app: `app/views/<name>/page.sui`.
pub fn default_app_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app")
}

/// Construct the bundler for `app_dir`, pointing deck's resolver at it.
fn make_bundler(app_dir: &Path) -> Bundler {
    // DECK_PROJECT_DIR is deck's canonical project-root env var; deck-core also
    // accepts the legacy SHIP_DEV_PROJECT_DIR as a fallback.
    // SAFETY: set once during single-threaded setup, before any request.
    unsafe {
        std::env::set_var("DECK_PROJECT_DIR", app_dir);
    }
    let home = ship_substrate::ShipHome::from_env();
    // repo_root == app_dir so bundle output (`<root>/.deck`) lands where the
    // resolver reads it (`SHIP_DEV_PROJECT_DIR/.deck`).
    Bundler::start(&home, app_dir.to_path_buf(), None)
}

/// deck's hosting surface + the two extensions its handlers read + the HMR SSE
/// route. Shared by both entry points.
fn build_router(bundler: Bundler) -> Router {
    router()
        // The canonical deck-axum HMR handler — no hand-rolled variant here.
        .route("/_sui/dev/hmr", get(serve_bundle_hmr))
        .layer(Extension(DaemonMode::Development))
        .layer(Extension(bundler))
}

/// A ready router with NO filesystem watcher (read-only host / tests).
pub fn deck_app(app_dir: PathBuf) -> Router {
    build_router(make_bundler(&app_dir))
}

/// A dev server with the FS watcher running: edit a `.sui` → rebuild the
/// affected target → push an HMR event. Async because `start_watching` spawns
/// the watch task.
pub async fn deck_dev_server(app_dir: PathBuf) -> Router {
    let bundler = make_bundler(&app_dir);
    if let Err(e) = bundler.start_watching().await {
        eprintln!("deck-demo: watcher failed to start: {e}");
    }
    build_router(bundler)
}
