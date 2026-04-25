//! MCP tool handlers for job lifecycle management.
//!
//! Jobs are event-sourced. These handlers emit the appropriate `job.*` event
//! and return the result. State is read back via the projection in `load_jobs`.

use runtime::events::job::{
    JobBlockedPayload, JobCompletedPayload, JobDispatchedPayload,
    JobFailedPayload, JobGateFailedPayload, JobGatePassedPayload, JobGateRequestedPayload,
    JobMergedPayload, StepDoneDeclaredPayload, event_types,
};
use runtime::events::{EventEnvelope, EventStore, SqliteEventStore};
use runtime::projections::job::load_jobs;

use crate::requests::{CreateJobRequest, StepDoneRequest, UpdateJobRequest};

/// Create a job by delegating to the daemon's REST API.
///
/// The daemon handles pipeline enrichment (reading .ship/pipelines/),
/// event persistence, kernel routing, and dispatch — one code path.
pub async fn create_job(req: CreateJobRequest) -> String {
    let body = serde_json::json!({
        "slug": req.slug,
        "agent": req.agent,
        "branch": req.branch,
        "spec_path": req.spec_path,
        "depends_on": req.depends_on,
    });
    match crate::network_client::create_job_via_daemon(body).await {
        Ok(resp) => resp,
        Err(e) => format!("Error creating job: {e}"),
    }
}

/// Emit the appropriate event for the requested status transition.
pub async fn update_job(req: UpdateJobRequest) -> String {
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => return format!("Error opening event store: {e}"),
    };

    let envelope_result = match req.status.as_str() {
        "dispatched" => {
            let worktree = match req.worktree {
                Some(w) => w,
                None => return "Error: worktree is required for status 'dispatched'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_DISPATCHED,
                &req.job_id,
                &JobDispatchedPayload {
                    job_id: req.job_id.clone(),
                    worktree,
                    pid: req.pid,
                    provider_meta: None,
                },
            )
        }
        "gate_requested" => {
            let gate_agent = match req.gate_agent {
                Some(g) => g,
                None => return "Error: gate_agent is required for status 'gate_requested'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_GATE_REQUESTED,
                &req.job_id,
                &JobGateRequestedPayload {
                    job_id: req.job_id.clone(),
                    gate_agent,
                },
            )
        }
        "gate_passed" => EventEnvelope::new(
            event_types::JOB_GATE_PASSED,
            &req.job_id,
            &JobGatePassedPayload {
                job_id: req.job_id.clone(),
            },
        ),
        "gate_failed" => {
            let reason = match req.error {
                Some(r) => r,
                None => return "Error: error is required for status 'gate_failed'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_GATE_FAILED,
                &req.job_id,
                &JobGateFailedPayload {
                    job_id: req.job_id.clone(),
                    reason,
                },
            )
        }
        "blocked" => {
            let blocker = match req.blocker {
                Some(b) => b,
                None => return "Error: blocker is required for status 'blocked'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_BLOCKED,
                &req.job_id,
                &JobBlockedPayload {
                    job_id: req.job_id.clone(),
                    blocker,
                    needs_human: true,
                },
            )
        }
        "completed" => {
            let slug = resolve_slug(&store, &req.job_id);
            EventEnvelope::new(
                event_types::JOB_COMPLETED,
                &req.job_id,
                &JobCompletedPayload {
                    job_id: req.job_id.clone(),
                    slug,
                },
            )
        }
        "merged" => {
            let slug = resolve_slug(&store, &req.job_id);
            EventEnvelope::new(
                event_types::JOB_MERGED,
                &req.job_id,
                &JobMergedPayload {
                    job_id: req.job_id.clone(),
                    slug,
                },
            )
        }
        "failed" => {
            let error = match req.error {
                Some(e) => e,
                None => return "Error: error is required for status 'failed'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_FAILED,
                &req.job_id,
                &JobFailedPayload {
                    job_id: req.job_id.clone(),
                    error,
                },
            )
        }
        other => {
            return format!(
                "Error: unknown status '{other}'. Valid values: \
                dispatched, gate_requested, gate_passed, gate_failed, blocked, completed, merged, failed"
            );
        }
    };

    let envelope = match envelope_result {
        Ok(e) => e,
        Err(e) => return format!("Error building event: {e}"),
    };

    // Route through daemon first — this is the authoritative path that triggers
    // pipeline advancement, job dispatch, and subscriber delivery. The daemon
    // persists to its own event store.
    let daemon_ok = match crate::network_client::event_route(&envelope, None, None).await {
        Ok(_) => true,
        Err(e) => {
            tracing::warn!("job.{} daemon routing failed, falling back to local persist: {e}", req.status);
            false
        }
    };

    // Local persist as fallback — ensures the event is recorded even if daemon is down.
    // On daemon restart, reconciliation replays from the local store.
    if !daemon_ok
        && let Err(e) = store.append(&envelope)
    {
        return format!("Error persisting event: {e}");
    }

    format!("Job {} updated to status '{}'", req.job_id, req.status)
}

/// Declare the current pipeline step complete.
///
/// Reads `SHIP_MESH_ID` to identify the job, resolves the current step from
/// the projection, and emits a `step.done_declared` event via the daemon.
pub async fn step_done(req: StepDoneRequest) -> String {
    let slug = match runtime::env::read_raw(runtime::env::Var::MeshId) {
        Ok(id) if !id.is_empty() => id,
        _ => return "Error: SHIP_MESH_ID not set — step_done can only be called from a dispatched job".to_string(),
    };

    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => return format!("Error loading jobs: {e}"),
    };

    let job = match jobs.values().find(|j| j.slug == slug) {
        Some(j) => j.clone(),
        None => return format!("Error: no job found for slug '{slug}'"),
    };

    let step_id = match &job.current_step {
        Some(id) => id.clone(),
        None => return format!("Error: no active step for job '{slug}'"),
    };

    let payload = StepDoneDeclaredPayload {
        job_id: job.job_id.clone(),
        slug: slug.clone(),
        step_id: step_id.clone(),
        summary: req.summary,
        outputs: req.outputs,
    };

    let envelope = match EventEnvelope::new(
        event_types::STEP_DONE_DECLARED,
        &job.job_id,
        &payload,
    ) {
        Ok(e) => e,
        Err(e) => return format!("Error building event: {e}"),
    };

    match crate::network_client::event_route(&envelope, None, None).await {
        Ok(_) => format!("Step '{step_id}' marked done for job '{slug}'"),
        Err(e) => {
            // Fallback: persist locally
            if let Ok(store) = SqliteEventStore::new() {
                let _ = EventStore::append(&store, &envelope);
            }
            tracing::warn!("step_done daemon routing failed, persisted locally: {e}");
            format!("Step '{step_id}' marked done (local fallback)")
        }
    }
}

/// Look up the slug for a job_id from the event store projection.
fn resolve_slug(_store: &SqliteEventStore, job_id: &str) -> String {
    load_jobs()
        .ok()
        .and_then(|jobs| jobs.get(job_id).map(|r| r.slug.clone()))
        .unwrap_or_default()
}
