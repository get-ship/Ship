//! Serves the bundled `@ship/ui-runtime` ESM module.
//!
//! `.sui` views and components import from `@ship/ui-runtime`; the
//! page wrapper inlines an importmap that resolves the bare specifier to
//! `/_sui/runtime.js`, served here.
//!
//! View serving lives in `ship_set_view`. SDK data lives in `ship_set_data`.
//! HMR lives in `ship_set_hmr`. This file is just the runtime asset.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

/// Bundled `@ship/ui-runtime` ESM. Built by `pnpm -F @ship/ui-runtime bundle`
/// (esbuild). Embedded at compile time so the daemon serves a self-contained
/// runtime without depending on a build artifact path at runtime.
const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");

/// `GET /_sui/runtime.js` — serves the bundled `@ship/ui-runtime` module.
pub async fn serve_sui_runtime() -> Response {
    let (body, cache_control) = match crate::dev_assets::override_for("sui-runtime").await {
        Some(bytes) => (Body::from(bytes), "no-cache"),
        None => (Body::from(SUI_RUNTIME_JS), "public, max-age=3600, immutable"),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, cache_control)
        .body(body)
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}
