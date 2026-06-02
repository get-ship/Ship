//! ShipRuntime simple accessor methods. Lazy managers and tasks are in `tasks`.

use std::path::Path;
use std::sync::Arc;

use ship_platform::services::mesh::SharedMeshRegistry;
use ship_substrate::task_health::HealthRegistry;
use crate::supervisor::run_dispatch::DispatchHandle;

use super::{
    ShipRuntime, RuntimeConfig, SharedKernel, AgentMailboxes, PtyConnections,
};

impl ShipRuntime {
    /// Resolve a workspace-id (branch name) to its worktree path.
    /// Wraps the configured `worktree_resolver` with a 404 response on miss.
    pub fn resolve_worktree(
        &self,
        id: &str,
    ) -> Result<std::path::PathBuf, (axum::http::StatusCode, axum::Json<crate::wire::MeshResponse>)>
    {
        (self.worktree_resolver)(id).ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                axum::Json(crate::wire::MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("workspace '{id}' not found")),
                }),
            )
        })
    }

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

    /// `RulesService` accessor — pilot for the modular-runtime spec 02
    /// service-layer pattern. HTTP handlers in `services::rules::*`
    /// call through this rather than reaching into runtime state.
    pub fn rules(&self) -> &dyn ship_platform::services::rules::service::RulesService {
        &*self.rules
    }

    /// `PermissionsService` accessor — second mr-02 pilot.
    pub fn permissions(
        &self,
    ) -> &dyn ship_platform::services::permissions::service::PermissionsService {
        &*self.permissions
    }

    /// `McpService` accessor — third mr-02 pilot.
    pub fn mcp(&self) -> &dyn ship_platform::services::mcp::service::McpService {
        &*self.mcp
    }

    /// `SkillsService` accessor.
    pub fn skills(&self) -> &dyn ship_platform::services::skills::service::SkillsService {
        &*self.skills
    }

    /// `AgentsService` accessor.
    pub fn agents(&self) -> &dyn ship_platform::services::agents::service::AgentsService {
        &*self.agents
    }

    /// `PipelinesService` accessor — definition CRUD; dispatch / cancel /
    /// runs still go through `runtime_api`.
    pub fn pipelines(&self) -> &dyn ship_platform::services::pipelines::service::PipelinesService {
        &*self.pipelines
    }

    /// `PipelinesService` as a cloned `Arc` — for diagnostic snapshot providers
    /// that need to hold the handle independently of the runtime borrow.
    pub fn pipelines_arc(
        &self,
    ) -> Arc<dyn ship_platform::services::pipelines::service::PipelinesService> {
        self.pipelines.clone()
    }

    /// `GitService` accessor — every git op except SSE goes through this.
    pub fn git(&self) -> &dyn ship_platform::services::git::service::GitService {
        &*self.git
    }

    /// `CollectionsService` accessor — db-bound CRUD + query.
    pub fn collections(
        &self,
    ) -> &dyn ship_platform::services::collections::service::CollectionsService {
        &*self.collections
    }

    /// `ArtifactsService` accessor — db-bound CRUD + scope promotion.
    pub fn artifacts(
        &self,
    ) -> &dyn ship_platform::services::artifacts::service::ArtifactsService {
        &*self.artifacts
    }

    /// `AccountService` accessor — gateway-bearer token store + PKCE
    /// state for in-flight logins. Shared between REST routes and the
    /// read-op registry.
    pub fn account(&self) -> &Arc<ship_platform::account::AccountService> {
        &self.account
    }

    pub fn mesh_registry(&self) -> &SharedMeshRegistry {
        &self.mesh_registry
    }

    /// Replace the mesh registry (used after spawning MeshService).
    pub fn set_mesh_registry(&mut self, registry: SharedMeshRegistry) {
        self.mesh_registry = registry;
    }

    /// Store the dispatch handle (set once after job manager init).
    pub fn set_dispatch_handle(&self, handle: DispatchHandle) {
        let _ = self.dispatch_handle.set(handle);
    }

    pub fn dispatch_handle(&self) -> Option<&DispatchHandle> {
        self.dispatch_handle.get()
    }

    pub fn agent_mailboxes(&self) -> &AgentMailboxes {
        &self.agent_mailboxes
    }

    pub fn pty_connections(&self) -> &PtyConnections {
        &self.pty_connections
    }

    pub fn pty_sessions(&self) -> &crate::sessions::SessionRegistry {
        &self.pty_sessions
    }

    pub fn actor_sessions(&self) -> &crate::sessions::ActorRegistry {
        &self.actor_sessions
    }

    pub fn session_registry(&self) -> &crate::auth::SessionRegistry {
        &self.session_registry
    }

    /// Get or lazily create the shared DataService for collections.
    ///
    /// On first call, registers built-in collections (currently `artifacts`)
    /// so they are guaranteed present without requiring a schema file.
    pub async fn data_service(&self) -> anyhow::Result<tokio::sync::MutexGuard<'_, Option<ship_substrate::data::DataService>>> {
        ship_platform::services::data::ensure_data_service(&self.data_service, &self.ship_dir).await
    }

    /// Arc handle to the shared `DataService` mutex — passed to service
    /// impls (`CollectionsService`, `ArtifactsService`) so they can lock
    /// the same mutex the runtime holds. Lazy-init is shared via
    /// `ensure_data_service`.
    pub fn data_service_arc(
        &self,
    ) -> Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>> {
        self.data_service.clone()
    }

    pub fn health_registry(&self) -> &HealthRegistry {
        &self.health_registry
    }

    /// Store a pre-built diagnostic registry. Called once after the dispatch
    /// handle is set so the supervisor snapshot has a live process registry.
    /// Idempotent — a second call is a no-op.
    pub fn set_diag_registry(&self, reg: ship_substrate::diagnostics::DiagnosticRegistry) {
        let _ = self.diag_registry.set(reg);
    }

    /// Return the diagnostic registry. Returns `None` before
    /// `set_diag_registry` is called (i.e., before full startup).
    pub fn diag_registry(&self) -> Option<&ship_substrate::diagnostics::DiagnosticRegistry> {
        self.diag_registry.get()
    }

    pub fn view_sessions(&self) -> &crate::views::session::SharedViewSessionRegistry {
        &self.view_sessions
    }

    /// Attached project workspace, if the daemon was launched with one.
    pub fn workspace_dir(&self) -> Option<&Path> {
        self.workspace_dir.as_deref()
    }
}
