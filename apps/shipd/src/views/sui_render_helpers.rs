//! Shared helpers for the `/_sui/render/{view}` (GET, walker
//! debug) and `/_sui/render-fragment/{view}` (POST, Phase E-2
//! control-flow swap) endpoints.
//!
//! Both endpoints look the same view source up by name and validate
//! against the same lowercase-ASCII charset, so the lookup + name
//! check live here.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

const AGENTS_VIEW_HTML_SET: &str =
    include_str!("../../../../views/agents/view.sui");
const GIT_VIEW_HTML_SET: &str =
    include_str!("../../../../views/git/view.sui");
const COUNTER_VIEW_HTML_SET: &str =
    include_str!("../../../../views/counter/view.sui");

pub(super) fn view_source(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_HTML_SET),
        "git" => Some(GIT_VIEW_HTML_SET),
        "counter" => Some(COUNTER_VIEW_HTML_SET),
        _ => None,
    }
}

/// Lowercase ASCII + hyphens only. Mirrors `ship_set_view::is_valid_view_name`
/// so the path segment can't escape the embedded view set.
pub(super) fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

pub(super) fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

pub(super) fn render_error(message: String) -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(message))
        .unwrap_or_else(|_| Response::new(Body::from("compile error")))
}

pub(super) fn bad_request(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("bad request")))
}
