//! HTTP handlers for listing and serving views (HTML + component bundles).

use std::collections::HashMap;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use super::resolve::{is_valid_view_name, read_manifest_file, resolve_view_dir, scan_views_dir};
use super::{ViewListEntry, ViewManifest, mesh_err, mesh_ok};

/// GET /api/v1/views
#[utoipa::path(
    operation_id = "listViews",
    get,
    path = "/api/v1/views",
    responses(
        (status = 200, description = "Catalog of available views"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn list_views() -> impl IntoResponse {
    let mut views: HashMap<String, ViewListEntry> = HashMap::new();
    if let Ok(d) = ship_application::project::get_project_dir(None) { scan_views_dir(&d.join("views"), "project", &mut views); }
    if let Ok(d) = ship_application::project::get_global_dir() { scan_views_dir(&d.join("views"), "global", &mut views); }
    let mut list: Vec<ViewListEntry> = views.into_values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    mesh_ok(serde_json::json!({ "views": list }))
}

/// GET /api/v1/view/{name}/asset/{file} — serve static assets from view dir.
pub async fn serve_view_asset(Path((name, file)): Path<(String, String)>) -> axum::response::Response {
    use axum::http::header;

    let err = |status: StatusCode, msg: String| {
        axum::response::Response::builder().status(status).body(axum::body::Body::from(msg)).unwrap()
    };

    if !is_valid_view_name(&name) || file.contains("..") { return err(StatusCode::BAD_REQUEST, "invalid path".into()); }
    let (view_dir, _) = match resolve_view_dir(&name, None) {
        Some(v) => v,
        None => return err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };

    // Also check parent views/ dir for shared assets like datastar.min.js
    let path = view_dir.join(&file);
    let path = if path.exists() { path } else {
        let parent = view_dir.parent().map(|p| p.join(&file));
        match parent {
            Some(p) if p.exists() => p,
            _ => return err(StatusCode::NOT_FOUND, format!("asset '{file}' not found")),
        }
    };

    let content = match std::fs::read(&path) {
        Ok(c) => c,
        Err(_) => return err(StatusCode::NOT_FOUND, format!("asset '{file}' not found")),
    };

    let content_type = if file.ends_with(".js") { "application/javascript" }
        else if file.ends_with(".css") { "text/css" }
        else if file.ends_with(".html") { "text/html" }
        else { "application/octet-stream" };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(content))
        .unwrap()
}

/// GET /api/v1/view/{name}/manifest
#[utoipa::path(
    operation_id = "serveViewManifest",
    get,
    path = "/api/v1/view/{name}/manifest",
    params(("name" = String, Path, description = "View name")),
    responses(
        (status = 200, description = "View manifest JSON"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "View not found", body = crate::wire::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn serve_view_manifest(Path(name): Path<String>) -> impl IntoResponse {
    if !is_valid_view_name(&name) { return mesh_err(StatusCode::BAD_REQUEST, "invalid view name"); }
    let (view_dir, _) = match resolve_view_dir(&name, None) {
        Some(v) => v,
        None => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    if (view_dir.join("manifest.yaml").exists()
        || view_dir.join("manifest.jsonc").exists()
        || view_dir.join("manifest.json").exists())
        && let Some(m) = read_manifest_file(&view_dir, &name) {
            return mesh_ok(serde_json::json!(m));
        }
    if view_dir.join("index.html").exists() {
        return mesh_ok(serde_json::json!(ViewManifest {
            name: name.clone(), view_type: "html".into(),
            element_name: None, bundle_url: None, claims: None, entry: None,
        }));
    }
    if view_dir.join("index.js").exists() {
        return mesh_ok(serde_json::json!(ViewManifest {
            name: name.clone(), view_type: "component".into(),
            element_name: Some(format!("ship-{name}-view")),
            bundle_url: Some(format!("/api/v1/view/{name}/bundle")),
            claims: None, entry: None,
        }));
    }
    mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' has no servable content"))
}

/// GET /api/v1/view/{name}/bundle — raw JS, not JSON-wrapped.
#[utoipa::path(
    operation_id = "serveViewBundle",
    get,
    path = "/api/v1/view/{name}/bundle",
    params(("name" = String, Path, description = "View name")),
    responses(
        (status = 200, description = "View module bundle (JavaScript)", content_type = "application/javascript"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "View not found", body = crate::wire::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn serve_view_bundle(Path(name): Path<String>) -> axum::response::Response {
    use axum::http::header;

    let err = |status: StatusCode, msg: String| {
        axum::response::Response::builder().status(status).body(axum::body::Body::from(msg)).unwrap()
    };

    if !is_valid_view_name(&name) { return err(StatusCode::BAD_REQUEST, "invalid view name".into()); }
    let (view_dir, _) = match resolve_view_dir(&name, None) {
        Some(v) => v,
        None => return err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    let entry = read_manifest_file(&view_dir, &name)
        .and_then(|m| m.entry)
        .unwrap_or_else(|| "index.js".into());
    if entry.contains("..") || entry.contains('/') || entry.contains('\\') {
        return err(StatusCode::BAD_REQUEST, "invalid entry path in manifest".into());
    }
    let content = match std::fs::read(view_dir.join(&entry)) {
        Ok(c) => c,
        Err(_) => return err(StatusCode::NOT_FOUND, format!("entry '{entry}' not found for view '{name}'")),
    };
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(content))
        .unwrap()
}
