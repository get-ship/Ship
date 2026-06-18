//! `run_network` — the daemon composition root.
//!
//! Reads as a top-to-bottom L2 startup story: dev warning, resolve paths,
//! resolve config, apply env vars, build the runtime, start subsystems,
//! register the active space, start the agent stack + human gateway + diag,
//! install session subsystems, build the router, bind, emit the startup
//! event, serve, drain.
//!
//! The L0/L1 init steps (path discovery, env-var wiring, port derivation,
//! runtime subsystem startup, identity files, session install) live in
//! `ship_runtime::init`. The L0 init module splits the work across
//! `init::paths` (paths + config + env) and `init::startup` (subsystem
//! boot + identity + session), re-exported at the `init` root.
//!
//! Split (file-cap): listener binding + direct event emit live in
//! `super::net`, the agent-stack startup in `super::agent_stack`, the leak
//! diagnostics in `super::diag`, and the post-serve drain in
//! `super::shutdown`. This file owns the sequential composition story.

use anyhow::{Result, anyhow};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::daemon::{StartupCompletedPayload, event_types as daemon_events};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::{auth, router, ship_runtime};

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

    let init_started_at = std::time::Instant::now();
    let paths = ship_runtime::init::resolve_daemon_paths(&workspace_dir)?;
    let config =
        ship_runtime::init::resolve_daemon_config(&paths, host, port)?;
    ship_runtime::init::apply_path_env_vars(&paths)?;

    let meta = ship_runtime::AppMetadata {
        global_dir: paths.global_dir.clone(),
        workspace_dir: paths.workspace_dir.clone(),
    };
    let rt_config = ship_runtime::RuntimeConfig {
        host: config.host.clone(),
        port: config.desired_port,
    };
    // The MeshService was the in-shipd mesh.* op surface; it was killed
    // along with the REST routes and `ship mesh broadcast` CLI. Real
    // inter-agent messaging is now native via the kernel's directed-event
    // delivery (`EventEnvelope.target_actor_id`).
    let runtime = ship_runtime::ShipRuntime::new(meta, rt_config).await?;
    let rt = Arc::new(runtime);

    // L0 subsystem boot + project space registration.
    ship_runtime::init::start_runtime_subsystems(&rt, &paths.global_dir).await?;
    ship_runtime::init::register_active_space(&rt, &paths.global_dir, &paths.project_root).await?;

    // L1 agent + diagnostic subsystems (split out per file-cap).
    super::agent_stack::start_agent_stack(&rt, &paths.global_dir).await;
    super::human_gateway::spawn_human_gateway(rt.kernel(), rt.health_registry()).await;
    super::diag::spawn_leak_diag(&rt);

    // L0 identity files + auth + sessions.
    let token = auth::read_or_generate_token(&paths.daemon_dir)?;
    // Record the *worktree* this daemon serves, not the main-checkout
    // root we'd get from ProjectRoot::discover's redirect. `just status`
    // and the e2e visual harness consult this to verify they're talking
    // to the daemon scoped to their cwd.
    let identity_root = paths
        .worktree_root
        .as_deref()
        .or_else(|| paths.project_root.as_ref().map(|p| p.path()))
        .ok_or_else(|| {
            anyhow!("shipd: no worktree_root or project_root; refusing to record identity")
        })?;
    ship_runtime::init::write_identity_files(rt.daemon_dir(), identity_root)?;

    // Build the parent signal token once; the install helper hands a
    // child to the runtime and the L2 loop uses the parent for the
    // axum graceful-shutdown and the host unix socket.
    let ct = CancellationToken::new();
    let ct2 = ct.clone();
    tokio::spawn(async move {
        super::shutdown_signal().await;
        ct2.cancel();
    });
    ship_runtime::init::install_session_subsystems(&rt, ct.clone(), dev).await?;

    let session_registry = rt.session_registry().clone();
    let pairing_registry = auth::pairing::new_shared();
    let attestation_verifier = auth::attest::AttestationVerifier::new();
    let app = router::build_router(
        rt.clone(),
        token,
        session_registry,
        pairing_registry,
        attestation_verifier,
        daemon_mode,
    );

    let (listener, port) =
        super::net::bind_with_retry(&config.host, config.desired_port, config.user_pinned, &paths.global_dir).await?;
    super::write_port_file(rt.daemon_dir(), port)?;

    let addr = format!("{}:{}", config.host, port);
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
        .unwrap_or_else(|| format!("{} (global-only)", paths.global_dir.display()));
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

    // First-run UX: if `data.db` doesn't exist yet, open the default
    // browser at `/setup` once the listener is bound. Harmless no-op on
    // subsequent starts; suppressible via `SHIP_NO_BROWSER=1`.
    super::first_run::maybe_open_setup(&paths.global_dir, port);

    // Emit the daemon.startup_completed event so external observers
    // (and the history projection) can correlate startup with later
    // supervisor / first-event timing.
    emit_startup_completed(&rt, port, init_started_at).await;

    // Start the in-process bundler's file watcher. HMR SSE clients subscribe the
    // bundler's broadcast directly via deck's `serve_bundle_hmr` handler, so the
    // daemon no longer forwards events through a parallel channel. The watcher
    // task's JoinHandle is captured (#256) so shutdown can abort it instead of
    // leaving it detached to hold the runtime alive past SIGTERM.
    let bundler_watcher: tokio::task::JoinHandle<()> = {
        let bundler = rt.bundler().clone();
        tokio::spawn(async move {
            if let Err(e) = bundler.start_watching().await {
                tracing::warn!("bundler: could not start file watching: {e}");
            }
        })
    };

    // Host unix socket — the per-agent surface. Same app + gate as TCP, but
    // auth comes from the connection's kernel peer credentials (no token).
    // Best-effort and additive: TCP stays the primary surface regardless.
    #[cfg(unix)]
    let _host_socket = auth::peer_socket::spawn_host_socket(rt.daemon_dir(), app.clone(), ct.clone());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move { ct.cancelled_owned().await })
    .await
    .map_err(|e| anyhow!("server error: {e}"))?;

    // Serve has returned (graceful shutdown drained the listener). Tear down the
    // bundler's detached tasks (#256): flip its watch flag so `run_watch` exits,
    // then abort the HMR event-forwarder so neither holds the runtime alive past
    // SIGTERM. Both are idempotent and best-effort.
    if let Err(e) = rt.bundler().shutdown().await {
        tracing::warn!("bundler: shutdown error: {e}");
    }
    bundler_watcher.abort();

    // Bounded agent drain + sandbox reap + runtime shutdown + the
    // daemon.shutdown_completed event — see `super::shutdown`.
    super::shutdown::drain_and_shutdown(&rt, &paths.global_dir).await;
    Ok(())
}

/// Emit the `daemon.startup_completed` event with init-duration timing.
/// Best-effort: a failed persist-and-route is logged at WARN and does
/// not fail the daemon.
async fn emit_startup_completed(
    rt: &Arc<ship_runtime::ShipRuntime>,
    port: u16,
    init_started_at: std::time::Instant,
) {
    let global_dir = rt.ship_dir();
    let summary = ship_substrate::db::event_log_summary(global_dir);
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
}
