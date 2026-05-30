//! Plain REST API for ship-mcp relay clients.
//!
//! These endpoints bypass the MCP protocol — no handshake needed.
//! ship-mcp calls these to forward mesh operations to the shared kernel.

use axum::{Extension, Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use axum::response::sse::{Event, KeepAlive, Sse};
use ship_substrate::events::{CallerKind, EventEnvelope};
use serde::{Deserialize, Serialize};

use crate::auth::{AuthContext, enforce_mcp_identity};

/// Axum shared state — an Arc-wrapped ShipRuntime.
pub use crate::ship_runtime::ApiState;

// Re-export for use in other modules.
pub type SharedKernel = crate::ship_runtime::SharedKernel;

// ---- Request / Response types ----

#[derive(Deserialize)]
pub struct MeshSendReq {
    pub from: String,
    pub to: String,
    /// Arbitrary JSON message body. Delivered verbatim to the target agent's
    /// `mesh.send` event; schema is caller-defined.
    pub body: serde_json::Value,
}

#[derive(Deserialize)]
pub struct MeshBroadcastReq {
    pub from: String,
    /// Arbitrary JSON broadcast payload. Delivered to every agent whose
    /// capabilities match `capability_filter` (or all agents when unset).
    pub body: serde_json::Value,
    pub capability_filter: Option<String>,
}

#[derive(Deserialize)]
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

pub async fn mesh_send(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshSendReq>,
) -> axum::response::Response {
    if let Err(e) = enforce_mcp_identity(&auth, Some(&req.from)) {
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
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })).into_response(),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("sent") })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })).into_response(),
    }
}

pub async fn mesh_broadcast(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshBroadcastReq>,
) -> axum::response::Response {
    if let Err(e) = enforce_mcp_identity(&auth, Some(&req.from)) {
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
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })).into_response(),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("broadcast sent") })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })).into_response(),
    }
}

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

pub async fn mesh_status_update(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<MeshStatusReq>,
) -> axum::response::Response {
    // Scope check is owned by job #80 (compound issue); this dispatch
    // only adds identity binding so workers can't update another agent's
    // status. Daemon-token callers (CLI/Studio) pass through.
    if let Err(e) = enforce_mcp_identity(&auth, Some(&req.agent_id)) {
        return e;
    }
    let envelope = match EventEnvelope::new(
        "mesh.status",
        &req.agent_id,
        &serde_json::json!({ "agent_id": &req.agent_id, "status": req.status }),
    )
    .map(|e| e.with_actor_id(&req.agent_id))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })).into_response(),
    };

    let ctx = auth.to_daemon_emit_context();
    match state.kernel().lock().await.emit(envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("status updated") })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })).into_response(),
    }
}

// ---- Capabilities ----

#[derive(Serialize)]
pub struct CapabilitiesResponse {
    pub authenticated: bool,
    pub mode: String,
    pub tier: String,
    pub features: Vec<String>,
}

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

#[derive(Deserialize)]
pub struct CheckoutHookReq {
    pub branch: String,
    pub path: String,
}

pub async fn hooks_checkout(
    Json(req): Json<CheckoutHookReq>,
) -> impl IntoResponse {
    let ship_dir_result = ship_substrate::project::get_project_dir(Some(std::path::PathBuf::from(&req.path)));
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
pub async fn mesh_events(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(agent_id): Path<String>,
) -> axum::response::Response {
    if let Err(e) = enforce_mcp_identity(&auth, Some(&agent_id)) {
        return e;
    }
    let mailbox = state.agent_mailboxes().lock().await.remove(&agent_id);
    let Some(mut mailbox) = mailbox else {
        return (
            StatusCode::NOT_FOUND,
            format!("no mailbox for agent '{agent_id}' — register first"),
        ).into_response();
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

    Sse::new(stream).keep_alive(KeepAlive::default()).into_response()
}

#[cfg(test)]
#[path = "rest_api_tests.rs"]
mod tests;
