//! `GET /_ship-set/render/{view}` — SSR walker debug endpoint.
//!
//! Phase F-1: an inspection surface that returns the
//! `ship_set_compiler::render::render` walker's output for a compiled
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

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use ship_set_compiler::{parse, render};

// View source — embedded the same way `ship_set_view.rs` embeds its
// templates so the daemon binary is self-contained and the test
// harness (which spawns shipd in a tempdir) sees a real fixture
// without needing an on-disk `views/` tree.
const AGENTS_VIEW_HTML_SET: &str =
    include_str!("../../../../views/agents/view.html.set");
const GIT_VIEW_HTML_SET: &str =
    include_str!("../../../../views/git/view.html.set");

#[derive(Debug, Deserialize, Default)]
pub struct RenderQuery {
    #[serde(default)]
    format: Option<String>,
}

/// `GET /_ship-set/render/{name}` — returns SSR walker output.
pub async fn serve_set_render(
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
                "view not found: views/{name}/view.html.set"
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

fn view_source(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_HTML_SET),
        "git" => Some(GIT_VIEW_HTML_SET),
        _ => None,
    }
}

/// Lowercase ASCII + hyphens only. Mirrors `ship_set_view::is_valid_view_name`
/// so the path segment can't escape the embedded view set.
fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

fn wrap_document(name: &str, body_fragment: &str) -> String {
    let safe_name = escape_html(name);
    format!(
        "<!doctype html>\n\
<html lang=\"en\">\n\
<head>\n\
  <meta charset=\"utf-8\">\n\
  <title>SSR debug: {safe_name}</title>\n\
</head>\n\
<body>\n\
<!-- BEGIN SSR walker output -->\n\
{body_fragment}\n\
<!-- END SSR walker output -->\n\
</body>\n\
</html>\n"
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

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

fn render_error(message: String) -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(message))
        .unwrap_or_else(|_| Response::new(Body::from("compile error")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_renders_html_with_markers() {
        let resp = serve_set_render(
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
        assert!(body.contains("data-set-"));
        assert!(body.contains("__set_init__"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_view_is_404_with_path() {
        let resp = serve_set_render(
            Path("does-not-exist".to_string()),
            Query(RenderQuery::default()),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains("views/does-not-exist/view.html.set"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_name_is_404() {
        let resp = serve_set_render(
            Path("BadName".to_string()),
            Query(RenderQuery::default()),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn json_format_returns_html_and_init() {
        let resp = serve_set_render(
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
