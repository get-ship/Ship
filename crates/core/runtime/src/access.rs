//! Sub-traits for the modular-runtime spec 03 (`RuntimeAccess`) substrate.
//!
//! The root `RuntimeAccess` trait composes per-concern sub-traits so a
//! handler that only needs `kernel()` doesn't pay the full god-state
//! surface. Three of those sub-traits live here because they reference
//! runtime-owned types only — `KernelRouter`, `SharedMeshRegistry`, and
//! filesystem paths the daemon was launched with.
//!
//! `ServicesAccess` and `AuthAccess` and the root `RuntimeAccess` itself
//! live in `apps/shipd/src/access.rs` because they reference shipd-side
//! types (the `*Service` traits, `SessionRegistry`). Promoting them to
//! the runtime crate happens when a second consumer (cloud worker) shows
//! up — the spec D5 "internal modules until external consumers" rule.

use std::path::Path;
use std::sync::Arc;

use crate::events::KernelRouter;
use crate::services::mesh::SharedMeshRegistry;

/// Shared kernel handle. Mirrors `apps/shipd/src/ship_runtime.rs::SharedKernel`
/// to keep the trait surface free of axum / shipd imports.
pub type SharedKernel = Arc<tokio::sync::Mutex<KernelRouter>>;

/// Kernel-bound access. Handlers that emit or subscribe to events go
/// through this rather than reaching into `state.kernel()` directly.
pub trait KernelAccess: Send + Sync {
    /// The shared kernel router. Locking semantics are unchanged — this
    /// is the same `Arc<Mutex<KernelRouter>>` callers already hold.
    fn kernel(&self) -> &SharedKernel;
}

/// Mesh-bound access. The mesh registry plus the agent-mailbox map that
/// lives next to it (kept together because every mesh handler needs both).
pub trait MeshAccess: Send + Sync {
    fn mesh_registry(&self) -> &SharedMeshRegistry;
}

/// Filesystem / launch-config access. Read-only — the daemon's roots
/// don't change after `ShipRuntime::new` returns.
pub trait ConfigAccess: Send + Sync {
    /// `~/.ship/` (or the override the daemon was started with).
    fn ship_dir(&self) -> &Path;
    /// Project workspace root, when the daemon was started with one.
    fn workspace_dir(&self) -> Option<&Path>;
}
