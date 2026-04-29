//! Serves the bundled `@ship-set/runtime` ESM module.
//!
//! `.html.set` views and components import from `@ship-set/runtime`; the
//! page wrapper inlines an importmap that resolves the bare specifier to
//! `/_ship-set/runtime.js`, served here.
//!
//! View serving lives in `ship_set_view`. SDK data lives in `ship_set_data`.
//! HMR lives in `ship_set_hmr`. This file is just the runtime asset.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

/// Bundled `@ship-set/runtime` ESM. Built by `pnpm -F @ship-set/runtime bundle`
/// (esbuild). Embedded at compile time so the daemon serves a self-contained
/// runtime without depending on a build artifact path at runtime.
const SHIP_SET_RUNTIME_JS: &str = include_str!("../../assets/ship-set-runtime.js");

/// `GET /_ship-set/runtime.js` — serves the bundled `@ship-set/runtime` module.
pub async fn serve_ship_set_runtime() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(Body::from(SHIP_SET_RUNTIME_JS))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}
