//! `run_network` — the daemon composition root: resolve dirs/ports, build the
//! runtime, start background tasks, bind, serve, and tear down.
//!
//! Split (file-cap): listener binding + direct event emit live in
//! `super::net`, the agent-stack startup in `super::agent_stack`, the leak
//! diagnostics in `super::diag`, and the post-serve drain in
//! `super::shutdown`. This file owns the sequential startup story.

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
    apps_config_override: Option<std::path::PathBuf>,
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

    super::cleanup_stale_daemon_files(&daemon_dir).map_err(|e| {
        eprintln!("\nshipd startup aborted: {e}\n");
        e
    })?;
    if let Some(ref w) = workspace_dir {
        tracing::info!(workspace = %w.display(), daemon_dir = %daemon_dir.display(), "shipd: project-scoped mode");
        // Make `--workspace` authoritative for the SUI resolver chain.
        // deck-core's resolver consults an in-process global set by
        // `deck::resolver::set_project_root` (the SAFE path) — see the
        // call below. The `Var::DevProjectDir` env var is kept for
        // non-deck readers (apps/shipd/src/views/discovery.rs, ship-agents
        // view_state.rs) that still key off it; deck-core no longer
        // reads it.
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
        // Authoritative project-root override for deck-core's resolver. Same
        // value the router reads at DeckConfig::load time (router.rs's
        // `rt.workspace_dir()`/cwd fallback) so the resolver and the
        // bundler agree on where `<root>/.deck` lives. Non-deck readers
        // (apps/shipd/src/views/discovery.rs, ship-agents view_state.rs)
        // still rely on the env var above.
        let repo_root = Some(w.clone()).or_else(|| std::env::current_dir().ok());
        if let Some(r) = repo_root {
            deck::resolver::set_project_root(r);
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

    // Floki dispatch closure + agent-sessions actor (full orchestrator with
    // ship-code, floki-only without) — see `super::agent_stack`.
    super::agent_stack::start_agent_stack(&rt, &global_dir).await;
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
    super::human_gateway::spawn_human_gateway(rt.kernel(), rt.health_registry()).await;
    rt.start_hook_engine().await;
    rt.start_config_watcher();
    // Live agent-def index (agents arc S5): build the read-only index from the
    // def files at boot, then keep it live on `agent.def.updated`. Shares the
    // gated-write NOTIFY's `KernelLog` so op-writes re-index deterministically.
    rt.start_agent_def_index().await;
    // Typed pipeline `Watch` (pipelines P0.4): stand up the reactive registry +
    // the bridge that taps the orchestrator's pipeline.* / session.* router
    // events into the Watch log so run/step update streams wake.
    rt.start_pipeline_reactive().await;
    // leak-193 diagnostics (SHIP_LEAK_DIAG=1) — see `super::diag`.
    super::diag::spawn_leak_diag(&rt);

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
    super::write_pid_file(&daemon_dir)?;
    // Record the *worktree* this daemon serves, not the main-checkout
    // root we'd get from ProjectRoot::discover's redirect. `just status`
    // and the e2e visual harness consult this to verify they're talking
    // to the daemon scoped to their cwd.
    if let Some(root) = worktree_root.as_ref() {
        super::write_workspace_file(&daemon_dir, root)?;
    } else if let Some(root) = project_root.as_ref() {
        super::write_workspace_file(&daemon_dir, root.path())?;
    }
    super::write_sha_file(&daemon_dir, build_info::build_info!().git_sha)?;

    let ct = CancellationToken::new();
    let ct2 = ct.clone();
    tokio::spawn(async move { super::shutdown_signal().await; ct2.cancel(); });
    // Hand the runtime a CHILD of the signal token (#256) so long-lived SSE
    // keepalive/data loops can `select!` against cancellation. When the signal
    // handler above cancels `ct`, the child fires and those streams close —
    // letting axum's graceful-shutdown drain finish instead of hanging on a
    // never-closing SSE connection until SIGKILL.
    rt.set_shutdown_token(ct.child_token());

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
    // Read the apps list from `apps/shipd/apps.jsonc`. The file is the
    // Ship-monorepo source of truth for "which apps ship, on what
    // routes, with what globals" — deck (L1) is per-app, the host
    // owns the list, so it lives with shipd (its only reader), not at the
    // repo root. Default to `<worktree>/apps/shipd/apps.jsonc`; the
    let apps_config_path = apps_config_override
        .or_else(|| worktree_root.as_ref().map(|r| r.join("apps/shipd/apps.jsonc")))
        .unwrap_or_else(|| std::path::PathBuf::from("apps/shipd/apps.jsonc"));
    let apps = match router::apps_config::load_apps_config(&apps_config_path) {
        Ok(apps) => {
            tracing::info!(
                path = %apps_config_path.display(),
                count = apps.len(),
                "shipd: loaded apps.jsonc"
            );
            apps
        }
        Err(e) => {
            tracing::warn!(
                path = %apps_config_path.display(),
                error = %e,
                "shipd: apps.jsonc failed to load — serving no apps"
            );
            Vec::new()
        }
    };
    let app = router::build_router(
        rt.clone(),
        token,
        session_registry,
        pairing_registry,
        attestation_verifier,
        daemon_mode,
        apps,
    );
    let user_pinned = port.is_some();
    let (listener, port) =
        super::net::bind_with_retry(&host, desired_port, user_pinned, &global_dir).await?;
    super::write_port_file(&daemon_dir, port)?;

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
    super::first_run::maybe_open_setup(&global_dir, port);

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
    super::shutdown::drain_and_shutdown(&rt, &global_dir).await;
    Ok(())
}
