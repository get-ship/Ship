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
    /// Optional project workspace. When set, view handlers and job dispatch
    /// read from `<workspace>/.ship/` instead of `<global>/`. Without this
    /// the daemon is workspace-agnostic and serves only global config —
    /// that surface bled into project views more than once. Projects pass
    /// the flag explicitly.
    pub workspace_dir: Option<PathBuf>,
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
    /// Shared data service for collections and extensions.
    data_service: tokio::sync::Mutex<Option<runtime::data::DataService>>,
    /// Agent-owned view sessions (Phase 1 canvas framework).
    view_sessions: crate::views::session::SharedViewSessionRegistry,
    /// PTY session registry — chat, pipelines, dock terminal all go
    /// through here. See `crate::sessions`.
    pty_sessions: crate::sessions::SessionRegistry,
    /// Project workspace dir (e.g. `/path/to/repo`) when the daemon was
    /// started with `--workspace`. Drives project-scoped artifact storage.
    workspace_dir: Option<PathBuf>,
    /// Wall-clock instant the daemon process started — used for uptime.
    started_at: std::time::Instant,
    /// Hook engine (external source adapters + hot-reload watcher).
    hook_engine: std::sync::OnceLock<std::sync::Arc<runtime::hooks::engine::HookEngine>>,
}

