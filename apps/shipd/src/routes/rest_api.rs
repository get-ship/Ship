//! Plain REST API for ship-mcp relay clients.
//!
//! These endpoints bypass the MCP protocol — no handshake needed.
//! ship-mcp calls these to forward mesh operations to the shared kernel.

use axum::{Extension, Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use axum::response::sse::{Event, KeepAlive, Sse};
use ship_substrate::events::{CallerKind, EventEnvelope};
use serde::{Deserialize, Serialize};

use crate::auth::{AuthContext, Scope, require_scope};

/// Axum shared state — an Arc-wrapped ShipRuntime.
pub use crate::ship_runtime::ApiState;

// Re-export for use in other modules.
pub type SharedKernel = crate::ship_runtime::SharedKernel;

// ---- Request / Response types ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "agent_id": "agent-writer",
    "capabilities": ["writes.prose", "reads.repo"]
}))]
pub struct MeshRegisterReq {
    pub agent_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "from": "agent-planner",
    "to": "agent-writer",
    "body": { "task": "draft README section: overview" }
}))]
pub struct MeshSendReq {
    pub from: String,
    pub to: String,
    /// Arbitrary JSON message body. Delivered verbatim to the target agent's
    /// `mesh.send` event; schema is caller-defined.
    #[schema(value_type = Object)]
    pub body: serde_json::Value,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "from": "agent-planner",
    "body": { "announcement": "build step complete" },
    "capability_filter": "reads.repo"
}))]
pub struct MeshBroadcastReq {
    pub from: String,
    /// Arbitrary JSON broadcast payload. Delivered to every agent whose
    /// capabilities match `capability_filter` (or all agents when unset).
    #[schema(value_type = Object)]
    pub body: serde_json::Value,
    pub capability_filter: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "agent_id": "agent-writer", "status": "idle" }))]
pub struct MeshStatusReq {
    pub agent_id: String,
    pub status: String,
}

// MeshResponse moved to `crate::wire::MeshResponse` (Phase 3 — module-DAG
// fix: `auth` couldn't reference shipd's routes/* without violating the
// layering). Kept as a re-export for handlers that still import from
// `crate::routes::rest_api::*`.
pub use crate::wire::MeshResponse;

// ---- Handlers ----

#[utoipa::path(
    operation_id = "meshRegister",
    post,
    path = "/api/v1/mesh/register",
    request_body = MeshRegisterReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::wire::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "mesh"
)]
pub async fn mesh_register(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshRegisterReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e;
    }
    // MCP callers must register as their authenticated identity — no impersonation.
    if auth.caller_kind == CallerKind::Mcp
        && let Some(ref bound_id) = auth.agent_id
        && bound_id != &req.agent_id
    {
        return (
            StatusCode::FORBIDDEN,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(format!(
                    "identity mismatch: authenticated as '{}', cannot register as '{}'",
                    bound_id, req.agent_id
                )),
            }),
        );
    }

    // Spawn (or re-spawn) an actor in the kernel for this remote agent
    // so messages with target_actor_id can be delivered to its mailbox.
    let mailbox = {
        let mut kernel = state.kernel().lock().await;
        // Stop existing actor so we get a fresh mailbox for the SSE endpoint.
        let _ = kernel.stop_actor(&req.agent_id);
        let config = ship_substrate::events::ActorConfig {
            namespace: req.agent_id.clone(),
            write_namespaces: vec!["mesh.".to_string()],
            read_namespaces: vec!["mesh.".to_string()],
            subscribe_namespaces: vec!["mesh.".to_string()],
        };
        match kernel.spawn_actor(&req.agent_id, config) {
            Ok((_store, mailbox)) => Some(mailbox),
            Err(ship_substrate::errors::KernelError::ActorAlreadyExists { .. }) => {
                tracing::warn!(agent_id = %req.agent_id, "actor already exists after stop");
                None
            }
            Err(e) => {
                tracing::warn!(agent_id = %req.agent_id, "actor spawn failed: {e}");
                None
            }
        }
    }; // kernel lock drops here

    if let Some(mailbox) = mailbox {
        state.agent_mailboxes().lock().await
            .insert(req.agent_id.clone(), mailbox);
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(format!("actor spawn failed for '{}'", req.agent_id)),
            }),
        );
    }

    let envelope = match EventEnvelope::new(
        "mesh.register",
        &req.agent_id,
        &serde_json::json!({ "agent_id": &req.agent_id, "capabilities": req.capabilities }),
    )
    .map(|e| e.with_actor_id(&req.agent_id))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!(format!("registered: {}", req.agent_id)) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

#[utoipa::path(
    operation_id = "meshSend",
    post,
    path = "/api/v1/mesh/send",
    request_body = MeshSendReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::wire::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 404, description = "target agent not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "mesh"
)]
pub async fn mesh_send(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshSendReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e;
    }
    // Route through MeshService via mesh.send event type so it validates
    // the target and sets target_actor_id for directed delivery.
    let envelope = match EventEnvelope::new(
        "mesh.send",
        &req.to,
        &serde_json::json!({ "from": &req.from, "to": &req.to, "body": req.body }),
    )
    .map(|e| e.with_actor_id(&req.from))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("sent") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

