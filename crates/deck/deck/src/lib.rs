//! deck — thin re-export facade over deck-server + deck-host.
//!
//! All logic lives in `deck-host` (pure framework) and `deck-server` (axum
//! handlers). This crate exists so shipd's imports (`deck::*`)
//! continue to resolve without changes.

pub mod assets;
pub mod bundle;
pub mod packs;
pub mod render_helpers;
pub mod resolver;
pub mod view;
pub mod view_host;
pub mod view_payload;

pub use deck_server::assets::{
    serve_sui_components, serve_sui_dom, serve_sui_island, serve_sui_lucide_icons,
    serve_sui_runtime, serve_sui_sdk_client,
};
pub use deck_server::packs::{
    serve_catalog_published, serve_catalog_workspace, serve_pack_asset,
};
pub use deck_server::view::{
    serve_sui_view_css, serve_sui_view_setup, serve_sui_view_template, ViewSetupQuery,
};
pub use deck_server::view_host::serve_view_host;
pub use deck_server::view_payload::{serve_view_payload, serve_view_payload_subpath};
pub use deck_server::DaemonMode;
pub use deck_server::meta::{serve_meta, DaemonBuildInfo};
