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
//! See `sui_patches.rs` for the encoding spec.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use runtime::events::ActorConfig;
use runtime::gen_ulid;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::rest_api::ApiState;
use crate::views::sui_data_resolvers::{parse_expr, resolve as resolve_value, subscribe_namespaces};
use crate::views::sui_patch_channel;
use crate::views::sui_patches::format_patch_signals;
use super::sui_render_helpers::{is_valid_view_name, render_view_source};

/// Buffer for SSE events across all spawned data-source tasks.
const CHANNEL_BUFFER: usize = 64;
/// Keepalive comment interval so proxies don't close idle connections.
const KEEPALIVE_SECS: u64 = 20;

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

    // Subscribe to the per-view broadcast so out-of-band frames pushed
    // by `sui_agent_push` (patch-elements, etc.) reach this connection.
    // Spawned BEFORE the live-data tasks so the subscription is in place
    // when those tasks send their initial frames.
    {
        let mut bcast_rx = sui_patch_channel::subscribe(&name).await;
        let tx_b = tx.clone();
        tokio::spawn(async move {
            loop {
                match bcast_rx.recv().await {
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
                }
            }
        });
    }

    if live_data.is_empty() {
        // No live data — hold the connection open with keepalives only.
        let tx2 = tx.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(KEEPALIVE_SECS));
            ticker.tick().await; // skip immediate tick
            loop {
                ticker.tick().await;
                if tx2.send(Ok(": ship-keepalive\n\n".to_string())).await.is_err() {
                    break;
                }
            }
        });
        drop(tx);
    } else {
        // Spawn one task per live data source.
        for (data_name, expr_source) in live_data {
            let tx = tx.clone();
            let state = state.clone();
            tokio::spawn(async move {
                run_data_source(data_name, expr_source, state, tx).await;
            });
        }
        // Keepalive task.
        let tx_hb = tx.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(KEEPALIVE_SECS));
            ticker.tick().await;
            loop {
                ticker.tick().await;
                if tx_hb.send(Ok(": ship-keepalive\n\n".to_string())).await.is_err() {
                    break;
                }
            }
        });
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

/// Drive one live data source: send initial value then re-resolve on
/// every kernel event in the subscribed namespaces.
async fn run_data_source(
    data_name: String,
    expr_source: String,
    state: ApiState,
    tx: mpsc::Sender<Result<String, std::io::Error>>,
) {
    let parsed = match parse_expr(&expr_source) {
        Some(p) => p,
        None => {
            // Unparseable expression — send a loading envelope and exit.
            tracing::warn!(expr = %expr_source, "sui_sse: unparseable data source expression");
            let frame = format_patch_signals(&data_name, &serde_json::json!({"kind": "loading"}));
            let _ = tx.send(Ok(frame)).await;
            return;
        }
    };

    // Resolve and send the initial value.
    let initial = resolve_initial(&parsed, &state, &expr_source);
    let frame = format_patch_signals(&data_name, &initial);
    if tx.send(Ok(frame)).await.is_err() {
        return;
    }

    let namespaces = subscribe_namespaces(&parsed);
    if namespaces.is_empty() {
        // No live updates available — initial value only, keep alive via hb task.
        return;
    }

    let actor_id = format!("sui-sse-{}-{}", data_name, gen_ulid());
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: namespaces,
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => Some(mb),
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "sui_sse spawn_actor failed");
            None
        }
    };
    let _guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };
    let Some(mut mb) = mailbox else { return };

    loop {
        let Some(_ev) = mb.recv().await else { break };
        // Coalesce burst events before re-resolving.
        while mb.try_recv().is_ok() {}

        let value = resolve_value(&parsed, &state).unwrap_or(serde_json::Value::Null);
        let live_value = serde_json::json!({"kind": "ok", "value": value});
        let frame = format_patch_signals(&data_name, &live_value);
        if tx.send(Ok(frame)).await.is_err() {
            break;
        }
    }
}

/// Resolve the initial value, wrapping in `{"kind":"ok","value":...}`.
/// Falls back to `{"kind":"loading"}` on error so the runtime never sees
/// a transport-level failure for an authoring mistake.
fn resolve_initial(
    parsed: &crate::views::sui_data_resolvers::ParsedExpr,
    state: &ApiState,
    expr: &str,
) -> serde_json::Value {
    match resolve_value(parsed, state) {
        Ok(v) => serde_json::json!({"kind": "ok", "value": v}),
        Err(e) => {
            tracing::warn!(error = %e, expr = %expr, "sui_sse: initial resolve failed");
            serde_json::json!({"kind": "loading"})
        }
    }
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
    // format_patch_signals is the encoding, tested in sui_patches.rs.
    // Integration tests (live round-trip) live in the Playwright probe.

    #[test]
    fn is_valid_view_name_rejects_traversal() {
        assert!(!is_valid_view_name("../secret"));
        assert!(!is_valid_view_name(""));
        assert!(!is_valid_view_name("A_B"));
    }

    #[test]
    fn is_valid_view_name_accepts_good_names() {
        assert!(is_valid_view_name("git"));
        assert!(is_valid_view_name("my-view"));
        assert!(is_valid_view_name("agents"));
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
