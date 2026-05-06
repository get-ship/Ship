//! ship-ui-framework — server-side hosting for SUI views.
//!
//! Owns every framework concern that needs to live on the server side:
//!
//! - **View payload endpoint** (`GET /_sui/views/<name>[/<sub>]`) —
//!   compile `.sui` on demand, return JSON the runtime's `mount()`
//!   consumes. Single data contract.
//! - **Asset bundles** (`GET /_sui/{runtime,dom,sdk-client,components}.js`)
//!   — embedded JS modules built by the npm bundle pipeline.
//! - **View-host static page** (`GET /sui/<name>[/<sub>]`,
//!   `GET /view-host/...`) — one HTML file that mounts any view by URL.
//! - **Per-view SSE channel** (`GET /_sui/views/<name>/sse`) — live data
//!   patches, agent-pushed DOM patches.
//! - **Mutation endpoint** (`POST /_sui/mutation/<name>`) — `<form
//!   mutation>` server-truth pattern.
//! - **HMR** (`GET /_sui/hmr`) — file edits in `.ship/views/` push
//!   `view-updated` events to mounted browsers (U1 lights this up).
//!
//! ## Daemon integration
//!
//! ```rust,ignore
//! use ship_ui_framework as sui;
//!
//! let app = axum::Router::new()
//!     .merge(sui::router(state))
//!     .merge(rest_api::router())
//!     .merge(static_assets::router());
//! ```
//!
//! The daemon supplies a project-dir resolver via [`SuiConfig`]; the
//! rest of the framework stays opaque to it.

pub mod bundle;
pub mod render_helpers;
pub mod resolver;
pub mod runtime;
pub mod view;
pub mod view_host;
pub mod view_payload;

#[cfg(test)]
pub(crate) mod test_lock {
    //! One process-wide mutex shared by every test that mutates the
    //! `SHIP_DEV_PROJECT_DIR` env var. Both the resolver and the
    //! view_payload suite did this with their OWN mutex; under
    //! `cargo test`'s parallel runner they raced and one suite saw
    //! the other's tempdir, surfacing as `view not found: cachey`
    //! (or similar) flakes. Funnel both through a single static.
    use std::sync::Mutex;

    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    pub fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }
}

// Re-exports the daemon (and any other axum host) consumes to register
// SUI's HTTP routes.
pub use runtime::serve_sui_runtime;
pub use view::{
    serve_sui_components, serve_sui_dom, serve_sui_island, serve_sui_lucide_icons,
    serve_sui_sdk_client, serve_sui_view_css, serve_sui_view_setup,
    serve_sui_view_template, ViewSetupQuery,
};
pub use view_host::serve_view_host;
pub use view_payload::{serve_view_payload, serve_view_payload_subpath};

use std::path::PathBuf;
use std::sync::Arc;

/// Configuration the daemon supplies. Everything SUI-specific lives in
/// this crate; the daemon only injects what's external (project dir
/// lookup tied to ship's per-worktree state).
#[derive(Clone)]
pub struct SuiConfig {
    /// Resolves the current worktree's project dir — the parent of
    /// `.ship/views/`. Returns `None` when called outside a worktree.
    pub project_dir: Arc<dyn Fn() -> Option<PathBuf> + Send + Sync>,
}

/// Shared state for the SUI framework's HTTP routes.
#[derive(Clone)]
pub struct SuiState {
    pub config: SuiConfig,
}

impl SuiState {
    pub fn new(config: SuiConfig) -> Self {
        Self { config }
    }
}

/// Build the axum router that owns every `/_sui/*`, `/sui/*`, and
/// `/view-host/*` route. Daemon mounts via [`axum::Router::merge`].
///
/// D1.4 wires the daemon caller. The handlers themselves still expect
/// `axum::extract::State<runtime::ApiState>` (not `SuiState`); the
/// daemon supplies that via `with_state`. SuiState exists for
/// framework-internal concerns (file watcher, etc.) that arrive in U1.
pub fn router(_state: SuiState) -> axum::Router {
    // Stub — the daemon's existing router still owns the route table
    // until D1.4 moves it here. This crate currently exists to be the
    // compile-time home of the moved Rust modules.
    axum::Router::new()
}
