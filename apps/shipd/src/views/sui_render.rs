#![allow(clippy::doc_lazy_continuation)]
//! `GET /_sui/render/{view}` — SSR walker debug endpoint.
//! `POST /_sui/render-fragment/{view}` — Phase E-2 swap-on-change.
//!
//! Two ship-set rendering endpoints share `views::sui_render_helpers`
//! for view-name validation and source lookup. The GET handler lives
//! here; the POST handler lives in `ship_set_render_fragment.rs` (file
//! cap).
//!
//! ## GET — debug surface
//!
//! Phase F-1: an inspection surface that returns the
//! `sui_compiler::render::render` walker's output for a compiled
//! view. Lets developers `curl` the markers, initial scope, and
//! control-flow rendering against real fixtures, independently of the
//! still-pending production SSR cut-over.
//!
//! Two response shapes:
//! * default → wrapped HTML document (browsable);
//! * `?format=json` → `{ "html": ..., "init": {...} }`.
//!
//! Compile errors render through `CompileError::render(source)` and
//! surface as 500 text. Unknown views surface as 404 with the path
//! that was looked up.
//!
//! ## POST — control-flow fragment render (Phase E-2)
//!
//! Path: `POST /_sui/render-fragment/{view}`
//! Body: `{ "init": { <state/data overrides> }, "key": "b<N>" }`
//!  - `init` is a JSON object whose keys are `<state>` / `<data>`
//!    declaration names; values replace the IR's first-paint defaults
//!    so the renderer evaluates expressions against the runtime's
//!    current scope. Unknown keys (e.g. `route`) are merged in too.
//!  - `key` is the `b<N>` boundary key the runtime is asking the
//!    daemon to re-render. Must be a control-flow binding (`If` /
//!    `For` / `Match` / `Await`); leaf bindings return 400.
//! Response: `200 application/json` with `{ "html": "...", "init": {...} }`
//!  - `html` is the body of the boundary — *not* surrounded by the
//!    `<sui-{kind} key="bN">` wrapper. The
//!    runtime swaps the wrapper's children, not the wrapper itself.
//!  - `init` is the merged scope used by the renderer (defaults +
//!    overrides), mirrored back so the client can confirm what was
//!    rendered against.
//! Errors:
//!  - 400: malformed body / `key` not in registry / `key` is a leaf
//!    (non-control-flow) binding.
//!  - 404: unknown `view`.
//!  - 500: compile / parse error in the embedded view source.

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use sui_compiler::{parse, render};

use super::sui_render_helpers::{
    is_valid_view_name, not_found, render_error, view_source,
};

#[derive(Debug, Deserialize, Default)]
pub struct RenderQuery {
    #[serde(default)]
    format: Option<String>,
}

/// `GET /_sui/render/{name}` — returns SSR walker output.
pub async fn serve_sui_render(
    Path(name): Path<String>,
    Query(query): Query<RenderQuery>,
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

    // Parse first — surfaces the same diagnostic family `compile()`
    // would. `render()` only consumes a `Program`, so we don't need the
    // emitted JS module for an inspection endpoint.
    let program = match parse::parse(source) {
        Ok(p) => p,
        Err(err) => return render_error(err.render(source)),
    };

    let opts = render::RenderOpts { view_name: name.clone() };
    let rendered = match render::render_default(&program, &opts) {
        Ok(r) => r,
        Err(err) => return render_error(err.render(source)),
    };

    let want_json = query
        .format
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if want_json {
        let body = serde_json::json!({
            "html": rendered.html,
            "init": rendered.init,
        });
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(body.to_string()))
            .unwrap_or_else(|_| Response::new(Body::from("response build failed")));
    }

    let document = wrap_document(&name, &rendered.html);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(document))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn wrap_document(name: &str, body_fragment: &str) -> String {
    let safe_name = escape_html(name);
    let importmap = super::sui_attach_boot::importmap_script();
    let components = super::sui_attach_boot::components_side_effect_script();
    let boot = super::sui_attach_boot::attach_boot_script(
        &safe_name,
        "document.querySelector('[data-ssr-root]')",
        "render",
    );
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>SSR debug: {safe_name}</title>
  <link rel="stylesheet" href="/_sui/views/{safe_name}/view.css">
  {importmap}
  {components}
</head>
<body>
<!-- BEGIN SSR walker output -->
<div data-ssr-root>{body_fragment}</div>
<!-- END SSR walker output -->
{boot}
</body>
</html>
"#
    )
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_renders_html_with_markers() {
        let resp = serve_sui_render(
            Path("agents".to_string()),
            Query(RenderQuery::default()),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(ct.starts_with("text/html"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains("<!-- BEGIN SSR walker output -->"));
        assert!(body.contains("<!-- END SSR walker output -->"));
        assert!(body.contains("<!--sui:text:") || body.contains("<sui-if") || body.contains("on:click="));
        assert!(body.contains("__set_init__"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_view_is_404_with_path() {
        let resp = serve_sui_render(
            Path("does-not-exist".to_string()),
            Query(RenderQuery::default()),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains("views/does-not-exist/view.sui"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_name_is_404() {
        let resp = serve_sui_render(
            Path("BadName".to_string()),
            Query(RenderQuery::default()),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn json_format_returns_html_and_init() {
        let resp = serve_sui_render(
            Path("agents".to_string()),
            Query(RenderQuery { format: Some("json".to_string()) }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(ct.starts_with("application/json"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(body.get("html").and_then(|v| v.as_str()).is_some());
        let init = body.get("init").unwrap();
        assert_eq!(
            init.get("agents").unwrap(),
            &serde_json::json!({"kind": "loading"})
        );
    }
}
