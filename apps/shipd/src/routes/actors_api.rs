//! `GET /api/v1/actors` — unified read surface for running actors
//! (chat agent, pipeline step, shell, service).
//!
//! Read endpoint projects from the `actors` table maintained by
//! `ship_substrate::projections::ActorProjection`. Liveness derives from
//! `last_event_at` (bumped by SQLite triggers on every event correlated
//! to the actor) — no heartbeat loop. The mesh.* op surface (mesh/agents/{id},
//! mesh/agents/{id}/events, mesh/register, mesh/send, mesh/broadcast,
//! mesh/discover, mesh/status, mesh/events/{id}) and the
//! `/api/v1/actors/{actor_id}` + `/api/v1/actors/{actor_id}/events`
//! introspection reads were removed: the kernel IS the mailbox primitive
//! (`EventEnvelope.target_actor_id` for directed delivery), the Connect
//! ops surface is the replacement, and the dashboard already has the
//! live `ListActors` projection it needs.

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ship_substrate::projections::actor::{ListActorsFilter, list_actors};
use serde::Deserialize;
use serde_json::json;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;
use crate::wire::MeshResponse;

#[derive(Deserialize)]
pub struct ListQuery {
    pub workspace_id: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/actors` — list all actors, ordered by recency. The
/// `live` field on each row is overlaid from the kernel's mailbox
/// map at query time — the kernel's actor registry IS the source of
/// truth for "alive and reachable." If an actor is in
/// `kernel.list_actors()`, you can send to it and it will receive.
/// If not, you can't. Period.
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