impl ShipRuntime {
    /// Create a new runtime. Performs integrity check, DB init, and kernel init.
    /// Does NOT spawn services — those are lazy via managers.
    pub async fn new(meta: AppMetadata, config: RuntimeConfig) -> Result<Self> {
        let global_dir = &meta.global_dir;
        let workspace_dir = meta.workspace_dir.clone();

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

        // Validate artifacts config from project settings (if a workspace is
        // attached and the block is present). Fail fast on invalid values.
        if let Some(ws) = &workspace_dir {
            match runtime::config::get_config(Some(ws.clone())) {
                Ok(proj) => {
                    if let Some(art_cfg) = &proj.artifacts {
                        if let Err(e) = art_cfg.validate() {
                            return Err(anyhow!(
                                "invalid [artifacts] config in .ship/ship.jsonc: {e}"
                            ));
                        }
                    }
                }
                Err(e) => tracing::warn!("could not read project config for artifacts validation: {e}"),
            }
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
            // Platform DB lives at ~/.ship/platform.db only. A workspace
            // attachment does NOT fork a second db into <workspace>/.ship/.
            // See crates/core/runtime/src/db/mod.rs:77 ("Never inside a project directory").
            ship_dir: global_dir.clone(),
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            data_service: tokio::sync::Mutex::new(None),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
        })
    }

    /// Process uptime since the runtime was constructed.
    pub fn uptime(&self) -> std::time::Duration {
        self.started_at.elapsed()
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

    pub fn pty_sessions(&self) -> &crate::sessions::SessionRegistry {
        &self.pty_sessions
    }

    pub fn session_registry(&self) -> &SessionRegistry {
        &self.session_registry
    }

    /// Get or lazily create the shared DataService for collections.
    ///
    /// On first call, registers built-in collections (currently `artifacts`)
    /// so they are guaranteed present without requiring a schema file.
    pub async fn data_service(&self) -> Result<tokio::sync::MutexGuard<'_, Option<runtime::data::DataService>>> {
        let mut guard = self.data_service.lock().await;
        if guard.is_none() {
            let mut svc = runtime::data::DataService::open(&self.ship_dir)?;
            runtime::artifacts::ensure_artifacts_collection(&mut svc)
                .context("registering built-in artifacts collection")?;
            *guard = Some(svc);
        }
        Ok(guard)
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

    /// Stable project slug for the attached workspace. `None` when the daemon
    /// is in global-only mode — callers that need a project scope should
    /// surface a clear error rather than inventing one.
    pub fn project_slug(&self) -> Option<String> {
        let ws = self.workspace_dir.as_ref()?;
        let project_ship = runtime::project::get_project_dir(Some(ws.clone())).ok()?;
        Some(runtime::project::project_slug_from_ship_dir(&project_ship))
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

    /// Rehash blobs for legacy artifact metadata rows that predate the
    /// content_hash column. One-shot; idempotent. Logged on failure but
    /// doesn't block startup.
    pub async fn backfill_artifact_content_hashes(&self) {
        let ship_dir = self.ship_dir.clone();
        tokio::task::spawn_blocking(move || {
            match runtime::data::DataService::open(&ship_dir) {
                Ok(mut svc) => {
                    if let Err(e) = runtime::artifacts::gc::backfill_content_hashes(&ship_dir, &mut svc) {
                        tracing::warn!("artifact content_hash backfill failed: {e}");
                    }
                }
                Err(e) => tracing::warn!("artifact backfill: failed to open DataService: {e}"),
            }
        })
        .await
        .ok();
    }

    /// Spawn the artifact GC loop. Runs immediately on startup then every hour.
    ///
    /// Best-effort: errors are logged but don't take down the daemon.
    pub fn start_artifact_gc(&self) {
        let ship_dir = self.ship_dir.clone();
        let kernel = self.kernel.clone();
        crate::supervised::spawn_supervised(
            &self.health_registry,
            "artifact-gc",
            move || {
                let ship_dir = ship_dir.clone();
                let kernel = kernel.clone();
                async move {
                    loop {
                        match runtime::data::DataService::open(&ship_dir) {
                            Ok(mut svc) => {
                                if let Err(e) = runtime::artifacts::gc::collect_expired(
                                    &ship_dir, &mut svc, &kernel,
                                ).await {
                                    tracing::warn!("artifact GC run failed: {e}");
                                }
                            }
                            Err(e) => tracing::warn!("artifact GC: failed to open DataService: {e}"),
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                    }
                }
            },
        );
    }

    /// Start the hook engine: spawns external source adapters (webhook, file, cron)
    /// and the hot-reload watcher. Idempotent — subsequent calls are no-ops.
    pub async fn start_hook_engine(&self) {
        if self.hook_engine.get().is_some() {
            return;
        }
        match runtime::hooks::engine::spawn_hook_service(
            self.kernel.clone(),
            self.ship_dir.clone(),
        )
        .await
        {
            Ok(engine) => {
                // Also start the filesystem watcher for hot-reload.
                if let Err(e) = runtime::hooks::watcher::spawn_watcher(engine.clone()) {
                    tracing::warn!("hook watcher failed to start: {e}");
                }
                let _ = self.hook_engine.set(engine);
                tracing::info!("hooks: engine started");
            }
            Err(e) => tracing::warn!("hooks: failed to start engine: {e}"),
        }
    }

    /// The running hook engine, if started.
    pub fn hook_engine(
        &self,
    ) -> Option<&std::sync::Arc<runtime::hooks::engine::HookEngine>> {
        self.hook_engine.get()
    }

    /// The webhook source for the public webhook route, if the engine is running.
    pub fn webhook_source(
        &self,
    ) -> Option<std::sync::Arc<runtime::hooks::sources::webhook::WebhookSource>> {
        self.hook_engine.get()?.webhook_source()
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
        let workspace_dir = ship_dir.parent().map(|p| p.to_path_buf());
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
            data_service: tokio::sync::Mutex::new(None),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
        }
    }

    /// Test constructor that also accepts a mesh registry.
    #[cfg(test)]
    pub fn new_test_with_registry(
        ship_dir: std::path::PathBuf,
        kernel: SharedKernel,
        mesh_registry: SharedMeshRegistry,
    ) -> Self {
        let workspace_dir = ship_dir.parent().map(|p| p.to_path_buf());
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
            data_service: tokio::sync::Mutex::new(None),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
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
                kernel.lock().await.emit(envelope, &ctx).await
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
