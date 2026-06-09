//! Plain REST API — non-mesh endpoints (capabilities).
//!
//! The `MeshResponse` envelope is the generic `(ok, data)` JSON shape
//! used by every `/api/v1/*` handler; the name is historical. Real
//! inter-agent messaging is now native: the kernel delivers directed
//! events via `EventEnvelope.target_actor_id` to actor mailboxes; the
//! mesh.* op surface is gone.
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
