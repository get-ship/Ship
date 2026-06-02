//! SUI bundle HMR channel.
//!
//! `GET /_sui/dev/hmr` forwards `module_changed` / `compile_error` events
//! from the Bun bundler worker to browser clients as an SSE stream. The
//! Deck host's live-reload client subscribes here to drive in-place view
//! reloads (and full-page reloads for component/runtime bundles).
//!
//! This is a Deck *control-plane* channel — host tooling, not Ship domain
//! data. Domain subscriptions (`<data live>`) ride a separate stream and
//! are migrating onto the generated Connect client. Keep the two planes
//! distinct.
//!
//! The route is registered only when the daemon runs in Development mode
//! (see `router.rs`); the channel itself is lazy — no cost until a client
//! subscribes.

use axum::{
    Extension,
    body::Body,
    http::{StatusCode, header},
    response::Response,
};
use futures_util::stream::StreamExt;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;

/// Event forwarded from the bundler worker to browser clients.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BundleModuleEvent {
    pub kind: BundleModuleEventKind,
    pub target: String,
    pub out_path: Option<String>,
    pub bytes: Option<usize>,
    pub message: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleModuleEventKind {
    ModuleChanged,
    CompileError,
}

struct BundleHmrChannel {
    tx: broadcast::Sender<BundleModuleEvent>,
}

static BUNDLE_CHANNEL: OnceLock<BundleHmrChannel> = OnceLock::new();

fn bundle_channel() -> &'static BundleHmrChannel {
    BUNDLE_CHANNEL.get_or_init(|| {
        let (tx, _) = broadcast::channel(128);
        BundleHmrChannel { tx }
    })
}

/// Publish a bundler event to all subscribed SSE clients.
///
/// Called by the bundler startup hook in `lib.rs` whenever the worker
/// emits a `module_changed` or `compile_error` event.
pub fn publish_bundle_event(event: BundleModuleEvent) {
    let _ = bundle_channel().tx.send(event);
}

/// `GET /_sui/dev/hmr` — SSE stream of bundler module-changed events.
///
/// The Deck host's live-reload client subscribes here. Events are
/// `module-changed` and `compile-error` kinds with JSON payloads.
///
/// The optional `Extension<CancellationToken>` (a child of the daemon shutdown
/// token) terminates the stream on SIGTERM (#256) so this long-lived SSE
/// connection doesn't hold the axum graceful-shutdown drain open until SIGKILL.
pub async fn serve_bundle_hmr(cancel: Option<Extension<CancellationToken>>) -> Response {
    let rx = bundle_channel().tx.subscribe();
    let events = BroadcastStream::new(rx).filter_map(|res| async move {
        let Ok(event) = res else { return None };
        let event_name = match event.kind {
            BundleModuleEventKind::ModuleChanged => "module-changed",
            BundleModuleEventKind::CompileError => "compile-error",
        };
        let payload = serde_json::to_string(&event).unwrap_or_default();
        Some(Ok::<_, std::io::Error>(format!("event: {event_name}\ndata: {payload}\n\n")))
    });
    // End the stream when the shutdown token fires. `take_until` completes the
    // stream as soon as the cancellation future resolves; with no token (tests)
    // the stream runs until the client disconnects, as before.
    let cancel = cancel.map(|e| e.0).unwrap_or_default();
    let stream = events.take_until(async move { cancel.cancelled().await });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_hmr_router(mode: deck::DaemonMode) -> axum::Router {
        use axum::routing;
        let mut r = axum::Router::new();
        if mode == deck::DaemonMode::Development {
            r = r.route("/_sui/dev/hmr", routing::get(serve_bundle_hmr));
        }
        r
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_hmr_route_available_in_dev() {
        use tower::ServiceExt;
        let app = build_hmr_router(deck::DaemonMode::Development);
        let req = axum::http::Request::get("/_sui/dev/hmr")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "/_sui/dev/hmr must be 200 in dev mode");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn bundle_hmr_route_absent_in_production() {
        use tower::ServiceExt;
        let app = build_hmr_router(deck::DaemonMode::Production);
        let req = axum::http::Request::get("/_sui/dev/hmr")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "/_sui/dev/hmr must be 404 in prod mode");
    }
}
