//! L0 init split into submodules — daemon composition (`paths`, `startup`,
//! `identity_files`) re-exported here so callers reach them as
//! `ship_runtime::init::X`.
//!
//! Submodules:
//! - `paths`          — path discovery, env-var wiring, port derivation.
//! - `startup`        — runtime subsystem startup, identity files, session install.
//! - `identity_files` — pid/port/workspace/sha writers + stale-file cleanup.
//! ShipRuntime construction and shutdown. Test ctors live in `test_ctors`.

mod paths;
mod startup;
pub mod identity_files;

pub use paths::{DaemonPaths, DaemonConfig, resolve_daemon_paths, resolve_daemon_config, apply_path_env_vars};
pub use startup::{
    start_runtime_subsystems, write_identity_files, install_session_subsystems,
};
pub use identity_files::*;

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
         // The kernel routes every event into its per-scope db
         // (runtime.db / account.db / spaces/<id>/space.db) via PartitionedSink.
         // The space/account dbs are provisioned lazily on first persist; the
         // runtime db is provisioned eagerly here. data.db is opened once at
         // startup so the per-collection schema migrations run (no-op on a
         // fresh install). platform.db is dead — no boot step touches it.
         //
         // Install platform-level projections + supervisor hooks before
         // any SqliteEventStore is constructed.
         // (Engine cut 2026-07-10: the pipeline-run projection is gone;
         // install_projections() is now a no-op and dropped from the boot path.)
         ship_platform::install_defaults();
 
         let home = ship_substrate::ShipHome::from_env();
         let dir = home.data_dir().to_path_buf();
         let kernel = tokio::task::spawn_blocking(move || {
             ship_substrate::home::ensure_layout(&home)
                 .context("ensure ship home layout")?;
             // data.db is gone (S5.3): the active data plane (collections,
             // _schema_registry) lives per-scope in space.db via
             // DataService::open_at; there is no machine-wide data.db to provision.
             ship_substrate::events::init_kernel_router(dir)
                 .map_err(|e| anyhow!("failed to initialize KernelRouter: {e}"))
         })
         .await
         .map_err(|e| anyhow!("spawn_blocking failed: {e}"))??;

        // One-line startup status: events persisted, latest event id. Makes
        // it easy to spot if a restart "lost" events (a mismatched count vs.
        // a known-good baseline is the first signal of database trouble).
        log_event_log_summary(global_dir);

        // Build the single active-space `SpaceContext`
        // and the ONE shared schema registry, here (C-2 §3.1 / UD-7). The
        // `collections()`/`artifacts()` accessors construct stateless services
        // over this per call; both wire mounts (collections.proto +
        // artifacts.proto) ride the SAME ctx so a wire-created schema is
        // visible to trigger inserts. Collections AND artifact metadata are
        // event-sourced (ADR D6/D7): the read path must bind to the SAME
        // per-scope `spaces/<id>/space.db` that the `CollectionProjection` /
        // `ArtifactProjection` materialize the table into, so a query reads
        // exactly what the projection wrote.
        // The active space is the daemon's served space — its mint-once
        // `space_id` (a nanoid) the runtime assigns at boot through
        // `ship_space::identity::ensure_identity_for_root`. Identity's
        // canonical home is the `spaces` Runtime-scoped projection (folded
        // from `space.opened`); the daemon caches the identity here so
        // every reader (`active_space_id`, `SpaceService.Info`, the
        // dispatch chain's `spaces/<id>/space.db` partition) reaches one
        // source of truth. With NO discoverable project the daemon boots
        // space-less: `served_space_id = None`, the ctx is NOT built, and
        // a space-scoped op errors cleanly. There is NO daemon home space
        // and NO `global` pseudo-space to fall back onto (S2 home-space
        // kill).
        //
        // The accessor takes a `&KernelLog` (the broadcast-on-emit wrapper
        // ship-space's space-service seam expects). The underlying router is
        // shared across reads; the `KernelLog` constructor just attaches a
        // broadcast channel.
        let served_space_id: Option<String> = {
            let kernel_log =
                std::sync::Arc::new(ship_substrate::ports::KernelLog::new(kernel.clone()));
            let probe = workspace_dir.clone().or_else(|| std::env::current_dir().ok());
            match probe.as_deref() {
                Some(p) => {
                    match ship_space::identity::ensure_identity_for_root(
                        &kernel_log, p, None,
                    )
                    .await
                    {
                        Ok(rec) => Some(rec.space_id),
                        Err(e) => {
                            // No `.ship/` presence-walk match, or kernel emit
                            // failed. Both are loud-but-non-fatal: shipd
                            // boots space-less (the S2 posture). The log
                            // carries a single warn for the operator.
                            tracing::warn!(
                                error = %e,
                                probe = %p.display(),
                                "shipd: identity mint-or-read failed; booting space-less"
                            );
                            None
                        }
                    }
                }
                None => None,
            }
        };
        // One `SpaceContext` for both collections AND artifacts — they share
        // the same per-scope `spaces/<id>/space.db` (ADR D6/D7), so binding
        // a single ctx with one `DataService::open_at` + one `KernelLog` is
        // correct. The previous code opened two `DataService` connections
        // over the same file (collection_ctx + artifact_ctx) and stood up two
        // `KernelLog` instances — collapsed here.
        let space_ctx = served_space_id
            .as_deref()
            .and_then(|id| build_space_db_context_arc(kernel.clone(), global_dir, id));
        let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
        // Hydrate user-authored collection schemas from the descriptor into
        // the shared registry so they are queryable alongside the built-ins.
        // Post format-migration Wave 4 phase A: collections arrive via
        // `defineCollection` declare-forms in the space's `ship.config.ts`
        // and land in the descriptor's `library.collections` (a
        // `Vec<compiler::CollectionDef>`). `hydrate_registry` converts
        // those to `SchemaDoc` and registers them; the previous
        // `.ship/collections/*.jsonc` file walk is gone — see
        // `census-format-migration.md` Section 4 row 3.
        {
            // Probe the active workspace; a missing one yields an empty
            // hydrate (built-ins only). The descriptor's own roll-up
            // is also done at `start_agent_def_index` time; we re-roll
            // here so the schema registry is seeded before any
            // collection op lands.
            let probe = workspace_dir.clone().or_else(|| std::env::current_dir().ok());
            if let Some(workspace) = probe.as_deref() {
                match ship_resolver::roll_up(workspace, None, None).await {
                    Ok(composed) => {
                        ship_store::collection_schemas::hydrate_registry(
                            &schemas,
                            &composed.library.collections,
                        );
                    }
                    Err(e) => tracing::warn!(
                        error = %e,
                        "shipd: descriptor roll-up for collection hydration failed"
                    ),
                }
            }
        }
        let global_blob_dir = global_dir.clone();

        // Account identity is served by the typed Connect `AccountService`
        // (account.proto, `ship-account` crate). Its `AccountWire` is built in
        // `rpc::ops::router` from `daemon_dir` + `SHIP_GATEWAY_URL`; the runtime
        // no longer holds an account service in `ApiState`.

        Ok(Self {
            config,
            kernel,
            // The struct gained a `ship_dir: PathBuf` field after the
            // production ctor was last edited. The test ctor already
            // passes it through (apps/shipd/src/ship_runtime/test_ctors.rs);
            // the production ctor must do the same or it fails E0063.
            // The ship's home root is the natural value here — every
            // downstream accessor (`build_space_db_context_arc`, the
            // daemon_dir computation, the workspace derivation) already
            // takes the same path. Test ctor takes it as an argument so
            // the test can use a tmpdir; production ctor derives it from
            // `meta.global_dir` (the canonical ship home per the docs at
            // `crates/core/ship-substrate/src/project/global.rs`).
            ship_dir: global_dir.clone(),
            // Per-scope dbs live at the ship-home root (`runtime.db`,
            // `account.db`, `spaces/<id>/space.db`); `data.db` + `kv.db`
            // live under `~/.ship/machine/db/`. A workspace attachment does
            // NOT fork a second db into `<workspace>/.ship/` — see
            // `crates/core/ship-substrate/src/db/mod.rs`. `platform.db` was
            // deleted in the Phase-4 storage partition; no boot step
            // opens it.
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
                let home = ship_substrate::ShipHome::from_env();
                match wt_root.as_ref() {
                    Some(r) => home.worktree_daemon_dir(&ship_substrate::home::path_hash(r)),
                    None => home.daemon_dir(),
                }
            },
            #[cfg(feature = "ship-code")]
            dispatch_handle: std::sync::OnceLock::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            space_ctx,
            schemas,
            global_blob_dir,
            pty_sessions: ship_terminal::SessionRegistry::new(),
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
            served_space_id: std::sync::OnceLock::from(served_space_id),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            agent_def_kernel: std::sync::OnceLock::new(),
            agent_def_index: std::sync::OnceLock::new(),
            agent_reactive: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            shutdown_token: std::sync::OnceLock::new(),
            live_resolvers: Arc::new(crate::ship_runtime::live_resolvers::build()),
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
        crate::ship_runtime::init::identity_files::remove_liveness_files(&self.daemon_dir);
        tracing::info!("shipd: shutdown complete");
    }
}

