//! Generic SSE handler for user-authored Ship views.
//!
//! `GET /api/runtime/view/{name}/sse` reads the view's `manifest.yaml` to
//! determine which kernel events to subscribe to, renders partials on each
//! event, and pushes them to the browser via Datastar's PatchElements.

use axum::extract::{Path, State};
use runtime::events::ActorConfig;
use runtime::gen_ulid;
use std::collections::HashMap;
use std::time::Duration;

use crate::rest_api::ApiState;
use crate::views::stream::{ViewSse, ViewStream};

/// Parse Datastar signals from the raw query string.
///
/// On GET requests, Datastar sends all signals as a JSON object in the
/// `datastar` query parameter (URL-encoded).
fn parse_datastar_signals(raw_query: &Option<String>) -> serde_json::Value {
    raw_query
        .as_ref()
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "datastar")
                .map(|(_, v)| v.to_string())
        })
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::json!({}))
}

/// Extract unique namespace prefixes from manifest event names.
///
/// e.g. `["collection.item_created", "collection.item_deleted"]` -> `["collection"]`
fn event_namespaces(events: &Option<Vec<String>>) -> Vec<String> {
    let Some(events) = events else {
        return vec!["".to_string()];
    };
    let mut ns: Vec<String> = events
        .iter()
        .filter_map(|e| e.split('.').next().map(String::from))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    ns.sort();
    if ns.is_empty() {
        ns.push(String::new());
    }
    ns
}

/// Render the content partial for a view from disk (hot-reload friendly).
fn render_partial(
    view_name: &str,
    view_dir: &std::path::Path,
    ctx: &serde_json::Value,
) -> Option<String> {
    let partial_path = view_dir.join("partials/content.html.j2");
    let source = std::fs::read_to_string(&partial_path).ok()?;
    let tpl_name = format!("{view_name}/partials/content");
    match super::templates::render_view_source(&tpl_name, &source, ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(view = %view_name, error = %e, "partial render failed");
            Some(super::error_overlay::render_error_overlay(
                &tpl_name, &source, &e,
            ))
        }
    }
}

/// GET /api/runtime/view/{name}/sse
#[utoipa::path(
    operation_id = "serveViewSse",
    get,
    path = "/api/runtime/view/{name}/sse",
    params(("name" = String, Path, description = "View name")),
    responses(
        (status = 200, description = "Server-sent-event stream of view fragments", content_type = "text/event-stream"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "View not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn serve_view_sse(
    Path(name): Path<String>,
    State(state): State<ApiState>,
    axum::extract::RawQuery(raw_query): axum::extract::RawQuery,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();

    let view_dir_opt = super::resolve_view_dir(&name);

    let actor_id = format!("view-sse-{name}-{}", gen_ulid());

    tokio::spawn(async move {
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        // Resolve view dir and read manifest.
        let Some((view_dir, _)) = view_dir_opt else {
            tracing::warn!(view = %name, "view not found for SSE");
            return;
        };
        let manifest = super::read_raw_manifest(&view_dir);
        let manifest_data = manifest.as_ref().and_then(|m| m.data.clone());
        let manifest_events = manifest.as_ref().and_then(|m| m.events.clone());
        let namespaces = event_namespaces(&manifest_events);

        // Parse Datastar signals from query string.
        let signals = parse_datastar_signals(&raw_query);
        let query_params: HashMap<String, String> = raw_query
            .as_ref()
            .map(|q| {
                form_urlencoded::parse(q.as_bytes())
                    .filter(|(k, _)| k != "datastar")
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Patch selector: convention is #<name>-content (e.g. #data-content).
        let selector = format!("#{name}-content");

        // Initial render.
        let ctx = super::context::build_generic_context(&name, &signals, &manifest_data, &query_params);
        if let Some(html) = render_partial(&name, &view_dir, &ctx) {
            if vs.patch(&selector, html).await.is_err() {
                return;
            }
        }

        // Spawn kernel actor for event subscription.
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: namespaces,
        };
        let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config)
        {
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
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let ctx =
                super::context::build_generic_context(&name, &signals, &manifest_data, &query_params);
            let Some(html) = render_partial(&name, &view_dir, &ctx) else {
                continue;
            };
            if vs.patch(&selector, html).await.is_err() {
                break;
            }
        }
    });

    sse
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_signals_empty() {
        let signals = parse_datastar_signals(&None);
        assert_eq!(signals, serde_json::json!({}));
    }

    #[test]
    fn parse_signals_from_query() {
        let q = Some(
            "datastar=%7B%22collection%22%3A%22task%22%7D&other=1".to_string(),
        );
        let signals = parse_datastar_signals(&q);
        assert_eq!(signals["collection"], "task");
    }

    #[test]
    fn event_namespaces_extracts_prefixes() {
        let events = Some(vec![
            "collection.item_created".into(),
            "collection.item_deleted".into(),
            "workspace.updated".into(),
        ]);
        let ns = event_namespaces(&events);
        assert!(ns.contains(&"collection".to_string()));
        assert!(ns.contains(&"workspace".to_string()));
        assert_eq!(ns.len(), 2);
    }

    #[test]
    fn event_namespaces_defaults_to_wildcard() {
        let ns = event_namespaces(&None);
        assert_eq!(ns, vec!["".to_string()]);
    }
}
