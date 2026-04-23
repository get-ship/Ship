//! Events view — live event-stream feed with filter chips and entity groupings.
//!
//! Endpoints:
//! - `GET /api/runtime/view/events` — full-page shell (fragment). Filter chips
//!   are driven by datastar signals; applying a filter re-fetches the content
//!   fragment via this same endpoint plus query params.
//! - `GET /api/runtime/view/events/sse` — Datastar SSE stream that patches
//!   `#events-content` on every newly-emitted event.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use runtime::events::ActorConfig;
use runtime::gen_ulid;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::rest_api::ApiState;
use crate::views::events_state::{
    DEFAULT_LIMIT, EntityGroup, EventFilter, EventRow, distinct_actors, distinct_types,
    group_by_entity, list_events,
};
use crate::views::resolve_view_dir;
use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EventsParams {
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
    /// When set, suppress the shell and render only the content fragment
    /// (used for datastar partial refresh on filter change).
    #[serde(default)]
    pub fragment: Option<String>,
}

impl EventsParams {
    fn filter(&self) -> EventFilter {
        EventFilter {
            event_type: self.event_type.clone(),
            actor: self.actor.clone(),
            entity: self.entity.clone(),
            q: self.q.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct EventsCtx {
    pub filter: EventFilter,
    pub types: Vec<String>,
    pub actors: Vec<String>,
    pub groups: Vec<EntityGroup>,
    pub rows: Vec<EventRow>,
    pub total: usize,
}

fn build_ctx(params: &EventsParams, ship_dir: &std::path::Path) -> EventsCtx {
    let filter = params.filter();
    let all_rows = list_events(ship_dir, &EventFilter::default(), DEFAULT_LIMIT);
    let types = distinct_types(&all_rows);
    let actors = distinct_actors(&all_rows);
    let rows = list_events(ship_dir, &filter, DEFAULT_LIMIT);
    let total = rows.len();
    let groups = group_by_entity(rows.clone());
    EventsCtx {
        filter,
        types,
        actors,
        groups,
        rows,
        total,
    }
}

/// Try rendering a template from `.ship/views/events/<file>`.
/// Returns `None` if the view directory or file does not exist, or if
/// rendering fails (in which case the caller falls back to embedded templates).
fn try_render_view(file: &str, ctx: &EventsCtx) -> Option<String> {
    let (view_dir, _source) = resolve_view_dir("events")?;
    let source = std::fs::read_to_string(view_dir.join(file)).ok()?;
    let template_name = format!("events/{file}");
    templates::render_view_source(&template_name, &source, ctx).ok()
}

#[utoipa::path(
    operation_id = "eventsShell",
    get,
    path = "/api/runtime/view/events",
    responses(
        (status = 200, description = "Rendered events view shell", content_type = "text/html"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn events_shell(
    State(state): State<ApiState>,
    Query(params): Query<EventsParams>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_ctx(&params, state.ship_dir());
    let is_fragment = params.fragment.as_deref().is_some_and(|s| !s.is_empty());

    if !is_fragment {
        // Try .ship/views/events/index.html.j2 first (new canonical location).
        if let Some(html) = try_render_view("index.html.j2", &ctx) {
            return Ok(Html(html));
        }
    }

    // Fallback to embedded template.
    let tpl = if is_fragment {
        "events/content.html.j2"
    } else {
        "events/shell.html.j2"
    };
    templates::render(tpl, &ctx).map(Html).map_err(|e| {
        tracing::error!(error = %e, "events shell render failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[utoipa::path(
    operation_id = "eventsSse",
    get,
    path = "/api/runtime/view/events/sse",
    responses(
        (status = 200, description = "Datastar SSE fragments for events view", content_type = "text/event-stream"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn events_sse(
    State(state): State<ApiState>,
    Query(params): Query<EventsParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();

    let actor_id = format!("events-view-sse-{}", gen_ulid());
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["".to_string()],
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => Some(mb),
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "spawn_actor failed for events SSE");
            None
        }
    };
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));
        if let Some(html) = render_content(&params, &ship_dir) {
            if vs.patch("#events-content", html).await.is_err() {
                return;
            }
        }
        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let Some(html) = render_content(&params, &ship_dir) else { continue };
            if vs.patch("#events-content", html).await.is_err() {
                break;
            }
        }
    });
    sse
}

fn render_content(params: &EventsParams, ship_dir: &std::path::Path) -> Option<String> {
    let ctx = build_ctx(params, ship_dir);
    match templates::render("events/content.html.j2", &ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(error = %e, "events content render failed");
            None
        }
    }
}

#[cfg(test)]
#[path = "views_events_tests.rs"]
mod tests;
