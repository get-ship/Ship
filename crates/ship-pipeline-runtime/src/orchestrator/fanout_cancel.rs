//! Fanout cancellation — tears down every registered lane under a job
//! when the operator cancels the parent.
//!
//! Extracted from `handlers.rs` so that file stays under the 300-line cap
//! and the cancellation logic can be unit-tested in isolation.

use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::KernelRouter;
use crate::orchestrator::OrchestratorContext;

/// Tear down every registered fanout lane under `pipeline_run_id`: call
/// `cleanup_job` on the lane's worktree/tmux session, drop the
/// oneshot sender so the aggregator stops blocking, and clear the
/// per-lane registry entries.
///
/// Idempotent: lanes not registered (already completed or never spawned)
/// are silently skipped. The aggregator's `rx.await` resolves to
/// `Err(RecvError)` as soon as the sender is dropped here, so the
/// fanout background task exits promptly instead of blocking on a
/// lane that will never report.
pub async fn cancel_job_fanout_lanes(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    octx: &OrchestratorContext,
) {
    // Snapshot the lane refs for this job under a short lock, then release
    // it so cleanup_job (which may be slow) doesn't hold the global mutex.
    let lane_slugs: Vec<(String, String)> = {
        let refs = octx.fanout_lane_refs.lock().await;
        refs.iter()
            .filter(|(_, r)| r.pipeline_run_id == pipeline_run_id)
            .map(|(slug, r)| {
                // Lane branch mirrors the naming in `dispatch_fanout_step`:
                // `{job.branch}-{step.id}-{lane.id}`. The job's branch is
                // derived from its slug (`job/{slug}`).
                let branch = format!(
                    "job/{}-{}-{}",
                    r.parent_slug, r.fanout_step_id, r.lane_id
                );
                (slug.clone(), branch)
            })
            .collect()
    };

    if lane_slugs.is_empty() {
        return;
    }

    tracing::info!(
        pipeline_run_id, lane_count = lane_slugs.len(),
        "job-dispatch: cancelling fanout lanes"
    );

    for (lane_slug, lane_branch) in &lane_slugs {
        if let Some(handle) = octx.registry.take(lane_slug).await {
            let pid = handle.pid;
            super::emit::emit_process_signal_sent(
                kernel, lane_slug, pid, "SIGTERM",
                "parent_job_cancelled", "operator",
            ).await;
            if let Err(e) = octx.executor.stop(&handle).await {
                tracing::warn!(
                    lane_slug, "job-dispatch: lane cancel stop returned err: {e}"
                );
            }
        }

        octx.platform.cleanup_job(lane_slug, lane_branch).await;
        octx.done_flags.lock().await.remove(lane_slug);
        octx.activity_txs.lock().await.remove(lane_slug);
    }

    // Drop the lane oneshot senders — dropping the Sender causes the
    // aggregator's `rx.await` to resolve to `Err(RecvError)` immediately
    // instead of blocking forever. Also remove the lane refs so a
    // late-arriving step.observed doesn't re-enter the aggregator path.
    let mut txs = octx.fanout_lane_txs.lock().await;
    let mut refs = octx.fanout_lane_refs.lock().await;
    for (lane_slug, _) in &lane_slugs {
        txs.remove(lane_slug);
        refs.remove(lane_slug);
    }
}
