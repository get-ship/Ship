//! `data live` resolver endpoint.
//!
//! `GET /_sail/data?expr=<expression>` opens an SSE stream that:
//! - Sends an initial `set` patch containing the current value
//! - Streams subsequent patches as the underlying schema source emits
//!   updates
//!
//! v1 is a SAFE STUB:
//! - Accepts requests, validates the expression syntax against a small
//!   allowlist (`ship.<noun>.<verb>(...)`)
//! - Sends a `set` patch with `null` as the initial value
//! - Holds the connection open so the runtime stays subscribed
//!
//! Plumbing real schema sources (actors, mesh, jobs, sessions, ...)
//! lives in a follow-up. The contract is the [`SailDataResolver`]
//! trait below; daemon plugins register resolvers keyed by namespace
//! prefix (`ship.actors.*`, `ship.mesh.*`, ...).

use axum::{
    body::Body,
    extract::Query,
    http::{StatusCode, header},
    response::Response,
};
use futures_util::stream::{self, StreamExt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DataQuery {
    pub expr: String,
}

/// Trait every Sail schema source implements.
///
/// `expr` is a parsed-shallow string like `ship.actors.list({ limit: 50 })`.
/// Resolvers decide whether they own the namespace and, if so, return
/// an initial value and an SSE patch stream.
pub trait SailDataResolver: Send + Sync {
    /// Namespace prefix this resolver owns, e.g. `"ship.actors"`.
    fn namespace(&self) -> &'static str;
    /// Resolve once and return JSON-serializable initial value.
    fn resolve_initial(&self, expr: &str) -> Result<serde_json::Value, String>;
}

/// `GET /_sail/data?expr=...`
pub async fn serve_data(Query(q): Query<DataQuery>) -> Response {
    let expr = q.expr.trim();
    if !is_valid_expr(expr) {
        return bad_request("invalid expr");
    }

    // v1 stub: emit a single set-patch with null as the initial value.
    // The connection stays open for future patch deliveries.
    let initial_patch = format!(
        "event: patch\ndata: {{\"op\":\"set\",\"value\":null,\"_expr\":{expr:?}}}\n\n",
    );
    let stream = stream::once(async move {
        Ok::<_, std::io::Error>(initial_patch)
    })
    .chain(stream::pending());

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}

/// Permissive `ship.namespace.verb(...)` shape check. Rejects anything
/// containing newlines, semicolons, or backticks. The actual schema
/// source decides whether to honor it.
fn is_valid_expr(expr: &str) -> bool {
    if expr.is_empty() || expr.len() > 1024 {
        return false;
    }
    if expr.contains('\n') || expr.contains(';') || expr.contains('`') {
        return false;
    }
    // Must start with an identifier (namespace root).
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

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_expr_rejected() {
        let resp = serve_data(Query(DataQuery { expr: "bad;".into() })).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn valid_expr_returns_event_stream() {
        let resp = serve_data(Query(DataQuery {
            expr: "ship.actors.list()".into(),
        }))
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/event-stream"
        );
    }
}
