//! ShipRuntime — central manager for the shipd daemon.
//!
//! Replaces the ad-hoc `ApiState` struct with a structured runtime that
//! owns the kernel, config, and lazily-initialized managers.

use anyhow::{Context, Result, anyhow};
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};
use ship_platform::services::mesh::SharedMeshRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::SessionRegistry;
use crate::managers::{RunManager, MeshManager, SessionScanner};
use ship_substrate::task_health::HealthRegistry;
use crate::supervisor::run_dispatch::DispatchHandle;

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

/// Canonical name for the axum shared state. Both `ApiState` and
/// `AppState` refer to the same `Arc<ShipRuntime>` handle.
pub type ApiState = Arc<ShipRuntime>;

// Re-export for backward compat in handlers.
pub type SharedKernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;
pub type AgentMailboxes = Arc<Mutex<HashMap<String, ship_substrate::events::Mailbox>>>;
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
    /// Daemon IPC directory — `<root>/machine/daemon/` or
    /// `<root>/machine/daemons/<hash>/` per devx-04.
    daemon_dir: PathBuf,
    mesh: tokio::sync::OnceCell<MeshManager>,
    jobs: tokio::sync::OnceCell<RunManager>,
    dispatch_handle: std::sync::OnceLock<DispatchHandle>,
    session_scanner: tokio::sync::OnceCell<SessionScanner>,
    /// Scoped session registry — maps session tokens to DaemonSession.
    session_registry: SessionRegistry,
    /// Health registry for supervised background tasks.
    health_registry: HealthRegistry,
    /// Shared data service for collections and extensions. Arc-wrapped
    /// so trait-bound service impls (`CollectionsService`,
    /// `ArtifactsService`) can hold a clone of the same handle and see
    /// the same lazily-opened DataService.
    data_service: Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>>,
    /// Agent-owned view sessions (Phase 1 canvas framework).
    view_sessions: crate::views::session::SharedViewSessionRegistry,
    /// PTY session registry — chat, pipelines, dock terminal all go
    /// through here. See `crate::sessions`.
    pty_sessions: crate::sessions::SessionRegistry,
    /// Actor-backed session registry — destination of the supervisor-
    /// primitive migration. Shells live here today; agents and pipelines
    /// follow as 1.1.c / 1.1.d land.
    actor_sessions: crate::sessions::ActorRegistry,
    /// Project workspace dir (e.g. `/path/to/repo`) when the daemon was
    /// started with `--workspace`. Drives project-scoped artifact storage.
    workspace_dir: Option<PathBuf>,
    /// Wall-clock instant the daemon process started — used for uptime.
    started_at: std::time::Instant,
    /// Hook engine (external source adapters + hot-reload watcher).
    hook_engine: std::sync::OnceLock<std::sync::Arc<ship_platform::triggers::engine::HookEngine>>,
    /// Declarative-tier config watcher (Phase 4) — emits domain.changed events.
    config_watcher: std::sync::OnceLock<ship_platform::config::ConfigWatcher>,
    /// Bun bundler worker — owned here, lazily started on first access.
    /// `Bundler::start` is synchronous (it spawns a child and returns a
    /// handle), so `std::sync::OnceLock` is the right primitive — using
    /// `tokio::sync::OnceCell` would require `get_or_init` to take an
    /// `async` initializer.
    pub bundler: std::sync::OnceLock<ship_platform::bundler::Bundler>,
    /// `RulesService` impl — pilot for the modular-runtime spec 02
    /// service-layer pattern. HTTP handlers in `services::rules` go
    /// through this trait so the cloud worker / browser embed / unit
    /// tests can swap implementations without standing up an HTTP server.
    rules: std::sync::Arc<dyn ship_platform::services::rules::service::RulesService>,
    /// `PermissionsService` impl — second mr-02 pilot, includes the
    /// cross-domain `tool_patterns` composition.
    permissions: std::sync::Arc<dyn ship_platform::services::permissions::service::PermissionsService>,
    /// `McpService` impl — third mr-02 pilot.
    mcp: std::sync::Arc<dyn ship_platform::services::mcp::service::McpService>,
    /// `SkillsService` impl — fourth mr-02 migration.
    skills: std::sync::Arc<dyn ship_platform::services::skills::service::SkillsService>,
    /// `AgentsService` impl — fifth mr-02 migration.
    agents: std::sync::Arc<dyn ship_platform::services::agents::service::AgentsService>,
    /// `PipelinesService` impl — sixth mr-02 migration (definition CRUD only;
    /// dispatch / cancel / runs still flow through `runtime_api`).
    pipelines: std::sync::Arc<dyn ship_platform::services::pipelines::service::PipelinesService>,
    /// `GitService` impl — seventh mr-02 migration. Project-bound; the
    /// SSE handler in `git/mod.rs` stays separate.
    git: std::sync::Arc<dyn ship_platform::services::git::service::GitService>,
    /// `CollectionsService` impl — eighth mr-02 migration. Db-bound;
    /// shares the `data_service` Arc with the runtime.
    collections: std::sync::Arc<dyn ship_platform::services::collections::service::CollectionsService>,
    /// `ArtifactsService` impl — ninth mr-02 migration. Db-bound;
    /// shares the `data_service` Arc with the runtime.
    artifacts: std::sync::Arc<dyn ship_platform::services::artifacts::service::ArtifactsService>,
    /// Account service — owns the gateway-bearer token store and the
    /// PKCE state for in-flight logins. Constructed once per daemon and
    /// shared between the REST routes (`account_api`) and the read-op
    /// registry (`registry::account`).
    account: std::sync::Arc<ship_platform::account::AccountService>,
    /// Worktree resolver — production uses `git worktree list`; tests
    /// override with an in-memory map. See `resolve_worktree` method and
    /// `new_test_with_worktree_resolver` test ctor.
    worktree_resolver: WorktreeResolver,
}

