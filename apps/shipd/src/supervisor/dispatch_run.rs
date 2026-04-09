//! First-run-step dispatch — handles pipelines whose first step is `run:`.
//!
//! When a pipeline starts with a run step, we skip agent spawning entirely.
//! Instead we prepare isolation, emit `job.dispatched`, and execute the shell
//! command directly. The run step executor emits `step_completed`/`step_failed`,
//! which triggers `advance_pipeline` for subsequent steps.

use runtime::events::job::JobCreatedPayload;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch_emit::emit_job_dispatched_with_meta;
use super::executors::run_step::execute_run_step;
use super::helpers::worktrees_dir;
use super::job_dispatch::DispatchContext;

pub(crate) async fn dispatch_first_run_step(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    dctx: &DispatchContext,
    step_id: &str,
    command: &str,
) {
    dispatch_first_run_step_inner(kernel, payload, dctx, step_id, command, None).await;
}

#[cfg(test)]
pub(crate) async fn dispatch_first_run_step_at(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    dctx: &DispatchContext,
    step_id: &str,
    command: &str,
    ship_dir: &std::path::Path,
) {
    dispatch_first_run_step_inner(kernel, payload, dctx, step_id, command, Some(ship_dir)).await;
}

async fn dispatch_first_run_step_inner(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    dctx: &DispatchContext,
    step_id: &str,
    command: &str,
    ship_dir: Option<&std::path::Path>,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    tracing::info!(slug, branch, step_id, %command, "job-dispatch: first step is run — executing directly");

    let job_ctx = runtime::services::dispatch_ports::JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: payload.agent.to_string(),
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

    // Prepare isolation (worktree)
    let worktree_path = match dctx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            return;
        }
    };

    // Emit job.dispatched
    emit_job_dispatched_with_meta(kernel, &payload.job_id, &worktree_path, slug, None, ship_dir).await;

    // Execute the run step — emits step_dispatched + step_completed/failed,
    // which triggers advance_pipeline for subsequent steps.
    execute_run_step(
        &payload.job_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
    ).await;
}
