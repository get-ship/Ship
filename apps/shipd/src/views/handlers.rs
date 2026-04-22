//! HTTP handlers for listing and serving views (HTML + component bundles).

use std::collections::HashMap;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use super::packs::{inject_packs, read_view_packs};
use super::resolve::{is_valid_view_name, read_manifest_file, resolve_view_dir, scan_views_dir};
use super::{ViewListEntry, ViewManifest, mesh_err, mesh_ok};

/// Build a rich context for Jinja view rendering.
/// Includes collection schemas and items so views can render server-side.
pub(super) fn build_view_context(view_name: &str, params: &HashMap<String, String>) -> serde_json::Value {
    let mut ctx = serde_json::json!({
        "view_name": view_name,
        "query": params,
    });
    if let Ok(ship_dir) = runtime::project::get_global_dir() {
        // Merge built-in schemas (notes, feedback, annotations) with
        // user-defined ones (global + project-local) so the view's
        // active_collection lookup + items query work for any navigable collection.
        let project_dir = runtime::project::get_project_dir(None).ok();
        let mut schemas = runtime::collections::builtin::all_schemas();
        schemas.extend(
            runtime::collections::schema::load_schemas_merged(&ship_dir, project_dir.as_deref())
                .unwrap_or_default(),
        );
        let summaries: Vec<serde_json::Value> = schemas.iter().map(|s| {
            let fields: Vec<serde_json::Value> = s.fields.iter().map(|(k, v)| {
                serde_json::json!([k, {
                    "field_type": format!("{:?}", v.field_type).to_lowercase(),
                    "required": v.required,
                    "values": v.values,
                    "default": v.default,
                }])
            }).collect();
            serde_json::json!({ "id": s.id, "name": s.name, "field_count": s.fields.len(), "fields": fields })
        }).collect();
        ctx["collections"] = serde_json::json!(summaries);

        let active_schema = params.get("collection")
            .and_then(|id| schemas.iter().find(|s| s.id == *id))
            .or_else(|| schemas.first());

        if let Some(first) = active_schema {
            ctx["active_collection"] = serde_json::json!(first.id);
            let fields_list: Vec<serde_json::Value> = first.fields.iter().map(|(k, v)| {
                serde_json::json!([k, {
                    "field_type": format!("{:?}", v.field_type).to_lowercase(),
                    "required": v.required, "values": v.values, "default": v.default,
                }])
            }).collect();
            ctx["schema"] = serde_json::json!({ "id": first.id, "name": first.name, "fields": fields_list });

            if let Ok(mut svc) = runtime::collections::store::open_data_db(&ship_dir) {
                let _ = runtime::collections::store::ensure_collection_table(&mut svc, first);
                let req = runtime::collections::query::QueryRequest {
                    collection: first.id.clone(),
                    filter: std::collections::BTreeMap::new(),
                    sort: std::collections::BTreeMap::new(),
                    limit: 200, offset: 0,
                };
                if let Ok(items) = runtime::collections::store::query_items(&mut svc, first, &req) {
                    let items_json: Vec<serde_json::Value> = items.into_iter()
                        .map(|i| serde_json::to_value(i).unwrap_or_default()).collect();
                    ctx["total_items"] = serde_json::json!(items_json.len());
                    ctx["items"] = serde_json::json!(items_json);
                }
            }
        }
    }
    ctx
}

/// GET /api/runtime/views
pub async fn list_views() -> impl IntoResponse {
    let mut views: HashMap<String, ViewListEntry> = HashMap::new();
    if let Ok(d) = runtime::project::get_project_dir(None) { scan_views_dir(&d.join("views"), "project", &mut views); }
    if let Ok(d) = runtime::project::get_global_dir() { scan_views_dir(&d.join("views"), "global", &mut views); }
    let mut list: Vec<ViewListEntry> = views.into_values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    mesh_ok(serde_json::json!({ "views": list }))
}

