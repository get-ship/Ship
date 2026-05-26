//! Fanout aggregation — lane completion, failed-lane markers, and the pure
//! `Advance` / `Fail` decision the aggregator applies once every lane has
//! reported.
//!
//! Extracted from `fanout.rs` so the decision function stays pure (testable
//! from integration tests without standing up a real tokio dispatch) and so
//! neither file exceeds the 300-line cap.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::{ArtifactScope, ArtifactService};
use ship_substrate::events::pipeline::{
    PipelineStepDoneDeclaredPayload, PipelineStepObservedPayload, StepVerdict, event_types,
};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use crate::services::orchestrator::emit::persist_and_route;
use crate::services::orchestrator::step_io::get_step_outputs_from_events;
use crate::services::orchestrator::{FanoutLaneRef, LaneOutcome, OrchestratorContext};

// `AggregateDecision` + `aggregate_lane_outcomes` moved to
// `ship-pipelines::fanout` per modular-runtime-01. Re-exported here so
// `fanout.rs`, integration tests, etc. keep importing from the legacy
// path without churn.
pub use ship_pipelines::fanout::{AggregateDecision, aggregate_lane_outcomes};

/// Reserved output name used for a failed-lane marker. Source-of-truth
/// is `ship-pipelines::validate::FAILED_MARKER_NAME`; re-exported here
/// for the daemon-side fanout machinery that runs the marker emission.
pub use ship_pipelines::validate::FAILED_MARKER_NAME;

/// Resolve a lane's `LaneOutcome` from the router when its `step.observed`
/// arrives. Reads lane outputs from the event store and sends through the
/// registered channel. On failure, emits a `<lane_id>.failed` sentinel
/// artifact so the judge's `consumes` resolves via the normal path.
pub async fn complete_fanout_lane(
    kernel: &Arc<Mutex<KernelRouter>>,
    payload: &PipelineStepObservedPayload,
    lane_ref: &FanoutLaneRef,
    octx: &OrchestratorContext,
) {
    let lane_step_id = format!("{}.{}", lane_ref.fanout_step_id, lane_ref.lane_id);
    let success = matches!(payload.verdict, StepVerdict::Advance | StepVerdict::Inconclusive);
    let mut outputs = get_step_outputs_from_events(&lane_ref.pipeline_run_id, &lane_step_id);

    if !success {
        tracing::warn!(
            pipeline_run_id = lane_ref.pipeline_run_id, lane_id = lane_ref.lane_id,
            step = lane_ref.fanout_step_id, reason = ?payload.reason,
            "fanout: lane failed"
        );
        // Emit a `<lane_id>.failed` marker artifact + synthetic step.done_declared
        // so the judge step's `consumes` resolves through the same mechanism
        // used for successful outputs. No special-casing required downstream.
        let reason = payload.reason.clone().unwrap_or_else(|| "lane failed".to_string());
        if let Some(marker_id) = emit_failed_marker(
            kernel, &lane_ref.pipeline_run_id, &lane_ref.parent_slug,
            &lane_step_id, &lane_ref.lane_id, &reason,
        ).await {
            outputs.insert(FAILED_MARKER_NAME.to_string(), marker_id);
        }
    } else {
        tracing::info!(
            pipeline_run_id = lane_ref.pipeline_run_id, lane_id = lane_ref.lane_id,
            step = lane_ref.fanout_step_id, outputs_count = outputs.len(),
            "fanout: lane succeeded"
        );
    }

    // Clean up registry entry for the lane.
    octx.registry.unregister(&payload.slug).await;

    let outcome = LaneOutcome { lane_id: lane_ref.lane_id.clone(), success, outputs };

    // Remove and send through the oneshot channel (if still registered).
    let tx = octx.fanout_lane_txs.lock().await.remove(payload.slug.as_str());
    if let Some(tx) = tx {
        let _ = tx.send(outcome);
    } else {
        tracing::warn!(
            slug = payload.slug, lane_id = lane_ref.lane_id,
            "fanout: lane completion channel already consumed or missing"
        );
    }
    // Remove lane ref now that it's handled.
    octx.fanout_lane_refs.lock().await.remove(payload.slug.as_str());
}

/// Store a sentinel "failed" artifact for a failed lane and emit a synthetic
/// `step.done_declared` event for the lane's step_id so the judge's
/// `consumes: [<lane_id>.failed]` resolves via the normal event-store path.
///
/// Returns the marker artifact id on success. Logs and returns `None` on
/// any failure — a failed lane should never block aggregator progress.
pub(crate) async fn emit_failed_marker(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    lane_step_id: &str,
    lane_id: &str,
    reason: &str,
) -> Option<String> {
    let ship_dir = match ship_substrate::project::get_global_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(pipeline_run_id, lane_id, "fanout: failed marker — cannot get ship dir: {e}");
            return None;
        }
    };
    let mut data = match ship_substrate::data::DataService::open(&ship_dir) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(pipeline_run_id, lane_id, "fanout: failed marker — cannot open data service: {e}");
            return None;
        }
    };
    let space_id = ship_substrate::project::project_slug_from_ship_dir(&ship_dir);
    let art_svc = ArtifactService::new(&ship_dir);
    let scope = ArtifactScope::Run {
        space_id,
        run_id: pipeline_run_id.to_string(),
    };
    let body = format!(
        "# Lane `{lane_id}` failed\n\nreason: {reason}\nstep_id: {lane_step_id}\n"
    );
    let rec = match art_svc
        .store(
            &mut data, kernel,
            &format!("{lane_id}.failed"),
            "text/plain",
            body.as_bytes(),
            &scope,
            None, None, Some("service.fanout"), None,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(pipeline_run_id, lane_id, "fanout: failed marker — artifact store failed: {e}");
            return None;
        }
    };

    // Synthetic step.done_declared so resolve_consumes finds the marker the
    // same way it finds any other lane output.
    let payload = PipelineStepDoneDeclaredPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        step_id: lane_step_id.to_string(),
        summary: Some(format!("lane failed: {reason}")),
        outputs: [(FAILED_MARKER_NAME.to_string(), rec.id.clone())].into(),
    };
    match EventEnvelope::new(event_types::PIPELINE_STEP_DONE_DECLARED, pipeline_run_id, &payload) {
        Ok(env) => {
            let env = env.with_actor_id("service.fanout");
            persist_and_route(kernel, env, slug, None).await;
        }
        Err(e) => {
            tracing::warn!(pipeline_run_id, lane_id, "fanout: failed marker — envelope build failed: {e}");
        }
    }
    Some(rec.id)
}
