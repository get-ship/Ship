// FILE_CAP_OVERRIDE: daemon/ was 9 small files; consolidation was the requested move; will split later if needed
//
// Daemon lifecycle helpers — folded from the previous agent_stack / diag /
// human_gateway / net / shutdown / first_run submodules. Each entry point
// keeps its original visibility (`pub` for ones called outside the daemon
// module, `pub(super)` for ones called only from `run_network`).
//
// Order: startup helpers (agent_stack, diag, human_gateway), then I/O
// (bind_with_retry, emit_daemon_event, maybe_open_setup), then shutdown
// (drain_and_shutdown).

use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::daemon::{
    CrashDetectedPayload, PortBindFailedPayload, ShutdownCompletedPayload,
    event_types as daemon_events,
};

use crate::ship_runtime::ShipRuntime;

// ── startup helpers ──────────────────────────────────────────────────────

/// Agent-stack startup: in-process Yoke + agent-sessions actor
/// (full orchestrator with `ship-code`, yoke-only without).
///
/// Native harness (`yoke`, alias `floki`) embeds Yoke in-process — no
/// Bridge/fleet required. Set `YOKE_RPC=1` to force optional Connect to fleet.
///
/// Mount split (2026-07-10): with ship-code off, there is no
/// `CodeService.Dispatch` route mounted — a coding-agent request surfaces
/// as a 404, not a refused call (R-A: absence is structural).
pub(super) async fn start_agent_stack(rt: &Arc<ShipRuntime>, global_dir: &std::path::Path) {
    let ops_router = crate::rpc::ops::router(rt.clone());
    let yoke_gateway = std::sync::Arc::new(crate::yoke::gateway::ShipdGateway::new(ops_router));
    let yoke_kernel = rt.kernel().clone();
    let yoke_rpc = std::env::var("YOKE_RPC").as_deref() == Ok("1");
    let yoke_run_fn: ship_agents::ports::YokeRunFn = {
        let gateway = yoke_gateway.clone();
        std::sync::Arc::new(move |payload| {
            let gateway = gateway.clone();
            let kernel = yoke_kernel.clone();
            Box::pin(async move {
                if yoke_rpc {
                    crate::yoke::yoke_rpc_client::dispatch_yoke_session_rpc(
                        payload, gateway, kernel,
                    )
                    .await;
                } else {
                    crate::yoke::embed::dispatch_yoke_session(payload, gateway, kernel).await;
                }
            })
        })
    };

    #[cfg(feature = "bridge")]
    {
        // Harness-on: hand the yoke closure to the full composition root
        // (RunManager → subscribe_job_events). That path builds the full
        // AgentPorts (with harness fields) AND starts the agent-sessions
        // actor with them — the actor is the SAME one that would otherwise
        // be started below, so we MUST NOT start it again here.
        let dispatch_handle = crate::dispatch::run_dispatch::subscribe_job_events(
            rt.kernel().clone(),
            global_dir.to_path_buf(),
            &rt.health_registry(),
            rt.session_registry().clone(),
            rt.daemon_origin(),
            rt.active_space_id(),
            Some(yoke_run_fn),
        )
        .await;
    }
    #[cfg(not(feature = "bridge"))]
    {
        // Flag-off: skip the orchestrator (harness ports are ship-code-typed),
        // but still start agent-sessions with yoke-only AgentPorts so native
        // sessions keep flowing. Non-yoke sessions no-op (harness-driver stack
        // cfg-omitted from `handle_agent_session_started`).
        let _ = global_dir;
        use ship_substrate::task_health::spawn_monitored;
        let ports = std::sync::Arc::new(ship_agents::AgentPorts::yoke_only(Some(yoke_run_fn)));
        let kernel_actor = rt.kernel().clone();
        let hr = rt.health_registry();
        spawn_monitored(hr, "agent-sessions", async move {
            ship_agents::start_agents(kernel_actor, ports).await;
        });
    }
}

