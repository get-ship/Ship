//! SessionScanner — background task that detects abandoned sessions.
//!
//! Runs every 60 seconds. For each active session, checks last_activity_at.
//! If idle beyond TTL (10 min), ends the session with exit_reason "abandoned".

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// TTL in seconds — sessions idle beyond this are considered abandoned.
const ABANDONED_TTL_SECS: i64 = 600;

/// Scan interval in seconds.
const SCAN_INTERVAL_SECS: u64 = 60;

/// Manages the session scanner background task.
pub struct SessionScanner {
    _handle: JoinHandle<()>,
}

impl SessionScanner {
    /// Start the scanner. Spawns a background tokio task.
    pub fn start(kernel: Arc<Mutex<runtime::events::KernelRouter>>) -> Self {
        let handle = tokio::spawn(async move {
            let _kernel = kernel; // held for lifetime
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(SCAN_INTERVAL_SECS)).await;
                scan_abandoned_sessions();
            }
        });
        Self { _handle: handle }
    }
}

/// Scan for abandoned sessions and end them.
fn scan_abandoned_sessions() {
    let sessions = match runtime::db::session_drain::list_active_sessions() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session scanner: failed to list active sessions: {e}");
            return;
        }
    };

    let cutoff = (chrono::Utc::now()
        - chrono::Duration::seconds(ABANDONED_TTL_SECS))
    .to_rfc3339();

    for session in &sessions {
        let is_abandoned = match &session.last_activity_at {
            Some(ts) => ts < &cutoff,
            // No heartbeat yet — check started_at as fallback.
            None => session.started_at < cutoff,
        };

        if is_abandoned {
            match runtime::db::session_drain::force_end_session(&session.id, "abandoned") {
                Ok(()) => {
                    tracing::info!(
                        session_id = %session.id,
                        workspace = %session.workspace_branch,
                        "session scanner: ended abandoned session"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        session_id = %session.id,
                        "session scanner: failed to end abandoned session: {e}"
                    );
                }
            }
        }
    }
}
