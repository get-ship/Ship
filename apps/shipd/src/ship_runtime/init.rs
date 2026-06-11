//! ShipRuntime construction and shutdown. Test ctors live in `test_ctors`.

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::EmitContext;
use ship_substrate::task_health::HealthRegistry;
use super::{ShipRuntime, RuntimeConfig, AppMetadata};

impl ShipRuntime {
    /// Create a new runtime. Performs integrity check, DB init, and kernel init.
    /// Does NOT spawn services — those are lazy via managers.
    pub async fn new(meta: AppMetadata, config: RuntimeConfig) -> Result<Self> {
        let global_dir = &meta.global_dir;
        let workspace_dir = meta.workspace_dir.clone();

        // Integrity check + rebuild if needed.
        //
        // On startup: check each per-scope event db's integrity. After the
        // storage partition every scope owns its own db (`runtime.db`,
        // `account.db`, each mounted `spaces/<id>/space.db`), each with its own
        // `events` log AND projection tables. If any scope's projections drift
        // from its log, replay every scope db's events into fresh projection
        // tables. If a scope's event log itself is corrupt, refuse to start —
        // the user needs to intervene rather than lose data silently. The
        // Runtime scope db (`runtime.db`) is the boot gate: it is the kernel's
        // primary scope and its presence signals a non-fresh install.
        if ship_substrate::db::runtime_db_path(global_dir).exists() {
            match ship_substrate::db::integrity::check_scope_integrity(global_dir) {
                Ok(ship_substrate::db::integrity::IntegrityStatus::Ok) => {
                    tracing::info!("shipd: per-scope db integrity check passed");
                }
                Ok(ship_substrate::db::integrity::IntegrityStatus::ProjectionsCorrupt {
                    details,
                }) => {
                    tracing::warn!(
                        "shipd: projection tables drifted from event log ({details}), rebuilding per-scope"
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
                        "shipd: per-scope projection rebuild complete"
                    );
                }
                Ok(ship_substrate::db::integrity::IntegrityStatus::EventsCorrupt {
                    details,
                }) => {
                    return Err(anyhow!(
                        "a scope event log is corrupt and cannot be recovered: \
                         {details}. Back up the named db under ~/.ship/, then either \
                         restore from a known-good copy or remove it (destroys that \
                         scope's history) and restart the daemon."
                    ));
                }
                Err(e) => {
                    return Err(anyhow!(
                        "shipd: integrity check failed: {e}. \
                         Refusing to start with unknown database health. \
                         Inspect the per-scope dbs under ~/.ship/ manually or remove \
                         them to reset."
                    ));
                }
            }
        }

        // Ensure DB exists and init kernel.
        //
        // platform.db is dead — the kernel routes every event into its per-scope
        // db (runtime.db / account.db / spaces/<id>/space.db) via PartitionedSink.
        // The only remaining reason to touch platform.db at boot is the ONE-TIME
        // legacy migration off it: a pre-existing install carries `kv_state` +
        // legacy collection tables in platform.db that the kv/collections
        // migrations drain out. Both steps run only when a platform.db actually
        // exists — a fresh install never creates one.
        //
        // Ordering for the legacy drain:
        // 1. Open data.db (collections migration copies legacy rows out of platform.db)
        //    BEFORE the platform.db migration — migration 0006 drops those tables.
        // 2. Migrate kv rows from platform.db kv_state into kv.db BEFORE the
        //    platform.db migration — migration 0002 drops kv_state.
        //
        // Use spawn_blocking because the DB layer uses block_in_place internally.
        // Install platform-level projections + supervisor hooks before
        // any SqliteEventStore is constructed.
        ship_platform::install_defaults();
        ship_pipeline_runtime::install_projections();

        let dir = global_dir.clone();
        let kernel = tokio::task::spawn_blocking(move || {
            ship_substrate::home::ensure_layout(&ship_substrate::ShipHome::at(dir.clone()))
                .context("ensure ship home layout")?;
            // Open data.db + run collections migration first.
            let _ = ship_substrate::data::DataService::open(&dir)?;
            // Legacy platform.db drain — one-time, only when a platform.db is
            // present. `migrate_from_platform_db` already early-returns when it
            // is absent; the platform.db schema migration (which drops the
            // drained legacy tables) is gated on the same `.exists()` so a fresh
            // install never materializes a dead platform.db.
            ship_agents::migrate_skill_vars_from_platform_db(&dir)
                .context("kv migration from platform.db")?;
            if ship_substrate::db::platform_db_path(&dir).exists() {
                ship_substrate::db::ensure_db_at(&dir)?;
            }
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

        // Build the collection + artifact `SpaceContext`s
        // and the ONE shared schema registry, here (C-2 §3.1 / UD-7). The
        // `collections()`/`artifacts()` accessors construct stateless services
        // over these per call; the collection wire is mounted with the SAME
        // registry so a wire-created schema is visible to trigger inserts.
        // Collections AND artifact metadata are event-sourced (ADR D6/D7): the
        // read path must bind to the SAME per-scope `spaces/<id>/space.db` that the
        // `CollectionProjection` / `ArtifactProjection` materialize the table into,
        // so a query reads exactly what the projection wrote.
        //
        // The active space is the path-hash of the DISCOVERED project root — the
        // identity the `projects` projection keys on. With NO discoverable project
        // the daemon boots space-less: `space_id` is `None`, the contexts are NOT
        // built, and a space-scoped op resolves the active space per-request and
        // errors cleanly if there is none. There is NO daemon home space and NO
        // `global` pseudo-space to fall back onto (S2 home-space kill).
        let space_id = {
            let probe = workspace_dir.clone().or_else(|| std::env::current_dir().ok());
            probe
                .as_deref()
                .and_then(ship_substrate::project::resolve_active_space)
        };
        let collection_ctx = space_id
            .as_deref()
            .map(|id| build_space_db_context_arc(kernel.clone(), global_dir, id));
        let artifact_ctx = space_id
            .as_deref()
            .map(|id| build_space_db_context_arc(kernel.clone(), global_dir, id));
        let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
        // Hydrate user-authored `.ship/collections/*.jsonc` schemas into the
        // shared registry so they are queryable alongside the built-ins.
        {
            let project_ship = ship_substrate::project::get_project_dir(
                workspace_dir.clone(),
            )
            .ok();
            ship_store::collection_schemas::hydrate_registry(
                &schemas,
                global_dir,
                project_ship.as_deref(),
            );
        }
        let global_blob_dir = global_dir.clone();

        // Account identity is served by the typed Connect `AccountService`
        // (account.proto, `ship-account` crate). Its `AccountWire` is built in
        // `ops_rpc::router` from `daemon_dir` + `SHIP_GATEWAY_URL`; the runtime
        // no longer holds an account service in `ApiState`.

        Ok(Self {
            config,
            kernel,
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
            #[cfg(feature = "ship-code")]
            dispatch_handle: std::sync::OnceLock::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            collection_ctx,
            artifact_ctx,
            schemas,
            global_blob_dir,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: ship_agents::services::sessions::SessionRegistry::new(),
            actor_sessions: ship_agents::services::sessions::ActorRegistry::new(),
            rules: ship_agents::services::rules::service::RulesServiceImpl::arc(global_dir.clone()),
            permissions: {
                // PermissionsServiceImpl needs an Arc<dyn McpService> for
                // tool_patterns; build the same impl the runtime exposes
                // so HTTP and trait-to-trait paths see one catalog.
                let mcp_for_perms =
                    ship_agents::services::mcp::service::McpServiceImpl::arc(global_dir.clone());
                std::sync::Arc::new(
                    ship_agents::services::permissions::service::PermissionsServiceImpl::with_roots(
                        global_dir.clone(),
                        None,
                        mcp_for_perms,
                    ),
                )
            },
            mcp: ship_agents::services::mcp::service::McpServiceImpl::arc(global_dir.clone()),
            skills: ship_agents::services::skills::service::SkillsServiceImpl::arc(global_dir.clone()),
            agents: ship_agents::services::agents::service::AgentsServiceImpl::arc(global_dir.clone()),
            pipelines: ship_pipeline_runtime::pipelines::service::PipelinesServiceImpl::arc(global_dir.clone()),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            agent_def_kernel: std::sync::OnceLock::new(),
            agent_reactive: std::sync::OnceLock::new(),
            pipeline_reactive: std::sync::OnceLock::new(),
            pipeline_watch_kernel: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            shutdown_token: std::sync::OnceLock::new(),
            live_resolvers: Arc::new(crate::live_resolvers::build()),
        })
    }

    /// Clean up daemon files on shutdown.
    ///
    /// Emits a `supervisor.shutting_down` event, then drains managers before
    /// removing daemon files.
    pub async fn shutdown(&self) {
        // Cancel the shutdown token (#256) so any SSE keepalive/data loops still
        // selecting on it exit promptly. The signal handler already cancels the
        // parent token before `axum::serve` returns; this is the belt-and-
        // suspenders for the post-serve teardown path.
        if let Some(token) = self.shutdown_token.get() {
            token.cancel();
        }

        // Emit supervisor.shutting_down event before drain/cleanup.
        // Use spawn_blocking-style guard: route() may call block_in_place
        // which panics on single-threaded runtimes, so we catch that.
        if let Ok(envelope) = ship_substrate::events::EventEnvelope::new(
            "supervisor.shutting_down",
            "supervisor",
            &serde_json::json!({}),
        ) {
            let ctx = EmitContext::daemon_internal(None);
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

        // Checkpoint the WAL across every per-scope db the kernel sink mounted
        // (`runtime.db`, `account.db`, each touched `spaces/<id>/space.db`) so
        // no scope's write-ahead log is left un-truncated at stop. Runs after
        // the shutting_down emit + drain, so the final events are durable
        // before the checkpoint. Best-effort: a flush failure must not block
        // shutdown.
        if let Err(e) = self.kernel.lock().await.flush_wal_all_scopes() {
            tracing::warn!("shipd: WAL flush across scope dbs failed: {e}");
        }

        // Remove pid and port (liveness indicators) but keep the token.
        // Preserving the token across restarts means CLI clients that cached
        // it don't need to re-handshake after a daemon restart.
        crate::daemon::remove_liveness_files(&self.daemon_dir);
        tracing::info!("shipd: shutdown complete");
    }
}

/// Construct a `SpaceContext` with the `DataStore` bound to the space's per-scope
/// event db (`spaces/<id>/space.db`) rather than the shared `data.db`.
/// Collections AND artifact metadata are event-sourced: the
/// `CollectionProjection` / `ArtifactProjection` write their tables into that
/// per-scope db inside the append transaction, so the read path must query the
/// SAME file (ADR D6/D7).
pub fn build_space_db_context_arc(
    kernel: super::SharedKernel,
    ship_dir: &Path,
    space_id: &str,
) -> Arc<ship_substrate::ports::SpaceContext> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

    let space_db = ship_substrate::db::space_db_path(ship_dir, space_id);
    let data_svc =
        DataService::open_at(&space_db).expect("data service over space.db");
    let data_store = Arc::new(SingleBackendDataStore::new(data_svc, space_id));
    let kernel_log = Arc::new(KernelLog::new(kernel));
    Arc::new(SpaceContext::new(SpaceId::new(space_id), data_store, kernel_log))
}

/// Emit a single INFO line with event-log health at startup. Gives operators
/// an immediate signal: how many events persisted, the newest id. If the
/// count ever drops between restarts without a reset, that's the first
/// thing to investigate.
fn log_event_log_summary(global_dir: &Path) {
    // The kernel sink writes the Runtime partition to runtime.db; its presence
    // is the "have we persisted any events yet" signal.
    if !ship_substrate::db::runtime_db_path(global_dir).exists() {
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
