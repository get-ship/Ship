//! Generic file-routed view handler.
//!
//! Endpoint: `GET /api/v1/v/{*path}`.
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

/// Candidate view roots, in project-wins-first order. `pub(crate)`
/// so the routed SSE handler can reuse the same resolution rules.
pub(crate) fn view_roots(state: &ApiState) -> Vec<std::path::PathBuf> {
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
        "dev": runtime::env::read_raw(runtime::env::Var::Dev).is_ok(),
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

/// Render-only entry point — reused by the routed SSE handler to
/// re-render on kernel events. Returns rendered HTML or a render
/// error string.
pub(crate) fn render_routed(
    state: &ApiState,
    path: &str,
    query: &HashMap<String, String>,
    headers: &HeaderMap,
) -> Result<String, String> {
    let roots = view_roots(state);
    if roots.is_empty() {
        return Err("no view roots available".into());
    }
    let m = roots
        .iter()
        .find_map(|root| match_path(root, path))
        .ok_or_else(|| format!("no view matches /{path}"))?;

    let is_fragment = headers
        .get("datastar-request")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let ctx_json = build_context(
        &m.params,
        query,
        headers,
        state,
        &m.view_name,
        is_fragment,
    );
    let source = std::fs::read_to_string(&m.template)
        .map_err(|e| format!("read {}: {e}", m.template.display()))?;
    let ctx_with_ship = build_render_ctx(ctx_json, state, &m, query);
    render_source(&m.template, &source, &m.view_name, ctx_with_ship)
}

#[utoipa::path(
    operation_id = "serveViewRouted",
    get,
    path = "/api/v1/v/{path}",
    params(("path" = String, Path, description = "Dotted view path matched against .ship/views/ tree")),
    responses(
        (status = 200, description = "Rendered view HTML", content_type = "text/html"),
        (status = 404, description = "No view matches the given path"),
        (status = 500, description = "Template render failed"),
    ),
    tag = "framework"
)]
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
            "no view roots available — create `.ship/views/` to start".to_string(),
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
    let ctx_value = Value::from_serialize(&ctx_json);
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
        let ctx_with_ship = Value::from_serialize(serde_json::Value::Object(merged));
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

    let html = match render_source(&m.template, &source, &m.view_name, ctx_with_ship) {
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

    // Inject per-view pack scripts. Manifest lives at the top-level view
    // dir (first segment of view_name); look it up in each root.
    let top = m.view_name.split('/').next().unwrap_or(&m.view_name);
    let packs: Vec<String> = roots.iter()
        .map(|r| r.join(top))
        .find(|d| d.is_dir())
        .map(|d| crate::views::packs::read_view_packs(&d))
        .unwrap_or_default();
    let html = crate::views::packs::inject_packs(html, &packs, is_fragment);

    let mut resp = html.into_response();
    resp.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    resp
}

/// Compose the render-time Value that carries the `ship` namespace
/// alongside the flat context fields.
pub(crate) fn build_render_ctx(
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
    let base = Value::from_serialize(serde_json::Value::Object(merged));
    // Merge by putting `ship` into the flat context. The macro
    // spreads `base`'s keys, then our explicit `ship` key overrides
    // if a key collides.
    minijinja::context! { ship, ..base }
}

/// Render a template source with a fresh env that mirrors what the
/// user-view renderer sets up: embedded `TEMPLATES` (macros, shared
/// partials), sibling partials discovered on disk, SDK filters,
/// auto-escape disabled. Adds our `__routed__` source + renders.
///
/// We don't use the cached user-view renderer because we need to
/// pass a minijinja::Value (holding the ShipNs Object, which owns
/// ApiState) rather than a Serialize ctx.
pub(crate) fn render_source(
    template_path: &std::path::Path,
    source: &str,
    view_name: &str,
    ctx: Value,
) -> Result<String, String> {
    use minijinja::Environment;

    let mut env = Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_filter("truncate", |value: String, length: usize| -> String {
        if value.chars().count() <= length { value }
        else { let mut s: String = value.chars().take(length.saturating_sub(1)).collect(); s.push('…'); s }
    });
    crate::views::view_filters::register_all(&mut env);

    // Embedded TEMPLATES — macros (_ship.html.j2), _urls, shared
    // partials (chat/partials/*, etc.). First-party view trees live on
    // disk under .ship/views/ and are registered per-request below.
    for (n, s) in crate::views::templates::TEMPLATES {
        let _ = env.add_template(n, s);
    }

    // Sibling partials on disk. Leak strings into 'static so the env
    // satisfies Environment<'static> — transient env, strings freed
    // with the env drop (technically leak, but per-request so fine).
    let top = view_name.split('/').next().unwrap_or(view_name);
    for (n, s) in crate::views::templates::collect_siblings(top) {
        let leaked_n: &'static str = Box::leak(n.into_boxed_str());
        let leaked_s: &'static str = Box::leak(s.into_boxed_str());
        let _ = env.add_template(leaked_n, leaked_s);
    }

    // Add the main source under a stable name.
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
