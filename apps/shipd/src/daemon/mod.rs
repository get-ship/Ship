//! Daemon lifecycle glue: shutdown signal, submodule re-exports.
//!
//! The actual serve loop lives in `daemon::run`; agent-stack startup,
//! leak diagnostics, human-gateway bootstrap, listener binding/emit,
//! first-run UX, and post-serve drain live in `daemon::lifecycle`.
//! Identity file writers and stale-file cleanup moved to
//! `ship_runtime::init::identity_files` (L0 init); the user-facing
//! `network status` / `network stop` CLI handlers moved to
//! `apps/shipctl/src/daemon/commands.rs`.

/// Wait for SIGINT or SIGTERM (UNIX) or just SIGINT (other platforms).
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut sigterm) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
            }
            Err(e) => {
                // SIGTERM handler unavailable — fall back to SIGINT only
                tracing::warn!("could not register SIGTERM handler: {e}; SIGINT only");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests;

// Daemon lifecycle helpers — agent-stack startup, leak diagnostics,
// human-gateway bootstrap, listener binding/emit, first-run UX, and
// post-serve drain. Folded from 6 small submodules for layout coherence.
pub mod lifecycle;
mod run;

pub use run::run_network;
