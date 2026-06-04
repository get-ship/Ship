//! deck — thin re-export facade over deck-axum + deck-core + deck-host.
//!
//! Hosting logic lives in `deck-core` (transport-agnostic, returns
//! `DeckResponse`); `deck-axum` is the axum adapter (`serve_*` handlers +
//! `router`). This crate re-exports both so shipd's imports (`deck::*`)
//! continue to resolve without changes.

// axum handlers + adapter surface.
pub use deck_axum::{
    into_axum, mode_from_ext, router, serve_bundle_hmr, serve_catalog_workspace,
    serve_shell, serve_shell_asset, serve_space, serve_space_asset,
    serve_meta, serve_sui_components,
    serve_sui_dom, serve_sui_icon, serve_sui_live_reload,
    serve_sui_runtime, serve_sui_sdk_client,
    serve_sui_view_setup, serve_view_host, serve_view_payload,
    serve_view_payload_subpath,
};

// Contract types (transport-agnostic), surfaced for hosts that name them.
pub use deck_core::{
    CachePolicy, DaemonBuildInfo, DaemonMode, DeckResponse, ViewSetupQuery, inject_mode_meta,
    mode_str,
};

// Submodule surfaces consumers reach into directly (shipd, xtask SDK gen).
//
// `render_helpers` lives in deck-axum (its `not_found` returns an axum
// response); `resolver` + `view_payload` are pure and live in deck-core.
pub use deck_axum::render_helpers;
pub use deck_core::{resolver, view_payload};

// The bundler — deck owns bundling (#167). Consumers that named
// `ship_platform::bundler::*` now use `deck::bundler::*`.
pub use deck_bundler as bundler;

// deck.jsonc config — the SSOT for consumer bundle targets.
pub use deck_config as config;
