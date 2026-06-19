//! Agent-driven lifecycle cores — `update_job_status_core` + `step_done_core`.
//!
//! These were previously the REST module `apps/shipd/src/project_api/jobs_status.rs`
//! and a back-call target for the ConnectRPC `pipeline.update_status` /
//! `pipeline.step_done` ops. Both are pure pipeline-domain logic: they build the
//! matching `pipeline.*`/`gate.*` envelope from a transport-neutral request and
//! persist it through the [`PipelineDispatchPort`]. Errors are transport-neutral
//! [`StatusCoreError`]; each surface maps them to its own wire shape.

use serde::Deserialize;
use serde_json::json;

use crate::pipeline_ports::PipelineDispatchPort;

#[derive(Debug)]
pub enum StatusCoreError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

#[derive(Deserialize)]
pub struct RunStatusReq {
    /// One of: dispatched, gate_requested, gate_passed, gate_failed, blocked,
    /// completed, merged, failed.
    pub status: String,
    pub worktree: Option<String>,
    pub pid: Option<u32>,
    pub gate_agent: Option<String>,
    pub blocker: Option<String>,
    pub error: Option<String>,
}

/// Core of the job-status command, shared by the REST handler and the
/// ConnectRPC `PipelinesService`. Builds the matching `pipeline.*`/`gate.*`
/// envelope from `body.status` and persists it.
pub async fn update_job_status_core(
    port: &dyn PipelineDispatchPort,
    pipeline_run_id: &str,
    body: RunStatusReq,
) -> Result<serde_json::Value, StatusCoreError> {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{
        PipelineRunBlockedPayload, PipelineRunCompletedPayload, PipelineRunDispatchedPayload, PipelineRunFailedPayload,
        PipelineGateFailedPayload, PipelineGatePassedPayload, PipelineGateRequestedPayload, PipelineRunMergedPayload,
        event_types,
    };

    let require = |field: Option<String>, what: &str| {
        field.ok_or_else(|| {
            StatusCoreError::BadRequest(format!("{what} is required for status '{}'", body.status))
        })
    };

    let envelope_result = match body.status.as_str() {
        "dispatched" => {
            let worktree = require(body.worktree.clone(), "worktree")?;
            EventEnvelope::new(
                event_types::PIPELINE_RUN_DISPATCHED,
                pipeline_run_id,
                &PipelineRunDispatchedPayload {
                    pipeline_run_id: pipeline_run_id.to_string(),
                    // External status-update endpoint only has pipeline_run_id;
                    // slug is unknown here. Consumers that need filtering by slug
                    // resolve it from the pipeline_run record or the dispatch path.
                    slug: String::new(),
                    worktree,
                    pid: body.pid,
                    harness_meta: None,
                },
            )
        }
        "gate_requested" => {
            let gate_agent = require(body.gate_agent.clone(), "gate_agent")?;
            EventEnvelope::new(
                event_types::PIPELINE_GATE_REQUESTED,
                pipeline_run_id,
                &PipelineGateRequestedPayload {
                    pipeline_run_id: pipeline_run_id.to_string(),
                    gate_agent,
                },
            )
        }
        "gate_passed" => EventEnvelope::new(
            event_types::PIPELINE_GATE_PASSED,
            pipeline_run_id,
            &PipelineGatePassedPayload { pipeline_run_id: pipeline_run_id.to_string() },
        ),
        "gate_failed" => {
            let reason = require(body.error.clone(), "error")?;
            EventEnvelope::new(
                event_types::PIPELINE_GATE_FAILED,
                pipeline_run_id,
                &PipelineGateFailedPayload { pipeline_run_id: pipeline_run_id.to_string(), reason },
            )
        }
        "blocked" => {
            let blocker = require(body.blocker.clone(), "blocker")?;
            EventEnvelope::new(
                event_types::PIPELINE_RUN_BLOCKED,
                pipeline_run_id,
                &PipelineRunBlockedPayload {
                    pipeline_run_id: pipeline_run_id.to_string(),
                    blocker,
                    needs_human: true,
                },
            )
        }
        "completed" => {
            let slug = resolve_slug(port.ship_dir(), pipeline_run_id);
            EventEnvelope::new(
                event_types::PIPELINE_RUN_COMPLETED,
                pipeline_run_id,
                &PipelineRunCompletedPayload { pipeline_run_id: pipeline_run_id.to_string(), slug },
            )
        }
        "merged" => {
            let slug = resolve_slug(port.ship_dir(), pipeline_run_id);
            EventEnvelope::new(
                event_types::PIPELINE_RUN_MERGED,
                pipeline_run_id,
                &PipelineRunMergedPayload { pipeline_run_id: pipeline_run_id.to_string(), slug },
            )
        }
        "failed" => {
            let error = require(body.error.clone(), "error")?;
            EventEnvelope::new(
                event_types::PIPELINE_RUN_FAILED,
                pipeline_run_id,
                &PipelineRunFailedPayload { pipeline_run_id: pipeline_run_id.to_string(), error },
            )
        }
        other => {
            return Err(StatusCoreError::BadRequest(format!(
                "unknown status '{other}'. Valid: dispatched, gate_requested, gate_passed, \
                gate_failed, blocked, completed, merged, failed"
            )));
        }
    };

    let envelope = envelope_result.map_err(|e| StatusCoreError::BadRequest(e.to_string()))?;
    port.persist_and_route(&envelope)
        .await
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?;

    Ok(json!({ "pipeline_run_id": pipeline_run_id, "status": body.status }))
}

