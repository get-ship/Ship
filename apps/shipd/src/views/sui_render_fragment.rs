//! `POST /_sui/render-fragment/{view}` — Phase E-2 control-flow
//! swap-on-change endpoint. See the docstring at the top of
//! `ship_set_render.rs` for the full contract (request body, response
//! shape, error codes); this module just wires the handler.

use axum::{
    body::{Body, Bytes},
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use sui_compiler::{bindings, parse, render};

use super::sui_render_helpers::{
    bad_request, is_valid_view_name, not_found, render_error, view_source,
};

/// Body schema for `POST /_sui/render-fragment/{view}`. Parsed
/// manually from raw bytes (rather than via `axum::Json<…>`) so missing
/// or wrong-shaped fields surface as 400, not axum's default 422.
#[derive(Debug, Deserialize)]
pub struct FragmentRequest {
    /// Caller-provided scope: state names → primitive values, data
    /// names → LiveState envelopes (`{kind, value?, error?}`). Merged
    /// on top of the IR's first-paint defaults.
    pub init: serde_json::Value,
    /// Boundary key (`b<N>`) to render. Must point at a control-flow
    /// binding in the registry.
    pub key: String,
}

/// `POST /_sui/render-fragment/{name}` — re-render a single
/// control-flow boundary's body against caller-supplied scope.
pub async fn serve_sui_render_fragment(
    Path(name): Path<String>,
    body: Bytes,
) -> Response {
    if !is_valid_view_name(&name) {
        return not_found(&format!("invalid view name: {name}"));
    }
    let source = match view_source(&name) {
        Some(s) => s,
        None => {
            return not_found(&format!(
                "view not found: views/{name}/view.sui"
            ));
        }
    };

    if body.is_empty() {
        return bad_request("missing JSON body with `init` and `key`");
    }
    let req: FragmentRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(err) => {
            return bad_request(&format!(
                "invalid JSON body (expected `init` object and `key` string): {err}"
            ));
        }
    };

    if !req.init.is_object() {
        return bad_request("`init` must be a JSON object");
    }
    if req.key.is_empty() {
        return bad_request("`key` must not be empty");
    }

    let program = match parse::parse(source) {
        Ok(p) => p,
        Err(err) => return render_error(err.render(source)),
    };
    let registry = bindings::build(&program);
    let opts = render::RenderOpts { view_name: name.clone() };

    let rendered =
        match render::render_fragment(&program, &registry, &opts, &req.init, &req.key) {
            Ok(r) => r,
            Err(err) => {
                let msg = err.message.clone();
                // Two error families surface as 400, not 500: unknown
                // key + non-control-flow key. Both are "the runtime
                // asked for something the registry doesn't support" —
                // a client bug, not a daemon bug.
                if msg.contains("unknown binding key")
                    || msg.contains("control-flow")
                {
                    return bad_request(&msg);
                }
                return render_error(err.render(source));
            }
        };

    let body = serde_json::json!({
        "html": rendered.html,
        "init": rendered.init,
    });
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn req(init: serde_json::Value, key: &str) -> Bytes {
        let body = serde_json::json!({ "init": init, "key": key });
        Bytes::from(serde_json::to_vec(&body).unwrap())
    }

    async fn body_json(resp: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn body_text(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// Walk the agents view's registry to find the outer `{#if
    /// agents().kind === 'ok'}` boundary key. Hard-coding `b<N>` would
    /// drift if the view source changes; resolve it through the
    /// compiler instead.
    fn outer_if_key() -> String {
        let src = include_str!("../../../../views/agents/view.sui");
        let p = parse::parse(src).expect("parse agents view");
        let r = bindings::build(&p);
        r.iter()
            .find(|b| matches!(b.kind, sui_compiler::keys::BindingKind::If))
            .expect("at least one if-binding in the agents view")
            .key
            .clone()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_render_fragment_for_loading_branch() {
        let key = outer_if_key();
        let resp = serve_sui_render_fragment(
            Path("agents".to_string()),
            req(json!({ "agents": { "kind": "loading" } }), &key),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let html = body.get("html").and_then(|v| v.as_str()).unwrap();
        // Loading branch in the agents view evaluates the inner `{#if
        // agents().kind === 'ok'}` to false, so the active body is
        // empty (no else) and the fragment HTML is empty.
        assert!(html.is_empty(), "expected empty body for loading, got: {html}");
        let init = body.get("init").unwrap();
        assert_eq!(
            init.get("agents").unwrap(),
            &json!({ "kind": "loading" })
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_render_fragment_for_ok_branch() {
        let key = outer_if_key();
        let resp = serve_sui_render_fragment(
            Path("agents".to_string()),
            req(
                json!({
                    "agents": { "kind": "ok", "value": { "items": [] } },
                    "items": []
                }),
                &key,
            ),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let html = body.get("html").and_then(|v| v.as_str()).unwrap();
        // Empty-state branch (items().length === 0) emits the
        // `<ship-empty-state>` element with `data-test="empty"`.
        assert!(
            html.contains(r#"data-test="empty""#),
            "expected empty-state markers in fragment HTML; got: {html}"
        );
        // The fragment must NOT include the outer wrapper for the
        // boundary itself — the runtime already has it in place.
        assert!(
            !html.contains(&format!(r#"data-set-if="{key}""#)),
            "fragment must not include the boundary wrapper; got: {html}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_fragment_unknown_key_400() {
        let resp = serve_sui_render_fragment(
            Path("agents".to_string()),
            req(json!({}), "b9999"),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let text = body_text(resp).await;
        assert!(
            text.contains("unknown binding key"),
            "expected helpful unknown-key message, got: {text}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_fragment_unknown_view_404() {
        let resp = serve_sui_render_fragment(
            Path("does-not-exist".to_string()),
            req(json!({}), "b0"),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let text = body_text(resp).await;
        assert!(text.contains("views/does-not-exist/view.sui"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_fragment_invalid_body_400() {
        let resp =
            serve_sui_render_fragment(Path("agents".to_string()), Bytes::new())
                .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let text = body_text(resp).await;
        assert!(text.contains("init") && text.contains("key"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_fragment_missing_required_fields_400() {
        // Valid JSON but missing `init` + `key`.
        let resp = serve_sui_render_fragment(
            Path("agents".to_string()),
            Bytes::from("{}"),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_fragment_init_must_be_object() {
        let resp = serve_sui_render_fragment(
            Path("agents".to_string()),
            req(json!("not-an-object"), "b0"),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_fragment_leaf_key_400() {
        // Counter view has only leaf bindings (text, on:click). Any of
        // them should fail the control-flow check.
        let src = include_str!("../../../../views/counter/view.sui");
        let p = parse::parse(src).expect("parse counter view");
        let r = bindings::build(&p);
        let leaf = r
            .iter()
            .find(|b| matches!(b.kind, sui_compiler::keys::BindingKind::Text))
            .expect("counter has at least one text binding")
            .key
            .clone();
        let resp = serve_sui_render_fragment(
            Path("counter".to_string()),
            req(json!({}), &leaf),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let text = body_text(resp).await;
        assert!(
            text.contains("control-flow"),
            "expected control-flow guard message, got: {text}"
        );
    }
}
