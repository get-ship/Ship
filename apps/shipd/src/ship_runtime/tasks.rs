//! ShipRuntime lazy managers, background tasks, event routing, and bundler init.

use anyhow::Context;

use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};

use super::ShipRuntime;

impl ShipRuntime {
    /// Lazily start and return the in-process Rolldown bundler (#163).
    ///
    /// `Bundler::start` locates the `sui-compiler` binary and returns a handle
    /// — no child process, no worker script, so nothing to supervise. The repo
    /// root defaults to the workspace dir (or cwd as fallback) since that's
    /// where `packages/` and `views/` live; the bundler compiles `.sui` by
    /// spawning that compiler in its Rolldown `load` hook.
    pub fn bundler(&self) -> &deck::bundler::Bundler {
        self.bundler.get_or_init(|| {
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
            // Load deck.jsonc from the repo root — the single source of truth
            // for consumer bundle targets, watch dirs, and host_assets_root.
            let cfg = deck::config::DeckConfig::load(&repo_root);
            // In-process Rolldown bundler (#163) — no worker script, no child
            // process. Bundles `repo_root`'s built-ins + the user `workspace`.
            deck::bundler::Bundler::start(repo_root, workspace, cfg)
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

    /// Spawn the artifact GC loop. Runs immediately on startup then every hour.
    ///
    /// Best-effort: errors are logged but don't take down the daemon. No-op when
    /// the daemon booted space-less (no active space → no artifact store to sweep).
    pub fn start_artifact_gc(&self) {
        let Some(space_ctx) = self.space_ctx.clone() else {
            tracing::info!("artifact GC: no active space, GC loop not started");
            return;
        };
        let global_dir = self.global_blob_dir.clone();
        ship_substrate::task_health::spawn_supervised(
            &self.health_registry,
            "artifact-gc",
            move || {
                let artifact_ctx = space_ctx.clone();
                let global_dir = global_dir.clone();
                async move {
                    loop {
                        let svc = ship_artifacts::gc::make_service_for_gc(
                            artifact_ctx.clone(),
                            global_dir.clone(),
                        );
                        if let Err(e) = ship_artifacts::gc::collect_expired(&svc).await {
                            tracing::warn!("artifact GC run failed: {e}");
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                    }
                }
            },
        );
    }

    /// Start the window-reaper loop (sweeps every 30s).
    ///
    /// Window actors self-spawn and report their pid; a clean close emits
    /// `deregister_window`, but a crash leaves the row non-terminal with a
    /// dead pid. The launcher's single-instance query already pid-probes, so
    /// it's honest either way — but the event log never recorded the death.
    /// This loop emits `actor.stopped` for vanished windows so the lifecycle
    /// is closed in the log. CLEANUP ONLY — the launcher owns (re)spawn.
    ///
    /// `reap_dead_windows` does a durable append (`block_on`), so it runs on a
    /// `spawn_blocking` thread — never inline on this async worker.
    pub fn start_window_reaper(&self) {
        ship_substrate::task_health::spawn_supervised(
            &self.health_registry,
            "window-reaper",
            move || async move {
                loop {
                    match tokio::task::spawn_blocking(
                        ship_substrate::db::actor_events::reap_dead_windows,
                    )
                    .await
                    {
                        Ok(Ok(reaped)) if !reaped.is_empty() => {
                            tracing::info!(count = reaped.len(), ?reaped, "window-reaper: closed dead windows")
                        }
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => tracing::warn!("window-reaper: sweep failed: {e}"),
                        Err(e) => tracing::warn!("window-reaper: join error: {e}"),
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
            },
        );
    }

    /// Start the sandbox reaper: periodically scans the dispatcher
    /// for sandboxes idle past their timeout and tears them down.
    ///
    /// Reap = registry state transition only (Idle → Reaping →
    /// Stopped). Per-sandbox backend teardown (Docker rm, youki
    /// delete) folds in once SandboxOwner moves from per-actor to
    /// per-sandbox (separate chunk). Today the registry's "idle
    /// past timeout" decision is the load-bearing bit; the actual
    /// container lifetime tracking happens in the existing
    /// per-actor SandboxOwner.Drop path.
    pub fn start_sandbox_reaper(&self) {
        ship_substrate::task_health::spawn_supervised(
            &self.health_registry,
            "sandbox-reaper",
            move || async move {
                loop {
                    let dispatcher = ship_device::sandbox::dispatcher();
                    let now = chrono::Utc::now();
                    let candidates = dispatcher.reap_candidates(now).await;
                    for id in candidates {
                        match dispatcher.reap(&id).await {
                            Ok(_) => tracing::info!(sandbox_id = %id.0, "sandbox reaped (idle)"),
                            Err(e) => tracing::warn!(
                                sandbox_id = %id.0,
                                "sandbox reap failed: {e}"
                            ),
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
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
        match ship_triggers::engine::spawn_hook_service(
            self.kernel.clone(),
            library_dir,
            self.schemas.clone(),
        )
        .await
        {
            Ok(engine) => {
                // Also start the filesystem watcher for hot-reload. Bind it to a
                // long-lived field — dropping the HookWatcher aborts the reload task,
                // so it MUST live for the daemon's lifetime.
                match ship_triggers::watcher::spawn_watcher(engine.clone()) {
                    Ok(w) => { let _ = self.hook_watcher.set(w); }
                    Err(e) => tracing::warn!("hook watcher failed to start: {e}"),
                }
                let _ = self.hook_engine.set(engine);
                tracing::info!("hooks: engine started");
            }
            Err(e) => tracing::warn!("hooks: failed to start engine: {e}"),
        }
    }

    /// The webhook source for the public webhook route, if the engine is running.
    pub fn webhook_source(
        &self,
    ) -> Option<std::sync::Arc<ship_triggers::sources::webhook::WebhookSource>> {
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

    /// Mount the live agent-def index (agents arc S5). Builds the read-only
    /// index over the active project space's `space.db` from the def files at
    /// boot, then spawns the change consumer that keeps it live on
    /// `agent.def.updated`. Both — plus the gated-write NOTIFY — ride ONE shared
    /// `KernelLog` (`agent_def_kernel`) so the op-emit and the consumer's
    /// `subscribe()` share a broadcast. No-op when the daemon booted space-less
    /// (no workspace) or when already mounted. Idempotent.
    pub async fn start_agent_def_index(&self) {
        use ship_substrate::data::DataService;
        use ship_substrate::ports::{KernelLog, SingleBackendDataStore};

        if self.agent_def_kernel.get().is_some() {
            return;
        }
        let Some(workspace) = self.workspace_dir.clone() else {
            tracing::info!("agent-def index: no workspace — skipping live mount");
            return;
        };
        let project_ship_dir = ship_substrate::ShipHome::project_ship_dir(&workspace);
        // Active-space identity = path-hash of the DISCOVERED project root — the
        // same resolution collection/artifact contexts use (init.rs). Space-less
        // workspace → no index; the daemon owns no home space to fall back on.
        let Some(space_id) = ship_substrate::project::resolve_active_space(&workspace) else {
            tracing::info!("agent-def index: workspace is space-less — skipping live mount");
            return;
        };

        // The index read-model lives in the active space's per-scope db under the
        // GLOBAL ship-home (~/.ship | $XDG_DATA_DIR), NEVER the project tree
        // (#279). `project_ship_dir` is for reading the def FILES (below), not
        // the db; rooting the db there forks a per-space store into the repo.
        let space_db = ship_substrate::db::space_db_path(self.ship_dir(), &space_id);
        let data_svc = match DataService::open_at(&space_db) {
            Ok(svc) => svc,
            Err(e) => {
                tracing::warn!("agent-def index: open space.db failed: {e}");
                return;
            }
        };
        let store = std::sync::Arc::new(SingleBackendDataStore::new(data_svc, &space_id));
        let index = std::sync::Arc::new(ship_platform::config::AgentDefIndex::new(store));
        if let Err(e) = index.ensure_table().await {
            tracing::warn!("agent-def index: ensure_table failed: {e}");
            return;
        }

        // ONE shared KernelLog for emit + subscribe. Set it before spawning so
        // the gated-write NOTIFY (`agent_def_kernel()`) routes through the same
        // channel the consumer reads.
        let kernel = std::sync::Arc::new(KernelLog::new(self.kernel.clone()));
        let _ = self.agent_def_kernel.set(kernel.clone());

        // Mount the agent `Watch` reactive registry on the SAME shared kernel —
        // its resolvers subscribe to this log's broadcast, so every gated-CRUD
        // `agent.def.updated` NOTIFY (which rides `agent_def_kernel`) re-runs the
        // affected snapshot. Reuses the existing shared log (no parallel
        // registry); mirrors how collection/space build a registry from their
        // scope kernel.
        let _ = self
            .agent_reactive
            .set(std::sync::Arc::new(ship_reactive::ReactiveRegistry::new(kernel.clone())));

        // Boot rebuild: index every def file on disk for the project scope.
        let fs = ship_substrate::ports::LocalFsBroker::new();
        if let Err(e) =
            ship_platform::config::agent_index::build_def_index(&index, &fs, &project_ship_dir, "project").await
        {
            tracing::warn!("agent-def index: boot rebuild failed: {e}");
        }

        // Spawn the live consumer over the shared KernelLog's broadcast.
        ship_platform::config::agent_index::spawn_def_index_consumer(
            index,
            kernel.subscribe(),
            project_ship_dir,
            "project".to_string(),
        );
        tracing::info!("agent-def index: live mount started (space {space_id})");
    }

    /// The shared `KernelLog` the live agent-def index rides, when mounted. The
    /// gated-write NOTIFY emits through this so the change reaches the consumer's
    /// broadcast subscription.
    pub fn agent_def_kernel(&self) -> Option<&std::sync::Arc<ship_substrate::ports::KernelLog>> {
        self.agent_def_kernel.get()
    }

    /// The agent `Watch` reactive registry (agents-arc E5), when the live
    /// def-index has been mounted. `None` in space-less boot — the Watch RPC
    /// then returns Unavailable rather than fabricate a registry.
    pub fn agent_reactive(&self) -> Option<&std::sync::Arc<ship_reactive::ReactiveRegistry>> {
        self.agent_reactive.get()
    }

    /// Stand up the typed pipeline `Watch` reactive registry (pipelines P0.4)
    /// and its bridge.
    ///
    /// The pipeline orchestrator emits `pipeline.*` events through the raw
    /// `KernelRouter` (no shared `KernelLog`), and child-step `session.*`
    /// terminals likewise. The registry's resolvers, however, wake off a
    /// `KernelLog` broadcast. This method bridges the two: it builds a dedicated
    /// `KernelLog`, mounts a `ReactiveRegistry` on it, and spawns a kernel actor
    /// subscribed to the `pipeline.` + `session.` namespaces whose mailbox drain
    /// re-`broadcast()`s each event into the log. N Watch subscribers on one
    /// `WatchKey` then share one resolver + one kernel subscription. Idempotent;
    /// non-fatal on actor-spawn failure (the Watch RPC then returns Unavailable).
    pub async fn start_pipeline_reactive(&self) {
        use ship_substrate::events::ActorConfig;
        use ship_substrate::ports::KernelLog;

        let kernel_log = std::sync::Arc::new(KernelLog::new(self.kernel.clone()));
        if self.pipeline_watch_kernel.set(kernel_log.clone()).is_err() {
            return; // already started
        }
        let _ = self
            .pipeline_reactive
            .set(std::sync::Arc::new(ship_reactive::ReactiveRegistry::new(kernel_log.clone())));

        let actor_id = format!("pipeline-watch-bridge-{}", ship_substrate::gen_ulid());
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            // Prefix-matched against event_type (`starts_with`): the run/step
            // lifecycle + the child-session terminals the projections fold.
            subscribe_namespaces: vec!["pipeline.".to_string(), "session.".to_string()],
            // session.* terminals are directed events — opt in to see them.
            see_directed_events: true,
        };
        let mailbox = {
            let mut kernel = self.kernel.lock().await;
            match kernel.spawn_actor(&actor_id, config) {
                Ok((_store, mb)) => mb,
                Err(e) => {
                    tracing::warn!("pipeline-watch bridge: actor spawn failed: {e}");
                    return;
                }
            }
        };
        tokio::spawn(async move {
            let mut mb = mailbox;
            while let Some(event) = mb.recv().await {
                kernel_log.broadcast(event);
            }
        });
        tracing::info!("pipeline Watch reactive registry + bridge started");
    }

    /// The pipeline `Watch` reactive registry (pipelines P0.4), when mounted.
    /// `None` until `start_pipeline_reactive` runs — the Watch RPC then returns
    /// Unavailable rather than fabricate a registry.
    pub fn pipeline_reactive(&self) -> Option<&std::sync::Arc<ship_reactive::ReactiveRegistry>> {
        self.pipeline_reactive.get()
    }

    /// Persist event to its scope db and deliver through the kernel.
    ///
    /// Delegates to `KernelRouter::emit` — the single write path. A successful
    /// return means the event is durable on disk AND delivered to in-memory
    /// subscribers.
    pub async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        // Relay: carry the envelope's own resolved space through the binding,
        // preserving its session. Daemon-internal mint (trusted relay).
        let ctx = EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            binding: ship_types::ScopeBinding::daemon_internal(
                envelope.space_id().map(ship_types::SpaceId::new),
                ship_types::Authority::Sdk,
            ),
            session_id: envelope.session_id.clone(),
        };
        self.kernel
            .lock()
            .await
            .emit(envelope.clone(), &ctx)
            .await
            .context("kernel emit")
    }
}
