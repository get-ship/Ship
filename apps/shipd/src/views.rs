//! View serving — resolves and serves view assets from project and global dirs.
//!
//! Supports HTML views (index.html) and web component views (JS bundles with
//! optional manifests). Resolution checks project `.ship/views/` first, then
//! falls back to global `~/.ship/views/`.
//!
//! ## R0a foundation submodules
//!
//! The `stream`, `state`, and `templates` submodules provide infrastructure
//! for daemon-rendered views built on minijinja + Datastar SSE. They are
//! scaffolding for the pipelines pilot (R0b) and subsequent view rewrites.
//! R0a intentionally does not change any existing route behavior.
//!
//! - [`stream`] — `ViewStream` helper wrapping Datastar's `PatchElements` /
//!   `PatchSignals` with an `mpsc`-backed SSE response.
//! - [`state`] — read helpers that centralize runtime state access for views.
//!   R0a ships only the subset the pipelines pilot will need (pipelines +
//!   workspace resolution). Helpers for specs, git, and settings are
//!   deliberately not included yet — they will be added by the agents that
//!   implement those pilots, alongside the handlers that consume them.
//! - [`templates`] — minijinja `Environment` loaded from
//!   `apps/shipd/templates/` via the `loader` feature, plus `render`
//!   convenience helpers.

pub mod stream;
pub mod state;
pub mod templates;

use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::rest_api::MeshResponse;

// ---- Types ----

#[derive(Serialize, Clone)]
pub struct ViewManifest {
    pub name: String,
    #[serde(rename = "type")]
    pub view_type: String,
    #[serde(rename = "elementName", skip_serializing_if = "Option::is_none")]
    pub element_name: Option<String>,
    #[serde(rename = "bundleUrl", skip_serializing_if = "Option::is_none")]
    pub bundle_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
}

#[derive(Deserialize)]
struct ManifestFile {
    name: Option<String>,
    #[serde(rename = "type")]
    view_type: Option<String>,
    element: Option<String>,
    claims: Option<Vec<String>>,
    entry: Option<String>,
}

#[derive(Serialize)]
struct ViewListEntry {
    name: String,
    #[serde(rename = "type")]
    view_type: String,
    source: String,
}

type MeshResult = (StatusCode, Json<MeshResponse>);

fn mesh_ok(data: serde_json::Value) -> MeshResult {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

fn mesh_err(status: StatusCode, msg: impl Into<String>) -> MeshResult {
    (StatusCode::from(status), Json(MeshResponse { ok: false, data: serde_json::json!(msg.into()) }))
}

// ---- Resolution ----

fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty() && !name.contains("..") && !name.contains('/') && !name.contains('\\')
}

/// Resolve a view directory by name. Checks project then global dirs.
fn resolve_view_dir(name: &str) -> Option<(PathBuf, &'static str)> {
    if !is_valid_view_name(name) { return None; }
    if let Ok(d) = runtime::project::get_project_dir(None) {
        let c = d.join("views").join(name);
        if c.is_dir() { return Some((c, "project")); }
    }
    if let Ok(d) = runtime::project::get_global_dir() {
        let c = d.join("views").join(name);
        if c.is_dir() { return Some((c, "global")); }
    }
    None
}

fn read_manifest_file(dir: &PathBuf, name: &str) -> Option<ViewManifest> {
    let raw: ManifestFile = if dir.join("manifest.yaml").exists() {
        serde_yaml::from_str(&std::fs::read_to_string(dir.join("manifest.yaml")).ok()?).ok()?
    } else if dir.join("manifest.json").exists() {
        serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).ok()?).ok()?
    } else {
        return None;
    };

    let vtype = raw.view_type.unwrap_or_else(|| "html".into());
    let mname = raw.name.unwrap_or_else(|| name.into());
    let (el, url) = if vtype == "component" {
        (Some(raw.element.unwrap_or_else(|| format!("ship-{name}-view"))),
         Some(format!("/api/runtime/view/{name}/bundle")))
    } else {
        (raw.element, None)
    };
    Some(ViewManifest { name: mname, view_type: vtype, element_name: el, bundle_url: url, claims: raw.claims, entry: raw.entry })
}