/// leak-193 diagnostics. When `SHIP_LEAK_DIAG=1`, log a periodic snapshot
/// of process RSS alongside every in-process container whose unbounded
/// growth could be the fanout RSS leak. Off by default — pure
/// observability, no behavior change.
pub(super) fn spawn_leak_diag(rt: &Arc<ShipRuntime>) {
    if std::env::var("SHIP_LEAK_DIAG").as_deref() != Ok("1") {
        return;
    }
    let diag_kernel = rt.kernel().clone();
    let diag_sessions = rt.live_sessions().clone();
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
                "DIAG rss={:.1}MB live_sessions={} per_turn_sessions={} {kline}",
                rss_mb(),
                diag_sessions.len(),
                per_turn,
            );
        }
    });
}

/// Boot-time crash response, log-native: detect an unclean previous run from
/// the log's own shape (a `daemon.startup_completed` with no
/// `daemon.shutdown_completed` after it), not by scanning full history. Must
/// run before this boot's own `daemon.startup_completed` lands, or that event
/// would shadow the prior run's. Cheap — two lookups against the
/// `(entity_id, event_type)` index, no table scan.
///
/// This responds (log + correlate + emit), it does not backfill: a crashed
/// run's missing shutdown event stays missing.
pub(super) async fn detect_prior_crash(rt: &Arc<ShipRuntime>, global_dir: &std::path::Path) {
    let db_path = ship_substrate::db::runtime_db_path(global_dir);
    let starts = match ship_substrate::db::events::query_events_by_entity_and_type_at(
        &db_path,
        "daemon",
        daemon_events::DAEMON_STARTUP_COMPLETED,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("shipd: crash detection: cannot read startup events: {e}");
            return;
        }
    };
    let Some(last_start) = starts.last() else {
        return; // first-ever boot: nothing to correlate against.
    };
    let stops = match ship_substrate::db::events::query_events_by_entity_and_type_at(
        &db_path,
        "daemon",
        daemon_events::DAEMON_SHUTDOWN_COMPLETED,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("shipd: crash detection: cannot read shutdown events: {e}");
            return;
        }
    };
    let clean_shutdown_after = stops.last().is_some_and(|s| s.id > last_start.id);
    if clean_shutdown_after {
        return;
    }

    let crashes_dir = ship_substrate::ShipHome::from_env().state_dir().join("crashes");
    let mut reports: Vec<String> = std::fs::read_dir(&crashes_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.starts_with("crash-") && name.ends_with(".json"))
        .collect();
    reports.sort();
    reports.reverse(); // most recent first (filename carries a sortable timestamp).

    tracing::error!(
        prior_started_event_id = %last_start.id,
        crash_report_count = reports.len(),
        most_recent_crash_report = reports.first().map(String::as_str).unwrap_or("<none>"),
        "shipd: prior run ended without a clean shutdown event — crash detected"
    );

    if let Ok(envelope) = EventEnvelope::new(
        daemon_events::DAEMON_CRASH_DETECTED,
        "daemon",
        &CrashDetectedPayload {
            prior_started_event_id: last_start.id.clone(),
            crash_report_count: reports.len(),
            correlated_crash_reports: reports,
        },
    ) {
        let envelope = envelope.with_actor_id("daemon");
        if let Err(e) = rt.persist_and_route(&envelope).await {
            tracing::warn!("shipd: cannot emit daemon.crash_detected: {e}");
        }
    }
}

// ── I/O: listener binding + daemon.* event emit ──────────────────────────

