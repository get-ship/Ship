//! `/api/actors/*` — unified read/control surface for every running
//! actor (chat agent, pipeline step, shell, service).
//!
//! Read endpoints project from the `actors` table maintained by
//! `runtime::projections::ActorProjection`. Liveness derives from
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
use runtime::events::EventEnvelope;
use runtime::events::types::event_types;
use runtime::projections::actor::{
    ListActorsFilter, actor_status, get_actor, list_actor_events, list_actors,
};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize)]
pub struct ListQuery {
    pub workspace_id: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/actors` — list all actors, ordered by recency.
pub async fn list_actors_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    let filter = ListActorsFilter {
        workspace_id: q.workspace_id,
        kind: q.kind,
        status: q.status,
        limit: q.limit.unwrap_or(0),
    };
    match list_actors(&filter) {
        Ok(actors) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: json!({ "actors": actors }) }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
        )
            .into_response(),
    }
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ActorInputReq {
    pub text: String,
}

/// `POST /api/actors/{id}/input` — push a turn into the actor's mailbox
/// via directed-delivery on `studio.chat_turn`. 404 if the actor isn't
/// in the projection; 409 if it's stopped or crashed.
pub async fn actor_input_handler(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(req): Json<ActorInputReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e.into_response();
    }
    let text = req.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: json!("empty text") }),
        )
            .into_response();
    }

    let (status, workspace_id): (String, Option<String>) = match actor_status(&id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: json!(format!("actor '{id}' not found")),
                }),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: json!(format!("status: {e}")) }),
            )
                .into_response();
        }
    };
    if status == "stopped" || status == "crashed" {
        return (
            StatusCode::CONFLICT,
            Json(MeshResponse {
                ok: false,
                data: json!({ "error": "actor not deliverable", "status": status }),
            }),
        )
            .into_response();
    }

    let payload = json!({ "actor_id": id, "text": text });
    let envelope = match EventEnvelope::new(event_types::STUDIO_CHAT_TURN, &id, &payload) {
        Ok(e) => e
            .with_context(workspace_id.as_deref(), Some(&id))
            .with_target(&id),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: json!(format!("envelope: {e}")) }),
            )
                .into_response();
        }
    };
    let ctx = auth.to_daemon_emit_context();
    if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("emit: {e}")) }),
        )
            .into_response();
    }
    (StatusCode::ACCEPTED, Json(MeshResponse { ok: true, data: json!({}) }))
        .into_response()
}
