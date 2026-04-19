//! Initial job dispatch — prepares isolation, spawns the first agent step.
//!
//! Also handles pipelines whose first step is `run:` (skips agent spawning).

use crate::events::job::{JobCreatedPayload, StepType};
use crate::services::dispatch_ports::{JobContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::{emit_job_dispatched, emit_job_dispatched_with_meta, emit_job_failed, emit_process_spawned, emit_spawn_failed};
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

    let slug_for_err = payload.slug.clone();
    let payload = match ensure_pipeline(payload) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug = slug_for_err, "pipeline: load failed: {e}");
            return;
        }
    };
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
                emit_job_failed(kernel, &payload.job_id, slug, "run step missing command").await;
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

    let first_step_info = payload.steps.as_ref().and_then(|p| {
        p.root_steps().first().map(|s| crate::services::dispatch_ports::StepInfo {
            index: 0,
            id: s.id.clone(),
        })
    });

    // Resolve provider/model from the first step when present — matches how
    // `advance_pipeline` resolves for steps 2+, so first-step and
    // later-step dispatch agree on the source of truth. Fall through to
    // the job-level payload fields when the step leaves them unset.
    let first_step = payload.steps.as_ref()
        .and_then(|p| p.root_steps().first().cloned());
    let launch_provider = first_step.as_ref()
        .and_then(|s| s.provider.clone())
        .or_else(|| payload.provider.clone());
    let launch_model = first_step.as_ref()
        .and_then(|s| s.model.clone())
        .or_else(|| payload.model.clone());

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            provider: launch_provider,
            model: launch_model,
            persist_session: true,
            ..Default::default()
        },
        step: first_step_info,
        headless: payload.headless,
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.job_id, slug, "isolation_prepare", &e).await;
            return;
        }
    };

    // Materialize `.artifacts/pipeline/` once at launch — overall spec plus
    // one pre-written body per step — then drop a dispatch prompt at
    // `spec_path` pointing the first agent at its own step file.
    if let Some(pipeline) = &payload.steps
        && let Some(root) = pipeline.root_steps().first()
    {
        let body = payload.spec_content.clone().unwrap_or_default();
        if let Err(e) = super::pipeline_specs::write_pipeline_specs(
            &worktree_path,
            slug,
            pipeline,
            &body,
            payload.step_specs.as_deref(),
        ) {
            tracing::error!(slug, "job-dispatch: failed to write pipeline specs: {e}");
        }
        match super::pipeline_specs::write_dispatch_prompt(
            &worktree_path,
            &payload.spec_path,
            payload.pipeline_name.as_deref(),
            pipeline,
            root,
        ) {
            Ok(path) => tracing::info!(
                slug, step_id = root.id,
                "job-dispatch: wrote dispatch prompt to {}", path.display()
            ),
            Err(e) => tracing::error!(
                slug, "job-dispatch: failed to write dispatch prompt: {e}"
            ),
        }
    }

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
            emit_spawn_failed(kernel, &payload.job_id, slug, "executor_spawn", &e).await;
            return;
        }
    };

    let executor_kind = if payload.headless { "process" } else { "pty" };
    emit_process_spawned(
        kernel, slug, &payload.job_id, handle.pid,
        "claude", &[], &[], &worktree_path.to_string_lossy(),
        executor_kind, job_ctx_with_dir.step.as_ref().map(|s| s.id.as_str()),
    ).await;

    octx.registry.register(slug, handle).await;

    let is_multi_step = payload.steps.as_ref().is_some_and(|s| s.steps.len() > 1);
    if is_multi_step {
        let root = payload.steps.as_ref().and_then(|s| s.root_steps().first().cloned().cloned());
        let (step_name, step_type) = root
            .map(|r| (r.id, r.step_type))
            .unwrap_or_else(|| (String::new(), crate::events::job::StepType::Agent));
        let monitor = StepMonitor::new(
            payload.job_id.clone(),
            0,
            slug.clone(),
            step_name,
            step_type,
            worktree_path.clone(),
        );
        let (activity_tx, activity_rx) = tokio::sync::mpsc::unbounded_channel();
        octx.done_flags.lock().await.insert(slug.clone(), monitor.done_flag());
        octx.activity_txs.lock().await.insert(slug.clone(), activity_tx);
        super::step_monitor::start_step_monitor(
            monitor,
            kernel.clone(),
            octx.registry.clone(),
            octx.executor.clone(),
            octx.probe.clone(),
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
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            provider: payload.provider.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
        step: Some(crate::services::dispatch_ports::StepInfo {
            index: 0,
            id: step_id.to_string(),
        }),
        headless: payload.headless,
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.job_id, slug, "isolation_prepare", &e).await;
            return;
        }
    };

    emit_job_dispatched_with_meta(kernel, &payload.job_id, &worktree_path, slug, None, ship_dir).await;

    super::run_step::execute_run_step(
        &payload.job_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
    ).await;
}
