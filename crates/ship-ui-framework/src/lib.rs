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
//! - **HMR** (`GET /_sui/hmr`, `notify`-based file watcher) — file edits
//!   in `.ship/views/` push `view-updated` events to mounted browsers.
//!   (Full HMR ships in U1; the route is registered now so the contract
//!   surface is stable.)
//!
//! ## Daemon integration
//!
//! ```rust,ignore
//! use ship_ui_framework as sui;
//!
//! let app = axum::Router::new()
//!     .merge(sui::router(sui::SuiState::new(sui::SuiConfig {
//!         project_dir: project_dir_resolver,
//!     })))
//!     .merge(rest_api::router())
//!     .merge(static_assets::router());
//! ```
//!
//! The daemon supplies a closure that resolves the current project dir
//! (where `.ship/views/` lives). Everything else stays internal.
//!
//! ## What's NOT here
//!
//! Daemon-specific things stay in `apps/shipd/`:
//!
//! - REST API (`/api/*`)
//! - Auth/session
//! - Lit asset serving (`ship-elements.js`, `datastar.js`)
//! - Workspace/agent/job lifecycle
//!
//! This crate is the *framework* layer; the daemon is the *product* it
//! plugs into. Both ship together as one ship binary.

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
///
/// One instance per daemon. Holds the SSE channel registry, the data-
/// resolver cache, and (post-U1) the file-watcher handle. The daemon
/// constructs once at startup and passes to [`router`].
#[derive(Clone)]
pub struct SuiState {
    pub config: SuiConfig,
    // Internals filled in by D1.3 as code moves over from apps/shipd.
}

impl SuiState {
    pub fn new(config: SuiConfig) -> Self {
        Self { config }
    }
}

/// Build the axum router that owns every `/_sui/*`, `/sui/*`, and
/// `/view-host/*` route. Daemon mounts via [`axum::Router::merge`].
pub fn router(_state: SuiState) -> axum::Router {
    // D1.3 wires the actual handlers as code moves in. For now this
    // crate compiles as a no-op placeholder so the workspace member
    // is present and apps/shipd's Cargo can switch its dependency.
    axum::Router::new()
}
