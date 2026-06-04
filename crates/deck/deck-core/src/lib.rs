//! deck-core — transport-agnostic SUI hosting logic.
//!
//! This crate owns *what* deck serves (compiled views, bundles, the catalog)
//! and produces a [`DeckResponse`] for each. It never touches axum or any
//! HTTP framework. Server adapters (`deck-axum`, a future `deck-bun`)
//! translate `DeckResponse` onto their own response type — the
//! ports-and-adapters seam.

pub mod response;

pub mod asset_graph;
pub mod assets;
pub mod error_page;
pub mod live_wire;
pub mod meta;
pub mod mode_html;
pub mod packs;
pub mod render_helpers;
pub mod resolver;
pub mod routes;
pub mod view;
pub mod view_host;
pub mod view_payload;

#[cfg(test)]
mod view_payload_tests;
#[cfg(test)]
mod view_payload_shape_tests;
#[cfg(test)]
mod mode_tests;

#[cfg(test)]
pub(crate) mod test_lock {
    use std::sync::Mutex;
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());
    pub fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub use response::{CachePolicy, DeckResponse, compute_etag};

pub use live_wire::{
    PatchElements, PatchElementsMode,
    format_keepalive, format_patch_elements, format_patch_set, format_patch_signals,
};

pub use assets::{
    default_shell_scaffold, default_space_scaffold,
    resolve_view_js, serve_scaffold, serve_scaffold_asset,
    serve_shell_asset, serve_space_asset,
    serve_sui_chunk, serve_sui_components, serve_sui_dom, serve_sui_icon,
    serve_sui_live_reload, serve_sui_runtime, serve_sui_sdk_client,
};
pub use error_page::view_not_found;
pub use meta::{serve_meta, DaemonBuildInfo};
pub use mode_html::{inject_mode_meta, mode_str};
pub use packs::serve_catalog_workspace;
pub use view::{serve_sui_view_setup, ViewSetupQuery};
pub use view_host::serve_view_host;
pub use view_payload::{serve_view_payload, serve_view_payload_subpath};

/// Daemon operating mode — controls cache headers, HMR availability, SSR cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaemonMode {
    Development,
    Production,
}

impl DaemonMode {
    pub fn parse(s: &str) -> Self {
        match s {
            "production" => Self::Production,
            _ => Self::Development,
        }
    }
}
