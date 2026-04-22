//! Dry-run render + context-echo endpoints for view authoring.
//!
//! Two entry points, both gated by dev mode OR `X-Ship-Dry-Run: allow`:
//!
//! - Middleware on `/api/runtime/view/{name}` that intercepts requests with
//!   `?__dry-run=1`. With `__context=<urlencoded JSON>` supplied, we render
//!   the view template against that context alone (no data fetch, no DB).
//!   Without `__context`, the request falls through to the normal handler
//!   and we attach an `X-Ship-Context-JSON` response header echoing the
//!   context that WOULD be used (currently: a fresh `build_view_context`).
//! - `serve_view_context` handler for `/api/runtime/view/{name}/context`
//!   returns just the context JSON (no HTML).
//!
//! Dry-run must not leak in production. We require SHIP_DEV/RUST_ENV=dev OR
//! an explicit `X-Ship-Dry-Run: allow` request header.
//!
//! Renders use `render_view_source` which already honors lenient undefined
//! behavior, so partial contexts don't crash.

use std::collections::HashMap;

use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::resolve::{is_valid_view_name, resolve_view_dir};

const MAX_CONTEXT_HEADER: usize = 8 * 1024;

/// Dev-mode OR explicit opt-in header required.
pub(super) fn is_dry_run_allowed(headers: &HeaderMap) -> bool {
    if super::lint_overlay::is_dev_mode() {
        return true;
    }
    headers
        .get("x-ship-dry-run")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("allow"))
        .unwrap_or(false)
}

/// Parse query string into a HashMap (axum's Query extractor not available
/// at middleware level when we haven't extracted Path yet).
fn query_map(uri: &axum::http::Uri) -> HashMap<String, String> {
    uri.query()
        .map(|q| {
            form_urlencoded::parse(q.as_bytes())
                .into_owned()
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}

/// Extract the view name from `/api/runtime/view/{name}` (rejects deeper
/// sub-paths like `/manifest`, `/bundle`, `/context`, `/sse`).
fn extract_view_name(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("/api/runtime/view/")?;
    if rest.contains('/') || rest.is_empty() { return None; }
    Some(rest)
}

/// Render a view's index template with a supplied context value. No data
/// fetch. Used by both the dry-run render path and (for context echo) the
/// handler response augmentation.
fn render_dry(name: &str, ctx: &serde_json::Value) -> Result<String, (StatusCode, String)> {
    let (view_dir, _) = resolve_view_dir(name)
        .ok_or((StatusCode::NOT_FOUND, format!("view '{name}' not found")))?;
    let (source, is_template) = if view_dir.join("index.html.j2").exists() {
        (std::fs::read_to_string(view_dir.join("index.html.j2"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("read index.html.j2: {e}")))?,
         true)
    } else {
        let h = std::fs::read_to_string(view_dir.join("index.html"))
            .map_err(|_| (StatusCode::NOT_FOUND, format!("view '{name}' has no index")))?;
        let tpl = h.contains("{%") || h.contains("{{");
        (h, tpl)
    };
    if !is_template {
        return Ok(source.replace("{{SHIP_TOKEN}}", ""));
    }
    super::templates::render_view_source(name, &source, ctx)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// Middleware: intercept `?__dry-run=1` on `/api/runtime/view/{name}` GETs.
pub async fn dry_run_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let Some(name) = extract_view_name(&path).map(|s| s.to_string()) else {
        return next.run(req).await;
    };
    let q = query_map(req.uri());
    let dry = q.get("__dry-run").map(|s| s == "1").unwrap_or(false);
    if !dry {
        return next.run(req).await;
    }
    if !is_dry_run_allowed(req.headers()) {
        return (StatusCode::FORBIDDEN, "dry-run disabled; set SHIP_DEV=1 or send X-Ship-Dry-Run: allow").into_response();
    }
    if !is_valid_view_name(&name) {
        return (StatusCode::BAD_REQUEST, "invalid view name").into_response();
    }

    // Branch A: caller supplied __context — render with that alone.
    if let Some(raw) = q.get("__context") {
        let ctx: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({
                    "error": "bad __context JSON",
                    "detail": e.to_string(),
                }))).into_response();
            }
        };
        return match render_dry(&name, &ctx) {
            Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap(),
            Err((s, msg)) => (s, msg).into_response(),
        };
    }

    // Branch B: no __context — fall through to real handler, then attach
    // the context JSON the generic handler would have built as a header so
    // agents can fetch-once-then-iterate. This is best-effort; dedicated
    // handlers (git_shell, etc.) build their own contexts we don't see here.
    let ctx = super::handlers::build_view_context(&name, &q);
    let mut response = next.run(req).await;
    let json = serde_json::to_string(&ctx).unwrap_or_else(|_| "{}".into());
    let (value, truncated) = if json.len() > MAX_CONTEXT_HEADER {
        (json[..MAX_CONTEXT_HEADER].to_string(), true)
    } else {
        (json, false)
    };
    if let Ok(hv) = HeaderValue::from_str(&value) {
        response.headers_mut().insert("x-ship-context-json", hv);
    }
    if truncated {
        response.headers_mut().insert(
            "x-ship-context-truncated",
            HeaderValue::from_static("1"),
        );
    }
    response
}

/// GET /api/runtime/view/{name}/context — return just the context JSON.
pub async fn serve_view_context(
    Path(name): Path<String>,
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    if !is_dry_run_allowed(&headers) {
        return (StatusCode::FORBIDDEN, "context endpoint requires dev mode or X-Ship-Dry-Run: allow").into_response();
    }
    if !is_valid_view_name(&name) {
        return (StatusCode::BAD_REQUEST, "invalid view name").into_response();
    }
    // If the caller supplied __context, echo the parsed value so they can
    // verify their JSON round-trips cleanly.
    if let Some(raw) = q.get("__context") {
        return match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(v) => axum::Json(v).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({
                "error": "bad __context JSON",
                "detail": e.to_string(),
            }))).into_response(),
        };
    }
    let ctx = super::handlers::build_view_context(&name, &q);
    axum::Json(ctx).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_view_name() {
        assert_eq!(extract_view_name("/api/runtime/view/git"), Some("git"));
        assert_eq!(extract_view_name("/api/runtime/view/git/sse"), None);
        assert_eq!(extract_view_name("/api/runtime/view/"), None);
        assert_eq!(extract_view_name("/other"), None);
    }

    #[test]
    fn parses_query() {
        let uri: axum::http::Uri = "/api/runtime/view/git?__dry-run=1&__context=%7B%7D".parse().unwrap();
        let q = query_map(&uri);
        assert_eq!(q.get("__dry-run").map(String::as_str), Some("1"));
        assert_eq!(q.get("__context").map(String::as_str), Some("{}"));
    }

    #[test]
    fn dry_run_gate_requires_opt_in() {
        let h = HeaderMap::new();
        // In non-dev environments without header, must be denied.
        if !super::super::lint_overlay::is_dev_mode() {
            assert!(!is_dry_run_allowed(&h));
        }
        let mut h = HeaderMap::new();
        h.insert("x-ship-dry-run", HeaderValue::from_static("allow"));
        assert!(is_dry_run_allowed(&h));
    }
}