/// Bind the listener with collision retry. On EADDRINUSE we increment
/// the port up to 9 times before giving up (unless the caller pinned a
/// port, in which case one attempt). The actually-bound port is what the
/// caller writes to `<daemon_dir>/port` — the derived port is just a hint.
pub(super) async fn bind_with_retry(
    host: &str,
    desired_port: u16,
    user_pinned: bool,
    rt: &Arc<ShipRuntime>,
) -> Result<(tokio::net::TcpListener, u16)> {
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
                    let already_bound_by_pid = pidfile_pid_for_port(desired_port);
                    emit_daemon_event(
                        rt,
                        daemon_events::DAEMON_PORT_BIND_FAILED,
                        "daemon",
                        &PortBindFailedPayload {
                            port: desired_port,
                            already_bound_by_pid,
                            error: e.to_string(),
                        },
                    )
                    .await;
                    return Err(anyhow!(
                        "bind {host}:{desired_port}..{}: {} (giving up after {max_retries} retries)",
                        bound_port - 1,
                        e,
                    ));
                }
            }
            Err(e) => {
                let already_bound_by_pid = pidfile_pid_for_port(bound_port);
                emit_daemon_event(
                    rt,
                    daemon_events::DAEMON_PORT_BIND_FAILED,
                    "daemon",
                    &PortBindFailedPayload {
                        port: bound_port,
                        already_bound_by_pid,
                        error: e.to_string(),
                    },
                )
                .await;
                return Err(anyhow!("bind {host}:{bound_port}: {e}"));
            }
        }
    };
    Ok((listener, bound_port))
}

/// Emit a daemon.* event through the live kernel router (daemon-internal
/// Runtime binding). Fire-and-forget: telemetry must never block lifecycle
/// decisions.
pub(super) async fn emit_daemon_event<P: serde::Serialize>(
    rt: &Arc<ShipRuntime>,
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
    if let Err(e) = rt.persist_and_route(&envelope).await {
        tracing::warn!("shipd: cannot emit {event_type}: {e}");
    }
}

/// Best-effort lookup of the pid currently holding `port`, sourced from the
/// supervisor's lock-based pidfile. Returns None if the file is missing,
/// unparseable, or records a different port.
fn pidfile_pid_for_port(port: u16) -> Option<u32> {
    let path = ship_substrate::ShipHome::from_env()
        .daemon_dir()
        .join("shipd.pid");
    let data = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(data.trim()).ok()?;
    let recorded_port = v.get("port")?.as_u64()? as u16;
    if recorded_port != port {
        return None;
    }
    Some(v.get("pid")?.as_u64()? as u32)
}

// ── I/O: first-run UX ────────────────────────────────────────────────────

/// Public entry point: if this is a first-run and browser open isn't
/// suppressed, spawn the platform's URL opener pointing at `/setup`.
///
/// Never errors — failing to launch a browser is a UX nit, not a reason to
/// block the daemon. Any failure is logged and discarded.
pub fn maybe_open_setup(ship_dir: &Path, port: u16) {
    if std::env::var_os("SHIP_NO_BROWSER").is_some() {
        tracing::debug!("first-run browser suppressed by SHIP_NO_BROWSER");
        return;
    }
    if !is_first_run(ship_dir) {
        return;
    }
    let url = format!("http://127.0.0.1:{port}/setup");
    tracing::info!(url = %url, "first-run: opening setup wizard in browser");
    spawn_open(&url);
}

/// True when the daemon has not initialized this ship-home before. A fresh
/// install has no `runtime.db` (the daemon materializes it on its first event);
/// this is the first-run sentinel after `data.db` was removed (it carried the
/// same "has the daemon started before" lifecycle).
fn is_first_run(ship_dir: &Path) -> bool {
    !ship_substrate::db::runtime_db_path(ship_dir).exists()
}

#[cfg(target_os = "linux")]
fn spawn_open(url: &str) {
    spawn_detached("xdg-open", &[url]);
}

#[cfg(target_os = "macos")]
fn spawn_open(url: &str) {
    spawn_detached("open", &[url]);
}

