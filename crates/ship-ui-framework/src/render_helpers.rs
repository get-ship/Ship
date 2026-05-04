//! Shared baked-view helpers — embed agents/git/counter `.sui` source
//! at compile time so the resolver and the SSE channel handler can
//! resolve them without a project filesystem.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

const AGENTS_VIEW_HTML_SET: &str =
    include_str!("../../../views/agents/view.sui");
const GIT_VIEW_HTML_SET: &str =
    include_str!("../../../views/git/view.sui");
const COUNTER_VIEW_HTML_SET: &str =
    include_str!("../../../views/counter/view.sui");

/// Baked SUI view set — `include_str!`'d at compile time. Used by the
/// resolver's "bake list" path (where Some(_) means: this is baked,
/// don't look on disk for source_path).
pub fn view_source(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_HTML_SET),
        "git" => Some(GIT_VIEW_HTML_SET),
        "counter" => Some(COUNTER_VIEW_HTML_SET),
        _ => None,
    }
}

/// Resolve a view's source by name. Tries the bake list first, then
/// falls back to per-worktree `.ship/views/<name>/index.sui` so
/// project-defined SUI views (e.g. ship-ui-studio) are still
/// reachable. Used by the SSE channel handler.
pub fn render_view_source(name: &str) -> Option<String> {
    if let Some(baked) = view_source(name) {
        return Some(baked.to_string());
    }
    let cwd = std::env::current_dir().ok()?;
    let ship_dir = runtime::project::resolve_project_ship_dir_no_redirect(&cwd)?;
    let path = ship_dir.join("views").join(name).join("index.sui");
    std::fs::read_to_string(&path).ok()
}

/// Lowercase ASCII + hyphens only. Mirrors `ship_set_view::is_valid_view_name`
/// so the path segment can't escape the embedded view set.
pub fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

pub fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

