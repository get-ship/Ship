//! Plain REST API — non-mesh endpoints (capabilities).
//!
//! The mesh.* op surface (mesh_register, mesh_send, mesh_broadcast,
//! mesh_discover, mesh_status_update, mesh_events) was removed with
//! the mesh.*/actor.* op cut. The L0 kernel mesh mechanism lives in
//! transport/connections.rs and is untouched.
//!
//! The `hooks_checkout` no-op was removed: actors carry their cwd,
//! sessions carry branch as a dimension, git is queried on demand —
//! nothing to reconcile on checkout.

use axum::{Extension, Json, response::IntoResponse};
use ship_substrate::events::CallerKind;
use serde::Serialize;

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
