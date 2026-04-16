//! Middleware that tags view responses with a Datastar morph target so
//! cross-view navigation inside the shell doesn't wipe shell chrome.
//!
//! When the shell does `@get('/api/runtime/view/skills')` from within an
//! already-mounted view, Datastar morphs the response into the current
//! DOM at `<body>` level by default — removing `<header>` and `<aside>`
//! because the response body only contains the view content. That's the
//! "chrome loss" bug.
//!
//! Fix: on all `/api/runtime/view/*` responses that are `text/html`, set
//! `datastar-selector: #ship-view-root` and `datastar-mode: outer`.
//! Datastar honors these headers and morphs only that specific element.
//! Non-Datastar callers (plain browser, curl) ignore the headers and
//! render the full page as usual.

use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Wrap responses from view routes with Datastar morph-target headers.
pub async fn view_morph_target_middleware(req: Request<Body>, next: Next) -> Response {
    let is_view_route = req.uri().path().starts_with("/api/runtime/view/");
    let mut response = next.run(req).await;

    if !is_view_route {
        return response;
    }

    let is_html = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/html"))
        .unwrap_or(false);

    if !is_html {
        return response;
    }

    // Don't override if handler already set these (e.g. SSE endpoints).
    let headers = response.headers_mut();
    headers
        .entry("datastar-selector")
        .or_insert(HeaderValue::from_static("#ship-view-root"));
    headers
        .entry("datastar-mode")
        .or_insert(HeaderValue::from_static("outer"));

    response
}