fn detect_view_type(dir: &PathBuf) -> Option<String> {
    if dir.join("manifest.yaml").exists() || dir.join("manifest.json").exists() {
        if let Some(m) = read_manifest_file(dir, "unknown") { return Some(m.view_type); }
    }
    if dir.join("index.html").exists() { return Some("html".into()); }
    if dir.join("index.js").exists() { return Some("component".into()); }
    None
}

fn scan_views_dir(dir: &PathBuf, source: &'static str, out: &mut HashMap<String, ViewListEntry>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
        let name = entry.file_name().to_string_lossy().to_string();
        if out.contains_key(&name) { continue; }
        if let Some(vtype) = detect_view_type(&entry.path()) {
            out.insert(name.clone(), ViewListEntry { name, view_type: vtype, source: source.into() });
        }
    }
}

// ---- Shared assets (embedded at compile time) ----

/// Datastar client — compile-time embedded, shipped with the daemon binary.
/// Upgraded by committing a new apps/shipd/assets/datastar.js.
const DATASTAR_JS: &str = include_str!("../assets/datastar.js");

/// Ship web-component element bundle (Lit + every `<ship-*>` primitive and
/// domain element). Generated by `pnpm -F @ship/ui bundle:elements`, which
/// runs esbuild over `packages/ui/src/web-components/index.ts`. Daemon-
/// rendered views load this once and every subsequent SSE fragment that
/// references a `<ship-*>` element renders immediately.
const SHIP_ELEMENTS_JS: &str = include_str!("../assets/ship-elements.js");

/// Custom Elements Manifest (CEM) — machine-readable catalog of every
/// `<ship-*>` element's attrs, slots, events, and CSS custom properties.
/// Generated by `pnpm -F @ship/ui manifest`. Consumed by IDE tooling
/// for attribute autocomplete.
const SHIP_ELEMENTS_MANIFEST: &str = include_str!("../../../packages/ui/custom-elements.json");

/// Token-efficient compact catalog. ~4KB vs the 189KB full CEM.
/// Derived from the CEM by `pnpm -F @ship/ui catalog`. This is what
/// agents should read into their context by default.
const SHIP_ELEMENTS_CATALOG: &str = include_str!("../../../packages/ui/catalog.md");

/// Shared design tokens + base styles for all daemon-served views.
/// Canonical OKLCH palette is sourced from `@ship/ui/tokens.css` so Studio,
/// daemon views, and web components can never drift. View-scoped base rules
/// live alongside in `assets/view-tokens.css`.
const VIEW_TOKENS_CSS: &str = concat!(
    include_str!("../../../packages/ui/src/tokens.css"),
    "\n",
    include_str!("../assets/view-tokens.css"),
);

/// GET /api/runtime/assets/datastar.js
pub async fn serve_datastar_js() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(axum::body::Body::from(DATASTAR_JS))
        .unwrap()
}

/// GET /api/runtime/assets/view-tokens.css
pub async fn serve_view_tokens() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(axum::body::Body::from(VIEW_TOKENS_CSS))
        .unwrap()
}

/// GET /api/runtime/assets/ship-elements.js
pub async fn serve_ship_elements() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(axum::body::Body::from(SHIP_ELEMENTS_JS))
        .unwrap()
}

/// GET /api/runtime/assets/custom-elements.json
pub async fn serve_elements_manifest() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(axum::body::Body::from(SHIP_ELEMENTS_MANIFEST))
        .unwrap()
}

/// GET /api/runtime/assets/ship-elements.md
/// Compact, token-efficient catalog for agent context.
pub async fn serve_elements_catalog() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(axum::body::Body::from(SHIP_ELEMENTS_CATALOG))
        .unwrap()
}

// ---- Handlers ----

