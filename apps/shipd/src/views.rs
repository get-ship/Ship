//! View serving — resolves and serves view assets from project and global dirs.
//!
//! Supports HTML views (index.html) and web component views (JS bundles with
//! optional manifests). Resolution checks project `.ship/views/` first, then
//! falls back to global `~/.ship/views/`.

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

/// Shared design tokens + base styles for all daemon-served views.
const VIEW_TOKENS_CSS: &str = include_str!("../assets/view-tokens.css");

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

/// GET /api/runtime/view/{name}/raw?token=... — raw HTML directly, not JSON-wrapped.
/// Used as iframe src so scripts load normally from the daemon origin.
/// Token is in the query string because iframes can't set auth headers.
pub async fn serve_view_raw(
    Path(name): Path<String>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::response::Response {
    use axum::http::header;

    let err = |status: StatusCode, msg: String| {
        axum::response::Response::builder().status(status).body(axum::body::Body::from(msg)).unwrap()
    };

    if !is_valid_view_name(&name) { return err(StatusCode::BAD_REQUEST, "invalid view name".into()); }
    let (view_dir, _) = match resolve_view_dir(&name) {
        Some(v) => v,
        None => return err(StatusCode::NOT_FOUND, format!("view '{name}' not found")),
    };
    let html = match std::fs::read_to_string(view_dir.join("index.html")) {
        Ok(h) => h,
        Err(_) => return err(StatusCode::NOT_FOUND, format!("no index.html for '{name}'")),
    };

    let token = query.get("token").cloned().unwrap_or_default();
    // Only token templating now. Views reference shared assets via HTTP:
    //   <link rel="stylesheet" href="/api/runtime/assets/view-tokens.css">
    //   <script type="module" src="/api/runtime/assets/datastar.js"></script>
    let html = html.replace("{{SHIP_TOKEN}}", &token);

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
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
    let token = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_string();
    // Views reference shared assets via HTTP:
    //   <link rel="stylesheet" href="/api/runtime/assets/view-tokens.css">
    //   <script type="module" src="/api/runtime/assets/datastar.js"></script>
    let html = html.replace("{{SHIP_TOKEN}}", &token);
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

// ---- Datastar SSE: Pipeline Monitor ----

/// GET /api/runtime/view/pipeline-monitor/sse — Datastar SSE stream.
///
/// Renders pipeline job state as HTML fragments pushed via SSE.
/// On connect: sends current state. On job/step events: re-renders and pushes.
pub async fn pipeline_monitor_sse(
    State(state): State<crate::rest_api::ApiState>,
) -> axum::response::sse::Sse<impl futures::stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    use axum::response::sse::{KeepAlive, Sse};
    use datastar::prelude::PatchElements;
    use runtime::events::ActorConfig;

    let actor_id = format!("pipeline-sse-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    };

    let mailbox = state.kernel().lock().await
        .spawn_actor(&actor_id, config)
        .ok()
        .map(|(_store, mb)| mb);

    let guard = super::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;

        // Send initial state.
        let html = render_pipeline_jobs();
        yield Ok::<_, std::convert::Infallible>(PatchElements::new(html).write_as_axum_sse_event());

        // Stream updates on job/step events.
        if let Some(mut mb) = mailbox {
            while let Some(_event) = mb.recv().await {
                let html = render_pipeline_jobs();
                yield Ok::<_, std::convert::Infallible>(PatchElements::new(html).write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Render all pipeline jobs as an HTML fragment for Datastar patching.
fn render_pipeline_jobs() -> String {
    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return r#"<div id="pipeline-content"><p style="color:#71717a;text-align:center;padding:24px 0">Failed to load jobs</p></div>"#.to_string(),
    };

    let mut sorted: Vec<_> = jobs.values().collect();
    sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    if sorted.is_empty() {
        return r#"<div id="pipeline-content"><p style="color:#71717a;text-align:center;padding:24px 0">No jobs</p></div>"#.to_string();
    }

    let mut html = String::from(r#"<div id="pipeline-content">"#);
    for job in &sorted {
        html.push_str(&render_job(job));
    }
    html.push_str("</div>");
    html
}

fn render_job(job: &runtime::projections::job::JobRecord) -> String {
    let steps = match &job.steps {
        Some(p) => &p.steps,
        None => {
            return format!(
                r#"<div class="job" id="job-{slug}">
                    <div class="job-header">
                        <div><span class="job-slug">{slug}</span><span class="job-agent">{agent}</span></div>
                        <span class="badge badge-{status}">{status}</span>
                    </div>
                </div>"#,
                slug = job.slug, agent = job.agent, status = format!("{:?}", job.status).to_lowercase(),
            );
        }
    };

    let completed = &job.completed_steps;
    let current = job.current_step.as_deref();
    let job_status = format!("{:?}", job.status).to_lowercase();

    let steps_html: String = steps.iter().enumerate().map(|(i, step)| {
        let (cls, status, icon) = if completed.contains(&step.id) {
            ("step step-completed", "completed", "&#10003;")
        } else if current == Some(step.id.as_str()) {
            if job_status == "failed" {
                ("step step-failed", "failed", "&#10007;")
            } else {
                ("step step-running", "running", "&#10227;")
            }
        } else {
            ("step", "pending", "&#9675;")
        };

        let agent_label = step.agent.as_deref()
            .map(|a| format!(" &middot; {a}"))
            .unwrap_or_default();
        let step_type = format!("{:?}", step.step_type).to_lowercase();
        let arrow = if i < steps.len() - 1 { r#"<span class="arrow">&rarr;</span>"# } else { "" };

        format!(
            r#"<div class="{cls}">
                <div class="step-name">{icon} {id}</div>
                <div class="step-type">{step_type}{agent_label}</div>
                <div class="step-status">{status}</div>
            </div>{arrow}"#,
            cls = cls, icon = icon, id = step.id, step_type = step_type,
            agent_label = agent_label, status = status, arrow = arrow,
        )
    }).collect();

    format!(
        r#"<div class="job" id="job-{slug}">
            <div class="job-header">
                <div><span class="job-slug">{slug}</span><span class="job-agent">{agent}</span></div>
                <span class="badge badge-{status}">{status}</span>
            </div>
            <div class="pipeline">{steps_html}</div>
        </div>"#,
        slug = job.slug, agent = job.agent, status = job_status, steps_html = steps_html,
    )
}

#[cfg(test)]
#[path = "views_tests.rs"]
mod views_tests;
