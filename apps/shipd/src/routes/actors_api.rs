//! `/api/actors/*` — unified read/control surface for every running
//! actor (chat agent, pipeline step, shell, service).
//!
//! Read endpoints project from the `actors` table maintained by
//! `ship_substrate::projections::ActorProjection`. Liveness derives from
//! `last_event_at` (bumped by SQLite triggers on every event correlated
//! to the actor) — no heartbeat loop. The input endpoint emits a
//! kernel event with `target_actor_id` set so directed delivery routes
//! it to the actor's mailbox without broadcast filtering.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::projections::actor::{
    ListActorsFilter, get_actor, list_actor_events, list_actors,
};
use serde::Serialize;
use serde::Deserialize;
use serde_json::json;

use crate::auth::AuthContext;
use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize)]
pub struct ListQuery {
    pub workspace_id: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/actors` — list all actors, ordered by recency. The
/// `live` field on each row is overlaid from the kernel's mailbox
/// map at query time — the mesh IS the source of truth for "alive
/// and reachable." If an actor is in `kernel.list_actors()`, you can
/// send to it and it will receive. If not, you can't. Period.
pub async fn list_actors_handler(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    let filter = ListActorsFilter {
        workspace_id: q.workspace_id,
        kind: q.kind,
        status: q.status,
        limit: q.limit.unwrap_or(0),
    };
    let mut rows = match list_actors(&filter) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
            )
                .into_response();
        }
    };
    let on_mesh: std::collections::HashSet<String> = state
        .kernel()
        .lock()
        .await
        .list_actors()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    for r in rows.iter_mut() {
        r.live = on_mesh.contains(&r.id);
    }
    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: json!({ "actors": rows }) }),
    )
        .into_response()
}

#[derive(Serialize)]
pub struct MeshActor {
    pub actor_id: String,
    pub subscribe_namespaces: Vec<String>,
    pub write_namespaces: Vec<String>,
}

/// `GET /api/mesh` — every actor with a live mailbox in the kernel,
/// straight from `kernel.list_actors()`. This is the heartbeat: if
/// it's here, it's reachable. Used by the dashboard and by anyone
/// who needs to know "what can I send to right now."
pub async fn list_mesh_handler(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> impl IntoResponse {
    let actors: Vec<MeshActor> = state
        .kernel()
        .lock()
        .await
        .list_actors()
        .into_iter()
        .map(|(id, cfg)| MeshActor {
            actor_id: id,
            subscribe_namespaces: cfg.subscribe_namespaces,
            write_namespaces: cfg.write_namespaces,
        })
        .collect();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: json!({ "count": actors.len(), "actors": actors }),
        }),
    )
        .into_response()
}

/// `GET /api/actors/{id}` — single actor detail.
pub async fn get_actor_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match get_actor(&id) {
        Ok(Some(a)) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: json!(a) }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: json!(format!("actor '{id}' not found")),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct EventsQuery {
    pub from_id: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/actors/{id}/events?from_id=<event_id>` — replay events
/// correlated to the actor in causal order. Used by the SSE reconnect
/// path: client tracks last seen event id, asks for everything after.
pub async fn list_actor_events_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<EventsQuery>,
) -> impl IntoResponse {
    match list_actor_events(&id, q.from_id.as_deref(), q.limit.unwrap_or(0)) {
        Ok(events) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: json!({ "events": events }) }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
        )
            .into_response(),
    }
}


#[cfg(test)]
#[path = "actors_api_tests.rs"]
mod tests;
