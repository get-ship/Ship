//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands.

/// Test-load initializer — runs once when the lib test binary loads.
/// Registers ship-platform's default projection set + supervisor hook so
/// any `SqliteEventStore` constructed inside a unit test sees the full
/// bus. Idempotent. Production path uses an explicit
/// `ship_platform::install_defaults()` call from `ship_runtime::new()`
/// (see apps/shipd/src/ship_runtime.rs).
#[cfg(test)]
#[ctor::ctor]
fn install_test_platform_defaults() {
    ship_platform::install_defaults();
    ship_pipeline_runtime::install_projections();
}

pub mod sessions;
pub mod auth;
pub mod wire;
pub mod transport;
mod daemon;
pub mod services;
pub mod managers;
pub mod routes;
pub mod router;
pub mod ops_rpc;
// diag_rpc: deleted — diagnostics.snapshot op cut; DiagnosticRegistry apparatus removed.
pub mod runtime_rpc;
pub mod pipelines_rpc;
// agent_rpc: policy + gate conformance tests for AgentsService.
// The impl AgentsService for OpsImpl is deleted — AgentsWire (ship-agents) is now
// the sole handler, mounted via AgentsServiceExt::register in ops_rpc.rs.
pub mod agent_rpc;
pub mod agent_event_ports;
pub mod pipeline_ports;
pub mod view_ports;
pub mod space_rpc;
pub mod settings_rpc;
pub mod device_ports;
pub mod account_rpc;
pub mod events_rpc;
pub mod view_rpc;
pub mod registry_rpc;
pub mod sui_rpc;
pub mod search_context;
pub mod search_rpc;
pub mod live_read;
// (former shipd/integrations/ — apps_bridge → views/mcp_apps_embed; registry → routes/mcp_registry)
pub mod views;
pub mod ship_runtime;
// Job + agent-session dispatch composition root — `ShipdPlatform`,
// `GitWorktreeIsolation`, `run_dispatch::subscribe_job_events`, and orphan
// cleanup. Lifted out of `ship-supervisor` in the supervisor split: this
// wiring depends on `ship-pipeline-runtime`, which depends on `ship-agents`,
// so it lives at the daemon composition layer (not in either L1 crate).
pub mod dispatch;
// `ship-supervisor` (L0) now holds only OS-level process primitives
// (terminal launcher, process-event emit, graceful_shutdown re-export). The
// existing `crate::supervisor::*` import path resolves through this re-export.
pub use ship_supervisor as supervisor;
// Tests for supervisor + shipd-internal integration (use ApiState,
// ShipRuntime, etc.) live under `apps/shipd/src/supervisor/`.
#[cfg(test)]
#[path = "supervisor/mod.rs"]
mod supervisor_tests;

#[cfg(test)]
pub mod test_harness;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
#[path = "ship_runtime_tests.rs"]
mod ship_runtime_tests;

pub use daemon::{network_status, network_stop};
pub use deck::DaemonMode;

/// Test-only: build a minimal router with the CORS middleware for testing.
#[cfg(test)]
pub fn test_cors_router() -> axum::Router {
    axum::Router::new()
        .route("/test", axum::routing::get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(transport::cors::cors_middleware))
}

