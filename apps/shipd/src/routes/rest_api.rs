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



/// Axum shared state — an Arc-wrapped ShipRuntime.
pub use crate::ship_runtime::ApiState;

// Re-export for use in other modules.
pub type SharedKernel = crate::ship_runtime::SharedKernel;

// MeshResponse re-export kept for handlers that still import from
// `crate::routes::rest_api::*`.
pub use crate::wire::MeshResponse;

