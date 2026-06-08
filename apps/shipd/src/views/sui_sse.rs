//! `GET /_sui/views/{name}/sse` — per-view live-data SSE channel.
//!
//! One connection per view instance. The handler:
//!
//! 1. Resolves the view's `.sui` source (bake list or on-disk).
//! 2. Parses it to enumerate `<data live />` declarations.
//! 3. Spawns one task per live data source. Each task sends the initial
//!    resolved value and then re-resolves whenever the underlying kernel
//!    namespaces emit events.
//! 4. Subscribes to the per-view `patch_channel` broadcast so out-of-band
//!    `patch-elements` frames pushed by `sui_agent_push` reach this client.
//! 5. Multiplexes all sources onto a single SSE stream of `patch-signals`
//!    + `patch-elements` events using an mpsc channel.
//!
//! Wire format: `event: patch-signals` with `data: {"name":"…","value":…}`,
//! and `event: patch-elements` with `data: {"selector":…,"mode":…,"html":…}`.
//! See `deck_core::live_wire` for the encoding spec.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::ship_runtime::ApiState;
use super::sui_patch_channel;
use super::sui_sse_sources as sources;
use deck::render_helpers::{is_valid_view_name, render_view_source};

/// Buffer for SSE events across all spawned data-source tasks.
const CHANNEL_BUFFER: usize = 64;

/// `GET /_sui/views/{name}/sse`
pub async fn serve_sui_sse(
    Path(name): Path<String>,
    axum::extract::State(state): axum::extract::State<ApiState>,
) -> Response {
    if !is_valid_view_name(&name) {
        return error_response(StatusCode::BAD_REQUEST, "invalid view name");
    }

    let source = match render_view_source(&name) {
        Some(s) => s,
        None => return error_response(StatusCode::NOT_FOUND, "view not found"),
    };

    // Parse the template and collect `<data live />` declarations.
    let program = match sui_compiler::parse::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(view = %name, error = %e.render(&source), "sui_sse: parse error");
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "view parse error");
        }
    };

    let live_data: Vec<(String, String)> = program
        .data
        .iter()
        .filter(|d| d.live)
        .map(|d| (d.name.clone(), d.source.source.clone()))
        .collect();

    let (tx, rx) = mpsc::channel::<Result<String, std::io::Error>>(CHANNEL_BUFFER);

    // Shutdown cancellation (#256): a SIGTERM cancels the daemon's token, whose
    // child fires here so every long-lived loop below `select!`s out and the
    // axum graceful-shutdown drain can finish — instead of hanging on this
    // never-closing SSE stream until SIGKILL.
    let shutdown = state.shutdown_token();

    // Subscribe to the per-view broadcast so out-of-band frames pushed
    // by `sui_agent_push` (patch-elements, etc.) reach this connection.
    // Spawned BEFORE the live-data tasks so the subscription is in place
    // when those tasks send their initial frames.
    {
        let mut bcast_rx = sui_patch_channel::subscribe(&name).await;
        let tx_b = tx.clone();
        let cancel = shutdown.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    recv = bcast_rx.recv() => match recv {
                        Ok(frame) => {
                            if tx_b.send(Ok(frame)).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(view = %name, dropped = n, "sui_sse: broadcast lagged");
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    },
                }
            }
        });
    }

    if live_data.is_empty() {
        // No live data — hold the connection open with keepalives only.
        let tx2 = tx.clone();
        let cancel = shutdown.clone();
        tokio::spawn(async move { sources::keepalive_loop(tx2, cancel).await });
        drop(tx);
    } else {
        // Spawn one task per live data source.
        for (data_name, expr_source) in live_data {
            let tx = tx.clone();
            let state = state.clone();
            let cancel = shutdown.clone();
            tokio::spawn(async move {
                // Each source loop also selects against shutdown internally; the
                // outer select here closes a source still in its initial resolve.
                tokio::select! {
                    _ = cancel.cancelled() => {}
                    // Signal-backed sources subscribe to the signal directly;
                    // namespace-event-driven sources use the existing path.
                    _ = async {
                        if sources::is_signal_backed_expr(&expr_source) {
                            sources::run_signal_data_source(data_name, expr_source, state, tx, cancel.clone()).await;
                        } else {
                            sources::run_data_source(data_name, expr_source, state, tx, cancel.clone()).await;
                        }
                    } => {}
                }
            });
        }
        // Keepalive task.
        let tx_hb = tx.clone();
        let cancel = shutdown.clone();
        tokio::spawn(async move { sources::keepalive_loop(tx_hb, cancel).await });
        drop(tx);
    }

    let stream = ReceiverStream::new(rx);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse stream init failed")))
}

fn error_response(status: StatusCode, msg: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("error")))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests that don't require a running daemon verify the pure logic:
    // format_patch_signals is the encoding, defined in `deck_core::live_wire`. The
    // per-source driver tests (signal-backed detection + the effect→mpsc
    // bridge) live alongside their code in `sui_sse_sources.rs`. Integration
    // tests (live round-trip) live in the Playwright probe.

    #[test]
    fn is_valid_view_name_rejects_traversal() {
        assert!(!is_valid_view_name("../secret"));
        assert!(!is_valid_view_name(""));
    }

    #[test]
    fn is_valid_view_name_accepts_good_names() {
        assert!(is_valid_view_name("git"));
        assert!(is_valid_view_name("my-view"));
        assert!(is_valid_view_name("agents"));
        // Same SSOT as the render path — names that render must validate here
        // too (digits/uppercase/underscore), so the SSE URL the payload mints
        // is never rejected by this handler.
        assert!(is_valid_view_name("A_B"));
        assert!(is_valid_view_name("v2"));
    }

    #[test]
    fn live_data_extracted_from_parsed_program() {
        let src = r#"<data name="overview" source="ship.git.overview()" live /><data name="log" source="ship.git.log()" /><p>x</p>"#;
        let program = sui_compiler::parse::parse(src).expect("parse");
        let live: Vec<_> = program
            .data
            .iter()
            .filter(|d| d.live)
            .map(|d| (d.name.as_str(), d.source.source.as_str()))
            .collect();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].0, "overview");
        assert_eq!(live[0].1, "ship.git.overview()");
    }

    #[test]
    fn non_live_data_not_included() {
        let src = r#"<data name="x" source="ship.git.log()" /><p>x</p>"#;
        let program = sui_compiler::parse::parse(src).expect("parse");
        let live_count = program.data.iter().filter(|d| d.live).count();
        assert_eq!(live_count, 0);
    }
}
