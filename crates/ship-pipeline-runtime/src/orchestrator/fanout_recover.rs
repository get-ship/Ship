//! Restart recovery for orphaned fanout lanes.
//!
//! `OrchestratorContext.fanout_lane_refs` / `fanout_lane_txs` live
//! in-memory only. If shipd crashes mid-fanout, the mapping between
//! parent step and in-flight lanes is gone and the aggregator task
//! that was going to emit the fanout outcome is gone with it.
//!
//! This module scans the event store on startup for lane-scoped
//! `job.step_dispatched` events that have no matching
//! `step.done_declared` and takes one of two actions per lane:
//!
//! * parent job terminal → clean up the lane's worktree/tmux session
//! * parent job live     → emit a synthetic `job.step_failed` for the
//!   lane so the (fresh) aggregator can resolve through the normal
//!   `min_successful_lanes` path
//!
//! Idempotent: running the reconcile twice emits no duplicate events
//! because the first pass writes a `step.done_declared` via the
//! `emit_failed_marker` path, which the second pass observes as
//! "already reported."

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::KernelRouter;
use ship_platform::projections::pipeline_run::{PipelineRun, PipelineRunStatus};
use crate::orchestrator::OrchestratorContext;
use ship_runner::emit::persist_and_route;
use crate::orchestrator::fanout_aggregate::emit_failed_marker;

/// Emit a lane-scoped `job.step_dispatched` with the metadata the restart
/// reconciler needs to find orphaned lanes after a daemon crash.
///
/// Payload fields beyond the standard step_dispatched shape:
/// `fanout_parent_step`, `lane_id`, `lane_slug`, and `step_type: "fanout_lane"`.
pub async fn emit_fanout_lane_dispatched(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    parent_slug: &str,
    fanout_step_id: &str,
    lane_id: &str,
    lane_slug: &str,
    lane_step_id: &str,
) {
    let payload = serde_json::json!({
        "pipeline_run_id": pipeline_run_id,
        "slug": parent_slug,
        "step_id": lane_step_id,
        "step_type": "fanout_lane",
        "fanout_parent_step": fanout_step_id,
        "lane_id": lane_id,
        "lane_slug": lane_slug,
    });
    let envelope = match EventEnvelope::new(event_types::PIPELINE_STEP_STARTED, pipeline_run_id, &payload) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(e) => {
            tracing::warn!(parent_slug, lane_id, "fanout: failed to build lane step_dispatched: {e}");
            return;
        }
    };
    persist_and_route(kernel, envelope, parent_slug, None).await;
}

/// Parsed lane dispatch record — the subset of a `job.step_dispatched`
/// payload we care about for recovery.
#[derive(Debug, Clone)]
pub(crate) struct LaneDispatchRecord {
    pub pipeline_run_id: String,
    pub parent_slug: String,
    pub fanout_step_id: String,
    pub lane_id: String,
    pub lane_slug: String,
    pub lane_step_id: String,
}

/// Scan `events` for lane `job.step_dispatched` entries that have no
/// matching `step.done_declared` (on the same lane_step_id). The result
/// is every lane that looked in-flight at the moment of the crash.
pub(crate) fn find_orphan_lanes(events: &[EventEnvelope]) -> Vec<LaneDispatchRecord> {
    let mut dispatched: HashMap<String, LaneDispatchRecord> = HashMap::new();
    let mut completed: HashSet<String> = HashSet::new();

    for env in events {
        match env.event_type.as_str() {
            event_types::PIPELINE_STEP_STARTED => {
                let Ok(p) = serde_json::from_str::<serde_json::Value>(&env.payload_json) else {
                    continue;
                };
                if p.get("step_type").and_then(|v| v.as_str()) != Some("fanout_lane") {
                    continue;
                }
                let pipeline_run_id = p.get("pipeline_run_id").and_then(|v| v.as_str()).unwrap_or_default();
                let parent_slug = p.get("slug").and_then(|v| v.as_str()).unwrap_or_default();
                let fanout_step_id = p.get("fanout_parent_step").and_then(|v| v.as_str()).unwrap_or_default();
                let lane_id = p.get("lane_id").and_then(|v| v.as_str()).unwrap_or_default();
                let lane_slug = p.get("lane_slug").and_then(|v| v.as_str()).unwrap_or_default();
                let lane_step_id = p.get("step_id").and_then(|v| v.as_str()).unwrap_or_default();
                if pipeline_run_id.is_empty() || lane_slug.is_empty() || lane_step_id.is_empty() {
                    continue;
                }
                dispatched.insert(
                    lane_step_id.to_string(),
                    LaneDispatchRecord {
                        pipeline_run_id: pipeline_run_id.to_string(),
                        parent_slug: parent_slug.to_string(),
                        fanout_step_id: fanout_step_id.to_string(),
                        lane_id: lane_id.to_string(),
                        lane_slug: lane_slug.to_string(),
                        lane_step_id: lane_step_id.to_string(),
                    },
                );
            }
            event_types::PIPELINE_STEP_DONE_DECLARED => {
                let Ok(p) = serde_json::from_str::<serde_json::Value>(&env.payload_json) else {
                    continue;
                };
                if let Some(step_id) = p.get("step_id").and_then(|v| v.as_str()) {
                    completed.insert(step_id.to_string());
                }
            }
            _ => {}
        }
    }

    dispatched
        .into_values()
        .filter(|r| !completed.contains(&r.lane_step_id))
        .collect()
}

