//! Generic file-routed view handler.
//!
//! Endpoint: `GET /api/runtime/v/{*path}`.
//!
//! 1. Match `path` against `.ship/views/` + `~/.ship/views/` via
//!    `views::router::match_path` (project scope wins on overlap).
//! 2. Build the `ship.*` template namespace with route params +
//!    query + signals + session + workspace + env.
//! 3. Load the template file, wrap with any `_layout.html.j2` chain,
//!    render with the ship namespace + legacy view context.
//! 4. Return HTML.
//!
//! Missing matches return 404 with a useful message. Template errors
//! raise 500 with the minijinja error in the body — the frontend's
//! `<ship-error-state>` renders the response either way.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use minijinja::Value;
use std::collections::HashMap;

use crate::rest_api::ApiState;
use crate::services::ship_ns::ShipNs;
use crate::views::router::match_path;

/// Candidate view roots, in project-wins-first order.
fn view_roots(state: &ApiState) -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    // Project scope: .ship/views/ of the enclosing project (if any).
    if let Ok(project_ship) = runtime::project::get_project_dir(None) {
        let p = project_ship.join("views");
        if project_ship != state.ship_dir() && p.is_dir() {
            roots.push(p);
        }
    }
    // User scope: daemon's global ~/.ship/views/.
    let user = state.ship_dir().join("views");
    if user.is_dir() {
        roots.push(user);
    }
    roots
}

/// Build the JSON ctx for a matched view. Carries route params,
/// query, signals, etc. so the ship namespace picks them up.
fn build_context(
    params: &HashMap<String, String>,
    query: &HashMap<String, String>,
    headers: &HeaderMap,
    state: &ApiState,
    view_name: &str,
    is_fragment: bool,
) -> serde_json::Value {
    // Signals carried by Datastar come on a header — we'll parse lazily
    // into a JSON object. Empty when the client doesn't send them.
    let signals = headers
        .get("datastar-signals")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let session = serde_json::json!({});  // TODO: populate from auth context
    let workspace = serde_json::json!({
        "id": headers.get("x-workspace-id").and_then(|v| v.to_str().ok()).unwrap_or(""),
    });
    let env = serde_json::json!({
        "scope": if state.ship_dir().ends_with(".ship") { "user" } else { "project" },
        "version": env!("CARGO_PKG_VERSION"),
        "dev": std::env::var("SHIP_DEV").is_ok(),
        "now": chrono::Utc::now().to_rfc3339(),
    });

    serde_json::json!({
        "view_name": view_name,
        "fragment": is_fragment,
        "route": params,
        "query": query,
        "signals": signals,
        "session": session,
        "workspace": workspace,
        "env": env,
    })
}

