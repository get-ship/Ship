//! Runtime subsystem startup, identity file writes, and session install.
//!
//! These are the L0/L1 startup steps that live in `ship_runtime::init`:
//! - `start_runtime_subsystems` — boot every background task the runtime
//!   owns (artifact GC, reapers, hook engine, config watcher, agent-def
//!   index, pipeline reactive registry).
//! - `write_identity_files` — write the pid/workspace/sha files into
//!   `<daemon_dir>/` so external tooling (`just status`, `ship doctor`)
//!   can introspect the running daemon.
//! - `install_session_subsystems` — install the process-wide session
//!   registry, the daemon's reachable origin, hydrate persisted
//!   sessions, and spawn the sweeper. Set the shutdown cancellation
//!   token child on the runtime so SSE loops can `select!` against it.

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use ship_substrate::home::read_or_generate_device_id;
use ship_substrate::ShipHome;
use ship_types::set_local_device_id;

use super::ShipRuntime;

/// Boot every background task the runtime owns. Idempotent — each
/// `start_*` on the runtime checks its own `OnceLock` / `subscribe`
/// guard and short-circuits on a second call. The bundler watcher is
/// intentionally NOT started here: it lives in the L2 daemon's serve
/// loop so the watcher `JoinHandle` is available to abort at shutdown.
/// `global_dir` is accepted for symmetry with the other init helpers
/// (and for callers that want to assert the runtime's `ship_dir()`
/// matches their own resolved `global_dir`).
pub async fn start_runtime_subsystems(
    rt: &Arc<ShipRuntime>,
    _global_dir: &Path,
) -> Result<()> {
    // Start the artifact GC loop (runs now then every hour).
    rt.start_artifact_gc();
    // Start the window-reaper loop (closes the log lifecycle of crashed
    // shell windows that never deregistered; every 30s).
    rt.start_window_reaper();
    // Sandbox reaper: idle-past-timeout sandboxes get marked Stopped.
    // Per-sandbox backend teardown folds in once SandboxOwner moves
    // from per-actor to per-sandbox.
    rt.start_sandbox_reaper();
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
    Ok(())
}

/// The launch-time project-registration step is now performed INLINE at
/// `daemon/run.rs` (the optionB-era `register_active_space` helper that used
/// to live here was deleted: the registry is event-only and folds the
/// `project.registered` event into the kernel's `projects` SQL projection in
/// the same transaction, so no `DataStore` argument is needed — wiring one
/// back in would re-introduce a load-bearing `.expect` over runtime.db).

/// Write the daemon identity files (pid, workspace, build sha) into
/// `<daemon_dir>/`. The runtime's own `daemon_dir` accessor returns the
/// same path the caller resolved up front; we keep this helper argument
/// explicit so it remains usable from contexts that have only the path.
pub fn write_identity_files(daemon_dir: &Path, root: &Path) -> Result<()> {
    crate::ship_runtime::init::identity_files::write_pid_file(daemon_dir)?;
    // Record the *worktree* this daemon serves, not the main-checkout
    // root we'd get from ProjectRoot::discover's redirect. `just status`
    // and the e2e visual harness consult this to verify they're talking
    // to the daemon scoped to their cwd.
    crate::ship_runtime::init::identity_files::write_workspace_file(daemon_dir, root)?;
    crate::ship_runtime::init::identity_files::write_sha_file(daemon_dir, build_info::build_info!().git_sha)?;
    Ok(())
}

/// Install the session subsystems the rest of the daemon depends on:
/// - hand the runtime a child of the signal token (#256) so long-lived
///   SSE keepalive/data loops can `select!` against cancellation.
/// - install the process-wide session registry so the host-side spawn
///   path (supervisor) can mint per-process tokens without threading a
///   registry through the event→orchestrator→executor chain.
/// - install the daemon's reachable origin so the spawn path can inject
///   SHIP_DAEMON_URL into sandboxed actors (rewritten to a
///   container-reachable host for Tier 2+).
/// - hydrate any persisted paired sessions from a previous daemon
///   lifetime. Dev mode starts fresh — don't carry prior sessions
///   across dev restarts.
/// - spawn the background sweeper that evicts expired sessions every
///   5 minutes and emits `auth.session.expired` for each.
pub async fn install_session_subsystems(
    rt: &Arc<ShipRuntime>,
    signal_token: CancellationToken,
    dev: bool,
) -> Result<()> {
    // Hand the runtime a CHILD of the signal token (#256) so long-lived SSE
    // keepalive/data loops can `select!` against cancellation. When the signal
    // handler above cancels the parent, the child fires and those streams
    // close — letting axum's graceful-shutdown drain finish instead of
    // hanging on a never-closing SSE connection until SIGKILL.
    rt.set_shutdown_token(signal_token.child_token());

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
    // Install the per-machine device id so every scope binding minted by
    // `ScopeBinding::daemon_internal` carries the stable id instead of the
    // `"local-device"` placeholder. Read+possible-mint under
    // `~/.ship/machine/device-id` (mode 0600). Propagating IO errors so a
    // mint failure blocks boot — mirrors `read_or_generate_token`'s posture
    // (the device id is a per-machine secret, not per-daemon).
    let home = ShipHome::at(rt.ship_dir().to_path_buf());
    let device_id = read_or_generate_device_id(&home)
        .map_err(|e| anyhow::anyhow!("mint device id at {}: {e}", home.device_id_path().display()))?;
    set_local_device_id(device_id);
    // Hydrate any persisted paired sessions from a previous daemon lifetime.
    // Dev mode starts fresh — don't carry prior sessions across dev restarts.
    if !dev {
        crate::auth::hydrate_sessions(&session_registry).await;
    }
    // Background sweep — evict expired sessions from the in-memory
    // registry every 5 minutes and emit `auth.session.expired` for each.
    // The HTTP lookup path already filters expired rows; this is the
    // audit-event source.
    crate::auth::spawn_session_sweeper(session_registry);
    Ok(())
}