/// Resolves a workspace-id (branch name) to its filesystem worktree path.
/// Returns None if the id doesn't match a known worktree.
pub type WorktreeResolver = Arc<dyn Fn(&str) -> Option<PathBuf> + Send + Sync>;

/// RAII guard that stops a kernel actor when dropped. Used by SSE
/// handlers (`routes::runtime_api`, `views::*`) so disconnect/cancel
/// cleans up the spawned actor without leaking mailbox state.
///
/// Lives here next to `SharedKernel` so consumers across `routes/`,
/// `views/`, and any future SSE-producing module can reach it without
/// the module-DAG-violating `views → routes` import path.
pub struct ActorGuard {
    pub actor_id: String,
    pub kernel: SharedKernel,
}

impl Drop for ActorGuard {
    fn drop(&mut self) {
        let actor_id = self.actor_id.clone();
        let kernel = self.kernel.clone();
        tokio::spawn(async move {
            let _ = kernel.lock().await.stop_actor(&actor_id);
        });
    }
}

/// Production worktree resolver: branch name → worktree path via
/// `git worktree list`, falling back to the current project root if the
/// id matches the current branch. Replaces the legacy free function
/// `crate::project_api::resolve_worktree` lookup; both share the
/// `worktree_path_for_branch` + `current_branch_at` helpers in
/// `project_api/mod.rs`.
pub fn default_worktree_resolver() -> WorktreeResolver {
    Arc::new(|id: &str| {
        if let Some(path) = crate::project_api::worktree_path_for_branch(id) {
            return Some(path);
        }
        if let Ok(project_ship) = ship_substrate::project::get_project_dir(None)
            && let Some(root) = project_ship.parent()
            && let Some(current_branch) = crate::project_api::current_branch_at(root)
            && current_branch == id
        {
            return Some(root.to_path_buf());
        }
        None
    })
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
        if ship_substrate::db::platform_db_path(global_dir).exists() {
            match ship_substrate::db::integrity::check_platform_integrity(global_dir) {
                Ok(ship_substrate::db::integrity::IntegrityStatus::Ok) => {
                    tracing::info!("shipd: platform.db integrity check passed");
                }
                Ok(ship_substrate::db::integrity::IntegrityStatus::ProjectionsCorrupt {
                    details,
                }) => {
                    tracing::warn!(
                        "shipd: projection tables drifted from event log ({details}), rebuilding"
                    );
                    let dir = global_dir.clone();
                    let report = tokio::task::spawn_blocking(move || {
                        ship_substrate::db::rebuild::rebuild_projections(&dir)
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
                Ok(ship_substrate::db::integrity::IntegrityStatus::EventsCorrupt {
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
        //
        // Ordering matters:
        // 1. Open data.db (collections migration copies legacy rows out of platform.db)
        //    BEFORE ensure_db_at — migration 0006 drops those legacy tables.
        // 2. Ensure kv.db schema and copy any rows from platform.db kv_state BEFORE
        //    ensure_db_at — migration 0002 drops kv_state from platform.db.
        //
        // Use spawn_blocking because the DB layer uses block_in_place internally.
        // Install platform-level projections + supervisor hooks before
        // any SqliteEventStore is constructed.
        ship_platform::install_defaults();

        let dir = global_dir.clone();
        let kernel = tokio::task::spawn_blocking(move || {
            ship_substrate::home::ensure_layout(&ship_substrate::ShipHome::at(dir.clone()))
                .context("ensure ship home layout")?;
            // Open data.db + run collections migration first.
            let _ = ship_substrate::data::DataService::open(&dir)?;
            // Migrate kv rows from platform.db to kv.db before migration 0002 drops them.
            ship_platform::kv::migrate_from_platform_db(&dir)
                .context("kv migration from platform.db")?;
            ship_substrate::db::ensure_db_at(&dir)?;
            ship_substrate::events::init_kernel_router(dir)
                .map_err(|e| anyhow!("failed to initialize KernelRouter: {e}"))
        })
        .await
        .map_err(|e| anyhow!("spawn_blocking failed: {e}"))??;

        // One-line startup status: events persisted, latest event id. Makes
        // it easy to spot if a restart "lost" events (a mismatched count vs.
        // a known-good baseline is the first signal of database trouble).
        log_event_log_summary(global_dir);

        // Best-effort auto-purge: remove trash entries older than 24 hours.
        match ship_substrate::trash::purge_old_trash(std::time::Duration::from_secs(86400)) {
            Ok(0) => {}
            Ok(n) => tracing::info!("auto-purged {n} expired trash entries"),
            Err(e) => tracing::warn!("trash auto-purge failed: {e}"),
        }

        let mesh_registry: SharedMeshRegistry =
            Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // End all previously-active sessions from prior daemon run.
        // Uses spawn_blocking because the DB layer uses block_in_place.
        let _ = tokio::task::spawn_blocking(cleanup_active_sessions_on_startup).await;

        // Build db-bound service impls before the struct literal so they
        // can share the same `data_service` Arc.
        let data_service: Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let collections =
            ship_platform::services::collections::service::CollectionsServiceImpl::arc(
                global_dir.clone(),
                kernel.clone(),
                data_service.clone(),
            );
        let artifacts = ship_platform::services::artifacts::service::ArtifactsServiceImpl::arc(
            global_dir.clone(),
            kernel.clone(),
            data_service.clone(),
        );

        // Account service — owns the gateway-bearer token store + PKCE
        // pending registry. Base URL defaults to production gateway;
        // overridable via SHIP_GATEWAY_URL so dev can point at staging or
        // a local `wrangler dev` instance. On client construction failure
        // we fall back to the production URL so route registration still
        // compiles; downstream handlers surface the construction error.
        let gateway_base = std::env::var("SHIP_GATEWAY_URL")
            .unwrap_or_else(|_| "https://api.getship.dev".to_string());
        let daemon_dir_for_account = {
            let probe = workspace_dir
                .clone()
                .or_else(|| std::env::current_dir().ok());
            let wt_root = probe
                .as_ref()
                .and_then(|p| ship_substrate::home::worktree_root_for(p))
                .or_else(|| {
                    probe
                        .as_ref()
                        .and_then(|p| ship_substrate::home::ProjectRoot::discover(p))
                        .map(|r| r.path().to_path_buf())
                });
            let home = ship_substrate::ShipHome::at(global_dir.clone());
            match wt_root.as_ref() {
                Some(r) => home.worktree_daemon_dir(&ship_substrate::home::path_hash(r)),
                None => home.daemon_dir(),
            }
        };
        let account_store = Arc::new(
            ship_platform::account::FileTokenStore::for_daemon_dir(&daemon_dir_for_account),
        );
        let account = Arc::new(
            ship_platform::account::AccountService::new(account_store, &gateway_base)
                .or_else(|e| {
                    tracing::warn!(
                        "account service: build failed ({e}); /api/v1/account/* will return 500"
                    );
                    ship_platform::account::AccountService::new(
                        Arc::new(ship_platform::account::FileTokenStore::for_daemon_dir(
                            &daemon_dir_for_account,
                        )),
                        "https://api.getship.dev",
                    )
                })
                .context("building fallback account service")?,
        );

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
            daemon_dir: {
                // Pick the worktree daemon dir if launched from inside a
                // git worktree, else the singleton. Mirrors lib.rs::run_network —
                // hash the worktree's own root so each linked worktree gets
                // its own slot.
                let probe = workspace_dir.clone().or_else(|| std::env::current_dir().ok());
                let wt_root = probe
                    .as_ref()
                    .and_then(|p| ship_substrate::home::worktree_root_for(p))
                    .or_else(|| {
                        probe
                            .as_ref()
                            .and_then(|p| ship_substrate::home::ProjectRoot::discover(p))
                            .map(|r| r.path().to_path_buf())
                    });
                let home = ship_substrate::ShipHome::at(global_dir.clone());
                match wt_root.as_ref() {
                    Some(r) => home.worktree_daemon_dir(&ship_substrate::home::path_hash(r)),
                    None => home.daemon_dir(),
                }
            },
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            data_service: Arc::new(tokio::sync::Mutex::new(None)),
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules: ship_platform::services::rules::service::RulesServiceImpl::arc(global_dir.clone()),
            permissions: {
                // PermissionsServiceImpl needs an Arc<dyn McpService> for
                // tool_patterns; build the same impl the runtime exposes
                // so HTTP and trait-to-trait paths see one catalog.
                let mcp_for_perms =
                    ship_platform::services::mcp::service::McpServiceImpl::arc(global_dir.clone());
                std::sync::Arc::new(
                    ship_platform::services::permissions::service::PermissionsServiceImpl::with_roots(
                        global_dir.clone(),
                        None,
                        mcp_for_perms,
                    ),
                )
            },
            mcp: ship_platform::services::mcp::service::McpServiceImpl::arc(global_dir.clone()),
            skills: ship_platform::services::skills::service::SkillsServiceImpl::arc(global_dir.clone()),
            agents: ship_platform::services::agents::service::AgentsServiceImpl::arc(global_dir.clone()),
            pipelines: ship_platform::services::pipelines::service::PipelinesServiceImpl::arc(global_dir.clone()),
            git: ship_platform::services::git::service::GitServiceImpl::arc(global_dir.clone()),
            collections,
            artifacts,
            account,
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            worktree_resolver: default_worktree_resolver(),
        })
    }

    /// Resolve a workspace-id (branch name) to its worktree path.
    /// Wraps the configured `worktree_resolver` with a 404 response on miss.
    pub fn resolve_worktree(
        &self,
        id: &str,
    ) -> Result<PathBuf, (axum::http::StatusCode, axum::Json<crate::wire::MeshResponse>)>
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

    pub fn session_registry(&self) -> &SessionRegistry {
        &self.session_registry
    }

    /// Get or lazily create the shared DataService for collections.
    ///
    /// On first call, registers built-in collections (currently `artifacts`)
    /// so they are guaranteed present without requiring a schema file.
    pub async fn data_service(&self) -> Result<tokio::sync::MutexGuard<'_, Option<ship_substrate::data::DataService>>> {
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

    pub fn view_sessions(&self) -> &crate::views::session::SharedViewSessionRegistry {
        &self.view_sessions
    }

    /// Attached project workspace, if the daemon was launched with one.
    pub fn workspace_dir(&self) -> Option<&Path> {
        self.workspace_dir.as_deref()
    }

    /// Lazily start and return the Bun bundler worker.
    ///
    /// The worker process is spawned on first call and supervised thereafter.
    /// The repo root defaults to the workspace dir (or cwd as fallback) since
    /// that's where `packages/` and `views/` live.
    ///
    /// Dev-mode override: if the workspace contains a checked-in
    /// `packages/deck/bundler/index.ts`, point the worker at *that* copy instead of
    /// the one stashed under `~/.ship/runtime/`. Without this, contributors
    /// editing `packages/deck/bundler/index.ts` in the repo see no effect until
    /// they re-run `ship runtime install` — a footgun that has bitten more
    /// than one debugging session.
    pub fn bundler(&self) -> &ship_platform::bundler::Bundler {
        self.bundler.get_or_init(|| {
            let home = ship_substrate::ShipHome::at(self.ship_dir.clone());
            // `repo_root` is the bundler's CWD + `SHIP_REPO_ROOT` — where
            // it looks for monorepo built-ins (`packages/`, contributor
            // `views/`). When the daemon has no attached project, fall
            // back to cwd so contributor-mode invocations behave as
            // before. `workspace` is the explicit user-project path
            // forwarded as `--workspace`; the bundler watches its
            // `views/*/page.sui` tree on top of the built-ins.
            let workspace = self.workspace_dir.clone();
            let repo_root = workspace
                .clone()
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| self.ship_dir.clone());
            // Prefer the canonical in-repo bundler source over the deployed
            // copy at `~/.ship/runtime/bundler.ts`. The deployed copy drifts
            // (only refreshed by `ship runtime install`), so a dev daemon
            // running against the repo must use the live source — otherwise a
            // stale worker emits to old asset paths. `packages/deck/bundler/
            // index.ts` IS the worker script (same stdin/stdout protocol).
            let bundler_override = repo_root
                .join("packages")
                .join("deck")
                .join("bundler")
                .join("index.ts");
            if bundler_override.is_file() {
                tracing::info!(
                    path = %bundler_override.display(),
                    "bundler: using in-repo bundler source (dev override)"
                );
                ship_platform::bundler::Bundler::start_with_script(
                    &home,
                    repo_root,
                    bundler_override,
                    workspace,
                )
            } else {
                ship_platform::bundler::Bundler::start(&home, repo_root, workspace)
            }
        })
    }

    /// Stable project slug for the attached workspace. `None` when the daemon
    /// is in global-only mode — callers that need a project scope should
    /// surface a clear error rather than inventing one.
    pub fn project_slug(&self) -> Option<String> {
        let ws = self.workspace_dir.as_ref()?;
        let project_ship = ship_substrate::project::get_project_dir(Some(ws.clone())).ok()?;
        Some(ship_substrate::project::project_slug_from_ship_dir(&project_ship))
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
    pub async fn jobs(&self) -> &RunManager {
        let hr = self.health_registry.clone();
        let registry = self.session_registry.clone();
        let daemon_url = self.daemon_origin();
        self.jobs
            .get_or_init(|| async {
                RunManager::init(
                    self.kernel.clone(),
                    self.ship_dir.clone(),
                    &hr,
                    registry,
                    daemon_url,
                )
                .await
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


    /// Spawn the artifact GC loop. Runs immediately on startup then every hour.
    ///
    /// Best-effort: errors are logged but don't take down the daemon.
    pub fn start_artifact_gc(&self) {
        let ship_dir = self.ship_dir.clone();
        let kernel = self.kernel.clone();
        ship_substrate::task_health::spawn_supervised(
            &self.health_registry,
            "artifact-gc",
            move || {
                let ship_dir = ship_dir.clone();
                let kernel = kernel.clone();
                async move {
                    loop {
                        match ship_substrate::data::DataService::open(&ship_dir) {
                            Ok(mut svc) => {
                                if let Err(e) = ship_platform::artifacts::gc::collect_expired(
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
        // Pass the user-scope library/ dir; engine joins `triggers/` for the
        // user-scope vector. Project-scope triggers are loaded by the hook
        // engine when a project workspace activates.
        let library_dir = ship_substrate::ShipHome::at(self.ship_dir.clone()).library_dir();
        match ship_platform::triggers::engine::spawn_hook_service(
            self.kernel.clone(),
            library_dir,
        )
        .await
        {
            Ok(engine) => {
                // Also start the filesystem watcher for hot-reload.
                if let Err(e) = ship_platform::triggers::watcher::spawn_watcher(engine.clone()) {
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
    ) -> Option<&std::sync::Arc<ship_platform::triggers::engine::HookEngine>> {
        self.hook_engine.get()
    }

    /// The webhook source for the public webhook route, if the engine is running.
    pub fn webhook_source(
        &self,
    ) -> Option<std::sync::Arc<ship_platform::triggers::sources::webhook::WebhookSource>> {
        self.hook_engine.get()?.webhook_source()
    }

    /// Start the declarative-tier config watcher (Phase 4).
    ///
    /// Watches agents, skills, views, pipelines, and rules under both the
    /// user-scope library dir and the active project scope (if set). Idempotent.
    pub fn start_config_watcher(&self) {
        if self.config_watcher.get().is_some() {
            return;
        }
        match ship_platform::config::spawn_config_watcher(
            self.kernel.clone(),
            self.ship_dir.clone(),
            self.workspace_dir.clone(),
        ) {
            Ok(w) => {
                let _ = self.config_watcher.set(w);
                tracing::info!("config watcher: started");
            }
            Err(e) => tracing::warn!("config watcher: failed to start: {e}"),
        }
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
            project_id: envelope.project_id.clone(),
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
        let daemon_dir = ship_substrate::ShipHome::at(ship_dir.clone()).daemon_dir();
        let rules = ship_platform::services::rules::service::RulesServiceImpl::arc(ship_dir.clone());
        let mcp = ship_platform::services::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        let skills = ship_platform::services::skills::service::SkillsServiceImpl::arc(ship_dir.clone());
        let agents = ship_platform::services::agents::service::AgentsServiceImpl::arc(ship_dir.clone());
        let pipelines =
            ship_platform::services::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        let git = ship_platform::services::git::service::GitServiceImpl::arc(ship_dir.clone());
        let data_service: Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let collections =
            ship_platform::services::collections::service::CollectionsServiceImpl::arc(
                ship_dir.clone(),
                kernel.clone(),
                data_service.clone(),
            );
        let artifacts = ship_platform::services::artifacts::service::ArtifactsServiceImpl::arc(
            ship_dir.clone(),
            kernel.clone(),
            data_service.clone(),
        );
        // Permissions reaches into Mcp via the trait for tool_patterns.
        // Wire the same arc both places so HTTP and trait-to-trait paths
        // observe the same catalog.
        let permissions: std::sync::Arc<dyn ship_platform::services::permissions::service::PermissionsService> =
            std::sync::Arc::new(
                ship_platform::services::permissions::service::PermissionsServiceImpl::with_roots(
                    ship_dir.clone(),
                    None,
                    mcp.clone(),
                ),
            );
        let account = test_account_service(&ship_dir);
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            daemon_dir,
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            data_service,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules,
            permissions,
            mcp,
            skills,
            agents,
            pipelines,
            git,
            collections,
            artifacts,
            account: account.clone(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            worktree_resolver: default_worktree_resolver(),
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
        let daemon_dir = ship_substrate::ShipHome::at(ship_dir.clone()).daemon_dir();
        let rules = ship_platform::services::rules::service::RulesServiceImpl::arc(ship_dir.clone());
        let mcp = ship_platform::services::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        let skills = ship_platform::services::skills::service::SkillsServiceImpl::arc(ship_dir.clone());
        let agents = ship_platform::services::agents::service::AgentsServiceImpl::arc(ship_dir.clone());
        let pipelines =
            ship_platform::services::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        let git = ship_platform::services::git::service::GitServiceImpl::arc(ship_dir.clone());
        let data_service: Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let collections =
            ship_platform::services::collections::service::CollectionsServiceImpl::arc(
                ship_dir.clone(),
                kernel.clone(),
                data_service.clone(),
            );
        let artifacts = ship_platform::services::artifacts::service::ArtifactsServiceImpl::arc(
            ship_dir.clone(),
            kernel.clone(),
            data_service.clone(),
        );
        // Permissions reaches into Mcp via the trait for tool_patterns.
        // Wire the same arc both places so HTTP and trait-to-trait paths
        // observe the same catalog.
        let permissions: std::sync::Arc<dyn ship_platform::services::permissions::service::PermissionsService> =
            std::sync::Arc::new(
                ship_platform::services::permissions::service::PermissionsServiceImpl::with_roots(
                    ship_dir.clone(),
                    None,
                    mcp.clone(),
                ),
            );
        let account = test_account_service(&ship_dir);
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry,
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            daemon_dir,
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            data_service,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules,
            permissions,
            mcp,
            skills,
            agents,
            pipelines,
            git,
            collections,
            artifacts,
            account: account.clone(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            worktree_resolver: default_worktree_resolver(),
        }
    }

    /// Test ctor variant — accepts a custom `WorktreeResolver` so tests
    /// can map workspace ids to their tempdir paths instead of consulting
    /// the daemon's CWD via `git worktree list`. See `project_api/tests.rs`.
    #[cfg(test)]
    pub fn new_test_with_worktree_resolver(
        ship_dir: std::path::PathBuf,
        kernel: SharedKernel,
        worktree_resolver: WorktreeResolver,
    ) -> Self {
        let mut rt = Self::new_test(ship_dir, kernel);
        rt.worktree_resolver = worktree_resolver;
        rt
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
                project_id: None,
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

        // Remove pid and port (liveness indicators) but keep the token.
        // Preserving the token across restarts means CLI clients that cached
        // it don't need to re-handshake after a daemon restart.
        crate::daemon::remove_liveness_files(&self.daemon_dir);
        tracing::info!("shipd: shutdown complete");
    }
}

// `ensure_data_service` moved to `ship_platform::services::data::ensure_data_service`
// so the migrated CollectionsService + ArtifactsService can call it without
// a back-reference to shipd. ShipRuntime's accessor below delegates.

/// End all previously-active sessions on daemon startup.
///
/// Handles the case where the daemon crashed and sessions were never ended.
fn cleanup_active_sessions_on_startup() {
    let sessions = match ship_substrate::db::session_drain::list_active_sessions() {
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
        match ship_substrate::db::session_drain::force_end_session(&session.id, "daemon_restart") {
            Ok(()) => {
                tracing::info!(
                    session_id = %session.id,
                    workspace = %session.branch,
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
    if !ship_substrate::db::platform_db_path(global_dir).exists() {
        tracing::info!("shipd: fresh install, no events yet");
        return;
    }
    let summary = ship_substrate::db::event_log_summary(global_dir);
    tracing::info!(
        events = summary.event_count,
        latest_event_id = summary.latest_event_id.as_deref().unwrap_or("-"),
        "shipd: event-log summary"
    );
}

/// Synthesise an `AccountService` for unit tests. Uses a tempdir-shaped
/// fake token store rooted at `ship_dir` (tests don't write to it) and a
/// localhost gateway URL so client construction succeeds without DNS.
#[cfg(test)]
fn test_account_service(
    ship_dir: &std::path::Path,
) -> std::sync::Arc<ship_platform::account::AccountService> {
    let store = std::sync::Arc::new(
        ship_platform::account::FileTokenStore::for_daemon_dir(ship_dir),
    );
    std::sync::Arc::new(
        ship_platform::account::AccountService::new(store, "http://127.0.0.1:1")
            .expect("test account service ctor"),
    )
}
