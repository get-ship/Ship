use axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};

use crate::{DaemonMode, asset_cache, compute_etag, mode_from_ext};

// Per-tag island bundles are served by file existence under
// `assets/sui-components/`. The bundler (packages/deck/bundler/index.ts +
// packages/deck/build via the umbrella's sui.config.ts `lazyOutSubdir`)
// is the source of truth for what lands there. The previous
// hand-maintained `is_known_island` allowlist was retired alongside
// the workspace-catalog cut-over — origin metadata in
// `.ship/sui-out/catalogs/workspace.json` makes per-tag gating
// redundant. Path-traversal is rejected explicitly in
// `serve_sui_island` so a missing-bundle 404 is the only failure mode.

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// deck's OWN baked product (sui-runtime/dom/sdk-client + primitives umbrella +
/// lucide). These ship inside the deck-server crate, like Next shipping its own
/// runtime. App + user content is NOT here — see `read_deck_asset`.
fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

/// App + user build output, read from the host-supplied `.deck/` tree (deck's
/// `.next`-equivalent). deck bakes none of this — view + chrome bundles are the
/// project's content, served from where the project built them. Returns None
/// when there is no project root (the on-demand compile path then takes over).
fn read_deck_asset(rel: &str) -> Option<String> {
    let root = crate::resolver::deck_out_root()?;
    std::fs::read_to_string(root.join(rel)).ok()
}

const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sui-sdk-client.js");
const SUI_COMPONENTS_JS: &str = include_str!("../assets/sui-components.js");
const SUI_LUCIDE_ICONS_JSON: &str = include_str!("../assets/lucide-icons.json");

// Public helper (used by view.rs). deck bakes NO app views — per-view bundles
// live in the project's `.deck/views/<name>/view.js`; absent that, view.rs
// compiles on demand from source.

pub fn resolve_view_js(name: &str) -> Option<String> {
    read_deck_asset(&format!("views/{name}/view.js"))
}

// Framework bundle handlers

pub async fn serve_sui_runtime(mode: Option<Extension<DaemonMode>>) -> Response {
    let mode = mode_from_ext(mode);
    match read_asset("sui-runtime.js") {
        Some(js) => js_response_mode(js, mode),
        None => js_response_static_mode(SUI_RUNTIME_JS, mode),
    }
}

pub async fn serve_sui_dom(mode: Option<Extension<DaemonMode>>) -> Response {
    let mode = mode_from_ext(mode);
    match read_asset("sui-dom.js") {
        Some(js) => js_response_mode(js, mode),
        None => js_response_static_mode(SUI_DOM_JS, mode),
    }
}

pub async fn serve_sui_sdk_client(mode: Option<Extension<DaemonMode>>) -> Response {
    let mode = mode_from_ext(mode);
    match read_asset("sui-sdk-client.js") {
        Some(js) => js_response_mode(js, mode),
        None => js_response_static_mode(SUI_SDK_CLIENT_JS, mode),
    }
}

pub async fn serve_sui_components(mode: Option<Extension<DaemonMode>>) -> Response {
    let mode = mode_from_ext(mode);
    match read_asset("sui-components.js") {
        Some(js) => js_response_mode(js, mode),
        None => js_response_static_mode(SUI_COMPONENTS_JS, mode),
    }
}

pub async fn serve_sui_lucide_icons(mode: Option<Extension<DaemonMode>>) -> Response {
    let mode = mode_from_ext(mode);
    let (body, is_dynamic) = match read_asset("lucide-icons.json") {
        Some(json) => (json, true),
        None => (SUI_LUCIDE_ICONS_JSON.to_string(), false),
    };
    let _ = is_dynamic; // same cache logic in both branches
    let cache = asset_cache(mode);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, cache);
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(body.as_bytes()));
    }
    builder.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

pub async fn serve_sui_island(mode: Option<Extension<DaemonMode>>, Path(file): Path<String>) -> Response {
    let mode = mode_from_ext(mode);
    // Reject path-traversal explicitly — the file segment is captured
    // verbatim from the URL, so a malicious `../../etc/passwd` would
    // otherwise escape the assets dir.
    if file.contains("..") || file.contains('/') {
        return not_found("invalid island name");
    }
    // The route captures the whole filename (`sui-rail.js`) because axum 0.8
    // forbids mixing path-params with literal suffixes in the same segment.
    // Strip the `.js` to recover the bundle name.
    let bundle = file.strip_suffix(".js").unwrap_or(&file).to_string();
    // Single canonical path: the project's `.deck/sui-components/<bundle>.js`.
    // Code-split component + chrome bundles are app/user content, so they live
    // in the project's deck build output — never baked into deck's own crate.
    // A missing bundle is a 404 with an actionable hint (no baked fallback).
    let rel = format!("sui-components/{bundle}.js");
    match read_deck_asset(&rel) {
        Some(js) => js_response_mode(js, mode),
        None => not_found("island bundle missing — run `bun run packages/deck/bundler/index.ts --all`"),
    }
}

// Response helpers

fn js_response_mode(body: String, mode: DaemonMode) -> Response {
    let cache = asset_cache(mode);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, cache);
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(body.as_bytes()));
    }
    builder.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn js_response_static_mode(body: &'static str, mode: DaemonMode) -> Response {
    js_response_mode(body.to_string(), mode)
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn runtime_js_returns_baked_fallback() {
        let resp = serve_sui_runtime(None).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_missing_returns_404() {
        // Islands are app/user content read from the project's `.deck/`; deck
        // bakes no island fallback (chrome is the shell app's, not the
        // framework's). A name with no `.deck/sui-components/<name>.js` 404s
        // with the bundler hint — there is no embedded copy to mask it.
        let resp = serve_sui_island(None, Path("definitely-not-a-real-bundle".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_path_traversal_rejected() {
        let resp = serve_sui_island(None, Path("../etc/passwd".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn lucide_icons_returns_baked_fallback() {
        let resp = serve_sui_lucide_icons(None).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
