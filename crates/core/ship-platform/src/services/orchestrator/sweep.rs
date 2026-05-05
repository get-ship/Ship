//! Periodic coherence sweep — detects and resolves drift between job
//! projections and live processes every 30 seconds.

use ship_substrate::events::job::{event_types as job_events, JobFailedPayload, StepType};
use ship_substrate::events::process::{event_types as proc_events, ProcessKilledPayload};
use ship_substrate::events::supervisor::{
    event_types as sup_events, CoherenceRestoredPayload, DriftDetectedPayload,
    SweepCompletedPayload, SweepStartedPayload,
};
use ship_substrate::events::EventEnvelope;
use crate::projections::job::{load_jobs, JobRecord, JobStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::emit::persist_and_route;
use super::OrchestratorContext;

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

    let jobs = load_jobs()?;
    let live_slugs = octx.registry.live_slugs().await;

    // Emit sweep_started
    let in_scope: Vec<_> = jobs
        .values()
        .filter(|j| matches!(j.status, JobStatus::Dispatched | JobStatus::GatePending))
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
        // resume_handle is tmux-only today (headless ProcessExecutor returns
        // None unconditionally — routing.rs), so relying on it alone drift-
        // fails every live headless agent within one sweep tick.
        if live_slugs.iter().any(|s| s == &job.slug) {
            continue;
        }

        let handle = octx
            .executor
            .resume_handle(
                &job.slug,
                job.provider.as_deref(),
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
                job_id: job.job_id.clone(),
                expected_status: status_str,
                actual_state: "no_process".to_string(),
                detail: "process missing for dispatched job".to_string(),
            },
        )
        .await;

        // Emit job.failed
        let fail_payload = JobFailedPayload {
            job_id: job.job_id.clone(),
            error: "coherence sweep: process missing for dispatched job".to_string(),
        };
        if let Ok(env) = EventEnvelope::new(job_events::JOB_FAILED, &job.job_id, &fail_payload) {
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

    // Check B: Stray processes in registry with no matching active job
    for slug in &live_slugs {
        let has_active_job = jobs.values().any(|j| {
            j.slug == *slug
                && matches!(j.status, JobStatus::Dispatched | JobStatus::GatePending)
        });
        if has_active_job {
            continue;
        }

        drift_count += 1;

        let job_id = jobs
            .values()
            .find(|j| j.slug == *slug)
            .map(|j| j.job_id.clone())
            .unwrap_or_default();

        emit_event(
            kernel,
            sup_events::SUPERVISOR_DRIFT_DETECTED,
            slug,
            &DriftDetectedPayload {
                slug: slug.clone(),
                job_id: job_id.clone(),
                expected_status: "none_or_terminal".to_string(),
                actual_state: "stray_process".to_string(),
                detail: "process registered but no active job".to_string(),
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
fn current_step_is_run(job: &JobRecord) -> bool {
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
