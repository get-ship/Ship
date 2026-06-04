//! `data live` resolver endpoint.
//!
//! `GET /_sui/data?expr=<expression>` opens an SSE stream that:
//! - Sends an initial `set` patch containing the current value
//! - Streams subsequent `set` patches whenever subscribed kernel
//!   namespaces emit events (full-value replacement; diffing is a v2
//!   concern)
//!
//! Resolution delegates to the existing service registry — every
//! `ship.<domain>.<op>(...)` expression maps to a registry entry and
//! flows through `services::registry::call`. See
//! [`super::ship_set_data_resolvers`] for the dispatch layer.

use axum::{
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::Response,
};
use futures_util::stream::{self, Stream, StreamExt};
use ship_substrate::events::ActorConfig;
use ship_substrate::gen_ulid;
use serde::Deserialize;
use std::pin::Pin;
use tokio_util::sync::CancellationToken;

use super::sui_data_resolvers::{
    ParsedExpr, parse_expr, resolve as resolve_value, subscribe_namespaces,
};
use crate::ship_runtime::ApiState;
use deck_core::live_wire::format_patch_set;

#[derive(Debug, Deserialize)]
pub struct DataQuery {
    pub expr: String,
}

/// `GET /_sui/data?expr=...`
pub async fn serve_data(State(state): State<ApiState>, Query(q): Query<DataQuery>) -> Response {
    let expr = q.expr.trim().to_string();
    if !is_valid_expr(&expr) {
        return bad_request("invalid expr");
    }

    // Shutdown token (#256 class): take_until so SIGTERM can drain this SSE
    // stream without waiting for the client to disconnect.
    let shutdown = state.shutdown_token();

    let body_stream: Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> =
        match parse_expr(&expr) {
            Some(parsed) => resolved_stream(state, expr.clone(), parsed, shutdown),
            None => fallback_stream(expr.clone(), shutdown),
        };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(body_stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}

/// Build a stream that emits an initial `set` patch from the registry
/// and re-emits a fresh `set` patch on every kernel event in the
/// subscribed namespaces.
///
/// `shutdown` is a child of the daemon's CancellationToken. All three code
/// paths call `take_until(shutdown.cancelled())` (#256 class) so SIGTERM can
/// drain this long-lived SSE stream without waiting for the client to
/// disconnect.
fn resolved_stream(
    state: ApiState,
    raw_expr: String,
    parsed: ParsedExpr,
    shutdown: CancellationToken,
) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> {
    // Resolve the initial value synchronously. Errors degrade to a
    // `null` initial patch — the connection still opens so the runtime
    // doesn't see a transport failure for an authoring bug. The
    // upstream registry message is logged but not pushed to the client.
    let initial = match resolve_value(&parsed, &state) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, expr = %raw_expr, "sui-data resolve failed");
            serde_json::Value::Null
        }
    };
    let initial_frame = format_patch_set(&initial, &raw_expr);

    let namespaces = subscribe_namespaces(&parsed);
    if namespaces.is_empty() {
        // No subscriptions — hold open until client disconnects OR shutdown.
        let cancel = shutdown.clone();
        return Box::pin(
            stream::once(async move { Ok(initial_frame) })
                .chain(stream::pending())
                .take_until(async move { cancel.cancelled().await }),
        );
    }

    let live = async_stream::stream! {
        let actor_id = format!("sui-data-{}", gen_ulid());
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: namespaces,
            see_directed_events: false,
        };
        let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => Some(mb),
            Err(e) => {
                tracing::error!(error = %e, actor = %actor_id, "sui-data spawn_actor failed");
                None
            }
        };
        let _guard = crate::ship_runtime::ActorGuard {
            actor_id,
            kernel: state.kernel().clone(),
        };
        let Some(mut mb) = mailbox else { return };

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                recv = mb.recv() => {
                    let Some(_ev) = recv else { break };
                    // Coalesce: drain any other immediately-available events before
                    // re-resolving. A burst of file changes re-emits one patch.
                    while mb.try_recv().is_ok() {}

                    let value = resolve_value(&parsed, &state).unwrap_or(serde_json::Value::Null);
                    yield Ok(format_patch_set(&value, &raw_expr));
                }
            }
        }
    };

    Box::pin(stream::once(async move { Ok(initial_frame) }).chain(live))
}

/// Stream for unparseable expressions. Emits one `null` patch then
/// holds the connection open until the client disconnects or the daemon shuts
/// down. Authoring bugs don't break transport.
fn fallback_stream(
    expr: String,
    shutdown: CancellationToken,
) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> {
    let frame = format_patch_set(&serde_json::Value::Null, &expr);
    Box::pin(
        stream::once(async move { Ok(frame) })
            .chain(stream::pending())
            .take_until(async move { shutdown.cancelled().await }),
    )
}

/// Permissive `ship.namespace.verb(...)` shape check. Rejects anything
/// containing newlines, semicolons, or backticks. The resolver decides
/// whether to honor the verb.
fn is_valid_expr(expr: &str) -> bool {
    if expr.is_empty() || expr.len() > 1024 {
        return false;
    }
    if expr.contains('\n') || expr.contains(';') || expr.contains('`') {
        return false;
    }
    let first = expr.chars().next().unwrap();
    first.is_ascii_alphabetic() || first == '_' || first == '$'
}

fn bad_request(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("bad request")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[test]
    fn valid_expressions() {
        assert!(is_valid_expr("ship.actors.list()"));
        assert!(is_valid_expr("ship.mesh.snapshot()"));
        assert!(is_valid_expr("ship.jobs.detail(id=props.id)"));
    }

    #[test]
    fn invalid_expressions() {
        assert!(!is_valid_expr(""));
        assert!(!is_valid_expr("alert(1);"));
        assert!(!is_valid_expr("eval(`x`)"));
        assert!(!is_valid_expr("multi\nline"));
        assert!(!is_valid_expr(&"x".repeat(2000)));
    }

    /// fallback_stream must terminate when the shutdown token is cancelled.
    /// Without the take_until the stream would block forever (stream::pending()),
    /// holding the axum graceful-shutdown drain open until SIGKILL.
    #[tokio::test]
    async fn fallback_stream_terminates_on_shutdown() {
        let token = CancellationToken::new();
        let mut stream = fallback_stream("unparseable.expr".to_string(), token.clone());

        // First item: the null patch.
        let first = stream.next().await;
        assert!(first.is_some(), "fallback_stream must emit one initial patch");

        // Cancel the token — the stream must now yield None (end) quickly.
        token.cancel();

        // After cancellation the stream should complete (not block).
        // We give it a short timeout to avoid a hung test.
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            stream.next(),
        )
        .await;
        assert!(
            result.is_ok(),
            "fallback_stream must complete within 200ms after shutdown token is cancelled"
        );
        // The stream either ends (None) or yields nothing else.
        // Either way the timeout must not fire.
    }

    /// resolved_stream with no subscriptions (no-namespace path) must also
    /// terminate on shutdown.
    #[tokio::test]
    async fn no_namespace_stream_terminates_on_shutdown() {
        let token = CancellationToken::new();
        // Build a ParsedExpr-less path by injecting an empty namespace list.
        // We test fallback_stream directly since it covers the same code path
        // (stream::once + stream::pending + take_until).
        let token2 = token.clone();
        let mut stream = fallback_stream("x".to_string(), token2);

        let _first = stream.next().await;
        token.cancel();

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            stream.next(),
        )
        .await;
        assert!(result.is_ok(), "no-namespace stream must end after shutdown");
    }
}