#[cfg(target_os = "windows")]
fn spawn_open(url: &str) {
    // `start` is a cmd builtin, not an executable — invoke via cmd /C.
    // The empty "" is cmd's window-title slot so a URL with spaces doesn't
    // get misinterpreted as one.
    spawn_detached("cmd", &["/C", "start", "", url]);
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn spawn_open(_url: &str) {
    tracing::debug!("first-run: browser open not supported on this platform");
}

fn spawn_detached(program: &str, args: &[&str]) {
    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    match cmd.spawn() {
        Ok(mut child) => {
            // Detach — we don't want a zombie if the opener exits after us.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => {
            tracing::warn!(program, error = %e, "first-run: failed to spawn browser opener");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn is_first_run_true_when_runtime_db_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(is_first_run(tmp.path()));
    }

    #[test]
    fn is_first_run_false_when_runtime_db_exists() {
        let tmp = TempDir::new().unwrap();
        let path = ship_substrate::db::runtime_db_path(tmp.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"").unwrap();
        assert!(!is_first_run(tmp.path()));
    }

    #[test]
    fn maybe_open_setup_is_noop_when_suppressed() {
        let tmp = TempDir::new().unwrap();
        // SAFETY: scoped to this test; SHIP_NO_BROWSER is a boolean flag, no
        // other thread in this crate depends on its exact value.
        unsafe { std::env::set_var("SHIP_NO_BROWSER", "1") };
        // Must not panic or block.
        maybe_open_setup(tmp.path(), 9315);
        unsafe { std::env::remove_var("SHIP_NO_BROWSER") };
    }
}

// ── shutdown ─────────────────────────────────────────────────────────────

/// Drain agents (bounded), reap sandboxes, emit `daemon.shutdown_completed`
/// while the kernel is still live, then shut the runtime down. Runs after
/// axum's graceful shutdown returned.
pub(super) async fn drain_and_shutdown(rt: &Arc<ShipRuntime>) {
    // Bound the agent drain so a stuck child can't hang shutdown forever (#256).
    // axum already drained connections; this cap protects the registry drain
    // (the SIGTERM→SIGKILL hang was an unbounded wait here + un-cancelled SSE).
    #[cfg(feature = "bridge")]
    let agents_drained = if let Some(dh) = rt.dispatch_handle() {
        const SHUTDOWN_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);
        let mut registry = dh.registry.lock().await;
        let count = registry.len();
        let sink: Arc<dyn bridge::lifecycle::LifecycleSink> =
            Arc::new(ship_agents::KernelLifecycleSink::new(rt.kernel().clone()));
        let drain = bridge::process_registry::graceful_shutdown(
            &sink,
            &mut registry,
            Arc::clone(&dh.executor),
        );
        if tokio::time::timeout(SHUTDOWN_DRAIN_TIMEOUT, drain).await.is_err() {
            tracing::warn!(
                timeout_secs = SHUTDOWN_DRAIN_TIMEOUT.as_secs(),
                "shipd: agent drain timed out; proceeding with shutdown"
            );
        }
        count
    } else {
        0
    };
    #[cfg(not(feature = "bridge"))]
    let agents_drained: usize = 0;

    // Reap all sandboxes so a clean restart doesn't orphan backend
    // containers. Each reap fires the SandboxOwner shutdown
    // (docker rm / youki delete). Best-effort — a reap failure here
    // shouldn't block shutdown.
    let sandboxes_reaped = ship_device::sandbox::dispatcher().reap_all().await;
    if sandboxes_reaped > 0 {
        tracing::info!(sandboxes_reaped, "reaped sandboxes on shutdown");
    }
    let uptime_secs = rt.uptime().as_secs();
    // Emit while the kernel is still alive — `rt.shutdown()` flushes the WAL
    // and removes liveness files, so the durable shutdown event must land
    // before that, not after.
    emit_daemon_event(
        rt,
        daemon_events::DAEMON_SHUTDOWN_COMPLETED,
        "daemon",
        &ShutdownCompletedPayload {
            pid: std::process::id(),
            uptime_secs,
            agents_drained,
            trigger: "sigterm".to_string(),
        },
    )
    .await;
    rt.shutdown().await;
}
