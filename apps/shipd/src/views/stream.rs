//! `ViewStream` — typed wrapper around a Datastar-over-SSE response.
//!
//! Axum SSE responses take an `impl Stream<Item = Result<Event, _>>`. This
//! module hides the channel plumbing so handlers can write:
//!
//! ```ignore
//! let (vs, sse) = ViewStream::new();
//! tokio::spawn(async move {
//!     vs.patch_html(initial).await;
//!     while let Some(ev) = events.next().await {
//!         vs.patch("#pipelines-content", render(ev)).await;
//!     }
//! });
//! sse
//! ```
//!
//! ## Wire protocol
//!
//! Every `patch*` call emits a `datastar-patch-elements` SSE event whose
//! `data: elements ...` lines carry the morph-target HTML. Signal writes
//! emit `datastar-patch-signals`. Both paths go through the datastar SDK's
//! `write_as_axum_sse_event` so we stay aligned with client versions as
//! the SDK bumps.
//!
//! Keep-alives are delivered by axum's `KeepAlive::default()` comment
//! frames, applied when `ViewStream::new` constructs the `Sse` response.
//! Applications that need a richer heartbeat (data events instead of
//! comments) can call [`ViewStream::heartbeat_every`].
//!
//! ## Lifetime
//!
//! The `ViewStream` handle can be freely cloned to multiple producers.
//! When the last handle drops, the receiver side closes and the SSE
//! response terminates cleanly. Handlers that need a guaranteed close
//! can call [`ViewStream::close`] which drops the handle explicitly.

use axum::response::sse::{Event, Sse};
use datastar::prelude::{PatchElements, PatchSignals};
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Default buffer for queued SSE events. Small because handlers should
/// keep up with the renderer; a backed-up channel indicates the client
/// is slow and we'd rather apply backpressure than buffer indefinitely.
const DEFAULT_BUFFER: usize = 32;

/// Producer handle for a Datastar SSE view. Cheap to clone — each clone
/// shares the same underlying channel.
#[derive(Clone)]
pub struct ViewStream {
    tx: mpsc::Sender<Result<Event, Infallible>>,
}

/// SSE response type produced by [`ViewStream::new`]. Exported so handlers
/// can name the return type when they need to.
pub type ViewSse = Sse<ReceiverStream<Result<Event, Infallible>>>;

impl ViewStream {
    /// Create a new `(ViewStream, Sse<...>)` pair.
    ///
    /// Return the second value from the axum handler; move the first into
    /// a spawned task that drives the view.
    pub fn new() -> (Self, ViewSse) {
        Self::with_buffer(DEFAULT_BUFFER)
    }

    /// Like [`new`] but lets the caller tune the channel buffer.
    pub fn with_buffer(buffer: usize) -> (Self, ViewSse) {
        let (tx, rx) = mpsc::channel(buffer.max(1));
        // Keep-alive intentionally omitted: callers drive liveness explicitly
        // via `heartbeat_every` when they need it, which keeps the type of
        // the SSE stream simple (see `ViewSse`).
        let sse = Sse::new(ReceiverStream::new(rx));
        (Self { tx }, sse)
    }

    /// Patch elements at `selector` with `html`. Uses Datastar's default
    /// morph mode — the target element and its children are morphed to
    /// match the incoming tree.
    ///
    /// Returns `Ok(())` on successful enqueue; `Err(())` if the receiver
    /// has already closed (client disconnected).
    pub async fn patch(&self, selector: &str, html: String) -> Result<(), ()> {
        let event = PatchElements::new(html)
            .selector(selector)
            .write_as_axum_sse_event();
        self.tx.send(Ok(event)).await.map_err(|_| ())
    }

    /// Patch using only the outer-element `id` of the incoming HTML —
    /// Datastar matches target by that id. Use this when your fragment
    /// is the whole component and selector is redundant.
    pub async fn patch_html(&self, html: String) -> Result<(), ()> {
        let event = PatchElements::new(html).write_as_axum_sse_event();
        self.tx.send(Ok(event)).await.map_err(|_| ())
    }

