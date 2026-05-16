use axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};

use crate::{DaemonMode, asset_cache, compute_etag, mode_from_ext};

/// Islands that the daemon serves a bundle for. Mirrors the bundles we
/// `include_str!` below; kept local to `serve_sui_island` so adding a
/// new bundle is a one-file change. Distinct from `sui_compiler`'s
/// `SUI_ISLANDS` table, which is consumed by the compiler to *preload*
/// bundles a view references — the route still has to serve `<sui-X>`
/// tags that show up unannounced at runtime.
fn is_known_island(name: &str) -> bool {
    matches!(
        name,
        "sui-terminal"
            | "sui-command-bar"
            | "sui-devtools"
            | "sui-launcher"
            | "sui-panel-dock"
            | "sui-nav-top"
            | "sui-rail"
            | "sui-code-editor"
            | "sui-markdown-editor"
            // View-authoring islands — both shipped as lazy bundles by
            // the asset pipeline (sui-view-authoring rolls up source /
            // components / activity / compile / annotation-pins; the
            // picker is the catalog primitive used in the Components
            // tab). The chrome edit-mode controller lazy-loads them on
            // first toggle into edit mode.
            | "sui-view-authoring"
            | "sui-component-picker"
    )
}

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

// Baked-in fallbacks (production / no-workspace).
//
// Canonical asset layout under `crates/sui/ship-sui-server/assets/`:
//   * top-level (`assets/<name>.js`) — always-loaded framework bundles
//     (sui-runtime, sui-dom, sui-sdk-client, sui-components) and
//     out-of-band assets (lucide-icons.json, sui-custom-elements.json).
//   * `assets/sui-components/<name>.js` — every code-split component bundle
//     (opted in via `<bundle separate />` in the component's .sui source).
//     Both writers (runtime-assets/bundler.ts and packages/sui-build via
//     ui-components/sui.config.ts `lazyOutSubdir`) emit here. Adding a
//     second location is forbidden; see job/bundle-output-canonical.
//   * `assets/views/<name>/view.js` — per-view bundles.

const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sui-sdk-client.js");
const SUI_COMPONENTS_JS: &str = include_str!("../assets/sui-components.js");
const SUI_LUCIDE_ICONS_JSON: &str = include_str!("../assets/lucide-icons.json");
const SUI_TERMINAL_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-terminal.js");
const SUI_COMMAND_BAR_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-command-bar.js");
const SUI_DEVTOOLS_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-devtools.js");
const SUI_LAUNCHER_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-launcher.js");
const SUI_PANEL_DOCK_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-panel-dock.js");
const SUI_NAV_TOP_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-nav-top.js");
const SUI_RAIL_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-rail.js");
const SUI_CODE_EDITOR_ISLAND_JS: &str = include_str!("../assets/sui-components/sui-code-editor.js");
const SUI_MARKDOWN_EDITOR_ISLAND_JS: &str =
    include_str!("../assets/sui-components/sui-markdown-editor.js");

// Baked view JS + CSS for first-party views (production fallback).

const AGENTS_VIEW_CSS: &str = include_str!("../../../../views/agents/style.css");
const AGENTS_VIEW_JS: &str = include_str!("../assets/views/agents/view.js");
const GIT_VIEW_CSS: &str = include_str!("../../../../views/git/style.css");
const GIT_VIEW_JS: &str = include_str!("../assets/views/git/view.js");

// Public helpers (used by view.rs)

pub fn resolve_view_js(name: &str) -> Option<String> {
    read_asset(&format!("views/{name}/view.js"))
}

pub fn resolve_view_js_baked(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_JS),
        "git" => Some(GIT_VIEW_JS),
        _ => None,
    }
}

pub fn resolve_view_css_baked(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_CSS),
        "git" => Some(GIT_VIEW_CSS),
        _ => None,
    }
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
    // The route captures the whole filename (`sui-rail.js`) because axum 0.8
    // forbids mixing path-params with literal suffixes in the same segment.
    // Strip the `.js` to recover the bundle name.
    let bundle = file.strip_suffix(".js").unwrap_or(&file).to_string();
    if !is_known_island(&bundle) {
        return not_found("unknown island");
    }
    // Single canonical path: `assets/sui-components/<bundle>.js`. Both
    // writers (runtime-assets bundler + packages/sui-build) target it.
    // Try the on-disk bundle first — that's the live-rebuilt artifact in
    // development and the canonical install layout in production.
    let rel = format!("sui-components/{bundle}.js");
    if let Some(js) = read_asset(&rel) {
        return js_response_mode(js, mode);
    }
    // Fall back to the baked-in copies for islands that have an
    // `include_str!` constant. Islands added to the allowlist after the
    // initial bake-in (e.g. sui-view-authoring, sui-component-picker)
    // rely on the on-disk path above and reach this arm only if the
    // bundler hasn't emitted the file yet — surface a 404 with a hint
    // rather than silently masking.
    let embedded: Option<&'static str> = match bundle.as_str() {
        "sui-terminal" => Some(SUI_TERMINAL_ISLAND_JS),
        "sui-command-bar" => Some(SUI_COMMAND_BAR_ISLAND_JS),
        "sui-devtools" => Some(SUI_DEVTOOLS_ISLAND_JS),
        "sui-launcher" => Some(SUI_LAUNCHER_ISLAND_JS),
        "sui-panel-dock" => Some(SUI_PANEL_DOCK_ISLAND_JS),
        "sui-nav-top" => Some(SUI_NAV_TOP_ISLAND_JS),
        "sui-rail" => Some(SUI_RAIL_ISLAND_JS),
        "sui-code-editor" => Some(SUI_CODE_EDITOR_ISLAND_JS),
        "sui-markdown-editor" => Some(SUI_MARKDOWN_EDITOR_ISLAND_JS),
        _ => None,
    };
    match embedded {
        Some(js) => js_response_mode(js.to_string(), mode),
        None => not_found("island bundle missing — run `bun run runtime-assets/bundler.ts --all`"),
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
    async fn island_terminal_returns_js_bundle() {
        let resp = serve_sui_island(None, Path("sui-terminal".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("application/javascript"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(bytes.len() > 1000, "island bundle unexpectedly small: {} bytes", bytes.len());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_unknown_returns_404() {
        let resp = serve_sui_island(None, Path("sui-unknown".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn lucide_icons_returns_baked_fallback() {
        let resp = serve_sui_lucide_icons(None).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
