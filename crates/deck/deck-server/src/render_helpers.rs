use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

pub use deck_host::render_helpers::is_valid_view_name;

pub fn render_view_source(name: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let ship_dir = ship_substrate::project::resolve_project_ship_dir(&cwd)?;
    let project_root = ship_dir.parent()?;
    deck_host::render_helpers::render_view_source(project_root, name)
}

pub fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}
