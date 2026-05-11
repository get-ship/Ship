//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use ship_substrate::events::{ActorConfig, EventEnvelope};
use ship_substrate::events::job::{JobCreatedPayload, event_types};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize, utoipa::IntoParams)]
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
#[utoipa::path(
    operation_id = "pollEvents",
    get,
    path = "/api/v1/events/poll",
    params(EventsPollQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn poll_events(
    State(state): State<ApiState>,
    Query(params): Query<EventsPollQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(200).min(2000);
    let ship_dir = state.ship_dir();
    if let Err(e) = ship_substrate::db::ensure_db_at(ship_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        );
    }
    // Read the most recent N events ascending; we filter in memory.
    let events = match ship_substrate::db::events::list_recent_events_at(ship_dir, 5_000) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    let cursor = params.since_id.as_deref();
    let slug = params.slug.as_deref();
    let ns = params.namespace.as_deref();

    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| match cursor {
            Some(c) => e.id.as_str() > c,
            None => true,
        })
        .filter(|e| match ns {
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

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "events": filtered }),
        }),
    )
}

pub(crate) struct ActorGuard {
    pub(crate) actor_id: String,
    pub(crate) kernel: crate::routes::rest_api::SharedKernel,
}

impl Drop for ActorGuard {
    fn drop(&mut self) {
        let actor_id = self.actor_id.clone();
        let kernel = self.kernel.clone();
        tokio::spawn(async move {
            let _ = kernel.lock().await.stop_actor(&actor_id);
        });
    }
}

/// GET /api/v1/runtime/events — SSE stream of all kernel events.
#[utoipa::path(
    operation_id = "eventStream",
    get,
    path = "/api/v1/events",
    responses(
        (status = 200, description = "SSE stream of kernel events"),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
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

    let guard = ActorGuard {
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
