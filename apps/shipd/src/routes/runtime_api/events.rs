//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use ship_substrate::events::ActorConfig;
use serde::Deserialize;

use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize)]
pub struct EventsPollQuery {
    /// ULID cursor — return events with id > since_id, ordered ASC.
    pub since_id: Option<String>,
    /// If set, only return events whose envelope `entity_id` matches OR
    /// whose payload contains `"slug": "<slug>"`.
    pub slug: Option<String>,
    /// Optional event-namespace prefix filter ("job", "process", "supervisor").
    pub namespace: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/v1/runtime/events/poll
///
/// Polling fallback for `ship tail`. SSE replays only future events;
/// poll allows the CLI to walk the log forward from a known cursor.
/// Use `since_id` (ULID) for incremental polling. Limit defaults to 200.
pub async fn poll_events(
    State(state): State<ApiState>,
    Query(params): Query<EventsPollQuery>,
) -> impl IntoResponse {
    match poll_events_core(
        state.ship_dir(),
        params.since_id.as_deref(),
        params.slug.as_deref(),
        params.namespace.as_deref(),
        params.limit.unwrap_or(200),
    ) {
        Ok(data) => (StatusCode::OK, Json(MeshResponse { ok: true, data })),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e) }),
        ),
    }
}

/// Cursor-walk poll core, shared by the REST handler and `EventsService.Poll`
/// (#141). Reads the most recent events ascending and filters in memory by
/// `since_id` cursor, namespace prefix, and slug (envelope `entity_id` or
/// payload `"slug"`). Returns `{ "events": [...] }`.
pub fn poll_events_core(
    ship_dir: &std::path::Path,
    since_id: Option<&str>,
    slug: Option<&str>,
    namespace: Option<&str>,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let limit = limit.min(2000);
    ship_substrate::db::ensure_db_at(ship_dir).map_err(|e| e.to_string())?;
    let events = ship_substrate::db::events::list_recent_events_at(ship_dir, 5_000)
        .map_err(|e| e.to_string())?;

    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| match since_id {
            Some(c) => e.id.as_str() > c,
            None => true,
        })
        .filter(|e| match namespace {
            Some(prefix) => e.event_type.starts_with(&format!("{prefix}.")),
            None => true,
        })
        .filter(|e| match slug {
            Some(s) => {
                if e.entity_id == s {
                    return true;
                }
                serde_json::from_str::<serde_json::Value>(&e.payload_json)
                    .ok()
                    .and_then(|v| v["slug"].as_str().map(str::to_string))
                    .map(|p| p == s)
                    .unwrap_or(false)
            }
            None => true,
        })
        .take(limit)
        .collect();

    Ok(serde_json::json!({ "events": filtered }))
}

// `ActorGuard` moved to `ship_runtime` (alongside `SharedKernel`) so
// views can use it without violating the module DAG (views → routes is
// banned by check-module-dag).

/// GET /api/v1/runtime/events — SSE stream of all kernel events.
pub async fn event_stream(
    State(state): State<ApiState>,
) -> Result<Sse<impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    let actor_id = format!(
        "studio-events-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["".to_string()],
        // SSE is the observability surface — clients opt in explicitly
        // when they want the full stream including directed events
        // (studio.chat_turn, hook.*, mesh.message). Without this flag,
        // the kernel's directed-event short-circuit hides those from
        // broadcast subscribers by design.
        see_directed_events: true,
    };

    let mailbox = {
        let mut kernel = state.kernel().lock().await;
        kernel.spawn_actor(&actor_id, config).map(|(_store, mb)| mb).map_err(|e| {
            tracing::warn!("event_stream: actor spawn failed: {e}");
            let status = match &e {
                ship_substrate::errors::KernelError::ActorAlreadyExists { .. } => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })?
    };

    let guard = crate::ship_runtime::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;
        let mut mb = mailbox;
        while let Some(event) = mb.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default().event("ship.event").data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