/// Construct a `SpaceContext` with the `DataStore` bound to the space's per-scope
/// event db (`spaces/<id>/space.db`) rather than the shared `data.db`.
/// Collections AND artifact metadata are event-sourced: the
/// `CollectionProjection` / `ArtifactProjection` write their tables into that
/// per-scope db inside the append transaction, so the read path must query the
/// SAME file (ADR D6/D7).
///
/// Returns `None` if the per-scope `space.db` couldn't be opened (locked,
/// corrupt, permission-denied, missing parent dir, etc.). The caller treats
/// `None` as "daemon boots space-less" — the same posture it already takes
/// when no active space is resolvable (see `ShipRuntime::new`). The daemon
/// continues to serve runtime-scope RPCs; space-scoped ops error cleanly via
/// the existing space-less accessors (`collections()`/`artifacts()`
/// return `anyhow::Error`, search fan-out degrades per-space to a stub
/// context, agent stack still runs). The loss is loud: a `tracing::error!`
/// at the open failure so it shows up in any daemon log ahead of the
/// health endpoint going unhealthy.
pub fn build_space_db_context_arc(
    kernel: super::SharedKernel,
    ship_dir: &Path,
    space_id: &str,
) -> Option<Arc<ship_substrate::ports::SpaceContext>> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

    let space_db = ship_substrate::db::space_db_path(ship_dir, space_id);
    let data_svc = match DataService::open_at(&space_db) {
        Ok(svc) => svc,
        Err(e) => {
            tracing::error!(
                space_id = %space_id,
                space_db = %space_db.display(),
                error = %e,
                "shipd: failed to open space.db at boot — daemon is degrading to space-less mode; \
                 space-scoped ops (collections/artifacts/items/search) will error until space.db is repaired"
            );
            return None;
        }
    };
    let data_store = Arc::new(SingleBackendDataStore::new(data_svc, space_id));
    let kernel_log = Arc::new(KernelLog::new(kernel));
    Some(Arc::new(SpaceContext::new(SpaceId::new(space_id), data_store, kernel_log)))
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
