//! Shared route-handler aliases — the `ApiState` / `SharedKernel` /
//! `MeshResponse` re-exports every `routes/*_api.rs` handler imports.
//!
//! The `MeshResponse` envelope is the generic `(ok, data)` JSON shape
//! used by every `/api/v1/*` handler; the name is historical. Real
//! inter-agent messaging is now native: the kernel delivers directed
//! events via `EventEnvelope.target_actor_id` to actor mailboxes; the
//! mesh.* op surface is gone.
//!
//! The `GET /api/v1/capabilities` handler was removed: zero consumers
//! (only shipd's own tests fetched it) and it served fabricated
//! tier/feature data naming retired concepts (jobs, workspaces).

/// Axum shared state — an Arc-wrapped ShipRuntime.
pub use crate::ship_runtime::ApiState;

// Re-export for use in other modules.
pub type SharedKernel = crate::ship_runtime::SharedKernel;

// MeshResponse re-export kept for handlers that still import from
// `crate::routes::rest_api::*`.
pub use crate::wire::MeshResponse;