/// GET /api/runtime/view/{name}/asset/{file} — serve static assets from view dir.
pub async fn serve_view_asset(Path((name, file)): Path<(String, String)>) -> axum::response::Response {
    use axum::http::header;

    let err = |status: StatusCode, msg: String| {
        axum::response::Response::builder().status(status).body(axum::body::Body::from(msg)).unwrap()
    };

    if !is_valid_view_name(&name) || file.contains("..") { return err(StatusCode::BAD_REQUEST, "invalid path".into()); }
    let (view_dir, _) = match resolve_view_dir(&name) {
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

/// GET /api/runtime/view/{name}
pub async fn serve_view(
    Path(name): Path<String>,
    axum::extract::Query(query_params): axum::extract::Query<std::collections::HashMap<String, String>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    if !is_valid_view_name(&name) { return mesh_err(StatusCode::BAD_REQUEST, "invalid view name").into_response(); }
    let (view_dir, _) = match resolve_view_dir(&name) {
        Some(v) => v,
        None => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' not found")).into_response(),
    };
    // Try index.html.j2 first (Jinja-processed), then index.html.
    let (source, is_template) = if view_dir.join("index.html.j2").exists() {
        match std::fs::read_to_string(view_dir.join("index.html.j2")) {
            Ok(h) => (h, true),
            Err(_) => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' cannot read index.html.j2")).into_response(),
        }
    } else {
        match std::fs::read_to_string(view_dir.join("index.html")) {
            Ok(h) => { let has_jinja = h.contains("{%") || h.contains("{{"); (h, has_jinja) },
            Err(_) => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' has no index.html")).into_response(),
        }
    };
    // Detect fragment mode: Datastar sends `Datastar-Request: true` when
    // navigating inside the shell. In that case the page chrome (DOCTYPE,
    // html, head, body) is unnecessary — the shell already provides it.
    let is_fragment = headers
        .get("datastar-request")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let html = if is_template {
        let mut ctx = build_view_context(&name, &query_params);
        if is_fragment {
            ctx["fragment"] = serde_json::json!(true);
        }
        match super::templates::render_view_source(&name, &source, &ctx) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(view = %name, "view template render failed: {e}");
                source.replace("{{SHIP_TOKEN}}", "")
            }
        }
    } else {
        source.replace("{{SHIP_TOKEN}}", "")
    };

    // Inject per-view bundle loaders declared via `packs:` in the manifest.
    // Core bundles (datastar.js, ship-elements.js) stay eager in the shell;
    // heavy islands arrive via pack declarations so views that don't need
    // them never pay the download cost.
    let packs = read_view_packs(&view_dir);
    let html = inject_packs(html, &packs, is_fragment);

    // In dev mode, run the linter and inject a findings overlay if any exist.
    let html = if super::lint_overlay::is_dev_mode() {
        let findings = lint::lint_view_dir(&view_dir);
        if findings.is_empty() {
            html
        } else {
            super::lint_overlay::inject_lint_overlay(html, &findings)
        }
    } else {
        html
    };

    // Content negotiation: if the caller wants HTML, return it raw.
    // Studio/MCP get the JSON envelope; shell and browsers get HTML.
    let wants_html = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/html"))
        .unwrap_or(false);
    if wants_html {
        // Middleware (view_morph_target) adds the datastar-selector header
        // so cross-view navigation inside the shell preserves shell chrome.
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(html))
            .unwrap();
    }
    mesh_ok(serde_json::json!({ "html": html })).into_response()
}

/// GET /api/runtime/view/{name}/manifest
pub async fn serve_view_manifest(Path(name): Path<String>) -> impl IntoResponse {
    if !is_valid_view_name(&name) { return mesh_err(StatusCode::BAD_REQUEST, "invalid view name"); }
    let (view_dir, _) = match resolve_view_dir(&name) {
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
            bundle_url: Some(format!("/api/runtime/view/{name}/bundle")),
            claims: None, entry: None,
        }));
    }
    mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' has no servable content"))
}

/// GET /api/runtime/view/{name}/bundle — raw JS, not JSON-wrapped.
pub async fn serve_view_bundle(Path(name): Path<String>) -> axum::response::Response {
    use axum::http::header;

    let err = |status: StatusCode, msg: String| {
        axum::response::Response::builder().status(status).body(axum::body::Body::from(msg)).unwrap()
    };

    if !is_valid_view_name(&name) { return err(StatusCode::BAD_REQUEST, "invalid view name".into()); }
    let (view_dir, _) = match resolve_view_dir(&name) {
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
