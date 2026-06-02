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
            // In-process Rolldown bundler (#163) — no worker script, no child
            // process. Bundles `repo_root`'s built-ins + the user `workspace`.
            deck::bundler::Bundler::start(&home, repo_root, workspace)
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
    pub async fn mesh(&self) -> &crate::managers::MeshManager {
        self.mesh
            .get_or_init(|| async {
                crate::managers::MeshManager::new(self.mesh_registry.clone())
            })
            .await
    }

    /// Lazily initialize and return the job manager.
    ///
    /// On first call, subscribes to job events and reconciles orphaned
    /// dispatched jobs from previous daemon runs.
    pub async fn jobs(&self) -> &crate::managers::RunManager {
        let hr = self.health_registry.clone();
        let registry = self.session_registry.clone();
        let daemon_url = self.daemon_origin();
        self.jobs
            .get_or_init(|| async {
                crate::managers::RunManager::init(
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
    pub async fn session_scanner(&self) -> &crate::managers::SessionScanner {
        let hr = self.health_registry.clone();
        self.session_scanner
            .get_or_init(|| async {
                crate::managers::SessionScanner::start(self.kernel.clone(), &hr)
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
                    let dispatcher = ship_platform::sandbox::dispatcher();
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