fn resolve_slug(ship_dir: &std::path::Path, pipeline_run_id: &str) -> String {
    crate::projections::pipeline_run::load_pipeline_runs_at(ship_dir)
        .ok()
        .and_then(|jobs| jobs.get(pipeline_run_id).map(|r| r.slug.clone()))
        .unwrap_or_default()
}

// ---- Step-done declaration ----

#[derive(Deserialize)]
pub struct StepDoneReq {
    pub summary: Option<String>,
    /// Map of `produces.name → artifact_id` for step outputs.
    #[serde(default)]
    pub outputs: std::collections::BTreeMap<String, String>,
}

/// Core of the step-done command, shared by the REST handler and the ConnectRPC
/// `PipelinesService`. Resolves the active step from the projection and emits
/// `step.done_declared`.
pub async fn step_done_core(
    port: &dyn PipelineDispatchPort,
    slug: &str,
    body: StepDoneReq,
) -> Result<serde_json::Value, StatusCoreError> {
    let jobs = crate::projections::pipeline_run::load_pipeline_runs_at(port.ship_dir())
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?;
    let job = jobs
        .values()
        .find(|j| j.slug == slug)
        .cloned()
        .ok_or_else(|| StatusCoreError::NotFound(format!("no job with slug '{slug}'")))?;
    let step_id = job
        .current_step
        .clone()
        .ok_or_else(|| StatusCoreError::BadRequest(format!("no active step for job '{slug}'")))?;

    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{PipelineStepDoneDeclaredPayload, event_types};

    let payload = PipelineStepDoneDeclaredPayload {
        pipeline_run_id: job.pipeline_run_id.clone(),
        slug: slug.to_string(),
        step_id: step_id.clone(),
        summary: body.summary,
        outputs: body.outputs,
    };
    let envelope =
        EventEnvelope::new(event_types::PIPELINE_STEP_DONE_DECLARED, &job.pipeline_run_id, &payload)
            .map_err(|e| StatusCoreError::BadRequest(e.to_string()))?;
    port.persist_and_route(&envelope)
        .await
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?;

    Ok(json!({ "slug": slug, "step_id": step_id }))
}

// ---- Cancel (operator-initiated run termination) ----

/// Core of the run-cancel command — the typed Connect `Cancel` verb (was
/// REST-only `POST /jobs/{slug}/cancel`). Resolves the run by slug via the
/// `pipeline_runs` projection, rejects an already-terminal run (no event), else
/// emits `PIPELINE_RUN_CANCELLED`. Returns the typed [`CancelRunResult`].
pub async fn cancel_run_core(
    port: &dyn PipelineDispatchPort,
    slug: &str,
    reason: &str,
) -> Result<ship_ops_rpc::CancelRunResult, StatusCoreError> {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{PipelineRunCancelledPayload, event_types};

    use crate::projections::pipeline_run::{PipelineRunStatus, load_pipeline_run_by_slug_at};

    let run = load_pipeline_run_by_slug_at(port.ship_dir(), slug)
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?
        .ok_or_else(|| StatusCoreError::NotFound(format!("no pipeline run with slug '{slug}'")))?;

    // Idempotent: a terminal run is not cancellable — caller error, no event.
    if matches!(
        run.status,
        PipelineRunStatus::Failed
            | PipelineRunStatus::Merged
            | PipelineRunStatus::Cancelled
            | PipelineRunStatus::Completed
    ) {
        return Err(StatusCoreError::BadRequest(format!(
            "run '{slug}' is already terminal ({})",
            format!("{:?}", run.status).to_lowercase()
        )));
    }

    let reason = if reason.trim().is_empty() {
        "operator-cancelled".to_string()
    } else {
        reason.to_string()
    };
    let payload = PipelineRunCancelledPayload {
        pipeline_run_id: run.pipeline_run_id.clone(),
        slug: slug.to_string(),
        reason,
        requested_by: "operator".to_string(),
    };
    let envelope =
        EventEnvelope::new(event_types::PIPELINE_RUN_CANCELLED, &run.pipeline_run_id, &payload)
            .map(|e| e.with_actor_id("operator"))
            .map_err(|e| StatusCoreError::BadRequest(e.to_string()))?;
    port.persist_and_route(&envelope)
        .await
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?;

    Ok(ship_ops_rpc::CancelRunResult {
        pipeline_run_id: run.pipeline_run_id,
        slug: slug.to_string(),
        status: "cancelling".to_string(),
        ..Default::default()
    })
}

