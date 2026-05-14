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
}

pub mod access;
pub mod sessions;
pub mod auth;
pub mod wire;
pub mod transport;
mod daemon;
pub mod services;
pub mod managers;
pub mod project_api;
pub mod routes;
pub mod router;
// (former shipd/integrations/ — apps_bridge → views/mcp_apps_embed; registry → routes/mcp_registry)
pub mod views;
pub mod ship_runtime;
// Production supervisor code lives in the `ship-supervisor` crate. The
// existing `crate::supervisor::*` import path across shipd resolves
// through this re-export so call sites don't churn.
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
pub use transport::NetworkServer;
pub use ship_sui::DaemonMode;

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
    daemon_mode: ship_sui::DaemonMode,
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
    let global_dir = ship_application::project::get_global_dir()?;
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
        // (ship-sui-server's resolver consults `SHIP_DEV_PROJECT_DIR`
        // first; without this it falls back to CWD + worktree-aware
        // resolution, which redirects into the parent checkout's
        // `views/` when the workspace lives inside a git worktree).
        // The explicit env var lets test fixtures and per-project
        // daemons stay scoped to their own `views/` directory.
        //
        // Leave any pre-existing value alone — callers that need a
        // different SUI resolver root than the daemon workspace
        // (rare; mostly studio in-place editing) can still override.
        // SAFETY: shipd's process is single-threaded at this point in
        // startup — set_var is unsound only when other threads can
        // observe the env mid-update.
        if std::env::var_os("SHIP_DEV_PROJECT_DIR").is_none() {
            // SAFETY: pre-async-runtime, no concurrent env access.
            unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", w); }
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
    let mut runtime = ship_runtime::ShipRuntime::new(meta, config).await?;
    let mesh_registry = transport::connections::spawn_mesh_service(runtime.kernel(), runtime.health_registry()).await?;
    runtime.set_mesh_registry(mesh_registry);
    let rt = Arc::new(runtime);

    let job_mgr = managers::JobManager::init(rt.kernel().clone(), global_dir.clone(), rt.health_registry()).await;
    if let Some(dh) = job_mgr.dispatch_handle {
        rt.set_dispatch_handle(dh);
    }
    // Start the abandoned session scanner.
    rt.session_scanner().await;
    // Start the artifact GC loop (runs now then every hour).
    rt.start_artifact_gc();
    daemon::human_gateway::spawn_human_gateway(rt.kernel(), rt.health_registry()).await;
    rt.start_hook_engine().await;
    rt.start_config_watcher();

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
                    Ok(ship_application::bundler::BundlerEvent::ModuleChanged { target, out_path, bytes }) => {
                        if mode == ship_sui::DaemonMode::Development {
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
                    Ok(ship_application::bundler::BundlerEvent::CompileError { target, message, line, col, file }) => {
                        if mode == ship_sui::DaemonMode::Development {
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
                    Ok(ship_application::bundler::BundlerEvent::WorkerRestarted) => {
                        tracing::info!("bundler: worker restarted");
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Register the launch-time project (if any) into the project registry
    // so the launcher / shell sees it on first list. Idempotent.
    if let Some(root) = project_root.as_ref()
        && let Err(e) = ship_platform::projects::register(root.path(), None) {
            tracing::warn!(path = %root.path().display(), "failed to register project on startup: {e}");
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
    // Hydrate any persisted paired sessions from a previous daemon lifetime.
    // Dev mode starts fresh — don't carry prior sessions across dev restarts.
    if !dev {
        auth::hydrate_sessions(&session_registry).await;
    }
    let pairing_registry = auth::pairing::new_shared();
    let attestation_verifier = auth::attest::AttestationVerifier::new();
    // Log the `.ship` components discovered at startup. The compile cache
    // is lazy (first-request-per-tag); this scan is purely diagnostic so
    // operators can see at a glance whether the components dir is wired.
    let comps = views::components::discover_components();
    tracing::info!(count = comps.len(), "shipd: .ship components discovered");
    for (tag, path) in &comps {
        tracing::info!(tag = %tag, path = %path.display(), "  component registered");
    }
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

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await.map_err(|e| anyhow!("server error: {e}"))?;

    let agents_drained = if let Some(dh) = rt.dispatch_handle() {
        let mut registry = dh.registry.lock().await;
        let count = registry.len();
        supervisor::graceful_shutdown(rt.kernel(), &mut registry, Arc::clone(&dh.executor)).await;
        count
    } else {
        0
    };
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
#[path = "routes/collection_tests.rs"]
mod collection_tests;
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
    let store = match SqliteEventStore::new_at(global_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("shipd: cannot open event store for {event_type}: {e}");
            return;
        }
    };
    if let Err(e) = store.append(&envelope) {
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
