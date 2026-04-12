//! Liveness monitoring — exit signal handling and periodic liveness sweep.
//!
//! Two independent mechanisms detect agent death without the SessionEnd hook:
//!
//! 1. **Exit signal handler** (`start_exit_handler`): each dispatched job gets
//!    an `UnboundedReceiver<ExitSignal>`. The executor's monitor thread sends on
//!    the paired sender when the process/session exits. The handler emits
//!    `job.completed` (exit 0 / unknown) or `job.failed` (non-zero exit).
//!
//! 2. **Liveness sweep** (`start_liveness_sweep`): a background task that
//!    runs every 30 seconds.  It calls `drain_dead` on the registry to catch
//!    any handle that never sent a signal, and scans the job projection for
//!    Dispatched jobs with no activity in >5 minutes.

use chrono::Utc;
use runtime::events::job::event_types;
use runtime::events::{EventEnvelope, KernelRouter};
use runtime::services::dispatch_ports::{ExitSignal, JobExecutor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use super::registry::ProcessRegistry;

// ── Exit signal handler ───────────────────────────────────────────────────────

/// Spawn a task that waits for an exit signal from a process monitor thread.
///
/// For non-pipeline jobs (single agent, no steps), this emits `job.completed`
/// or `job.failed` directly. Multi-step pipeline jobs route through the
/// step monitor instead — see `step_monitor::start_step_monitor`.
pub(crate) fn start_exit_handler(
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<Mutex<ProcessRegistry>>,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
) {
    tokio::spawn(async move {
        if let Some(signal) = exit_rx.recv().await {
            tracing::info!(
                slug = signal.slug,
                exit_code = ?signal.exit_code,
                "liveness: exit signal received"
            );
            registry.lock().await.unregister(&signal.slug);

            match signal.exit_code {
                Some(0) | None => {
                    // Single-step or no-pipeline job — original behavior.
                    let payload = serde_json::json!({
                        "job_id": signal.job_id, "slug": signal.slug,
                    });
                    emit_job_event(
                        &kernel, event_types::JOB_COMPLETED,
                        &signal.job_id, &signal.slug, payload,
                    ).await;
                }
                Some(code) => {
                    let payload = serde_json::json!({
                        "job_id": signal.job_id,
                        "error": format!("process exited with code {code}"),
                    });
                    emit_job_event(
                        &kernel, event_types::JOB_FAILED,
                        &signal.job_id, &signal.slug, payload,
                    ).await;
                }
            }
        }
    });
}

// ── Periodic liveness sweep ───────────────────────────────────────────────────

/// Start the background liveness sweep task.
///
/// Runs every 30 seconds:
/// - Checks registered handles via `executor.is_alive()`; marks failed if dead.
/// - Scans the job projection for Dispatched jobs idle for >5 min with no
///   live handle in the registry.
pub(crate) fn start_liveness_sweep(
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<Mutex<ProcessRegistry>>,
    executor: Arc<dyn JobExecutor>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            sweep_registered(&kernel, &registry, &*executor).await;
            sweep_stale_dispatched(&kernel, &registry).await;
        }
    });
}

/// Check registered handles; emit `job.failed` for any that are no longer alive.
async fn sweep_registered(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<Mutex<ProcessRegistry>>,
    executor: &dyn JobExecutor,
) {
    let dead = registry.lock().await.drain_dead(executor).await;
    if dead.is_empty() {
        return;
    }
    tracing::warn!(count = dead.len(), "liveness: sweep found dead handles");

    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("liveness: failed to load jobs for dead slug lookup: {e}");
            return;
        }
    };

    for slug in dead {
        if let Some(job) = jobs.values().find(|j| j.slug == slug) {
            let payload = serde_json::json!({
                "job_id": job.job_id,
                "error": "process died without completing",
            });
            emit_job_event(kernel, event_types::JOB_FAILED, &job.job_id, &slug, payload).await;
        } else {
            tracing::warn!(slug, "liveness: dead slug not found in job projection");
        }
    }
}

/// Scan the job projection for Dispatched jobs with no activity for >5 min
/// and no live handle in the registry; mark them failed.
async fn sweep_stale_dispatched(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<Mutex<ProcessRegistry>>,
) {
    use runtime::projections::job::JobStatus;

    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("liveness: stale sweep failed to load jobs: {e}");
            return;
        }
    };

    let now = Utc::now();
    let stale_threshold = chrono::Duration::minutes(5);

    let live = registry.lock().await.live_slugs();

    let stale: Vec<_> = jobs
        .values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .filter(|j| (now - j.updated_at) > stale_threshold)
        .filter(|j| !live.contains(&j.slug))
        .collect();

    if stale.is_empty() {
        return;
    }
    tracing::warn!(count = stale.len(), "liveness: stuck Dispatched jobs with no live handle");

    for job in stale {
        let payload = serde_json::json!({
            "job_id": job.job_id,
            "error": "no activity for >5min and no live process handle",
        });
        emit_job_event(
            kernel,
            event_types::JOB_FAILED,
            &job.job_id,
            &job.slug,
            payload,
        )
        .await;
    }
}

// ── Shared emit helper ────────────────────────────────────────────────────────

async fn emit_job_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    job_id: &str,
    slug: &str,
    payload: serde_json::Value,
) {
    let envelope = match EventEnvelope::new(event_type, job_id, &payload) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, event_type, "liveness: failed to build envelope: {e}");
            return;
        }
    };

    runtime::services::orchestrator::emit::persist_and_route(
        kernel, envelope, slug, None,
    ).await;
}
