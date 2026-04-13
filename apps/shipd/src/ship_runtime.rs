//! ShipRuntime — central manager for the shipd daemon.
//!
//! Replaces the ad-hoc `ApiState` struct with a structured runtime that
//! owns the kernel, config, and lazily-initialized managers.

use anyhow::{Context, Result, anyhow};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::services::mesh::SharedMeshRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::SessionRegistry;
use crate::managers::{JobManager, MeshManager, SessionScanner};
use crate::supervised::HealthRegistry;
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
    session_scanner: tokio::sync::OnceCell<SessionScanner>,
    /// Scoped session registry — maps session tokens to DaemonSession.
    session_registry: SessionRegistry,
    /// Health registry for supervised background tasks.
    health_registry: HealthRegistry,
    /// Agent-owned view sessions (Phase 1 canvas framework).
    view_sessions: crate::views::session::SharedViewSessionRegistry,
}

impl ShipRuntime {
    /// Create a new runtime. Performs integrity check, DB init, and kernel init.
    /// Does NOT spawn services — those are lazy via managers.
    pub async fn new(meta: AppMetadata, config: RuntimeConfig) -> Result<Self> {
        let global_dir = &meta.global_dir;

        // Integrity check + rebuild if needed.
        //
        // On startup: check platform.db integrity. If projections drift from
        // the event log, replay events into fresh projection tables. If the
        // event log itself is corrupt, refuse to start — the user needs to
        // intervene rather than lose data silently.
        if global_dir.join("platform.db").exists() {
            match runtime::db::integrity::check_platform_integrity(global_dir) {
                Ok(runtime::db::integrity::IntegrityStatus::Ok) => {
                    tracing::info!("shipd: platform.db integrity check passed");
                }
                Ok(runtime::db::integrity::IntegrityStatus::ProjectionsCorrupt {
                    details,
                }) => {
                    tracing::warn!(
                        "shipd: projection tables drifted from event log ({details}), rebuilding"
                    );
                    let dir = global_dir.clone();
                    let report = tokio::task::spawn_blocking(move || {
                        runtime::db::rebuild::rebuild_projections(&dir)
                    })
                    .await
                    .map_err(|e| anyhow!("projection rebuild task failed: {e}"))??;
                    tracing::info!(
                        events = report.events_replayed,
                        projections = report.projections_rebuilt.len(),
                        duration_ms = report.duration_ms,
                        "shipd: projection rebuild complete"
                    );
                }
                Ok(runtime::db::integrity::IntegrityStatus::EventsCorrupt {
                    details,
                }) => {
                    return Err(anyhow!(
                        "platform.db events table is corrupt and cannot be recovered: \
                         {details}. Back up ~/.ship/platform.db, then either restore \
                         from a known-good copy or remove it (destroys all workspace \
                         and job history) and restart the daemon."
                    ));
                }
                Err(e) => {
                    return Err(anyhow!(
                        "shipd: integrity check failed: {e}. \
                         Refusing to start with unknown database health. \
                         Inspect ~/.ship/platform.db manually or remove it to reset."
                    ));
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

        // One-line startup status: events persisted, latest event id. Makes
        // it easy to spot if a restart "lost" events (a mismatched count vs.
        // a known-good baseline is the first signal of database trouble).
        log_event_log_summary(global_dir);

        // Best-effort auto-purge: remove trash entries older than 24 hours.
        match runtime::workspace::trash::purge_old_trash(std::time::Duration::from_secs(86400)) {
            Ok(0) => {}
            Ok(n) => tracing::info!("auto-purged {n} expired trash entries"),
            Err(e) => tracing::warn!("trash auto-purge failed: {e}"),
        }

        let mesh_registry: SharedMeshRegistry =
            Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // End all previously-active sessions from prior daemon run.
        // Uses spawn_blocking because the DB layer uses block_in_place.
        let _ = tokio::task::spawn_blocking(cleanup_active_sessions_on_startup).await;

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
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
        })
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

    pub fn session_registry(&self) -> &SessionRegistry {
        &self.session_registry
    }

    pub fn health_registry(&self) -> &HealthRegistry {
        &self.health_registry
    }

    pub fn view_sessions(&self) -> &crate::views::session::SharedViewSessionRegistry {
        &self.view_sessions
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
        let hr = self.health_registry.clone();
        self.jobs
            .get_or_init(|| async {
                JobManager::init(self.kernel.clone(), self.ship_dir.clone(), &hr).await
            })
            .await
    }

    /// Lazily initialize and return the session scanner.
    ///
    /// On first call, starts a background task that scans for abandoned sessions.
    pub async fn session_scanner(&self) -> &SessionScanner {
        let hr = self.health_registry.clone();
        self.session_scanner
            .get_or_init(|| async {
                SessionScanner::start(self.kernel.clone(), &hr)
            })
            .await
    }

    /// Persist event to platform.db and deliver through the kernel.
    ///
    /// Delegates to `KernelRouter::emit` — the single write path. A successful
    /// return means the event is durable on disk AND delivered to in-memory
    /// subscribers.
    pub async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        let ctx = EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            workspace_id: envelope.workspace_id.clone(),
            session_id: envelope.session_id.clone(),
        };
        self.kernel
            .lock()
            .await
            .emit(envelope.clone(), &ctx)
            .await
            .context("kernel emit")
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
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
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
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
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

        // Remove PID and port (liveness indicators) but keep the token.
        // Preserving the token across restarts means CLI clients that cached
        // it don't need to re-handshake after a daemon restart.
        let _ = std::fs::remove_file(self.ship_dir.join("daemon.port"));
        let _ = std::fs::remove_file(self.ship_dir.join("daemon.pid"));
        tracing::info!("shipd: shutdown complete");
    }
}

/// End all previously-active sessions on daemon startup.
///
/// Handles the case where the daemon crashed and sessions were never ended.
fn cleanup_active_sessions_on_startup() {
    let sessions = match runtime::db::session_drain::list_active_sessions() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("startup cleanup: failed to list active sessions: {e}");
            return;
        }
    };

    if sessions.is_empty() {
        return;
    }

    tracing::info!(
        count = sessions.len(),
        "startup cleanup: ending stale active sessions from previous run"
    );

    for session in &sessions {
        match runtime::db::session_drain::force_end_session(&session.id, "daemon_restart") {
            Ok(()) => {
                tracing::info!(
                    session_id = %session.id,
                    workspace = %session.workspace_branch,
                    "startup cleanup: ended stale session"
                );
            }
            Err(e) => {
                tracing::warn!(
                    session_id = %session.id,
                    "startup cleanup: failed to end stale session: {e}"
                );
            }
        }
    }
}

/// Emit a single INFO line with event-log health at startup. Gives operators
/// an immediate signal: how many events persisted, the newest id. If the
/// count ever drops between restarts without a reset, that's the first
/// thing to investigate.
fn log_event_log_summary(global_dir: &Path) {
    if !global_dir.join("platform.db").exists() {
        tracing::info!("shipd: fresh install, no events yet");
        return;
    }
    let summary = runtime::db::event_log_summary(global_dir);
    tracing::info!(
        events = summary.event_count,
        latest_event_id = summary.latest_event_id.as_deref().unwrap_or("-"),
        "shipd: event-log summary"
    );
}
