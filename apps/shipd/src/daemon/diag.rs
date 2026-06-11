//! leak-193 diagnostics. When `SHIP_LEAK_DIAG=1`, log a periodic snapshot
//! of process RSS alongside every in-process container whose unbounded
//! growth could be the fanout RSS leak. Off by default — pure
//! observability, no behavior change.

use std::sync::Arc;

use crate::ship_runtime::ShipRuntime;

pub(super) fn spawn_leak_diag(rt: &Arc<ShipRuntime>) {
    if std::env::var("SHIP_LEAK_DIAG").as_deref() != Ok("1") {
        return;
    }
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
