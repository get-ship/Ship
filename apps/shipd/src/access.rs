//! `RuntimeAccess` substrate (modular-runtime spec 03 PR 1).
//!
//! Replaces the implicit `Arc<ShipRuntime>` god-state with a composable
//! trait surface. Handlers state what they need (`R: RuntimeAccess` plus
//! the relevant sub-trait); production wires `ShipRuntime` through;
//! tests can build a `MockRuntimeAccess` once it lands.
//!
//! Three sub-traits (`KernelAccess`, `MeshAccess`, `ConfigAccess`) live
//! in the runtime crate so other consumers (cloud worker, eventually)
//! can implement them without depending on shipd. The other two
//! (`ServicesAccess`, `AuthAccess`) live here because they reference
//! shipd-side traits (`*Service`, `SessionRegistry`); they relocate
//! when a second consumer appears.
//!
//! The substrate ships in PR 1 — handlers haven't been migrated yet.
//! Direct field access through `Arc<ShipRuntime>` still works; the
//! trait shape exists so the next PR can flip handlers domain-by-domain.

use std::sync::Arc;

pub use ship_substrate::access::{ConfigAccess, KernelAccess, MeshAccess, SharedKernel};

use crate::auth::SessionRegistry;
use ship_agents::services::agents::service::AgentsService;
use ship_agents::services::mcp::service::McpService;
use ship_agents::services::permissions::service::PermissionsService;
use ship_pipeline_runtime::pipelines::service::PipelinesService;
use ship_agents::services::rules::service::RulesService;
use ship_agents::services::skills::service::SkillsService;
use crate::ship_runtime::ShipRuntime;

/// Per-domain service surface. Each migrated mr-02 service appears here
/// as a trait getter. Handlers reach `rt.services().rules()` instead of
/// `state.rules()`; the only thing that changes between v1 and v2 of
/// `RuntimeAccess` is which type the `State<…>` extractor binds.
pub trait ServicesAccess: Send + Sync {
    fn rules(&self) -> &dyn RulesService;
    fn permissions(&self) -> &dyn PermissionsService;
    fn mcp(&self) -> &dyn McpService;
    fn skills(&self) -> &dyn SkillsService;
    fn agents(&self) -> &dyn AgentsService;
    fn pipelines(&self) -> &dyn PipelinesService;
    /// Build a `CollectionService` over the active-space `SpaceContext` + shared
    /// schema registry (C-2 §3.1). Errors cleanly when the daemon is space-less
    /// (no active space) rather than fabricate a `global` partition (S2).
    fn collections(&self) -> anyhow::Result<ship_collection::service::CollectionService>;
    /// Build an `ArtifactService` over the active-space `SpaceContext` + blob
    /// root. Errors cleanly when the daemon is space-less.
    fn artifacts(&self) -> anyhow::Result<ship_artifacts::service::ArtifactService>;
}

/// Auth + session registry surface. Today this is just the scoped
/// session registry — verify_token lives on `AuthContext` extension
/// extractors and stays there until handlers actually request it from
/// the trait.
pub trait AuthAccess: Send + Sync {
    fn session_registry(&self) -> &SessionRegistry;
}

/// Root runtime-access trait. Composed of `ServicesAccess` (shipd-side
/// services), `KernelAccess` (event router), `MeshAccess` (mesh
/// registry), `AuthAccess` (session registry), and `ConfigAccess`
/// (filesystem roots).
pub trait RuntimeAccess: Send + Sync + 'static {
    fn services(&self) -> &dyn ServicesAccess;
    fn kernel_access(&self) -> &dyn KernelAccess;
    fn mesh_access(&self) -> &dyn MeshAccess;
    fn auth(&self) -> &dyn AuthAccess;
    fn config(&self) -> &dyn ConfigAccess;
}

// ── Production impls on ShipRuntime ─────────────────────────────────

impl ServicesAccess for ShipRuntime {
    fn rules(&self) -> &dyn RulesService {
        ShipRuntime::rules(self)
    }
    fn permissions(&self) -> &dyn PermissionsService {
        ShipRuntime::permissions(self)
    }
    fn mcp(&self) -> &dyn McpService {
        ShipRuntime::mcp(self)
    }
    fn skills(&self) -> &dyn SkillsService {
        ShipRuntime::skills(self)
    }
    fn agents(&self) -> &dyn AgentsService {
        ShipRuntime::agents(self)
    }
    fn pipelines(&self) -> &dyn PipelinesService {
        ShipRuntime::pipelines(self)
    }
    fn collections(&self) -> anyhow::Result<ship_collection::service::CollectionService> {
        ShipRuntime::collections(self)
    }
    fn artifacts(&self) -> anyhow::Result<ship_artifacts::service::ArtifactService> {
        ShipRuntime::artifacts(self)
    }
}

impl KernelAccess for ShipRuntime {
    fn kernel(&self) -> &SharedKernel {
        ShipRuntime::kernel(self)
    }
}

impl MeshAccess for ShipRuntime {
    fn mesh_registry(&self) -> &ship_substrate::services::mesh::SharedMeshRegistry {
        ShipRuntime::mesh_registry(self)
    }
}

impl AuthAccess for ShipRuntime {
    fn session_registry(&self) -> &SessionRegistry {
        ShipRuntime::session_registry(self)
    }
}

impl ConfigAccess for ShipRuntime {
    fn ship_dir(&self) -> &std::path::Path {
        ShipRuntime::ship_dir(self)
    }
    fn workspace_dir(&self) -> Option<&std::path::Path> {
        ShipRuntime::workspace_dir(self)
    }
}

impl RuntimeAccess for ShipRuntime {
    fn services(&self) -> &dyn ServicesAccess {
        self
    }
    fn kernel_access(&self) -> &dyn KernelAccess {
        self
    }
    fn mesh_access(&self) -> &dyn MeshAccess {
        self
    }
    fn auth(&self) -> &dyn AuthAccess {
        self
    }
    fn config(&self) -> &dyn ConfigAccess {
        self
    }
}

// `Arc<R: RuntimeAccess>` — axum's `State<Arc<R>>` derefs to `&R`, so
// every accessor on `RuntimeAccess` is reachable directly. No blanket
// impl needed; the deref chain handles it.

/// `RuntimeAccess` blanket impl over `Arc<R>` so handlers can take
/// `State<Arc<R>>` and call `state.services()` without an explicit
/// deref. Only matters when the migration starts using the generic
/// extractor — production registration today still uses `ApiState`
/// (concrete `Arc<ShipRuntime>`).
impl<R: RuntimeAccess + ?Sized> RuntimeAccess for Arc<R> {
    fn services(&self) -> &dyn ServicesAccess {
        (**self).services()
    }
    fn kernel_access(&self) -> &dyn KernelAccess {
        (**self).kernel_access()
    }
    fn mesh_access(&self) -> &dyn MeshAccess {
        (**self).mesh_access()
    }
    fn auth(&self) -> &dyn AuthAccess {
        (**self).auth()
    }
    fn config(&self) -> &dyn ConfigAccess {
        (**self).config()
    }
}
