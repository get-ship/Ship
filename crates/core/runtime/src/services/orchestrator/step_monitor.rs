//! Step monitor — unified per-step execution tracker.
//!
//! Each dispatched step gets a `StepMonitor` that waits for:
//! 1. `step.done_declared` event (agent called `step_done`)
//! 2. Process exit signal (executor sends `ExitSignal`)
//! 3. Timeout (configurable, default 30 minutes)
//!
//! The monitor emits `step.observed` with the authoritative success/failure
//! verdict for pipeline advancement.

use crate::events::job::{StepObservedPayload, event_types};
use crate::events::{CallerKind, EmitContext, EventEnvelope, EventStore, KernelRouter, SqliteEventStore};
use crate::services::dispatch_ports::{ExitSignal, ProcessRegistryPort};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

/// Default step timeout: 30 minutes.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Per-step execution monitor.
///
/// Spawned as a tokio task when a pipeline step begins executing.
/// Tracks whether `step_done` was called and correlates it with process exit.
pub struct StepMonitor {
    pub job_id: String,
    pub step_index: usize,
    pub slug: String,
    pub step_name: String,
    pub started_at: Instant,
    pub timeout: Duration,
    pub done_declared: Arc<AtomicBool>,
}

impl StepMonitor {
    pub fn new(
        job_id: String,
        step_index: usize,
        slug: String,
        step_name: String,
    ) -> Self {
        Self {
            job_id,
            step_index,
            slug,
            step_name,
            started_at: Instant::now(),
            timeout: DEFAULT_TIMEOUT,
            done_declared: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns a clone of the `done_declared` flag for external signaling.
    pub fn done_flag(&self) -> Arc<AtomicBool> {
        self.done_declared.clone()
    }
}

/// Spawn the step monitor task.
///
/// Waits for the process exit signal, then emits `step.observed` based on
/// whether `step_done` was called before exit.
pub fn start_step_monitor(
    monitor: StepMonitor,
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<dyn ProcessRegistryPort>,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
) {
    tokio::spawn(async move {
        let result = tokio::time::timeout(monitor.timeout, exit_rx.recv()).await;

        let duration_secs = monitor.started_at.elapsed().as_secs();
        let done_declared = monitor.done_declared.load(Ordering::Acquire);

        // Unregister from process registry
        registry.unregister(&monitor.slug).await;

        let (succeeded, exit_code, reason) = match result {
            Ok(Some(signal)) => {
                let code = signal.exit_code;
                match code {
                    Some(0) | None => {
                        if done_declared {
                            (true, code, None)
                        } else {
                            (false, code, Some("no step_done".to_string()))
                        }
                    }
                    Some(_) => {
                        (false, code, Some(format!(
                            "process exited with code {}",
                            code.unwrap_or(-1)
                        )))
                    }
                }
            }
            Ok(None) => {
                (false, None, Some("exit channel closed unexpectedly".to_string()))
            }
            Err(_) => {
                (false, None, Some("timeout".to_string()))
            }
        };

        let payload = StepObservedPayload {
            job_id: monitor.job_id.clone(),
            slug: monitor.slug.clone(),
            step_index: monitor.step_index,
            step_name: monitor.step_name.clone(),
            succeeded,
            exit_code,
            duration_secs,
            done_declared,
            reason,
        };

        emit_step_observed(&kernel, &monitor.job_id, &monitor.slug, &payload).await;
    });
}

/// Emit a `step.observed` event through the kernel.
async fn emit_step_observed(
    kernel: &Arc<Mutex<KernelRouter>>,
    job_id: &str,
    slug: &str,
    payload: &StepObservedPayload,
) {
    let envelope = match EventEnvelope::new(event_types::STEP_OBSERVED, job_id, payload) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "step-monitor: failed to build envelope: {e}");
            return;
        }
    };

    if let Ok(store) = SqliteEventStore::new()
        && let Err(e) = EventStore::append(&store, &envelope)
    {
        tracing::warn!(slug, "step-monitor: failed to persist event: {e}");
    }

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!(slug, "step-monitor: failed to route event: {e}");
    }
}
