//! HTTP handlers for listing and serving views (HTML + component bundles).

use std::collections::HashMap;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};


use super::resolve::{is_valid_view_name, read_manifest_file, resolve_view_dir, scan_views_dir};
use super::{ViewListEntry, ViewManifest, mesh_err, mesh_ok};

/// Views hidden from user-facing pickers (cmdk find, view-picker).
/// Mirrors `HIDDEN_FROM_PICKERS` in routes/search_api.rs — both
/// endpoints feed the same picker surfaces so the lists must agree.
/// When a `hidden: true` manifest field lands, this moves to per-view.
const HIDDEN_FROM_PICKERS: &[&str] = &[
    "setup",
    "sessions", "mesh", "events",
    "specs",
];

/// Value-returning core for the view catalog (`{ views: [...] }`). Backs the
/// `DiscoveryService.List` Connect op (#139); the REST `GET /api/v1/views`
/// handler it replaced is gone.
pub fn list_views_catalog() -> serde_json::Value {
    let mut views: HashMap<String, ViewListEntry> = HashMap::new();
    for d in super::discovery::view_search_dirs() {
        scan_views_dir(&d.path, d.source, &mut views);
    }
    views.retain(|name, _| !HIDDEN_FROM_PICKERS.contains(&name.as_str()));
    let mut list: Vec<ViewListEntry> = views.into_values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    serde_json::json!({ "views": list })
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
    if view_dir.join("manifest.yaml").exists()
        || view_dir.join("manifest.jsonc").exists()
        || view_dir.join("manifest.json").exists()
    {
        if let Some(m) = read_manifest_file(&view_dir, &name) {
            return mesh_ok(serde_json::json!(m));
        }
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