use anyhow::{Result, anyhow};
use ship_substrate::events::daemon::{
    PortBindFailedPayload, ShutdownCompletedPayload, StartupCompletedPayload,
    event_types as daemon_events,
};
use ship_substrate::events::{EventEnvelope, EventStore, SqliteEventStore};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Start the network daemon and block until shutdown (SIGINT/SIGTERM).
///
/// `dev` enables relaxed auth for local development (auto-pair, localhost-only).
/// Never set `dev = true` in production.
///
/// `daemon_mode` controls cache headers, HMR route availability, and SSR cache
/// behavior. Default is `Development`, which preserves all existing behaviour.
///
/// `port = None` means "auto-derive": pick the port via
/// `ship_substrate::home::worktree_port_for(project_root)`, which honors
/// `~/.ship/machine/local.jsonc:worktrees.ports[<branch>]` pins and
/// otherwise hashes the project root into `9315 + N % 100`. If the
/// derived port is busy, increments up to 10 times before giving up.
/// Outside any project, the default falls back to 9315.
pub async fn run_network(
    host: String,
    port: Option<u16>,
    dev: bool,
    workspace_dir: Option<std::path::PathBuf>,
    daemon_mode: deck::DaemonMode,
) -> Result<()> {
    if dev {
        tracing::warn!(
            "╔══════════════════════════════════════════════════════════════════╗"
        );
        tracing::warn!(
            "║  SHIPD DEV MODE — auth relaxed, browser clients auto-pair        ║"
        );
        tracing::warn!(
            "║  Listening on localhost only. NOT FOR PRODUCTION.                ║"
        );
        tracing::warn!(
            "╚══════════════════════════════════════════════════════════════════╝"
        );
    }
    // Dev mode forces localhost bind. Agents can set SHIP_HOST env vars,
    // but they can't flip the --dev flag on the daemon process they didn't start.
    let host = if dev { "127.0.0.1".to_string() } else { host };
    auth::pairing::set_dev_mode(dev);
    let global_dir = ship_substrate::project::get_global_dir()?;
    let init_started_at = std::time::Instant::now();

    // Per devx-04, pick the daemon IPC dir based on workspace cwd. A daemon
    // launched from inside a git worktree writes to
    // `<root>/machine/daemons/<hash(worktree_root)>/`; bare/global startup
    // writes to the singleton `<root>/machine/daemon/`. Hashing the
    // worktree's own root (not the redirected main project root) is what
    // lets each linked worktree run its own daemon without trampling the
    // others' IPC handshakes.
    let project_probe = workspace_dir.clone().or_else(|| std::env::current_dir().ok());
    let project_root = project_probe
        .as_ref()
        .and_then(|p| ship_substrate::home::ProjectRoot::discover(p));
    let worktree_root = project_probe
        .as_ref()
        .and_then(|p| ship_substrate::home::worktree_root_for(p))
        .or_else(|| project_root.as_ref().map(|r| r.path().to_path_buf()));
    let home = ship_substrate::ShipHome::at(global_dir.clone());
    let daemon_dir = match worktree_root.as_ref() {
        Some(root) => home.worktree_daemon_dir(&ship_substrate::home::path_hash(root)),
        None => home.daemon_dir(),
    };

    // The ship host configures deck's first-party view root — deck itself knows
    // no app layout (see .artifacts/asset-architecture.md). Gate on the shell
    // source actually being present (a dev worktree of this repo), NOT on the
    // `--dev` flag, which dev launchers pass inconsistently. In a user's project
    // / prod install there is no `apps/shell/src`, so the env stays unset and
    // the shell bundle owns first-party views.
    if let Some(root) = worktree_root.as_ref() {
        let fp_root = root.join("apps").join("shell").join("src");
        if fp_root.is_dir() {
            // SAFETY: set once at startup, before any request thread reads it.
            unsafe {
                std::env::set_var("SHIP_FIRST_PARTY_VIEWS", &fp_root);
            }
        }
    }

    // One-time hint for the worktree footgun: if `ProjectRoot::discover`
    // redirects to a different (main) checkout and there's a live daemon
    // sitting at the OLD (main-root) hash, surface it so the operator can
    // clean up at their leisure. Don't auto-kill — that's their call.
    if let (Some(pr), Some(wt)) = (project_root.as_ref(), worktree_root.as_ref())
        && pr.path() != wt.as_path()
    {
        let main_hash = ship_substrate::home::path_hash(pr.path());
        let main_dir = home.worktree_daemon_dir(&main_hash);
        if let Some(handle) = ship_substrate::home::DaemonHandle::load(&main_dir)
            && handle.is_alive()
        {
            tracing::warn!(
                old_dir = %main_dir.display(),
                old_pid = handle.pid.unwrap_or(0),
                "found a daemon at the main-worktree hash for this repo; \
                 kill it if you no longer use shipd from {}",
                pr.path().display()
            );
        }
    }

    daemon::cleanup_stale_daemon_files(&daemon_dir).map_err(|e| {
        eprintln!("\nshipd startup aborted: {e}\n");
        e
    })?;
    if let Some(ref w) = workspace_dir {
        tracing::info!(workspace = %w.display(), daemon_dir = %daemon_dir.display(), "shipd: project-scoped mode");
        // Make `--workspace` authoritative for the SUI resolver chain
        // (deck-core's resolver consults `SHIP_DEV_PROJECT_DIR`
        // first; without this it falls back to CWD + worktree-aware
        // resolution, which redirects into the parent checkout's
        // `views/` when the workspace lives inside a git worktree).
        // The explicit env var lets test fixtures and per-project
        // daemons stay scoped to their own `views/` directory.
        //
        // Leave any pre-existing value alone — callers that need a
        // different SUI resolver root than the daemon workspace
        // (rare; mostly studio in-place editing) can still override.
        // Routed through the typed `Var` registry rather than a raw env
        // literal, so the override stays accountable to the env.rs
        // taxonomy (no raw `set_var` of a string name).
        // SAFETY: shipd's process is single-threaded at this point in
        // startup — set_var is unsound only when other threads can
        // observe the env mid-update.
        use ship_substrate::env::{self, Var};
        if env::read(Var::DevProjectDir).is_none() {
            // SAFETY: pre-async-runtime, no concurrent env access.
            unsafe { env::set(Var::DevProjectDir, w); }
        }
    } else {
        tracing::info!(daemon_dir = %daemon_dir.display(), "shipd: global-only mode (no --workspace given)");
    }

    // Resolve the desired port. Caller-supplied --port wins; otherwise we
    // derive from the worktree root (path-hash + local.jsonc:worktrees.ports
    // pin, see ship_substrate::home::worktree_port_for). Using the worktree
    // root rather than the redirected main project root means each linked
    // worktree gets its own derived port slot. Outside any project, use the
    // historical default.
    let desired_port = port.unwrap_or_else(|| match worktree_root.as_ref() {
        Some(root) => ship_substrate::home::worktree_port_for(root),
        None => 9315,
    });
    let meta = ship_runtime::AppMetadata {
        global_dir: global_dir.clone(),
        workspace_dir: workspace_dir.clone(),
    };
    let config = ship_runtime::RuntimeConfig { host: host.clone(), port: desired_port, ..Default::default() };
    let runtime = ship_runtime::ShipRuntime::new(meta, config).await?;
    // The MeshService was the in-shipd mesh.* op surface; it was killed
    // along with the REST routes and `ship mesh broadcast` CLI. Real
    // inter-agent messaging is now native via the kernel's directed-event
    // delivery (`EventEnvelope.target_actor_id`).
    let rt = Arc::new(runtime);

    let run_mgr = managers::RunManager::init(
        rt.kernel().clone(),
        global_dir.clone(),
        rt.health_registry(),
        rt.session_registry().clone(),
        rt.daemon_origin(),
    )
    .await;
    if let Some(dh) = run_mgr.dispatch_handle {
        rt.set_dispatch_handle(dh);
    }
    // diagnostics.snapshot op deleted — no DiagnosticRegistry to build.
    // Start the artifact GC loop (runs now then every hour).
    rt.start_artifact_gc();
    // Start the window-reaper loop (closes the log lifecycle of crashed
    // shell windows that never deregistered; every 30s).
    rt.start_window_reaper();
    // Sandbox reaper: idle-past-timeout sandboxes get marked Stopped.
    // Per-sandbox backend teardown folds in once SandboxOwner moves
    // from per-actor to per-sandbox.
    rt.start_sandbox_reaper();
    daemon::human_gateway::spawn_human_gateway(rt.kernel(), rt.health_registry()).await;
    rt.start_hook_engine().await;
    rt.start_config_watcher();
    // Live agent-def index (agents arc S5): build the read-only index from the
    // def files at boot, then keep it live on `agent.def.updated`. Shares the
    // gated-write NOTIFY's `KernelLog` so op-writes re-index deterministically.
    rt.start_agent_def_index().await;
    // leak-193 diagnostics. When `SHIP_LEAK_DIAG=1`, log a periodic snapshot
    // of process RSS alongside every in-process container whose unbounded
    // growth could be the fanout RSS leak. Off by default — pure
    // observability, no behavior change.
    if std::env::var("SHIP_LEAK_DIAG").as_deref() == Ok("1") {
        let diag_kernel = rt.kernel().clone();
        let diag_sessions = rt.actor_sessions().clone();
        let diag_per_turn = rt.session_registry().clone();
        tokio::spawn(async move {
            fn rss_mb() -> f64 {
                let s = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
                let pages: u64 =
                    s.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                (pages * 4096) as f64 / (1024.0 * 1024.0)
            }
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                tick.tick().await;
                let kline = diag_kernel.lock().await.diag_line();
                let per_turn = diag_per_turn.lock().await.len();
                tracing::info!(
                    target: "leak193",
                    "DIAG rss={:.1}MB actor_sessions={} per_turn_sessions={} \
                     pre_reg_mailboxes={} {kline}",
                    rss_mb(),
                    diag_sessions.len(),
                    per_turn,
                    ship_substrate::supervisor::pre_registered_mailbox_count(),
                );
            }
        });
    }

    // Start the Bun bundler worker. In dev mode, also forward module_changed
    // events to the HMR SSE channel so browser clients get in-place updates.
    {
        let bundler = rt.bundler().clone();
        let mut events = bundler.subscribe_events();
        let mode = daemon_mode;
        tokio::spawn(async move {
            if let Err(e) = bundler.start_watching().await {
                tracing::warn!("bundler: could not start file watching: {e}");
            }
            loop {
                match events.recv().await {
                    Ok(deck::bundler::BundlerEvent::ModuleChanged { target, out_path, bytes }) => {
                        // Mirror view-targeted compiles onto the per-view
                        // SSE channel so view-edit-mode UI can flash a
                        // "saved" indicator without subscribing to /_sui/dev/hmr.
                        if let Some(view_name) =
                            views::sui_view_edit_sse::view_name_from_bundler_target(&target)
                        {
                            let payload = views::sui_view_edit_sse::CompileEventPayload {
                                view_name: view_name.to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                message: None,
                                line: None,
                                col: None,
                                file: None,
                            };
                            let view_name_owned = view_name.to_string();
                            tokio::spawn(async move {
                                views::sui_view_edit_sse::publish_compiled(
                                    &view_name_owned,
                                    &payload,
                                )
                                .await;
                            });
                        }
                        if mode == deck::DaemonMode::Development {
                            views::sui_hmr::publish_bundle_event(views::sui_hmr::BundleModuleEvent {
                                kind: views::sui_hmr::BundleModuleEventKind::ModuleChanged,
                                target,
                                out_path: Some(out_path),
                                bytes: Some(bytes),
                                message: None,
                                line: None,
                                col: None,
                                file: None,
                            });
                        }
                    }
                    Ok(deck::bundler::BundlerEvent::CompileError { target, message, line, col, file }) => {
                        if let Some(view_name) =
                            views::sui_view_edit_sse::view_name_from_bundler_target(&target)
                        {
                            let payload = views::sui_view_edit_sse::CompileEventPayload {
                                view_name: view_name.to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                message: Some(message.clone()),
                                line,
                                col,
                                file: file.clone(),
                            };
                            let view_name_owned = view_name.to_string();
                            tokio::spawn(async move {
                                views::sui_view_edit_sse::publish_errored(
                                    &view_name_owned,
                                    &payload,
                                )
                                .await;
                            });
                        }
                        if mode == deck::DaemonMode::Development {
                            views::sui_hmr::publish_bundle_event(views::sui_hmr::BundleModuleEvent {
                                kind: views::sui_hmr::BundleModuleEventKind::CompileError,
                                target,
                                out_path: None,
                                bytes: None,
                                message: Some(message),
                                line,
                                col,
                                file,
                            });
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Register the launch-time project (if any) into the space registry so the
    // launcher / shell sees it on first list. Idempotent — ship-space emits
    // `project.seen` if already known.
    //
    // A discoverable project root IS the active space; the daemon has no separate
    // "home space" to stamp into a process-global (S2 home-space kill). The active
    // space is resolved per-request / at boot from the probe path via
    // `ship_substrate::project::resolve_active_space`. With no discoverable
    // project the daemon boots space-less. See spec
    // `.artifacts/specs/space/116-event-partition.md`.
    if let Some(root) = project_root.as_ref() {
        use ship_substrate::ports::{KernelLog, SingleBackendDataStore};
        use ship_substrate::data::DataService;
        let runtime_db = ship_substrate::db::runtime_db_path(&global_dir);
        let store = {
            let svc = DataService::open_at(&runtime_db).expect("data service over runtime.db");
            std::sync::Arc::new(SingleBackendDataStore::new(svc, "runtime"))
                as std::sync::Arc<dyn ship_substrate::ports::DataStore>
        };
        let kernel = std::sync::Arc::new(KernelLog::new(rt.kernel().clone()));
        let canonical = std::fs::canonicalize(root.path()).unwrap_or_else(|_| root.path().to_path_buf());
        if let Err(e) = ship_space::registry::add(&*store, &kernel, &canonical, None).await {
            tracing::warn!(path = %root.path().display(), "failed to register project on startup: {e}");
        }
    }

    let token = match auth::read_token(&daemon_dir) {
        Ok(existing) if !existing.is_empty() => Arc::new(existing),
        _ => {
            let new_token = auth::generate_token()
                .map_err(|e| anyhow!("failed to generate daemon token: {e}"))?;
            auth::write_token_file(&daemon_dir, &new_token).map_err(|e| anyhow!("write daemon.token: {e}"))?;
            Arc::new(new_token)
        }
    };
    daemon::write_pid_file(&daemon_dir)?;
    // Record the *worktree* this daemon serves, not the main-checkout
    // root we'd get from ProjectRoot::discover's redirect. `just status`
    // and the e2e visual harness consult this to verify they're talking
    // to the daemon scoped to their cwd.
    if let Some(root) = worktree_root.as_ref() {
        daemon::write_workspace_file(&daemon_dir, root)?;
    } else if let Some(root) = project_root.as_ref() {
        daemon::write_workspace_file(&daemon_dir, root.path())?;
    }
    daemon::write_sha_file(&daemon_dir, build_info::build_info!().git_sha)?;

    let ct = CancellationToken::new();
    let ct2 = ct.clone();
    tokio::spawn(async move { daemon::shutdown_signal().await; ct2.cancel(); });

    let session_registry = rt.session_registry().clone();
    // Install the process-wide registry handle so the host-side spawn path
    // (supervisor) can mint per-process tokens for sandboxed actors without
    // threading a registry through the event→orchestrator→executor chain.
    ship_auth::scopes::set_session_registry(session_registry.clone());
    // Install the daemon's reachable origin so the spawn path can inject
    // SHIP_DAEMON_URL into sandboxed actors (rewritten to a
    // container-reachable host for Tier 2+). Same boot-install pattern as
    // the session registry above.
    ship_platform::isolation::set_daemon_origin(rt.daemon_origin());
    // Hydrate any persisted paired sessions from a previous daemon lifetime.
    // Dev mode starts fresh — don't carry prior sessions across dev restarts.
    if !dev {
        auth::hydrate_sessions(&session_registry).await;
    }
    // Background sweep — evict expired sessions from the in-memory
    // registry every 5 minutes and emit `auth.session.expired` for each.
    // The HTTP lookup path already filters expired rows; this is the
    // audit-event source.
    auth::spawn_session_sweeper(session_registry.clone());
    let pairing_registry = auth::pairing::new_shared();
    let attestation_verifier = auth::attest::AttestationVerifier::new();
    let app = router::build_router(
        rt.clone(),
        token,
        session_registry,
        pairing_registry,
        attestation_verifier,
        ct.child_token(),
        daemon_mode,
    );
    // Bind the listener with collision retry. On EADDRINUSE we increment
    // the port up to 9 times before giving up. The actually-bound port is
    // what we write to <daemon_dir>/port — the derived port is just a hint.
    let user_pinned = port.is_some();
    let max_retries = if user_pinned { 1 } else { 10 };
    let mut bound_port = desired_port;
    let listener = loop {
        let addr = format!("{host}:{bound_port}");
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => break l,
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && !user_pinned => {
                tracing::warn!(
                    port = bound_port,
                    "shipd: port busy, trying next"
                );
                bound_port += 1;
                if bound_port - desired_port >= max_retries {
                    let already_bound_by_pid = pidfile_pid_for_port(&global_dir, desired_port);
                    emit_daemon_event(
                        &global_dir,
                        daemon_events::DAEMON_PORT_BIND_FAILED,
                        "daemon",
                        &PortBindFailedPayload {
                            port: desired_port,
                            already_bound_by_pid,
                            error: e.to_string(),
                        },
                    );
                    return Err(anyhow!(
                        "bind {host}:{desired_port}..{}: {} (giving up after {max_retries} retries)",
                        bound_port - 1,
                        e,
                    ));
                }
            }
            Err(e) => {
                let already_bound_by_pid = pidfile_pid_for_port(&global_dir, bound_port);
                emit_daemon_event(
                    &global_dir,
                    daemon_events::DAEMON_PORT_BIND_FAILED,
                    "daemon",
                    &PortBindFailedPayload {
                        port: bound_port,
                        already_bound_by_pid,
                        error: e.to_string(),
                    },
                );
                return Err(anyhow!("bind {host}:{bound_port}: {e}"));
            }
        }
    };
    let port = bound_port;
    daemon::write_port_file(&daemon_dir, port)?;

    let addr = format!("{host}:{port}");
    tracing::info!("shipd: listening on http://{addr}/mcp");
    tracing::info!("shipd: shell: http://{addr}/shell/");
    eprintln!("shell: http://{addr}/shell/");

    // Daemon identity line — single source of truth for "which binary,
    // which checkout, which project dir is serving :{port}". Surfaced in
    // both the structured log and on stderr so `scripts/dev-daemon.sh`
    // can confirm the worktree's daemon (not the installed one) is live.
    let identity_ship_dir = rt
        .workspace_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!("{} (global-only)", global_dir.display()));
    let bi = build_info::build_info!();
    let identity = format!(
        "shipd {}-{} listening on {} · ship_dir={} · pid={}",
        env!("CARGO_PKG_VERSION"),
        bi.short(),
        port,
        identity_ship_dir,
        std::process::id(),
    );
    tracing::info!("{identity}");
    eprintln!("{identity}");

    let summary = ship_substrate::db::event_log_summary(&global_dir);
    let startup = StartupCompletedPayload {
        pid: std::process::id(),
        port,
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_sha: build_info::build_info!().git_sha.to_string(),
        event_count_at_startup: summary.event_count,
        latest_event_id: summary.latest_event_id,
        init_duration_ms: init_started_at.elapsed().as_millis() as u64,
    };
    if let Ok(envelope) =
        EventEnvelope::new(daemon_events::DAEMON_STARTUP_COMPLETED, "daemon", &startup)
    {
        let envelope = envelope.with_actor_id("daemon");
        if let Err(e) = rt.persist_and_route(&envelope).await {
            tracing::warn!("shipd: failed to emit daemon.startup_completed: {e}");
        }
    }

    // First-run UX: if `data.db` doesn't exist yet, open the default
    // browser at `/setup` once the listener is bound. Harmless no-op on
    // subsequent starts; suppressible via `SHIP_NO_BROWSER=1`.
    daemon::first_run::maybe_open_setup(&global_dir, port);

    // Host unix socket — the per-agent surface. Same app + gate as TCP, but
    // auth comes from the connection's kernel peer credentials (no token).
    // Best-effort and additive: TCP stays the primary surface regardless.
    #[cfg(unix)]
    let _host_socket = auth::peer_socket::spawn_host_socket(&daemon_dir, app.clone(), ct.clone());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await.map_err(|e| anyhow!("server error: {e}"))?;

    let agents_drained = if let Some(dh) = rt.dispatch_handle() {
        let mut registry = dh.registry.lock().await;
        let count = registry.len();
        ship_code::process_registry::graceful_shutdown(rt.kernel(), &mut registry, Arc::clone(&dh.executor)).await;
        count
    } else {
        0
    };

    // Reap all sandboxes so a clean restart doesn't orphan backend
    // containers. Each reap fires the SandboxOwner shutdown
    // (docker rm / youki delete). Best-effort — a reap failure here
    // shouldn't block shutdown.
    let sandboxes_reaped = ship_device::sandbox::dispatcher().reap_all().await;
    if sandboxes_reaped > 0 {
        tracing::info!(sandboxes_reaped, "reaped sandboxes on shutdown");
    }
    let uptime_secs = rt.uptime().as_secs();
    rt.shutdown().await;
    emit_daemon_event(
        &global_dir,
        daemon_events::DAEMON_SHUTDOWN_COMPLETED,
        "daemon",
        &ShutdownCompletedPayload {
            pid: std::process::id(),
            uptime_secs,
            agents_drained,
            trigger: "sigterm".to_string(),
        },
    );
    Ok(())
}

#[cfg(test)]
#[cfg(test)]
#[path = "routes/artifacts_api_tests.rs"]
mod artifacts_api_tests;

/// Append a daemon.* event directly to platform.db.
///
/// Used in paths where the kernel may not be live (port bind failure) or where
/// the runtime is already torn down (post-serve shutdown). Fire-and-forget:
/// telemetry must never block lifecycle decisions.
fn emit_daemon_event<P: serde::Serialize>(
    global_dir: &std::path::Path,
    event_type: &str,
    entity_id: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, payload) {
        Ok(e) => e.with_actor_id("daemon"),
        Err(e) => {
            tracing::warn!("shipd: cannot build {event_type} envelope: {e}");
            return;
        }
    };
    // daemon.* is Runtime-scoped → write runtime.db (not the dead platform.db).
    let store = match SqliteEventStore::new_for_scope(
        global_dir,
        ship_substrate::events::EventScope::Runtime,
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("shipd: cannot open event store for {event_type}: {e}");
            return;
        }
    };
    // Enforced write: daemon.* is Runtime-scoped and this path may run with no
    // live kernel/session, so stamp + enforce from the daemon-internal (Runtime)
    // binding before persist.
    let binding = ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk);
    if let Err(e) = store.append_enforced(&envelope, &binding) {
        tracing::warn!("shipd: cannot append {event_type}: {e}");
    }
}

/// Best-effort lookup of the pid currently holding `port`, sourced from the
/// supervisor's lock-based pidfile. Returns None if the file is missing,
/// unparseable, or records a different port.
fn pidfile_pid_for_port(global_dir: &std::path::Path, port: u16) -> Option<u32> {
    let path = ship_substrate::ShipHome::at(global_dir.to_path_buf())
        .machine_dir()
        .join("daemon")
        .join("shipd.pid");
    let data = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(data.trim()).ok()?;
    let recorded_port = v.get("port")?.as_u64()? as u16;
    if recorded_port != port {
        return None;
    }
    Some(v.get("pid")?.as_u64()? as u32)
}
