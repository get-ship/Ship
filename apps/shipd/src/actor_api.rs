//! Actor spawning and event routing endpoints.
//!
//! These generalize mesh operations so the MCP process can delegate
//! actor lifecycle and event routing to the daemon's shared kernel.

use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use runtime::events::{EmitContext, EventEnvelope};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "actor_id": "agent-writer",
    "config": {
        "kind": "agent",
        "provider": "claude",
        "spec_path": ".ship/specs/write-readme.md"
    },
    "capabilities": ["writes.prose", "reads.repo"]
}))]
pub struct ActorSpawnReq {
    pub actor_id: String,
    #[schema(value_type = Object)]
    pub config: runtime::events::ActorConfig,
    /// Optional mesh capabilities for auto-registration.
    pub capabilities: Option<Vec<String>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "envelope": {
        "kind": "job.completed",
        "slug": "write-readme",
        "ts": "2026-04-22T10:15:00Z"
    },
    "workspace_id": "default",
    "session_id": "sess-2f9c"
}))]
pub struct EventRouteReq {
    #[schema(value_type = Object)]
    pub envelope: runtime::events::EventEnvelope,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
}

#[utoipa::path(
    operation_id = "actorSpawn",
    post,
    path = "/api/v1/actor/spawn",
    request_body = ActorSpawnReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "actor not found", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "actor already exists", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "actor"
)]
pub async fn actor_spawn(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ActorSpawnReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e;
    }
    {
        let mut kernel = state.kernel().lock().await;
        let _ = kernel.stop_actor(&req.actor_id);
        match kernel.spawn_actor(&req.actor_id, req.config) {
            Ok((_store, mailbox)) => {
                state.agent_mailboxes().lock().await
                    .insert(req.actor_id.clone(), mailbox);
            }
            Err(e) => {
                let status = match &e {
                    runtime::KernelError::ActorAlreadyExists { .. } => StatusCode::CONFLICT,
                    runtime::KernelError::ActorNotFound { .. } => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                return (status, Json(MeshResponse {
                    ok: false, data: serde_json::json!(e.to_string()),
                }));
            }
        }
    }

    if let Some(caps) = req.capabilities {
        let envelope = match EventEnvelope::new(
            "mesh.register",
            &req.actor_id,
            &serde_json::json!({ "agent_id": &req.actor_id, "capabilities": caps }),
        )
        .map(|e| e.with_actor_id(&req.actor_id))
        {
            Ok(e) => e,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse {
                ok: false, data: serde_json::json!(e.to_string()),
            })),
        };
        let ctx = auth.to_daemon_emit_context();
        if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
            tracing::warn!(actor_id = %req.actor_id, "mesh.register emit failed: {e}");
        }
    }

    (StatusCode::OK, Json(MeshResponse {
        ok: true, data: serde_json::json!(format!("spawned: {}", req.actor_id)),
    }))
}

#[utoipa::path(
    operation_id = "eventRoute",
    post,
    path = "/api/v1/events/route",
    request_body = EventRouteReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "actor"
)]
pub async fn event_route(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<EventRouteReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e;
    }
    // MCP tools route through the daemon's HTTP API. The MCP server is a
    // trusted Ship component — it validates caller permissions before emitting.
    // Use the authenticated caller kind but allow CLI callers (default) to route
    // system-namespace events (the bridge has already validated permissions).
    let ctx = EmitContext {
        caller_kind: auth.caller_kind,
        skill_id: None,
        workspace_id: req.workspace_id.or(auth.workspace_id),
        session_id: req.session_id.or(auth.session_id),
    };
    match state.kernel().lock().await.emit(req.envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("emitted") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}
