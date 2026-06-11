//! ShipRuntime simple accessor methods. Lazy managers and tasks are in `tasks`.

use std::path::Path;
use std::sync::Arc;

use ship_substrate::task_health::HealthRegistry;
#[cfg(feature = "ship-code")]
use crate::dispatch::run_dispatch::DispatchHandle;

use super::{
    ShipRuntime, RuntimeConfig, SharedKernel,
};

impl ShipRuntime {
    /// Process uptime since the runtime was constructed.
    pub fn uptime(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    /// Install the shutdown cancellation token (#256). Called once in
    /// `run_network` with a child of the daemon's signal token. Idempotent —
    /// a second set is ignored.
    pub fn set_shutdown_token(&self, token: tokio_util::sync::CancellationToken) {
        let _ = self.shutdown_token.set(token);
    }

    /// A child of the daemon shutdown token (#256). SSE handlers `select!`
    /// their long-lived loops against `.cancelled()` so a SIGTERM closes them
    /// promptly. Returns a detached child token when none is installed (tests),
    /// so handlers always have something to await.
    pub fn shutdown_token(&self) -> tokio_util::sync::CancellationToken {
        self.shutdown_token
            .get()
            .map(|t| t.child_token())
            .unwrap_or_default()
    }

    pub fn kernel(&self) -> &SharedKernel {
        &self.kernel
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Absolute origin (scheme+host+port) the daemon is reachable at.
    ///
    /// Used by HTML view templates so Datastar `@get`/`@post` calls resolve
    /// against the daemon regardless of whether the page is inline-mounted
    /// into another origin (e.g. Studio at :3000).
    pub fn daemon_origin(&self) -> String {
        let host = if self.config.host == "0.0.0.0" {
            "localhost".to_string()
        } else {
            self.config.host.clone()
        };
        format!("http://{}:{}", host, self.config.port)
    }

    pub fn ship_dir(&self) -> &Path {
        &self.ship_dir
    }

    pub fn daemon_dir(&self) -> &Path {
        &self.daemon_dir
    }

    /// `SkillsService` accessor.
    pub fn skills(&self) -> &dyn ship_agents::services::skills::service::SkillsService {
        &*self.skills
    }

    /// `RulesService` as a cloned `Arc` — for wire mounts that need to own the handle.
    pub fn rules_arc(
        &self,
    ) -> Arc<dyn ship_agents::services::rules::service::RulesService> {
        self.rules.clone()
    }

    /// `McpService` as a cloned `Arc` — for wire mounts.
    pub fn mcp_arc(&self) -> Arc<dyn ship_agents::services::mcp::service::McpService> {
        self.mcp.clone()
    }

    /// `PermissionsService` as a cloned `Arc` — for wire mounts.
    pub fn permissions_arc(
        &self,
    ) -> Arc<dyn ship_agents::services::permissions::service::PermissionsService> {
        self.permissions.clone()
    }

    /// `SkillsService` as a cloned `Arc` — for wire mounts.
    pub fn skills_arc(
        &self,
    ) -> Arc<dyn ship_agents::services::skills::service::SkillsService> {
        self.skills.clone()
    }

    /// `AgentsService` accessor.
    pub fn agents(&self) -> &dyn ship_agents::services::agents::service::AgentsService {
        &*self.agents
    }

    /// `AgentsService` as a cloned `Arc` — for the crate-owned `AgentsWire`.
    pub fn agents_arc(
        &self,
    ) -> std::sync::Arc<dyn ship_agents::services::agents::service::AgentsService> {
        self.agents.clone()
    }

    /// `PipelinesService` as a cloned `Arc` — for diagnostic snapshot providers
    /// that need to hold the handle independently of the runtime borrow.
    pub fn pipelines_arc(
        &self,
    ) -> Arc<dyn ship_pipeline_runtime::pipelines::service::PipelinesService> {
        self.pipelines.clone()
    }

    /// Build a `CollectionService` over the active-space `SpaceContext` + the
    /// shared schema registry. Cheap + stateless-but-for-ctx — constructed per
    /// call (C-2 §3.1). Errors cleanly when the daemon is space-less (no active
    /// space) rather than fabricate a `global` partition (S2 home-space kill).
    pub fn collections(&self) -> anyhow::Result<ship_collection::service::CollectionService> {
        let ctx = self.collection_ctx.clone().ok_or_else(|| {
            anyhow::anyhow!("no active space: collections are unavailable in space-less mode")
        })?;
        Ok(ship_collection::service::CollectionService {
            ctx,
            schemas: self.schemas.clone(),
        })
    }

    /// Build an `ArtifactService` over the active-space `SpaceContext` + blob
    /// root. Errors cleanly when the daemon is space-less.
    pub fn artifacts(&self) -> anyhow::Result<ship_artifacts::service::ArtifactService> {
        let ctx = self.artifact_ctx.clone().ok_or_else(|| {
            anyhow::anyhow!("no active space: artifacts are unavailable in space-less mode")
        })?;
        Ok(ship_artifacts::service::ArtifactService::new(
            ctx,
            self.global_blob_dir.clone(),
        ))
    }

    /// The shared collection schema registry (C-2 UD-7) — used by the wire mount
    /// and any caller that needs the same registry instance.
    pub fn collection_schemas(
        &self,
    ) -> Arc<ship_collection::projection::SchemaRegistry> {
        self.schemas.clone()
    }

    /// The active-space collection `SpaceContext` — used by the wire mount.
    /// `None` when the daemon booted space-less; the wire mount is skipped.
    pub fn collection_ctx(&self) -> Option<Arc<ship_substrate::ports::SpaceContext>> {
        self.collection_ctx.clone()
    }

    /// The active-space artifact `SpaceContext` — used by the wire mount + GC
    /// sweep. `None` when the daemon booted space-less.
    pub fn artifact_ctx(&self) -> Option<Arc<ship_substrate::ports::SpaceContext>> {
        self.artifact_ctx.clone()
    }

    /// Blob root for artifacts (global ship dir).
    pub fn global_blob_dir(&self) -> &std::path::Path {
        &self.global_blob_dir
    }
    /// Store the dispatch handle (set once after job manager init).
    #[cfg(feature = "ship-code")]
    pub fn set_dispatch_handle(&self, handle: DispatchHandle) {
        let _ = self.dispatch_handle.set(handle);
    }
    #[cfg(feature = "ship-code")]
    pub fn dispatch_handle(&self) -> Option<&DispatchHandle> {
        self.dispatch_handle.get()
    }

    pub fn pty_sessions(&self) -> &ship_agents::services::sessions::SessionRegistry {
        &self.pty_sessions
    }

    pub fn actor_sessions(&self) -> &ship_agents::services::sessions::ActorRegistry {
        &self.actor_sessions
    }

    pub fn session_registry(&self) -> &crate::auth::SessionRegistry {
        &self.session_registry
    }

    pub fn health_registry(&self) -> &HealthRegistry {
        &self.health_registry
    }

    pub fn view_sessions(&self) -> &crate::views::session::SharedViewSessionRegistry {
        &self.view_sessions
    }

    /// Attached project workspace, if the daemon was launched with one.
    pub fn workspace_dir(&self) -> Option<&Path> {
        self.workspace_dir.as_deref()
    }

    /// The live-read resolver registry — SSOT for snapshot `(domain, op)` reads,
    /// consumed by both the per-op `<Op>Live` streams and the multiplex.
    pub fn live_resolvers(&self) -> &Arc<crate::live_resolvers::LiveResolverRegistry> {
        &self.live_resolvers
    }
}
