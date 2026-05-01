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
use runtime::events::ActorConfig;
use runtime::gen_ulid;
use serde::Deserialize;
use std::pin::Pin;

use super::sui_data_resolvers::{
    ParsedExpr, parse_expr, resolve as resolve_value, subscribe_namespaces,
};
use crate::rest_api::ApiState;

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

    let body_stream: Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> =
        match parse_expr(&expr) {
            Some(parsed) => resolved_stream(state, expr.clone(), parsed),
            None => fallback_stream(expr.clone()),
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
fn resolved_stream(
    state: ApiState,
    raw_expr: String,
    parsed: ParsedExpr,
) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> {
    // Resolve the initial value synchronously. Errors degrade to a
    // `null` initial patch — the connection still opens so the runtime
    // doesn't see a transport failure for an authoring bug. The
    // upstream registry message is logged but not pushed to the client.
    let initial = match resolve_value(&parsed, &state) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, expr = %raw_expr, "ship-set-data resolve failed");
            serde_json::Value::Null
        }
    };
    let initial_frame = patch_frame(&initial, &raw_expr);

    let namespaces = subscribe_namespaces(&parsed);
    if namespaces.is_empty() {
        return Box::pin(stream::once(async move { Ok(initial_frame) }).chain(stream::pending()));
    }

    let live = async_stream::stream! {
        let actor_id = format!("ship-set-data-{}", gen_ulid());
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: namespaces,
        };
        let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => Some(mb),
            Err(e) => {
                tracing::error!(error = %e, actor = %actor_id, "ship-set-data spawn_actor failed");
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
            // Coalesce: drain any other immediately-available events before
            // re-resolving. A burst of file changes re-emits one patch.
            while mb.try_recv().is_ok() {}

            let value = resolve_value(&parsed, &state).unwrap_or(serde_json::Value::Null);
            yield Ok(patch_frame(&value, &raw_expr));
        }
    };

    Box::pin(stream::once(async move { Ok(initial_frame) }).chain(live))
}

/// Stream for unparseable expressions. Emits one `null` patch then
/// holds the connection open. Authoring bugs don't break transport.
fn fallback_stream(
    expr: String,
) -> Pin<Box<dyn Stream<Item = Result<String, std::io::Error>> + Send>> {
    let frame = patch_frame(&serde_json::Value::Null, &expr);
    Box::pin(stream::once(async move { Ok(frame) }).chain(stream::pending()))
}

fn patch_frame(value: &serde_json::Value, expr: &str) -> String {
    let payload = serde_json::json!({
        "op": "set",
        "value": value,
        "_expr": expr,
    });
    format!(
        "event: patch\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
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
}