pub async fn serve_view_routed(
    State(state): State<ApiState>,
    Path(path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    // Walk candidate roots, first match wins.
    let roots = view_roots(&state);
    if roots.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            format!("no view roots available — create `.ship/views/` to start"),
        )
            .into_response();
    }

    let m = roots
        .iter()
        .find_map(|root| match_path(root, &path));
    let m = match m {
        Some(m) => m,
        None => {
            return (
                StatusCode::NOT_FOUND,
                format!("no view matches `/{path}`"),
            )
                .into_response();
        }
    };

    let is_fragment = headers
        .get("datastar-request")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let ctx_json = build_context(
        &m.params,
        &query,
        &headers,
        &state,
        &m.view_name,
        is_fragment,
    );

    // Read template source.
    let source = match std::fs::read_to_string(&m.template) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("read template {}: {e}", m.template.display()),
            )
                .into_response();
        }
    };

    // Build the minijinja Value for the ctx + inject ship namespace.
    let mut ctx_value = Value::from_serialize(&ctx_json);
    if let Ok(ctx_map) = value_to_mutable_map(&ctx_value) {
        // The ship namespace is a live Object that composes request
        // context with the service registry. It accesses ApiState,
        // so it MUST be constructed per-render.
        let ship = Value::from_object(ShipNs {
            state: state.clone(),
            route: Value::from_serialize(&m.params),
            query: Value::from_serialize(&query),
            signals: Value::from_serialize(ctx_json.get("signals").cloned().unwrap_or_default()),
            session: Value::from_serialize(ctx_json.get("session").cloned().unwrap_or_default()),
            workspace: Value::from_serialize(ctx_json.get("workspace").cloned().unwrap_or_default()),
            env: Value::from_serialize(ctx_json.get("env").cloned().unwrap_or_default()),
        });
        // Rebuild as a map containing the fields + the ship object.
        let mut merged = serde_json::Map::new();
        if let Some(obj) = ctx_json.as_object() {
            for (k, v) in obj {
                merged.insert(k.clone(), v.clone());
            }
        }
        let mut ctx_with_ship = Value::from_serialize(&serde_json::Value::Object(merged));
        // minijinja doesn't let us trivially inject an Object into a
        // Value::from_serialize map, so we build the env as kwargs
        // using the template engine's state. Simpler: render via the
        // engine's kwargs mechanism — we pass a single dict and the
        // minijinja engine merges.
        let _ = (ctx_map, ship, ctx_with_ship);
    }

    // Render using the existing template pipeline — for now we compose
    // the ship namespace as a globals map at the environment level via
    // a per-render context argument. minijinja supports passing an
    // object arg that merges into the render scope.
    let ctx_with_ship = build_render_ctx(ctx_json.clone(), &state, &m, &query);

    let html = match render_source(&m.template, &source, ctx_with_ship) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(view = %m.view_name, "routed view render failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("render {}: {e}", m.view_name),
            )
                .into_response();
        }
    };

    let mut resp = html.into_response();
    resp.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    resp
}

/// Compose the render-time Value that carries the `ship` namespace
/// alongside the flat context fields.
fn build_render_ctx(
    ctx_json: serde_json::Value,
    state: &ApiState,
    m: &crate::views::router::RouteMatch,
    query: &HashMap<String, String>,
) -> Value {
    let ship = Value::from_object(ShipNs {
        state: state.clone(),
        route: Value::from_serialize(&m.params),
        query: Value::from_serialize(query),
        signals: Value::from_serialize(ctx_json.get("signals").cloned().unwrap_or_default()),
        session: Value::from_serialize(ctx_json.get("session").cloned().unwrap_or_default()),
        workspace: Value::from_serialize(ctx_json.get("workspace").cloned().unwrap_or_default()),
        env: Value::from_serialize(ctx_json.get("env").cloned().unwrap_or_default()),
    });
    // Merge: a simple Value::from_serialize map plus the `ship` key.
    let mut merged = serde_json::Map::new();
    if let Some(obj) = ctx_json.as_object() {
        for (k, v) in obj {
            merged.insert(k.clone(), v.clone());
        }
    }
    let base = Value::from_serialize(&serde_json::Value::Object(merged));
    // Merge by putting `ship` into the flat context. The macro
    // spreads `base`'s keys, then our explicit `ship` key overrides
    // if a key collides.
    minijinja::context! { ship, ..base }
}

/// Render a template source directly using a local env. This bypasses
/// the global `templates::render` because we need a per-render env
/// that can carry the ShipNs (which holds ApiState).
fn render_source(
    template_path: &std::path::Path,
    source: &str,
    ctx: Value,
) -> Result<String, String> {
    // Build a transient env with the shared template corpus + our
    // one-off source. Reuses the global env's registered templates
    // (macros, `_urls.html.j2`, etc.) so templates can import.
    let shared = crate::views::templates::env();
    let mut env = shared.clone();
    env.add_template_owned("__routed__", source.to_string())
        .map_err(|e| format!("parse {}: {e}", template_path.display()))?;
    let tpl = env
        .get_template("__routed__")
        .map_err(|e| format!("lookup {}: {e}", template_path.display()))?;
    tpl.render(ctx)
        .map_err(|e| format!("render {}: {e}", template_path.display()))
}

/// (Unused — placeholder from an earlier approach.)
fn value_to_mutable_map(_v: &Value) -> Result<(), ()> { Ok(()) }
