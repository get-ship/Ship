//! SessionScanner — detects abandoned sessions and ends them.
//!
//! Tracks per-session `last_activity_at` in memory by subscribing to
//! `session.*` events through the kernel. A periodic 60s tick walks
//! the in-memory map and ends any session whose last activity is older
//! than `ABANDONED_TTL_SECS`. Idle detection inherently needs a timer
//! (the *absence* of activity is what we're watching for) but the
//! per-tick cost is now an in-memory scan instead of a DB query —
//! event-coverage/04 fix.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::task_health::HealthRegistry;

/// TTL in seconds — sessions idle beyond this are considered abandoned.
const ABANDONED_TTL_SECS: i64 = 600;

/// Interval at which the in-memory abandonment check runs.
const SCAN_INTERVAL_SECS: u64 = 60;

/// Manages the session scanner background task.
pub struct SessionScanner {
    _handle: JoinHandle<()>,
}

impl SessionScanner {
    /// Start the scanner. Subscribes to `session.*` events to maintain
    /// per-session activity timestamps in memory; spawns the periodic
    /// scan task supervised by the health registry.
    pub fn start(
        kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
        health_registry: &HealthRegistry,
    ) -> Self {
        let activity: Arc<Mutex<HashMap<String, DateTime<Utc>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn the activity pump. We hold `kernel` only long enough
        // to register the actor; the mailbox loop owns its receiver.
        let activity_pump = activity.clone();
        let kernel_for_pump = kernel.clone();
        tokio::spawn(async move {
            let mailbox = {
                let mut kr = kernel_for_pump.lock().await;
                let config = ship_types::ActorConfig {
                    namespace: "service.session_scanner".to_string(),
                    write_namespaces: vec![],
                    read_namespaces: vec![],
                    subscribe_namespaces: vec!["session.".to_string()],
                };
                match kr.spawn_actor("service.session_scanner", config) {
                    Ok((_store, mb)) => mb,
                    Err(e) => {
                        tracing::warn!(
                            "session-scanner: kernel spawn_actor failed ({e}); \
                             abandonment detection falls back to DB poll"
                        );
                        return;
                    }
                }
            };
            run_activity_pump(activity_pump, mailbox).await;
        });

        let activity_for_scan = activity.clone();
        let handle = crate::task_health::spawn_supervised(
            health_registry,
            "session-scanner",
            move || {
                let activity = activity_for_scan.clone();
                async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(SCAN_INTERVAL_SECS))
                            .await;
                        scan_in_memory(&activity).await;
                    }
                }
            },
        );
        Self { _handle: handle }
    }
}

/// Drain mailbox events into the activity map. Returns when all
/// senders are dropped (kernel shutdown).
async fn run_activity_pump(
    activity: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    mut mailbox: ship_substrate::events::Mailbox,
) {
    while let Some(envelope) = mailbox.recv().await {
        update_activity_from_event(&activity, &envelope).await;
    }
}

async fn update_activity_from_event(
    activity: &Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    envelope: &ship_substrate::events::EventEnvelope,
) {
    use ship_types::event_types::SESSION_ENDED;

    let session_id = envelope.entity_id.clone();
    let mut map = activity.lock().await;
    if envelope.event_type == SESSION_ENDED {
        map.remove(&session_id);
    } else {
        // Any other session.* event counts as activity — the agent is
        // doing *something*. Specific events that mean "the agent is
        // alive" (heartbeat, progress, started, attached, status_changed)
        // are exactly the cases we care about; treating the whole
        // `session.` namespace uniformly avoids missing future additions.
        map.insert(session_id, Utc::now());
    }
}

/// In-memory abandonment scan: ends any session whose last-known
/// activity is older than ABANDONED_TTL_SECS. Cheap; no DB query.
async fn scan_in_memory(activity: &Arc<Mutex<HashMap<String, DateTime<Utc>>>>) {
    let cutoff = Utc::now() - chrono::Duration::seconds(ABANDONED_TTL_SECS);
    let abandoned: Vec<String> = {
        let map = activity.lock().await;
        map.iter()
            .filter(|(_, ts)| **ts < cutoff)
            .map(|(id, _)| id.clone())
            .collect()
    };
    if abandoned.is_empty() {
        return;
    }

    for session_id in abandoned {
        match ship_substrate::db::session_drain::force_end_session(&session_id, "abandoned") {
            Ok(()) => {
                tracing::info!(%session_id, "session scanner: ended abandoned session");
                let mut map = activity.lock().await;
                map.remove(&session_id);
            }
            Err(e) => {
                tracing::warn!(
                    %session_id,
                    "session scanner: failed to end abandoned session: {e}"
                );
            }
        }
    }
}