/// Run recovery for every orphaned lane found in the event store.
///
/// For each lane:
///   * parent job is terminal → `platform.cleanup_job` the lane slug
///   * parent job is live     → emit a synthetic failed marker so the
///     aggregator can resolve via `min_successful_lanes`
pub async fn reconcile_fanout_lanes(
    kernel: &Arc<Mutex<KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let events = match ship_substrate::db::events::list_all_events() {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("fanout-recover: failed to load events: {e}");
            return;
        }
    };
    let orphans = find_orphan_lanes(&events);
    if orphans.is_empty() {
        tracing::debug!("fanout-recover: no orphan lanes");
        return;
    }

    let jobs = match ship_platform::projections::pipeline_run::load_pipeline_runs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("fanout-recover: failed to load jobs projection: {e}");
            return;
        }
    };

    tracing::info!(
        orphan_count = orphans.len(),
        "fanout-recover: reconciling orphan fanout lanes"
    );

    for lane in &orphans {
        let parent = jobs.values().find(|j| j.pipeline_run_id == lane.pipeline_run_id);
        let terminal = matches!(
            parent.map(|j| &j.status),
            Some(PipelineRunStatus::Cancelled) | Some(PipelineRunStatus::Failed) | Some(PipelineRunStatus::Merged)
        );
        if terminal {
            let branch = lane_branch(parent, &lane.parent_slug, &lane.fanout_step_id, &lane.lane_id);
            tracing::info!(
                pipeline_run_id = lane.pipeline_run_id, lane_slug = lane.lane_slug,
                "fanout-recover: parent terminal — cleaning lane worktree"
            );
            octx.platform.cleanup_job(&lane.lane_slug, &branch).await;
            continue;
        }

        // Parent still live: synthesize a failed outcome so the fresh
        // aggregator (if the parent job replays) has a lane result via
        // min_successful_lanes. Uses the same failed-marker path that
        // the runtime uses for ordinary lane failures, so the judge's
        // `consumes: [<lane_id>.failed]` resolves identically.
        let reason = "daemon restart: lane orphaned";
        tracing::info!(
            pipeline_run_id = lane.pipeline_run_id, lane_slug = lane.lane_slug,
            "fanout-recover: parent live — emitting synthetic failed marker"
        );
        let _ = emit_failed_marker(
            kernel, &lane.pipeline_run_id, &lane.parent_slug,
            &lane.lane_step_id, &lane.lane_id, reason,
        ).await;
    }
}

/// Derive a lane's branch name from the projection (preferred) or fall
/// back to the dispatch naming convention.
fn lane_branch(
    parent: Option<&PipelineRun>,
    parent_slug: &str,
    fanout_step_id: &str,
    lane_id: &str,
) -> String {
    let base = parent
        .map(|j| j.branch.clone())
        .unwrap_or_else(|| format!("job/{parent_slug}"));
    format!("{base}-{fanout_step_id}-{lane_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(event_type: &str, entity_id: &str, payload: serde_json::Value) -> EventEnvelope {
        EventEnvelope::new(event_type, entity_id, &payload).unwrap()
    }

    #[test]
    fn orphan_detection_matches_dispatched_minus_completed() {
        let events = vec![
            ev(event_types::PIPELINE_STEP_STARTED, "j1", serde_json::json!({
                "pipeline_run_id": "j1", "slug": "parent",
                "step_id": "draft.lane-a", "step_type": "fanout_lane",
                "fanout_parent_step": "draft", "lane_id": "lane-a",
                "lane_slug": "parent-draft-lane-a",
            })),
            ev(event_types::PIPELINE_STEP_STARTED, "j1", serde_json::json!({
                "pipeline_run_id": "j1", "slug": "parent",
                "step_id": "draft.lane-b", "step_type": "fanout_lane",
                "fanout_parent_step": "draft", "lane_id": "lane-b",
                "lane_slug": "parent-draft-lane-b",
            })),
            ev(event_types::PIPELINE_STEP_DONE_DECLARED, "j1", serde_json::json!({
                "pipeline_run_id": "j1", "slug": "parent",
                "step_id": "draft.lane-a",
                "outputs": {}, "summary": "ok",
            })),
        ];
        let orphans = find_orphan_lanes(&events);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].lane_id, "lane-b");
        assert_eq!(orphans[0].lane_step_id, "draft.lane-b");
    }

    #[test]
    fn non_fanout_step_dispatched_events_are_ignored() {
        let events = vec![ev(event_types::PIPELINE_STEP_STARTED, "j1", serde_json::json!({
            "pipeline_run_id": "j1", "slug": "parent",
            "step_id": "build", "step_type": "agent",
        }))];
        assert!(find_orphan_lanes(&events).is_empty());
    }

    #[test]
    fn completed_lanes_are_not_orphans() {
        let events = vec![
            ev(event_types::PIPELINE_STEP_STARTED, "j1", serde_json::json!({
                "pipeline_run_id": "j1", "slug": "parent",
                "step_id": "draft.l", "step_type": "fanout_lane",
                "fanout_parent_step": "draft", "lane_id": "l",
                "lane_slug": "parent-draft-l",
            })),
            ev(event_types::PIPELINE_STEP_DONE_DECLARED, "j1", serde_json::json!({
                "pipeline_run_id": "j1", "slug": "parent",
                "step_id": "draft.l", "outputs": {}, "summary": "",
            })),
        ];
        assert!(find_orphan_lanes(&events).is_empty());
    }

    #[test]
    fn malformed_payloads_are_skipped_silently() {
        let events = vec![ev(event_types::PIPELINE_STEP_STARTED, "j1", serde_json::json!({
            "step_type": "fanout_lane",
            // missing pipeline_run_id, lane_slug, step_id
        }))];
        assert!(find_orphan_lanes(&events).is_empty());
    }
}
