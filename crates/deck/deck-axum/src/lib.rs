//! deck-axum — the axum adapter for `deck-core`.
//!
//! deck-core never speaks HTTP; it produces a [`DeckResponse`]. This crate
//! is the plug: thin `serve_*` async shims extract axum params, call the
//! deck-core fn, and map the `DeckResponse` onto `axum::response::Response`
//! via [`into_axum`]. [`router`] registers deck's whole hosting surface as
//! one mountable unit.

mod handlers;
pub mod render_helpers;

pub use handlers::{
    serve_catalog_published, serve_catalog_workspace, serve_meta, serve_pack_asset,
    serve_sui_components, serve_sui_dom, serve_sui_island, serve_sui_live_reload,
    serve_sui_lucide_icons, serve_sui_runtime, serve_sui_sdk_client, serve_sui_view_css,
    serve_sui_view_setup, serve_sui_view_template, serve_view_host, serve_view_payload,
    serve_view_payload_subpath,
};

// Re-export the contract types so `deck`'s facade (and shipd through it)
// can name them without depending on deck-core directly.
pub use deck_core::{
    CachePolicy, DaemonBuildInfo, DaemonMode, DeckResponse, ViewSetupQuery,
    inject_mode_meta, mode_str,
};

use axum::{
    Router,
    body::Body,
    extract::Extension,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};

/// Extract the daemon mode from an optional axum `Extension`, defaulting to
/// Development. The whole `/_sui/*` surface reads it this way so handlers
/// need no router-state plumbing.
pub fn mode_from_ext(ext: Option<Extension<DaemonMode>>) -> DaemonMode {
    ext.map(|e| e.0).unwrap_or(DaemonMode::Development)
}

/// Map a transport-agnostic [`DeckResponse`] onto an `axum::response::Response`.
///
/// A free function rather than `impl IntoResponse`: the orphan rule forbids
/// implementing axum's trait for deck-core's foreign type from this crate.
pub fn into_axum(resp: DeckResponse) -> Response {
    let status = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, resp.content_type)
        .header(header::CACHE_CONTROL, resp.cache_control);
    if let Some(etag) = resp.etag {
        builder = builder.header(header::ETAG, etag);
    }
    for (name, value) in resp.extra_headers {
        builder = builder.header(name, value);
    }
    builder
        .body(Body::from(resp.body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// deck's whole hosting surface as one mountable `Router`.
///
/// Registers: framework bundles (runtime/dom/sdk-client/components/lucide),
/// per-tag island bundles, the workspace + published catalogs, pack assets,
/// per-view template/setup/css, view-host (bare + sub-path), and
/// view-payload (top-level + sub-path). Paths mirror shipd's canonical
/// `Route` enum so a host can mount this in lieu of wiring each handler.
///
/// `/api/_meta`, HMR, and the state-aware SUI endpoints (sse/data/mutation/
/// patch) are NOT registered here — they need host-supplied extensions or
/// router state.
pub fn router() -> Router {
    Router::new()
        .route("/_sui/runtime.js", get(serve_sui_runtime))
        .route("/_sui/dom.js", get(serve_sui_dom))
        .route("/_sui/sdk-client.js", get(serve_sui_sdk_client))
        .route("/_sui/components.js", get(serve_sui_components))
        .route("/_sui/lucide-icons.json", get(serve_sui_lucide_icons))
        .route("/_sui/live-reload.js", get(serve_sui_live_reload))
        .route("/_sui/sui-components/{file}", get(serve_sui_island))
        .route("/api/sui/catalog/workspace", get(serve_catalog_workspace))
        .route("/api/sui/catalog/published", get(serve_catalog_published))
        .route("/_sui/packs/{pack_id}/{file}", get(serve_pack_asset))
        .route("/_sui/views/{name}/view.sui", get(serve_sui_view_template))
        .route("/_sui/views/{name}/view.js", get(serve_sui_view_setup))
        .route("/_sui/views/{name}/view.css", get(serve_sui_view_css))
        .route("/_sui/views/{name}", get(serve_view_payload))
        .route("/_sui/views/{name}/{*sub_path}", get(serve_view_payload_subpath))
        .route("/view-host", get(serve_view_host))
        .route("/view-host/{*rest}", get(serve_view_host))
        .route("/sui/{name}", get(serve_view_host))
        .route("/sui/{name}/{*sub_path}", get(serve_view_host))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deck_core::DaemonMode;

    async fn body_string(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_maps_status_content_type_and_cache() {
        let dr = DeckResponse::js_asset("console.log(1)", DaemonMode::Development);
        let resp = into_axum(dr);
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
        assert!(ct.contains("application/javascript"));
        let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
        assert_eq!(cc, "no-cache");
        assert!(!resp.headers().contains_key(header::ETAG));
        assert_eq!(body_string(resp).await, "console.log(1)");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_prod_emits_etag() {
        let dr = DeckResponse::js_asset("x", DaemonMode::Production);
        let resp = into_axum(dr);
        assert!(resp.headers().contains_key(header::ETAG), "prod must carry ETag");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_preserves_extra_headers() {
        let dr = DeckResponse::js_view("v", DaemonMode::Development)
            .with_header("X-Sui-View", "counter");
        let resp = into_axum(dr);
        assert_eq!(
            resp.headers().get("X-Sui-View").unwrap().to_str().unwrap(),
            "counter"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn into_axum_not_found_status() {
        let resp = into_axum(DeckResponse::not_found("nope"));
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