#[utoipa::path(
    operation_id = "meshBroadcast",
    post,
    path = "/api/v1/mesh/broadcast",
    request_body = MeshBroadcastReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::wire::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "mesh"
)]
pub async fn mesh_broadcast(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshBroadcastReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshBroadcast) {
        return e;
    }
    let mut payload = serde_json::json!({ "from": &req.from, "body": req.body });
    if let Some(ref cap) = req.capability_filter {
        payload["capability_filter"] = serde_json::json!(cap);
    }
    let envelope = match EventEnvelope::new("mesh.broadcast", &req.from, &payload)
        .map(|e| e.with_actor_id(&req.from))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("broadcast sent") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

#[utoipa::path(
    operation_id = "meshDiscover",
    get,
    path = "/api/v1/mesh/discover",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "mesh"
)]
pub async fn mesh_discover(
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let registry = state.mesh_registry().read().await;
    let agents: Vec<serde_json::Value> = registry
        .values()
        .map(|e| {
            serde_json::json!({
                "agent_id": e.agent_id,
                "label": e.label,
                "capabilities": e.capabilities,
                "status": e.status,
            })
        })
        .collect();
    (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!({ "agents": agents }) }))
}

#[utoipa::path(
    operation_id = "meshStatusUpdate",
    post,
    path = "/api/v1/mesh/status",
    request_body = MeshStatusReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::wire::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "mesh"
)]
pub async fn mesh_status_update(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshStatusReq>,
) -> impl IntoResponse {
    let envelope = match EventEnvelope::new(
        "mesh.status",
        &req.agent_id,
        &serde_json::json!({ "agent_id": &req.agent_id, "status": req.status }),
    )
    .map(|e| e.with_actor_id(&req.agent_id))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("status updated") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

// ---- Capabilities ----

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "authenticated": true,
    "mode": "local",
    "tier": "owner",
    "features": ["mesh", "artifacts", "collections", "pipelines"]
}))]
pub struct CapabilitiesResponse {
    pub authenticated: bool,
    pub mode: String,
    pub tier: String,
    pub features: Vec<String>,
}

#[utoipa::path(
    operation_id = "capabilities",
    get,
    path = "/api/v1/capabilities",
    responses(
        (status = 200, body = CapabilitiesResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "capabilities"
)]
pub async fn capabilities(
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    let mode = match auth.caller_kind {
        CallerKind::Sdk => "cloud",
        _ => "local",
    };

    Json(CapabilitiesResponse {
        authenticated: true,
        mode: mode.to_string(),
        tier: "free".to_string(),
        features: vec![
            "workspaces".to_string(),
            "pipelines".to_string(),
            "jobs".to_string(),
            "events".to_string(),
        ],
    })
}

// ---- Checkout hook ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "branch": "job/write-readme",
    "path": "/home/me/project"
}))]
pub struct CheckoutHookReq {
    pub branch: String,
    pub path: String,
}

#[utoipa::path(
    operation_id = "hooksCheckout",
    post,
    path = "/api/v1/hooks/checkout",
    request_body = CheckoutHookReq,
    responses(
        (status = 200, description = "Workspace reconciled for the checked-out branch", body = MeshResponse),
        (status = 400, description = "Invalid project path", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "hooks"
)]
pub async fn hooks_checkout(
    Json(req): Json<CheckoutHookReq>,
) -> impl IntoResponse {
    let ship_dir_result = ship_application::project::get_project_dir(Some(std::path::PathBuf::from(&req.path)));
    let ship_dir = match ship_dir_result {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("no .ship/ found: {e}")),
                }),
            );
        }
    };

    let _ = ship_dir;
    // The workspace primitive retired; this hook used to refresh workspace
    // state on git checkout. Now there's nothing to reconcile — actors carry
    // their cwd, sessions carry branch as a dimension, git is queried on
    // demand. The endpoint stays as an OK no-op for backward compat.
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "branch": req.branch }),
        }),
    )
}

/// SSE endpoint — drains an agent's mailbox as server-sent events.
/// The agent must register first (which spawns the actor + stashes the mailbox).
/// Only one SSE connection per agent is supported (mailbox is taken, not cloned).
#[utoipa::path(
    operation_id = "meshEvents",
    get,
    path = "/api/v1/mesh/events/{agent_id}",
    params(("agent_id" = String, Path, description = "Agent ID")),
    responses(
        (status = 200, description = "SSE stream of mesh events"),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 404, description = "no mailbox for agent — register first", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "mesh"
)]
pub async fn mesh_events(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    let mailbox = state.agent_mailboxes().lock().await.remove(&agent_id);
    let Some(mut mailbox) = mailbox else {
        return Err((
            StatusCode::NOT_FOUND,
            format!("no mailbox for agent '{agent_id}' — register first"),
        ));
    };

    let stream = async_stream::stream! {
        while let Some(event) = mailbox.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default()
                    .event("mesh.event")
                    .data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
#[path = "rest_api_tests.rs"]
mod tests;
