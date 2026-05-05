//! Routed SSE handler — live server-pushed re-renders for file-routed
//! views.
//!
//! Endpoint: `GET /api/v1/sse/{*path}`.
//!
//! 1. Match `path` against `.ship/views/` via `router::match_path`.
//! 2. Look for `manifest.jsonc` / `manifest.yaml` next to the matched
//!    template or any ancestor dir; read the `events:` list.
//! 3. Subscribe to the union of event namespaces in the kernel.
//! 4. On each event, re-render the view through the same pipeline the
//!    routed-render handler uses (`routed_handler`) and push a
//!    Datastar patch to `#ship-view-root`.
//!
//! Views without a `manifest.events` list still get an SSE stream —
//! it just never fires server-side. Safe to open; client keeps the
//! heartbeat alive.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use ship_substrate::events::ActorConfig;
use ship_substrate::gen_ulid;
use std::collections::HashMap;
use std::time::Duration;

use crate::rest_api::ApiState;
use crate::views::router::match_path;
use crate::views::stream::{ViewSse, ViewStream};

/// Default patch target — the id the shell mounts full-page views into.
/// Callers can override via `?target=<selector>` on the SSE URL when the
/// view is embedded (drawer, dock, iframe) so re-renders don't clobber
/// the main content area.
const DEFAULT_PATCH_SELECTOR: &str = "#ship-view-root";

/// Resolve the nearest `manifest.{jsonc,yaml}` walking up from the
/// template file to any view-root parent. Returns the parsed events
/// list when present.
fn resolve_events(template: &std::path::Path, roots: &[std::path::PathBuf]) -> Vec<String> {
    let mut dir = template.parent();
    while let Some(d) = dir {
        // Stop if we climb above every known root.
        if !roots.iter().any(|r| d.starts_with(r) || d == r.as_path()) {
            break;
        }
        for fname in ["manifest.jsonc", "manifest.yaml", "manifest.json"] {
            let p = d.join(fname);
            if p.is_file() {
                if let Some(evs) = parse_manifest_events(&p) {
                    return evs;
                }
            }
        }
        dir = d.parent();
    }
    Vec::new()
}

fn parse_manifest_events(path: &std::path::Path) -> Option<Vec<String>> {
    let raw = std::fs::read_to_string(path).ok()?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let value: serde_json::Value = match ext {
        "yaml" | "yml" => serde_yaml::from_str(&raw).ok()?,
        _ => {
            let stripped = crate::services::jsonc::strip_jsonc(&raw);
            serde_json::from_str(&stripped).ok()?
        }
    };
    value
        .get("events")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
}

/// Derive kernel subscription namespaces from fully-qualified event
/// names — `config.agent.updated` → `config`.
fn namespaces_from(events: &[String]) -> Vec<String> {
    let mut ns: Vec<String> = events
        .iter()
        .filter_map(|e| e.split('.').next().map(String::from))
        .filter(|s| !s.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    ns.sort();
    ns
}

fn view_roots(state: &ApiState) -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    if let Ok(project_ship) = runtime::project::get_project_dir(None) {
        let p = project_ship.join("views");
        if project_ship != state.ship_dir() && p.is_dir() {
            roots.push(p);
        }
    }
    let user = state.ship_dir().join("views");
    if user.is_dir() {
        roots.push(user);
    }
    roots
}

#[utoipa::path(
    operation_id = "serveViewSseRouted",
    get,
    path = "/api/v1/sse/{path}",
    params(("path" = String, Path, description = "View path; matches the /api/v1/v/{path} tree")),
    responses(
        (status = 200, description = "Datastar SSE stream of view re-renders on subscribed kernel events", content_type = "text/event-stream"),
    ),
    tag = "framework"
)]
pub async fn serve_view_sse_routed(
    State(state): State<ApiState>,
    Path(path): Path<String>,
    axum::extract::RawQuery(raw_query): axum::extract::RawQuery,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let actor_id = format!("routed-sse-{}", gen_ulid());

    tokio::spawn(async move {
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        let roots = view_roots(&state);
        let m = match roots.iter().find_map(|r| match_path(r, &path)) {
            Some(m) => m,
            None => {
                tracing::warn!(path = %path, "routed SSE: no view match");
                return;
            }
        };

        let events = resolve_events(&m.template, &roots);
        let namespaces = namespaces_from(&events);

        // Parse datastar signals + query.
        let (_signals, query_params) = parse_query(&raw_query);

        // Optional per-connection patch target so embedded renders
        // (drawer chat, panel dock, etc.) don't clobber main content.
        let patch_selector: String = query_params
            .get("target")
            .cloned()
            .unwrap_or_else(|| DEFAULT_PATCH_SELECTOR.to_string());

        // Initial render — push the view root immediately so reconnects
        // pick up current state without waiting for an event.
        if let Ok(html) = render_once(&state, &path, &query_params).await {
            if vs.patch(&patch_selector, html).await.is_err() {
                return;
            }
        }

        // No events declared → keep the stream alive (heartbeat) but
        // never subscribe. Useful for views that just want a live
        // connection for signal updates.
        if namespaces.is_empty() {
            return;
        }

        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: namespaces,
        };
        let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => Some(mb),
            Err(e) => {
                tracing::error!(error = %e, actor = %actor_id, "spawn_actor failed");
                None
            }
        };
        let _guard = crate::runtime_api::ActorGuard {
            actor_id,
            kernel: state.kernel().clone(),
        };

        let Some(mut mb) = mailbox else { return };

        // Coalesce: a burst of events re-renders once. Simple: drain
        // pending events before re-render.
        loop {
            let Some(_ev) = mb.recv().await else { break };
            // Drain any other immediately-available events.
            while mb.try_recv().is_ok() {}

            let Ok(html) = render_once(&state, &path, &query_params).await else {
                continue;
            };
            if vs.patch(&patch_selector, html).await.is_err() {
                break;
            }
        }
    });

    sse
}

/// Render the routed view as HTML, using the same pipeline as the
/// HTTP render handler. Synchronous-feeling but runs off the async
/// task; blocking FS / template work stays in this task.
async fn render_once(
    state: &ApiState,
    path: &str,
    query: &HashMap<String, String>,
) -> Result<String, String> {
    // Delegate to the shared render fn in routed_handler. We construct
    // a dummy HeaderMap (no datastar-request, no signals) — the SSE
    // stream is conceptually always a fragment request.
    let mut headers = HeaderMap::new();
    headers.insert("datastar-request", "true".parse().unwrap());
    super::routed_handler::render_routed(state, path, query, &headers)
        .map_err(|e| e.to_string())
}

fn parse_query(
    raw_query: &Option<String>,
) -> (serde_json::Value, HashMap<String, String>) {
    let signals = raw_query
        .as_ref()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "datastar")
                .map(|(_, v)| v.to_string())
        })
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let params: HashMap<String, String> = raw_query
        .as_ref()
        .map(|q| {
            form_urlencoded::parse(q.as_bytes())
                .filter(|(k, _)| k != "datastar")
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default();
    (signals, params)
}
