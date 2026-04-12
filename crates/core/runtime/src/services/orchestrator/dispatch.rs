//! Initial job dispatch — prepares isolation, spawns the first agent step.
//!
//! Also handles pipelines whose first step is `run:` (skips agent spawning).

use crate::events::job::{JobCreatedPayload, StepType};
use crate::services::dispatch_ports::{JobContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::{emit_job_dispatched, emit_job_dispatched_with_meta};
use super::pipeline_loader::ensure_pipeline;
use super::step_monitor::StepMonitor;
use super::OrchestratorContext;

pub async fn handle_job_created(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &crate::events::EventEnvelope,
    octx: &OrchestratorContext,
) {
    let payload: JobCreatedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    if let Some(deps) = &payload.depends_on
        && !deps.is_empty()
    {
        tracing::info!(
            slug = payload.slug, deps = ?deps,
            "job-dispatch: deferring job until dependencies complete"
        );
        return;
    }

    let payload = ensure_pipeline(payload);
    dispatch_job_internal(kernel, &payload, octx).await;
}

pub async fn dispatch_job_internal(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    octx: &OrchestratorContext,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    // Check if the first step is a run step
    if let Some(ref steps) = payload.steps
        && let Some(first) = steps.root_steps().first()
        && first.step_type == StepType::Run
    {
        let command = match &first.command {
            Some(c) => c.clone(),
            None => {
                tracing::error!(slug, step_id = first.id, "job-dispatch: run step missing command");
                return;
            }
        };
        return dispatch_first_run_step(kernel, payload, octx, &first.id, &command).await;
    }

    let agent = if let Some(ref steps) = payload.steps {
        steps.root_steps().first()
            .and_then(|s| s.agent.as_deref())
            .unwrap_or(payload.agent.as_str())
    } else {
        payload.agent.as_str()
    };

    tracing::info!(slug, branch, agent, "job-dispatch: dispatching job");

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            provider: payload.provider.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            return;
        }
    };

    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
    let job_ctx_with_dir = JobContext {
        work_dir: worktree_path.clone(),
        exit_tx: Some(exit_tx),
        ..job_ctx
    };
    let handle = match octx.executor.spawn(&job_ctx_with_dir).await {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: executor spawn failed: {e}");
            return;
        }
    };

    octx.registry.register(slug, handle).await;

    let is_multi_step = payload.steps.as_ref().is_some_and(|s| s.steps.len() > 1);
    if is_multi_step {
        let step_name = payload.steps.as_ref()
            .and_then(|s| s.root_steps().first().map(|rs| rs.id.clone()))
            .unwrap_or_default();
        let monitor = StepMonitor::new(
            payload.job_id.clone(),
            0,
            slug.clone(),
            step_name,
            worktree_path.clone(),
        );
        let (activity_tx, activity_rx) = tokio::sync::mpsc::unbounded_channel();
        octx.done_flags.lock().await.insert(slug.clone(), monitor.done_flag());
        octx.activity_txs.lock().await.insert(slug.clone(), activity_tx);
        super::step_monitor::start_step_monitor(
            monitor,
            kernel.clone(),
            octx.registry.clone(),
            exit_rx,
            activity_rx,
        );
    } else {
        octx.platform.start_exit_handler(kernel.clone(), exit_rx);
    }

    emit_job_dispatched(kernel, &payload.job_id, &worktree_path, slug).await;
}

/// Test-friendly variant with explicit ship_dir for event store.
pub async fn dispatch_first_run_step_at(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    octx: &OrchestratorContext,
    step_id: &str,
    command: &str,
    ship_dir: &std::path::Path,
) {
    dispatch_first_run_step_inner(kernel, payload, octx, step_id, command, Some(ship_dir)).await;
}

async fn dispatch_first_run_step(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    octx: &OrchestratorContext,
    step_id: &str,
    command: &str,
) {
    dispatch_first_run_step_inner(kernel, payload, octx, step_id, command, None).await;
}

async fn dispatch_first_run_step_inner(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    octx: &OrchestratorContext,
    step_id: &str,
    command: &str,
    ship_dir: Option<&std::path::Path>,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    tracing::info!(slug, branch, step_id, %command, "job-dispatch: first step is run — executing directly");

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: payload.agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            provider: payload.provider.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            return;
        }
    };

    emit_job_dispatched_with_meta(kernel, &payload.job_id, &worktree_path, slug, None, ship_dir).await;

    super::run_step::execute_run_step(
        &payload.job_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
    ).await;
}
