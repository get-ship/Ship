//! Periodic coherence sweep — detects and resolves drift between job
//! projections and live processes every 30 seconds.

use ship_substrate::events::pipeline::{event_types as job_events, PipelineRunFailedPayload, StepType};
use ship_substrate::events::process::{event_types as proc_events, ProcessKilledPayload};
use ship_substrate::events::supervisor::{
    event_types as sup_events, CoherenceRestoredPayload, DriftDetectedPayload,
    SweepCompletedPayload, SweepStartedPayload,
};
use ship_substrate::events::EventEnvelope;
use crate::projections::agent_session::{load_sessions, AgentSession};
use crate::projections::pipeline_run::{load_pipeline_runs, PipelineRun, PipelineRunStatus};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::emit::persist_and_route;
use super::OrchestratorContext;

/// Is this slug owned by ANY known intentional dispatch? Source of
/// truth combines two projections:
///   - `jobs` (pipeline composition runs)
///   - `agent_sessions` (the dispatch primitive — bare ship use,
///     ship dispatch, MCP dispatch_agent, future pipeline steps)
///
/// True orphan processes — registered with no record in either
/// projection — are the only legitimate stray-kill targets. Phase 6
/// of the agent-primitive inversion: previously this check only
/// looked at jobs.status=Dispatched|GatePending, which false-flagged
/// every bare agent session that travels through agent_sessions.
fn is_known_dispatch(
    jobs: &HashMap<String, PipelineRun>,
    sessions: &HashMap<String, AgentSession>,
    slug: &str,
) -> bool {
    jobs.values().any(|j| j.slug == slug)
        || sessions.values().any(|s| s.slug == slug)
}

/// Spawn the coherence sweep loop — runs every 30s, first tick immediate.
pub async fn start_coherence_sweep(
    kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: Arc<OrchestratorContext>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = run_sweep(&kernel, &octx).await {
            tracing::error!("coherence sweep failed: {e}");
        }
    }
}