// ---- ResolveStep (the typed resume verb, step-id-keyed) ----

/// Core of the `ResolveStep` verb — the TYPED resume seam, keyed by step id.
///
/// FOR NOW it maps onto the EXISTING advance flow: it emits
/// `PIPELINE_STEP_DONE_DECLARED` for the named step (the same signal `step_done`
/// rides), so the orchestrator advances the run today. At the dissolution (P2.x)
/// this becomes engine `resume(decision)` with retry/fail branches; `decision`
/// is already on the wire so callers don't have to change shape then. Returns the
/// typed [`ResolveStepResult`].
pub async fn resolve_step_core(
    port: &dyn PipelineDispatchPort,
    slug: &str,
    step_id: &str,
    summary: Option<String>,
) -> Result<ship_ops_rpc::ResolveStepResult, StatusCoreError> {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{PipelineStepDoneDeclaredPayload, event_types};

    use crate::projections::pipeline_run::load_pipeline_run_by_slug_at;

    if step_id.trim().is_empty() {
        return Err(StatusCoreError::BadRequest("step_id is required".to_string()));
    }
    let run = load_pipeline_run_by_slug_at(port.ship_dir(), slug)
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?
        .ok_or_else(|| StatusCoreError::NotFound(format!("no pipeline run with slug '{slug}'")))?;

    let payload = PipelineStepDoneDeclaredPayload {
        pipeline_run_id: run.pipeline_run_id.clone(),
        slug: slug.to_string(),
        step_id: step_id.to_string(),
        summary,
        outputs: Default::default(),
    };
    let envelope =
        EventEnvelope::new(event_types::PIPELINE_STEP_DONE_DECLARED, &run.pipeline_run_id, &payload)
            .map_err(|e| StatusCoreError::BadRequest(e.to_string()))?;
    port.persist_and_route(&envelope)
        .await
        .map_err(|e| StatusCoreError::Internal(e.to_string()))?;

    Ok(ship_ops_rpc::ResolveStepResult {
        slug: slug.to_string(),
        step_id: step_id.to_string(),
        advanced: true,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::event_types;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// In-test `PipelineDispatchPort` fake: records emitted `(event_type,
    /// payload_json)` pairs and exposes a real (empty) ship_dir so the
    /// projection reads in the cores resolve deterministically.
    struct FakePort {
        ship_dir: PathBuf,
        _tmp: TempDir,
        emitted: Mutex<Vec<(String, String)>>,
    }

    impl FakePort {
        fn new() -> Self {
            let tmp = TempDir::new().unwrap();
            let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
            std::fs::create_dir_all(&ship_dir).unwrap();
            Self { ship_dir, _tmp: tmp, emitted: Mutex::new(Vec::new()) }
        }
    }

    #[async_trait::async_trait]
    impl PipelineDispatchPort for FakePort {
        async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
            self.emitted
                .lock()
                .unwrap()
                .push((envelope.event_type.clone(), envelope.payload_json.clone()));
            Ok(())
        }
        fn ship_dir(&self) -> &Path {
            &self.ship_dir
        }
    }

    fn req(status: &str) -> RunStatusReq {
        RunStatusReq {
            status: status.into(),
            worktree: None,
            pid: None,
            gate_agent: None,
            blocker: None,
            error: None,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatched_requires_worktree() {
        let port = FakePort::new();
        let err = update_job_status_core(&port, "run-1", req("dispatched")).await.unwrap_err();
        assert!(matches!(err, StatusCoreError::BadRequest(_)));
        assert!(port.emitted.lock().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_status_is_bad_request() {
        let port = FakePort::new();
        let err = update_job_status_core(&port, "run-1", req("invented")).await.unwrap_err();
        assert!(matches!(err, StatusCoreError::BadRequest(_)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gate_passed_maps_to_gate_passed_event() {
        let port = FakePort::new();
        let v = update_job_status_core(&port, "run-1", req("gate_passed")).await.unwrap();
        assert_eq!(v["pipeline_run_id"], "run-1");
        assert_eq!(v["status"], "gate_passed");
        let emitted = port.emitted.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, event_types::PIPELINE_GATE_PASSED);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dispatched_maps_to_dispatched_event() {
        let port = FakePort::new();
        let body = RunStatusReq {
            status: "dispatched".into(),
            worktree: Some("/tmp/ws".into()),
            pid: Some(4321),
            gate_agent: None,
            blocker: None,
            error: None,
        };
        update_job_status_core(&port, "run-1", body).await.unwrap();
        let emitted = port.emitted.lock().unwrap();
        assert_eq!(emitted[0].0, event_types::PIPELINE_RUN_DISPATCHED);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn completed_resolves_empty_slug_without_projection_row() {
        let port = FakePort::new();
        let v = update_job_status_core(&port, "run-1", req("completed")).await.unwrap();
        assert_eq!(v["status"], "completed");
        let emitted = port.emitted.lock().unwrap();
        assert_eq!(emitted[0].0, event_types::PIPELINE_RUN_COMPLETED);
        // No projection row exists for run-1 → slug resolves to the empty fallback.
        let payload: serde_json::Value = serde_json::from_str(&emitted[0].1).unwrap();
        assert_eq!(payload["slug"], "");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn step_done_not_found_for_unknown_slug() {
        let port = FakePort::new();
        let err = step_done_core(&port, "ghost-slug", StepDoneReq { summary: None, outputs: Default::default() })
            .await
            .unwrap_err();
        assert!(matches!(err, StatusCoreError::NotFound(_)));
    }

    // ── Cancel + ResolveStep (typed verbs, P0.3) ────────────────────────────

    /// Seed a run row into the port's `runtime.db` by writing the raw run
    /// events; the cores resolve it via `load_pipeline_run_by_slug_at`.
    fn seed_run(ship_dir: &Path, run_id: &str, slug: &str, terminal: bool) {
        use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
        drop(ship_substrate::db::open_runtime_db(ship_dir).unwrap());
        let store =
            SqliteEventStore::new_at_db_path(&ship_substrate::db::runtime_db_path(ship_dir)).unwrap();
        let started = EventEnvelope::new(
            event_types::PIPELINE_RUN_STARTED,
            run_id,
            &serde_json::json!({
                "pipeline_run_id": run_id, "slug": slug, "agent": "r",
                "branch": "job/x", "spec_path": "s",
                "steps": { "steps": [{ "id": "a", "type": "agent" }], "auto_merge": false }
            }),
        )
        .unwrap();
        store.append(&started).unwrap();
        if terminal {
            let m = EventEnvelope::new(
                event_types::PIPELINE_RUN_MERGED,
                run_id,
                &serde_json::json!({ "pipeline_run_id": run_id, "slug": slug }),
            )
            .unwrap();
            store.append(&m).unwrap();
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_run_emits_cancelled_and_returns_cancelling() {
        let port = FakePort::new();
        seed_run(port.ship_dir(), "run-c1", "doomed", false);

        let result = cancel_run_core(&port, "doomed", "").await.unwrap();
        assert_eq!(result.slug, "doomed");
        assert_eq!(result.pipeline_run_id, "run-c1");
        assert_eq!(result.status, "cancelling");

        let emitted = port.emitted.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, event_types::PIPELINE_RUN_CANCELLED);
        let payload: serde_json::Value = serde_json::from_str(&emitted[0].1).unwrap();
        assert_eq!(payload["slug"], "doomed");
        assert_eq!(payload["reason"], "operator-cancelled", "empty reason defaults");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_run_rejects_terminal_run() {
        let port = FakePort::new();
        seed_run(port.ship_dir(), "run-c2", "done", true); // merged → terminal
        let err = cancel_run_core(&port, "done", "stop").await.unwrap_err();
        assert!(matches!(err, StatusCoreError::BadRequest(_)));
        assert!(port.emitted.lock().unwrap().is_empty(), "terminal cancel emits nothing");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_run_not_found_for_unknown_slug() {
        let port = FakePort::new();
        let err = cancel_run_core(&port, "ghost", "").await.unwrap_err();
        assert!(matches!(err, StatusCoreError::NotFound(_)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resolve_step_emits_done_declared_and_advances() {
        let port = FakePort::new();
        seed_run(port.ship_dir(), "run-rs", "myrun", false);

        let result = resolve_step_core(&port, "myrun", "a", Some("looks good".into()))
            .await
            .unwrap();
        assert_eq!(result.slug, "myrun");
        assert_eq!(result.step_id, "a");
        assert!(result.advanced, "the step advanced");

        let emitted = port.emitted.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, event_types::PIPELINE_STEP_DONE_DECLARED);
        let payload: serde_json::Value = serde_json::from_str(&emitted[0].1).unwrap();
        assert_eq!(payload["step_id"], "a");
        assert_eq!(payload["summary"], "looks good");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn resolve_step_requires_step_id() {
        let port = FakePort::new();
        let err = resolve_step_core(&port, "myrun", "", None).await.unwrap_err();
        assert!(matches!(err, StatusCoreError::BadRequest(_)));
        assert!(port.emitted.lock().unwrap().is_empty());
    }
}
