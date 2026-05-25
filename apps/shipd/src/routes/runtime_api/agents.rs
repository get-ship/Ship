//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::auth::{AuthContext};
use crate::routes::rest_api::{ApiState, MeshResponse};

/// GET /api/v1/runtime/agents
pub async fn list_agents(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> impl IntoResponse {
    // An agent is either (a) defined on disk as a .ship/agents/<id> profile,
    // (b) live in the mesh registry (registered via REST/MCP), or both.
    // Return the union keyed by agent_id so tests that populate only the
    // mesh see live entries, and production users see their compiled profiles.
    //
    // Uses state.ship_dir() — walking up from CWD bleeds the main repo's
    // agents into isolated test harnesses or any caller running from a
    // different working directory.
    let mut by_id: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();

    let agents_dir = state.ship_dir().join("agents");
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".jsonc") && !name.ends_with(".toml") {
                continue;
            }
            let id = name.trim_end_matches(".jsonc").trim_end_matches(".toml").to_string();
            let label = std::fs::read_to_string(entry.path())
                .ok()
                .and_then(|content| {
                    let cleaned: String = content
                        .lines()
                        .map(|l| if l.trim_start().starts_with("//") { "" } else { l })
                        .collect::<Vec<_>>()
                        .join("\n");
                    serde_json::from_str::<serde_json::Value>(&cleaned).ok()
                })
                .and_then(|v| v["name"].as_str().map(str::to_string))
                .unwrap_or_else(|| id.clone());
            by_id.insert(id.clone(), serde_json::json!({ "agent_id": id, "label": label }));
        }
    }

    // Mesh registry entries. If an id already has a profile, mesh adds
    // status/capabilities to the existing entry; otherwise a fresh row.
    let mesh = state.mesh_registry().read().await;
    for (id, entry) in mesh.iter() {
        let mut merged = by_id.remove(id).unwrap_or_else(|| {
            serde_json::json!({ "agent_id": id, "label": entry.label.clone() })
        });
        merged["status"] = serde_json::json!(format!("{:?}", entry.status).to_lowercase());
        merged["capabilities"] = serde_json::json!(entry.capabilities);
        by_id.insert(id.clone(), merged);
    }

    let agents: Vec<serde_json::Value> = by_id.into_values().collect();
    (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!({ "agents": agents }) }))
        .into_response()
}

// ---- Agent control ----

#[derive(Deserialize)]
pub struct SteerAgentRequest {
    pub message: String,
}

/// POST /api/v1/runtime/agents/{slug}/stop — stop a running agent by slug.
///
/// Uses the process registry and executor to terminate the agent process.
pub async fn stop_agent(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let handle = {
        let mut reg = dh.registry.lock().await;
        match reg.take(&slug) {
            Some(h) => h,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
                );
            }
        }
    };

    match dh.executor.stop(&handle).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("stopped: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to stop agent: {e}")) }),
        ),
    }
}

/// POST /api/v1/runtime/agents/{slug}/steer — inject a message into a running agent.
pub async fn steer_agent(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<SteerAgentRequest>,
) -> impl IntoResponse {
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let reg = dh.registry.lock().await;
    let handle = match reg.get(&slug) {
        Some(h) => h,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
            );
        }
    };

    match dh.executor.send(handle, &req.message).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("steered: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to steer agent: {e}")) }),
        ),
    }
}