    /// Merge a JSON signal object into the client's signal store.
    pub async fn signal(&self, json: String) -> Result<(), ()> {
        let event = PatchSignals::new(json).write_as_axum_sse_event();
        self.tx.send(Ok(event)).await.map_err(|_| ())
    }

    /// Send an SSE comment heartbeat every `dur`. Returns a handle that
    /// cancels the heartbeat when dropped. The heartbeat stops on its
    /// own when the client disconnects (send errors out).
    pub fn heartbeat_every(&self, dur: Duration) -> HeartbeatHandle {
        let tx = self.tx.clone();
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(dur);
            // Skip the immediate tick — first heartbeat fires after `dur`.
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let ev = Event::default().comment("ship-heartbeat");
                if tx.send(Ok(ev)).await.is_err() {
                    break;
                }
            }
        });
        HeartbeatHandle { join: Some(handle) }
    }

    /// Drop this handle explicitly. The SSE stream terminates once every
    /// clone of the handle is dropped — if you've cloned the stream, all
    /// clones must go away before the client sees end-of-stream.
    pub fn close(self) {
        drop(self);
    }

    /// True if the client has disconnected (no receivers remain).
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

/// Guard that stops a heartbeat task on drop.
pub struct HeartbeatHandle {
    join: Option<tokio::task::JoinHandle<()>>,
}

impl Drop for HeartbeatHandle {
    fn drop(&mut self) {
        if let Some(j) = self.join.take() {
            j.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    /// Pull the body of an SSE response as a string.
    async fn body_string(resp: axum::response::Response) -> String {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn patch_emits_datastar_patch_elements() {
        let (vs, sse) = ViewStream::new();
        tokio::spawn(async move {
            vs.patch("#target", "<div id=\"target\">hi</div>".into())
                .await
                .unwrap();
            vs.close();
        });
        let body = body_string(sse.into_response()).await;
        assert!(
            body.contains("event: datastar-patch-elements"),
            "expected datastar-patch-elements event, got: {body}"
        );
        assert!(body.contains("hi"));
    }

    #[tokio::test]
    async fn patch_html_without_selector_still_emits_event() {
        let (vs, sse) = ViewStream::new();
        tokio::spawn(async move {
            vs.patch_html("<ul id=\"list\"><li>x</li></ul>".into())
                .await
                .unwrap();
        });
        let body = body_string(sse.into_response()).await;
        assert!(body.contains("datastar-patch-elements"));
        assert!(body.contains("<li>x</li>"));
    }

    #[tokio::test]
    async fn signal_emits_datastar_patch_signals() {
        let (vs, sse) = ViewStream::new();
        tokio::spawn(async move {
            vs.signal(r#"{"counter":42}"#.into()).await.unwrap();
        });
        let body = body_string(sse.into_response()).await;
        assert!(
            body.contains("datastar-patch-signals"),
            "expected datastar-patch-signals event, got: {body}"
        );
        assert!(body.contains("42"));
    }

    #[tokio::test]
    async fn dropping_producer_closes_stream() {
        let (vs, sse) = ViewStream::new();
        drop(vs);
        let body = body_string(sse.into_response()).await;
        // No events emitted, body should be empty or contain only keepalives.
        assert!(!body.contains("datastar-patch-elements"));
    }

    #[tokio::test]
    async fn is_closed_reflects_receiver_drop() {
        let (vs, sse) = ViewStream::new();
        assert!(!vs.is_closed());
        drop(sse);
        // The inner ReceiverStream dropped — sender sees closed.
        // Give the executor a tick to propagate the close.
        tokio::task::yield_now().await;
        assert!(vs.is_closed());
    }

    #[tokio::test]
    async fn heartbeat_sends_comment_frames() {
        let (vs, sse) = ViewStream::new();
        let hb = vs.heartbeat_every(Duration::from_millis(10));
        // Let the heartbeat fire at least once, then shut the stream down
        // so `body_string` can return. Dropping the producer closes the
        // channel; dropping the heartbeat guard aborts its task.
        tokio::time::sleep(Duration::from_millis(35)).await;
        drop(hb);
        drop(vs);
        let body = body_string(sse.into_response()).await;
        assert!(
            body.contains("ship-heartbeat"),
            "expected a heartbeat comment, got: {body:?}"
        );
    }
}
