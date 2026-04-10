//! ShipRuntime — central manager for the shipd daemon.
//!
//! Replaces the ad-hoc `ApiState` struct with a structured runtime that
//! owns the kernel, config, and lazily-initialized managers.

use anyhow::{Result, anyhow};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::services::mesh::SharedMeshRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::managers::{JobManager, MeshManager};
use crate::supervisor::job_dispatch::DispatchHandle;

// ---- Config types ----

/// How the daemon authenticates incoming requests.
#[derive(Debug, Clone)]
pub enum AuthMode {
    /// Token-based auth for local daemon (default).
    Local,
}

/// Runtime configuration — host, port, auth.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub host: String,
    pub port: u16,
    pub auth_mode: AuthMode,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9315,
            auth_mode: AuthMode::Local,
        }
    }
}

/// Paths and metadata for the daemon process.
pub struct AppMetadata {
    pub global_dir: PathBuf,
}

/// Type alias used as axum shared state.
pub type AppState = Arc<ShipRuntime>;

// Re-export for backward compat in handlers.
pub type SharedKernel = Arc<Mutex<runtime::events::KernelRouter>>;
pub type AgentMailboxes = Arc<Mutex<HashMap<String, runtime::events::Mailbox>>>;
pub type PtyConnections = Arc<Mutex<HashMap<String, usize>>>;

// ---- ShipRuntime ----

/// Central runtime for shipd. Owns the kernel, mesh registry, and config.
/// Managers (mesh, jobs) are lazily initialized on first access.
pub struct ShipRuntime {
    config: RuntimeConfig,
    kernel: SharedKernel,
    mesh_registry: SharedMeshRegistry,
    agent_mailboxes: AgentMailboxes,
    pty_connections: PtyConnections,
    ship_dir: PathBuf,
    mesh: tokio::sync::OnceCell<MeshManager>,
    jobs: tokio::sync::OnceCell<JobManager>,
    dispatch_handle: std::sync::OnceLock<DispatchHandle>,
}

impl ShipRuntime {
    /// Create a new runtime. Performs integrity check, DB init, and kernel init.
    /// Does NOT spawn services — those are lazy via managers.
    pub async fn new(meta: AppMetadata, config: RuntimeConfig) -> Result<Self> {
        let global_dir = &meta.global_dir;

        // Integrity check (skip if no DB yet).
        if global_dir.join("platform.db").exists() {
            match runtime::db::integrity::check_platform_integrity(global_dir) {
                Ok(runtime::db::integrity::IntegrityStatus::Ok) => {
                    tracing::info!("shipd: platform.db integrity check passed");
                }
                Ok(runtime::db::integrity::IntegrityStatus::ProjectionsCorrupt {
                    details,
                }) => {
                    tracing::warn!(
                        "shipd: projection tables corrupt ({details}), will rebuild"
                    );
                }
                Ok(runtime::db::integrity::IntegrityStatus::EventsCorrupt {
                    details,
                }) => {
                    return Err(anyhow!(
                        "platform.db is corrupt and cannot be recovered: {details}. \
                         Back up ~/.ship/platform.db and reinitialize with `ship init --global`."
                    ));
                }
                Err(e) => {
                    tracing::warn!("shipd: integrity check failed to run: {e}");
                }
            }
        }

        // Ensure DB exists and init kernel.
        // Use spawn_blocking because the DB layer uses block_in_place internally,
        // which requires the multi-threaded runtime. spawn_blocking works in both.
        let dir = global_dir.clone();
        let kernel = tokio::task::spawn_blocking(move || {
            runtime::db::ensure_db_at(&dir)?;
            runtime::events::init_kernel_router(dir)
                .map_err(|e| anyhow!("failed to initialize KernelRouter: {e}"))
        })
        .await
        .map_err(|e| anyhow!("spawn_blocking failed: {e}"))??;

        let mesh_registry: SharedMeshRegistry =
            Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        Ok(Self {
            config,
            kernel,
            mesh_registry,
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir: global_dir.clone(),
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
        })
    }

    pub fn kernel(&self) -> &SharedKernel {
        &self.kernel
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn ship_dir(&self) -> &Path {
        &self.ship_dir
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

    /// Lazily initialize and return the mesh manager.
    pub async fn mesh(&self) -> &MeshManager {
        self.mesh
            .get_or_init(|| async {
                MeshManager::new(self.mesh_registry.clone())
            })
            .await
    }

    /// Lazily initialize and return the job manager.
    ///
    /// On first call, subscribes to job events and reconciles orphaned
    /// dispatched jobs from previous daemon runs.
    pub async fn jobs(&self) -> &JobManager {
        self.jobs
            .get_or_init(|| async {
                JobManager::init(self.kernel.clone(), self.ship_dir.clone()).await
            })
            .await
    }

    /// Persist event to platform.db then route through kernel.
    pub async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        if let Ok(store) = runtime::events::SqliteEventStore::new() {
            runtime::events::EventStore::append(&store, envelope)?;
        }
        let ctx = EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        self.kernel.lock().await.route(envelope.clone(), &ctx).await
    }

    /// Build a ShipRuntime for tests — synchronous, takes a pre-initialized
    /// kernel and ship_dir. Does NOT run integrity checks or DB init.
    #[cfg(test)]
    pub fn new_test(ship_dir: std::path::PathBuf, kernel: SharedKernel) -> Self {
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
        }
    }

    /// Test constructor that also accepts a mesh registry.
    #[cfg(test)]
    pub fn new_test_with_registry(
        ship_dir: std::path::PathBuf,
        kernel: SharedKernel,
        mesh_registry: SharedMeshRegistry,
    ) -> Self {
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry,
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
        }
    }

    /// Clean up daemon files on shutdown.
    ///
    /// Emits a `supervisor.shutting_down` event, then drains managers before
    /// removing daemon files.
    pub async fn shutdown(&self) {
        // Emit supervisor.shutting_down event before drain/cleanup.
        // Use spawn_blocking-style guard: route() may call block_in_place
        // which panics on single-threaded runtimes, so we catch that.
        if let Ok(envelope) = EventEnvelope::new(
            "supervisor.shutting_down",
            "supervisor",
            &serde_json::json!({}),
        ) {
            let ctx = EmitContext {
                caller_kind: CallerKind::Runtime,
                skill_id: None,
                workspace_id: None,
                session_id: None,
            };
            let kernel = self.kernel.clone();
            let result = tokio::task::spawn(async move {
                kernel.lock().await.route(envelope, &ctx).await
            })
            .await;
            match result {
                Ok(Err(e)) => {
                    tracing::warn!("shipd: failed to emit shutting_down event: {e}");
                }
                Err(e) => {
                    tracing::warn!("shipd: shutting_down event task failed: {e}");
                }
                Ok(Ok(())) => {}
            }
        }

        // Drain — allow managers to finish in-flight work.
        tracing::info!("shipd: drain complete, cleaning up daemon files");

        let _ = std::fs::remove_file(self.ship_dir.join("daemon.token"));
        let _ = std::fs::remove_file(self.ship_dir.join("daemon.port"));
        let _ = std::fs::remove_file(self.ship_dir.join("daemon.pid"));
        tracing::info!("shipd: shutdown complete");
    }
}
