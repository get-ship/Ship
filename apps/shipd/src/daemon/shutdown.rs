//! Post-serve teardown: bounded agent drain, sandbox reap, runtime shutdown,
//! and the daemon.shutdown_completed event.

use std::sync::Arc;

use ship_substrate::events::daemon::{ShutdownCompletedPayload, event_types as daemon_events};

use crate::ship_runtime::ShipRuntime;

/// Drain agents (bounded), reap sandboxes, shut the runtime down, and emit
/// `daemon.shutdown_completed`. Runs after axum's graceful shutdown returned.
pub(super) async fn drain_and_shutdown(rt: &Arc<ShipRuntime>, global_dir: &std::path::Path) {
    // Bound the agent drain so a stuck child can't hang shutdown forever (#256).
    // axum already drained connections; this cap protects the registry drain
    // (the SIGTERM→SIGKILL hang was an unbounded wait here + un-cancelled SSE).
    #[cfg(feature = "ship-code")]
    let agents_drained = if let Some(dh) = rt.dispatch_handle() {
        const SHUTDOWN_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);
        let mut registry = dh.registry.lock().await;
        let count = registry.len();
        let drain = ship_code::process_registry::graceful_shutdown(rt.kernel(), &mut registry, Arc::clone(&dh.executor));
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
    #[cfg(not(feature = "ship-code"))]
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
    rt.shutdown().await;
    super::net::emit_daemon_event(
        global_dir,
        daemon_events::DAEMON_SHUTDOWN_COMPLETED,
        "daemon",
        &ShutdownCompletedPayload {
            pid: std::process::id(),
            uptime_secs,
            agents_drained,
            trigger: "sigterm".to_string(),
        },
    );
}