/// Execute a single coherence sweep pass.
pub async fn run_sweep(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    let jobs = load_pipeline_runs()?;
    // The agent_sessions projection is the dispatch-primitive read
    // model. Sweep needs to consult it to avoid false-positive
    // stray-process kills for bare ship use / dispatch flows. See
    // is_known_dispatch above for the rule.
    let sessions = load_sessions().unwrap_or_default();

    // Evict dead handles from the registry before reading live_slugs. The
    // registry tracks "what was registered"; this drains anything the
    // executor reports as no-longer-alive so the drift checks below see
    // truth. Each drained slug becomes a job.failed below — that was
    // previously the job of `liveness::sweep_registered` in shipd, folded
    // here so a single sweep owns the coherence story.
    let dead_drained = octx.registry.drain_dead(&*octx.executor).await;
    for slug in &dead_drained {
        if let Some(job) = jobs.values().find(|j| &j.slug == slug) {
            let fail_payload = PipelineRunFailedPayload {
                pipeline_run_id: job.pipeline_run_id.clone(),
                error: "coherence sweep: process died without completing".to_string(),
            };
            if let Ok(env) =
                EventEnvelope::new(job_events::PIPELINE_RUN_FAILED, &job.pipeline_run_id, &fail_payload)
            {
                let env = env.with_actor_id("service.coherence-sweep");
                persist_and_route(kernel, env, &job.slug, None).await;
            }
        } else {
            tracing::warn!(
                slug,
                "coherence sweep: drained dead handle has no matching job"
            );
        }
    }

    let live_slugs = octx.registry.live_slugs().await;

    // Emit sweep_started
    let in_scope: Vec<_> = jobs
        .values()
        .filter(|j| matches!(j.status, PipelineRunStatus::Dispatched | PipelineRunStatus::GatePending))
        .collect();

    emit_event(
        kernel,
        sup_events::SUPERVISOR_SWEEP_STARTED,
        "supervisor",
        &SweepStartedPayload {
            jobs_in_scope: in_scope.len(),
            processes_known: live_slugs.len(),
        },
    )
    .await;

    let mut drift_count: usize = 0;
    let mut actions_taken: usize = 0;

    // Check A: Dispatched/GatePending jobs with no live process
    for job in &in_scope {
        // Run steps run synchronously inside `execute_run_step` and never
        // register a handle; they self-emit step.completed / step.failed.
        // The live-process check would false-flag any Run step that outlives
        // the 30s sweep interval (e.g. `cargo test`).
        if current_step_is_run(job) {
            continue;
        }

        // Registry is the authoritative "is this process alive?" signal.
        // resume_handle is TmuxWindow-only — headless ProcessExecutor and
        // ServerExecutor return None unconditionally (routing.rs), so
        // relying on it alone would drift-fail every live headless agent
        // within one sweep tick.
        if live_slugs.iter().any(|s| s == &job.slug) {
            continue;
        }

        let handle = octx
            .executor
            .resume_handle(
                &job.slug,
                job.harness.as_deref(),
                job.current_step.as_deref(),
            )
            .await;

        if handle.is_some() {
            continue;
        }

        // Process missing — drift detected
        drift_count += 1;
        let status_str = format!("{:?}", job.status);

        emit_event(
            kernel,
            sup_events::SUPERVISOR_DRIFT_DETECTED,
            &job.slug,
            &DriftDetectedPayload {
                slug: job.slug.clone(),
                pipeline_run_id: job.pipeline_run_id.clone(),
                expected_status: status_str,
                actual_state: "no_process".to_string(),
                detail: "process missing for dispatched job".to_string(),
            },
        )
        .await;

        // Emit job.failed
        let fail_payload = PipelineRunFailedPayload {
            pipeline_run_id: job.pipeline_run_id.clone(),
            error: "coherence sweep: process missing for dispatched job".to_string(),
        };
        if let Ok(env) = EventEnvelope::new(job_events::PIPELINE_RUN_FAILED, &job.pipeline_run_id, &fail_payload) {
            let env = env.with_actor_id("service.coherence-sweep");
            persist_and_route(kernel, env, &job.slug, None).await;
        }

        emit_event(
            kernel,
            sup_events::SUPERVISOR_COHERENCE_RESTORED,
            &job.slug,
            &CoherenceRestoredPayload {
                slug: job.slug.clone(),
                action: "marked_failed".to_string(),
                detail: "emitted job.failed for process-missing drift".to_string(),
            },
        )
        .await;

        actions_taken += 1;
    }

    // Check B: ORPHAN processes — registered but unknown to any
    // dispatch record (neither jobs nor agent_sessions). Sweep is the
    // safety net for true orphans (stale handles from a daemon
    // restart, registry entries from spawn paths that didn't emit a
    // dispatch event). It is NOT for cleaning up processes whose
    // session/job has completed/failed — those are owned by the
    // exit-handler / step-monitor and will be reaped when the harness
    // actually exits.
    //
    // The previous check (`Dispatched | GatePending` only) reaped
    // legitimate processes whose status had transitioned (Completed,
    // Failed, etc.) but whose process was still cleaning up — the
    // ship-use-auto false-positive bug.
    for slug in &live_slugs {
        if is_known_dispatch(&jobs, &sessions, slug) {
            // Lifecycle owns this process. Don't interfere.
            continue;
        }

        drift_count += 1;

        // Best-effort pipeline_run_id for the drift payload; empty when no
        // record (true orphan).
        let pipeline_run_id = jobs
            .values()
            .find(|j| j.slug == *slug)
            .map(|j| j.pipeline_run_id.clone())
            .unwrap_or_default();

        emit_event(
            kernel,
            sup_events::SUPERVISOR_DRIFT_DETECTED,
            slug,
            &DriftDetectedPayload {
                slug: slug.clone(),
                pipeline_run_id: pipeline_run_id.clone(),
                expected_status: "none_or_terminal".to_string(),
                actual_state: "stray_process".to_string(),
                detail: "process registered but no dispatch record \
                         (orphan: not in jobs or agent_sessions)"
                    .to_string(),
            },
        )
        .await;

        // Kill via executor — take handle from registry first
        let killed_pid = if let Some(handle) = octx.registry.take(slug).await {
            let pid = handle.pid;
            let _ = octx.executor.stop(&handle).await;
            pid
        } else {
            None
        };

        // Emit process.killed — the audit record for the kill itself.
        // Distinct from supervisor.coherence_restored (which records the
        // sweep's action) because process.killed belongs to the process.*
        // lifecycle stream that operators query for "what happened to this
        // process."
        emit_event(
            kernel,
            proc_events::PROCESS_KILLED,
            slug,
            &ProcessKilledPayload {
                slug: slug.clone(),
                pid: killed_pid,
                signal: "SIGTERM+SIGKILL".to_string(),
                reason: "stray_process_sweep".to_string(),
                actor: "supervisor".to_string(),
            },
        )
        .await;

        emit_event(
            kernel,
            sup_events::SUPERVISOR_COHERENCE_RESTORED,
            slug,
            &CoherenceRestoredPayload {
                slug: slug.clone(),
                action: "process_killed".to_string(),
                detail: "killed stray process with no active job".to_string(),
            },
        )
        .await;

        actions_taken += 1;
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    emit_event(
        kernel,
        sup_events::SUPERVISOR_SWEEP_COMPLETED,
        "supervisor",
        &SweepCompletedPayload {
            duration_ms,
            drift_count,
            actions_taken,
        },
    )
    .await;

    if drift_count > 0 {
        tracing::info!(
            drift_count,
            actions_taken,
            duration_ms,
            "coherence sweep resolved drift"
        );
    }

    Ok(())
}

/// True iff the job's current step has `StepType::Run`. Returns false for
/// jobs with no DAG pipeline, no `current_step`, or a non-Run step type — so
/// only the Run-step path short-circuits the sweep.
fn current_step_is_run(job: &PipelineRun) -> bool {
    let Some(pipeline) = job.steps.as_ref() else {
        return false;
    };
    let Some(current_id) = job.current_step.as_deref() else {
        return false;
    };
    pipeline
        .steps
        .iter()
        .find(|s| s.id == current_id)
        .map(|s| matches!(s.step_type, StepType::Run))
        .unwrap_or(false)
}

async fn emit_event(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    event_type: &str,
    entity_id: &str,
    payload: &impl serde::Serialize,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, payload) {
        Ok(e) => e.with_actor_id("service.coherence-sweep"),
        Err(e) => {
            tracing::warn!(event_type, "sweep: failed to build envelope: {e}");
            return;
        }
    };
    persist_and_route(kernel, envelope, entity_id, None).await;
}
