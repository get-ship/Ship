use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use sui_compiler::islands::is_island_bundle;

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

// Baked-in fallbacks (production / no-workspace)

const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sui-sdk-client.js");
const SUI_COMPONENTS_JS: &str = include_str!("../assets/sui-components.js");
const SUI_LUCIDE_ICONS_JSON: &str = include_str!("../assets/lucide-icons.json");
const SUI_TERMINAL_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-terminal.js");
const SUI_COMMAND_BAR_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-command-bar.js");
const SUI_DEVTOOLS_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-devtools.js");
const SUI_LAUNCHER_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-launcher.js");
const SUI_PANEL_DOCK_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-panel-dock.js");
const SUI_NAV_TOP_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-nav-top.js");
const SUI_RAIL_ISLAND_JS: &str = include_str!("../assets/sui-islands/sui-rail.js");

// Baked view JS + CSS for first-party views (production fallback).
// `counter` moved out to packages/sui-runtime/fixtures/ as a framework
// smoke fixture — no longer a first-party view, no fallback bake.

const AGENTS_VIEW_CSS: &str = include_str!("../../../views/agents/style.css");
const AGENTS_VIEW_JS: &str = include_str!("../assets/views/agents/view.js");
const GIT_VIEW_CSS: &str = include_str!("../../../views/git/style.css");
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

pub async fn serve_sui_runtime() -> Response {
    match read_asset("sui-runtime.js") {
        Some(js) => js_response_owned(js),
        None => js_response(SUI_RUNTIME_JS),
    }
}

pub async fn serve_sui_dom() -> Response {
    match read_asset("sui-dom.js") {
        Some(js) => js_response_owned(js),
        None => js_response(SUI_DOM_JS),
    }
}

pub async fn serve_sui_sdk_client() -> Response {
    match read_asset("sui-sdk-client.js") {
        Some(js) => js_response_owned(js),
        None => js_response(SUI_SDK_CLIENT_JS),
    }
}

pub async fn serve_sui_components() -> Response {
    match read_asset("sui-components.js") {
        Some(js) => js_response_owned(js),
        None => js_response(SUI_COMPONENTS_JS),
    }
}

pub async fn serve_sui_lucide_icons() -> Response {
    if let Some(json) = read_asset("lucide-icons.json") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(json))
            .unwrap_or_else(|_| Response::new(Body::from("response build failed")));
    }
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(Body::from(SUI_LUCIDE_ICONS_JSON))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

pub async fn serve_sui_island(Path(bundle): Path<String>) -> Response {
    if !is_island_bundle(&bundle) {
        return not_found("unknown island");
    }
    let embedded: &'static str = match bundle.as_str() {
        "sui-terminal" => SUI_TERMINAL_ISLAND_JS,
        "sui-command-bar" => SUI_COMMAND_BAR_ISLAND_JS,
        "sui-devtools" => SUI_DEVTOOLS_ISLAND_JS,
        "sui-launcher" => SUI_LAUNCHER_ISLAND_JS,
        "sui-panel-dock" => SUI_PANEL_DOCK_ISLAND_JS,
        "sui-nav-top" => SUI_NAV_TOP_ISLAND_JS,
        "sui-rail" => SUI_RAIL_ISLAND_JS,
        _ => return not_found("unknown island"),
    };
    let rel = format!("sui-islands/{bundle}.js");
    match read_asset(&rel) {
        Some(js) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(js))
            .unwrap_or_else(|_| Response::new(Body::from("response build failed"))),
        None => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
            .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
            .body(Body::from(embedded))
            .unwrap_or_else(|_| Response::new(Body::from("response build failed"))),
    }
}

// Response helpers

fn js_response(body: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn js_response_owned(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
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
        let resp = serve_sui_runtime().await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_terminal_returns_js_bundle() {
        let resp = serve_sui_island(Path("sui-terminal".to_string())).await;
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
        let resp = serve_sui_island(Path("sui-unknown".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn lucide_icons_returns_baked_fallback() {
        let resp = serve_sui_lucide_icons().await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
