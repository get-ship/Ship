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
    serve_bundle_hmr, serve_catalog_workspace, serve_shell, serve_shell_asset,
    serve_meta,
    serve_sui_chunk, serve_sui_dom, serve_sui_icon,
    serve_sui_live_reload, serve_space, serve_space_asset, serve_sui_runtime,
    serve_sui_sdk_client, serve_sui_view_setup,
    serve_view_host, serve_view_payload, serve_view_payload_subpath,
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
/// Registers: framework bundles (runtime/dom/sdk-client/components),
/// per-icon lucide serving, the workspace catalog, per-view setup (compiled
/// `view.js`), view-host (two paths under `/_sui/view/`), and view-payload
/// (top-level + sub-path). Paths mirror shipd's canonical `Route` enum so a
/// host can mount this in lieu of wiring each handler.
///
/// HMR and the state-aware SUI endpoints (sse/data/mutation/patch) are NOT
/// registered here — they need host-supplied extensions or router state.
/// `/_sui/meta` is registered here; hosts that need to inject a
/// `DaemonBuildInfo` extension (shipd) simply layer it before mounting.
pub fn router() -> Router {
    Router::new()
        .route("/_sui/runtime.js", get(serve_sui_runtime))
        .route("/_sui/dom.js", get(serve_sui_dom))
        .route("/_sui/icons/{file}", get(serve_sui_icon))
        .route("/_sui/live-reload.js", get(serve_sui_live_reload))

        // Shared chunks from the multi-entry view build (#267), served
        // content-addressed from `.deck/` root. The exact singleton routes
        // above win for `/_sui/runtime.js` etc.; this catches hashed chunks.
        .route("/_sui/{file}", get(serve_sui_chunk))
        .route("/_sui/catalog/workspace", get(serve_catalog_workspace))
        .route("/_sui/meta", get(serve_meta))
        .route("/_sui/views/{name}/view.js", get(serve_sui_view_setup))
        .route("/_sui/views/{name}", get(serve_view_payload))
        .route("/_sui/views/{name}/{*sub_path}", get(serve_view_payload_subpath))
        .route("/space/_assets/{file}", get(serve_space_asset))
        .route("/space/{id}", get(serve_space))
        // Shell assets at /shell/_assets/{file} (symmetric with /space/_assets).
        .route("/shell/_assets/{file}", get(serve_shell_asset))
        // Single-view host surface under deck's /_sui/ namespace (singular
        // "view" to avoid colliding with the plural "views" payload routes).
        .route("/_sui/view/{name}", get(serve_view_host))
        .route("/_sui/view/{name}/{*sub_path}", get(serve_view_host))
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
        let resp = into_axum(DeckResponse::not_found("nope", DaemonMode::Development));
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn router_builds_without_route_conflict() {
        // The `/_sui/{file}` shared-chunk route is a param sibling of the exact
        // `/_sui/runtime.js` etc.; matchit (axum 0.8) must accept the overlap.
        // A conflict would otherwise only surface as a daemon-startup panic.
        let _ = router();
    }

    /// serve_bundle_hmr with no bundler must return 200 (empty SSE body) and
    /// not hang. This is a safety check for the no-bundler path.
    #[tokio::test(flavor = "multi_thread")]
    async fn serve_bundle_hmr_no_bundler_returns_200() {
        use crate::handlers::serve_bundle_hmr;

        let resp = serve_bundle_hmr(None, None).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
        assert!(ct.contains("text/event-stream"), "must be SSE content-type; got {ct}");
    }

    /// serve_bundle_hmr with a CancellationToken that fires immediately must
    /// not hang. Before this fix the BroadcastStream had no take_until and would
    /// block the axum graceful-shutdown drain until every client disconnected.
    #[tokio::test(flavor = "multi_thread")]
    async fn serve_bundle_hmr_terminates_on_cancel_token() {
        use axum::body::to_bytes;
        use crate::handlers::serve_bundle_hmr;
        use tokio_util::sync::CancellationToken;

        let token = CancellationToken::new();
        // Fire the token immediately before the handler even starts reading.
        token.cancel();

        // serve_bundle_hmr without a bundler (empty stream) — the test gates
        // that the handler accepts and honours the cancel extension without
        // panicking.
        let resp = serve_bundle_hmr(
            None,
            Some(Extension(token)),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Consuming the body must complete (not block forever).
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            to_bytes(resp.into_body(), usize::MAX),
        )
        .await;
        assert!(
            result.is_ok(),
            "response body must drain within 500ms after cancel token fires"
        );
    }
}