/// GET /api/runtime/views
pub async fn list_views() -> impl IntoResponse {
    let mut views: HashMap<String, ViewListEntry> = HashMap::new();
    if let Ok(d) = runtime::project::get_project_dir(None) { scan_views_dir(&d.join("views"), "project", &mut views); }
    if let Ok(d) = runtime::project::get_global_dir() { scan_views_dir(&d.join("views"), "global", &mut views); }
    let mut list: Vec<ViewListEntry> = views.into_values().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    mesh_ok(serde_json::json!({ "views": list }))
}

/// GET /api/runtime/view/{name}/raw — raw HTML directly, not JSON-wrapped.
/// Used as iframe src so scripts load normally from the daemon origin.
///
/// Cookie-first auth: when the caller presents a valid daemon token via
/// `?token=` (iframes cannot set `Authorization` headers), we mint a
/// `ship_session` browser cookie and return it alongside the HTML. The
/// bootstrap `@get(...)` call inside the HTML fires with the cookie
/// attached and never sees the token.
///
/// If the auth middleware has already run (cookie-authenticated caller) we
/// skip cookie issuance — the caller already has one.
///
/// `{{SHIP_TOKEN}}` placeholders in the bootstrap HTML are replaced with an
/// empty string. Views must use relative URLs; the cookie handles auth.
pub async fn serve_view_raw(
    State(state): State<crate::rest_api::ApiState>,
    Path(name): Path<String>,
) -> axum::response::Response {
    use axum::http::header;

    let err = |status: StatusCode, msg: String| {
        axum::response::Response::builder()
            .status(status)
            .body(axum::body::Body::from(msg))
            .unwrap()
    };

    if !is_valid_view_name(&name) {
        return err(StatusCode::BAD_REQUEST, "invalid view name".into());
    }
    let (view_dir, _) = match resolve_view_dir(&name) {
        Some(v) => v,
        None => return err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    let html = match std::fs::read_to_string(view_dir.join("index.html")) {
        Ok(h) => h,
        Err(_) => return err(StatusCode::NOT_FOUND, format!("no index.html for '{name}'")),
    };

    // Strip legacy placeholders; cookie auth replaces query-string tokens.
    let html = html.replace("{{SHIP_TOKEN}}", "");

    // Mint a browser-session cookie so subsequent fetches authenticate via
    // cookie. The auth middleware has already validated the caller's
    // token/session, so this is safe to unconditionally issue here.
    let session_id =
        crate::session::issue_browser_session(state.session_registry()).await;
    let cookie = format!(
        "{name}={id}; HttpOnly; SameSite=Strict; Path=/api/runtime; Max-Age={ttl}",
        name = crate::session::COOKIE_NAME,
        id = session_id,
        ttl = crate::session::BROWSER_SESSION_TTL_SECS,
    );

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::from(html))
        .unwrap()
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
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    if !is_valid_view_name(&name) { return mesh_err(StatusCode::BAD_REQUEST, "invalid view name"); }
    let (view_dir, _) = match resolve_view_dir(&name) {
        Some(v) => v,
        None => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    let html = match std::fs::read_to_string(view_dir.join("index.html")) {
        Ok(h) => h,
        Err(_) => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' has no index.html")),
    };
    let _ = headers; // cookie-first auth; headers unused here
    // Views reference shared assets via relative URLs:
    //   <link rel="stylesheet" href="/api/runtime/assets/view-tokens.css">
    //   <script type="module" src="/api/runtime/assets/datastar.js"></script>
    // Legacy `{{SHIP_TOKEN}}` placeholders are stripped — cookie handles auth.
    let html = html.replace("{{SHIP_TOKEN}}", "");
    mesh_ok(serde_json::json!({ "html": html }))
}

/// GET /api/runtime/view/{name}/manifest
pub async fn serve_view_manifest(Path(name): Path<String>) -> impl IntoResponse {
    if !is_valid_view_name(&name) { return mesh_err(StatusCode::BAD_REQUEST, "invalid view name"); }
    let (view_dir, _) = match resolve_view_dir(&name) {
        Some(v) => v,
        None => return mesh_err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    if view_dir.join("manifest.yaml").exists() || view_dir.join("manifest.json").exists() {
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

#[cfg(test)]
#[path = "views_tests.rs"]
mod views_tests;
