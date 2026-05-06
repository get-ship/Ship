//! Hot-module reload channels.
//!
//! Two independent channels:
//!
//! 1. **ship-set HMR** (`/_sui/hmr`): watches `.ship/ship-set/` for `.set`
//!    writes, emits `component:update` events. Used by ship-set component
//!    authors and the Studio UI.
//!
//! 2. **SUI bundle HMR** (`/_sui/dev/hmr`): forwards `module_changed` events
//!    from the Bun bundler worker. Emits `module-changed` and `compile-error`
//!    events. The browser-side `subscribeHmr()` in `@ship/ui-runtime/dev`
//!    opens this stream to get in-place view updates without reload.
//!    Only active when the bundler worker is running.
//!
//! Both channels are lazy — no startup cost if no client subscribes.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};
use futures_util::stream::StreamExt;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Debug, Clone)]
pub struct ShipSetUpdate {
    pub tag: String,
}

struct HmrChannel {
    tx: broadcast::Sender<ShipSetUpdate>,
    /// Hold the watcher so it doesn't drop.
    _watcher: Option<RecommendedWatcher>,
}

static CHANNEL: OnceLock<HmrChannel> = OnceLock::new();

fn channel() -> &'static HmrChannel {
    CHANNEL.get_or_init(|| {
        let (tx, _) = broadcast::channel(64);
        let watcher = spawn_watcher(tx.clone());
        HmrChannel { tx, _watcher: watcher }
    })
}

fn spawn_watcher(tx: broadcast::Sender<ShipSetUpdate>) -> Option<RecommendedWatcher> {
    let dirs: Vec<PathBuf> = [
        runtime::project::get_project_dir(None).ok(),
        runtime::project::get_global_dir().ok(),
    ]
    .into_iter()
    .flatten()
    .map(|d| d.join("sui"))
    .filter(|p| p.is_dir())
    .collect();
    if dirs.is_empty() {
        return None;
    }

    let mut last_emit: std::collections::HashMap<String, std::time::Instant> =
        std::collections::HashMap::new();

    let cb = move |res: Result<Event, notify::Error>| {
        let Ok(event) = res else { return };
        if !matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
        ) {
            return;
        }
        for path in &event.paths {
            if path.extension().and_then(|s| s.to_str()) != Some("set") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            let tag = stem.to_string();
            // Debounce per-tag (notify often fires twice per save).
            let now = std::time::Instant::now();
            if let Some(prev) = last_emit.get(&tag)
                && now.duration_since(*prev) < Duration::from_millis(150) {
                    continue;
                }
            last_emit.insert(tag.clone(), now);
            let _ = tx.send(ShipSetUpdate { tag });
        }
    };

    let mut watcher = match notify::recommended_watcher(cb) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("sui-hmr: watcher init failed: {e}");
            return None;
        }
    };
    for dir in &dirs {
        if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
            tracing::warn!("sui-hmr: watch {} failed: {e}", dir.display());
        }
    }
    tracing::info!("sui-hmr: watching {} dir(s)", dirs.len());
    Some(watcher)
}

/// `GET /_sui/hmr` — long-lived SSE stream of `component:update`
/// events. Each event payload is `{ "tag": "<tag>" }` JSON.
pub async fn serve_hmr() -> Response {
    let rx = channel().tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| async move {
        match res {
            Ok(ShipSetUpdate { tag }) => {
                let payload = format!("{{\"tag\":{tag:?}}}");
                Some(Ok::<_, std::io::Error>(format!(
                    "event: component:update\ndata: {payload}\n\n"
                )))
            }
            Err(_) => None, // lagged receiver — drop and let next event reset
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}

// ── SUI bundle HMR — forwarded from the Bun worker ───────────────────────────

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
/// The browser-side `subscribeHmr()` helper in `@ship/ui-runtime/dev`
/// subscribes here. Events are `module-changed` and `compile-error`
/// kinds with JSON payloads.
pub async fn serve_bundle_hmr() -> Response {
    let rx = bundle_channel().tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| async move {
        let Ok(event) = res else { return None };
        let event_name = match event.kind {
            BundleModuleEventKind::ModuleChanged => "module-changed",
            BundleModuleEventKind::CompileError => "compile-error",
        };
        let payload = serde_json::to_string(&event).unwrap_or_default();
        Some(Ok::<_, std::io::Error>(format!("event: {event_name}\ndata: {payload}\n\n")))
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}

/// `GET /_sui/hmr-client.js` — minimal client that subscribes and
/// reloads on update. Inline because it's tiny and we don't want to
/// pull in a build step.
pub async fn serve_hmr_client_js() -> Response {
    let body = r#"
// ship-set HMR client. Subscribes to /_sui/hmr; reloads on component updates.
// v1: full reload (preserves nothing). v1.5: swap customElements
// definition + walk instances. Listed at docs/ship-set/10-dev-tooling.md.
(function () {
  if (typeof EventSource === 'undefined') return;
  const es = new EventSource('/_sui/hmr');
  es.addEventListener('component:update', (ev) => {
    try {
      const { tag } = JSON.parse(ev.data);
      console.info('[ship-set:hmr] reloading for', tag);
    } catch {}
    location.reload();
  });
  es.addEventListener('error', () => {
    // Reconnect handled by browser; suppress noise.
  });
})();
"#;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response failed")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn hmr_client_serves_js() {
        let resp = serve_hmr_client_js().await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().starts_with("application/javascript"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn hmr_stream_is_event_stream() {
        let resp = serve_hmr().await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(ct.to_str().unwrap(), "text/event-stream");
    }

    #[test]
    fn channel_init_is_idempotent() {
        let a = channel() as *const HmrChannel;
        let b = channel() as *const HmrChannel;
        assert_eq!(a, b);
    }
}
