//! ship-sui — thin re-export facade over ship-sui-server + ship-sui-host.
//!
//! All logic lives in `ship-sui-host` (pure framework) and `ship-sui-server` (axum
//! handlers). This crate exists so shipd's imports (`ship_sui::*`)
//! continue to resolve without changes.

pub mod assets;
pub mod bundle;
pub mod render_helpers;
pub mod resolver;
pub mod view;
pub mod view_host;
pub mod view_payload;

pub use ship_sui_server::assets::{
    serve_sui_components, serve_sui_dom, serve_sui_island, serve_sui_lucide_icons,
    serve_sui_runtime, serve_sui_sdk_client,
};
pub use ship_sui_server::view::{
    serve_sui_view_css, serve_sui_view_setup, serve_sui_view_template, ViewSetupQuery,
};
pub use ship_sui_server::view_host::serve_view_host;
pub use ship_sui_server::view_payload::{serve_view_payload, serve_view_payload_subpath};
pub use ship_sui_server::DaemonMode;
pub use ship_sui_server::meta::{serve_meta, DaemonBuildInfo};
