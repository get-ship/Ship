//! Initial job dispatch — prepares isolation, spawns the first agent step.

use runtime::events::job::{JobCreatedPayload, StepType};
use runtime::services::dispatch_ports::JobContext;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch_pipeline::emit_job_dispatched;
use super::dispatch_run::dispatch_first_run_step;
use super::helpers::worktrees_dir;
use super::job_dispatch::DispatchContext;
use super::pipeline_loader::ensure_pipeline;

pub(crate) async fn handle_job_created(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &runtime::events::EventEnvelope,
    dctx: &DispatchContext,
) {
    let payload: JobCreatedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    if let Some(deps) = &payload.depends_on {
        if !deps.is_empty() {
            tracing::info!(
                slug = payload.slug,
                deps = ?deps,
                "job-dispatch: deferring job until dependencies complete (projection-backed)"
            );
            return;
        }
    }

    let payload = ensure_pipeline(payload);
    dispatch_job_internal(kernel, &payload, dctx).await;
}

pub(crate) async fn dispatch_job_internal(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    dctx: &DispatchContext,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    // Check if the first step is a run step
    if let Some(ref steps) = payload.steps {
        if let Some(first) = steps.root_steps().first() {
            if first.step_type == StepType::Run {
                let command = match &first.command {
                    Some(c) => c.clone(),
                    None => {
                        tracing::error!(slug, step_id = first.id, "job-dispatch: run step missing command");
                        return;
                    }
                };
                return dispatch_first_run_step(kernel, payload, dctx, &first.id, &command).await;
            }
        }
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
        work_dir: worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: runtime::services::dispatch_ports::LaunchProfile {
            provider: payload.provider.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
    };

    let worktree_path = match dctx.isolation.prepare(&job_ctx).await {
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
    let handle = match dctx.executor.spawn(&job_ctx_with_dir).await {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: executor spawn failed: {e}");
            return;
        }
    };

    dctx.registry.lock().await.register(slug, handle);

    let is_multi_step = payload.steps.as_ref().is_some_and(|s| s.steps.len() > 1);
    if is_multi_step {
        let step_name = payload.steps.as_ref()
            .and_then(|s| s.root_steps().first().map(|rs| rs.id.clone()))
            .unwrap_or_default();
        let monitor = super::step_monitor::StepMonitor::new(
            payload.job_id.clone(),
            0,
            slug.clone(),
            step_name,
        );
        dctx.done_flags.lock().await.insert(slug.clone(), monitor.done_flag());
        super::step_monitor::start_step_monitor(
            monitor,
            kernel.clone(),
            dctx.registry.clone(),
            exit_rx,
        );
    } else {
        super::liveness::start_exit_handler(kernel.clone(), dctx.registry.clone(), exit_rx);
    }

    emit_job_dispatched(kernel, &payload.job_id, &worktree_path, slug).await;
}
