//! Plain REST API — non-mesh endpoints (capabilities, hooks).
//!
//! The mesh.* op surface (mesh_register, mesh_send, mesh_broadcast,
//! mesh_discover, mesh_status_update, mesh_events) was removed with
//! the mesh.*/actor.* op cut. The L0 kernel mesh mechanism lives in
//! transport/connections.rs and is untouched.

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use ship_substrate::events::CallerKind;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;

/// Axum shared state — an Arc-wrapped ShipRuntime.
pub use crate::ship_runtime::ApiState;

// Re-export for use in other modules.
pub type SharedKernel = crate::ship_runtime::SharedKernel;

// MeshResponse re-export kept for handlers that still import from
// `crate::routes::rest_api::*`.
pub use crate::wire::MeshResponse;

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
