//! Pipeline DAG — step-based advancement for job dispatch.
//!
//! A pipeline is a DAG of steps. Each step declares dependencies via `after`.
//! When a step completes, unblocked steps are found and dispatched.

use ship_substrate::events::pipeline::StepType;
use ship_substrate::events::EventEnvelope;
use crate::projections::pipeline_run::load_pipeline_runs_at;
use crate::services::dispatch_ports::{SessionContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::{
    emit_job_dispatched_with_meta, emit_process_spawned, emit_step_dispatched_event,
    emit_step_failed,
};
use super::step_io::resolve_consumes;
use super::step_monitor::{StepMonitor, start_step_monitor};
use super::OrchestratorContext;

// Re-export for callers that import emit_job_dispatched from this module.
pub use super::emit::emit_job_dispatched;

/// Advance the pipeline after a step completes. Returns true if a next step was dispatched.
pub async fn advance_pipeline(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) -> bool {
    advance_pipeline_inner(kernel, envelope, octx, None, None).await
}

/// Test-friendly variant with explicit paths.
pub async fn advance_pipeline_at(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
    ship_dir: &std::path::Path,
) -> bool {
    let worktree_root = ship_dir.parent().map(|p| p.join("worktrees"));
    advance_pipeline_inner(kernel, envelope, octx, Some(ship_dir), worktree_root.as_deref()).await
}

async fn advance_pipeline_inner(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
    ship_dir_override: Option<&std::path::Path>,
    worktree_root_override: Option<&std::path::Path>,
) -> bool {
    // First pass: resolve the pipeline_run_id so we can acquire the per-job lock.
    // Lookups are index-backed: entity_id (PK), payload pipeline_run_id (PK),
    // payload slug (idx_jobs_slug). Test paths use load_pipeline_runs_at to honor
    // ship_dir_override; production paths use the O(1) single-row helpers.
    let pipeline_run_id = {
        let payload: serde_json::Value =
            serde_json::from_str(&envelope.payload_json).unwrap_or_default();
        let payload_slug = payload["slug"].as_str().unwrap_or("");
        let payload_job_id = payload["pipeline_run_id"].as_str().unwrap_or("");
        let entity_id = &envelope.entity_id;

        let record = match ship_dir_override {
            Some(dir) => match load_pipeline_runs_at(dir) {
                Ok(jobs) => jobs
                    .values()
                    .find(|j| j.pipeline_run_id == *entity_id)
                    .or_else(|| jobs.values().find(|j| !payload_job_id.is_empty() && j.pipeline_run_id == payload_job_id))
                    .or_else(|| jobs.values().find(|j| !payload_slug.is_empty() && j.slug == payload_slug))
                    .cloned(),
                Err(e) => {
                    tracing::warn!(entity_id = %envelope.entity_id, "pipeline: failed to load jobs: {e}");
                    return false;
                }
            },
            None => crate::projections::pipeline_run::load_pipeline_run_by_id(entity_id)
                .ok()
                .flatten()
                .or_else(|| {
                    (!payload_job_id.is_empty())
                        .then(|| crate::projections::pipeline_run::load_pipeline_run_by_id(payload_job_id).ok().flatten())
                        .flatten()
                })
                .or_else(|| {
                    (!payload_slug.is_empty())
                        .then(|| crate::projections::pipeline_run::load_pipeline_run_by_slug(payload_slug).ok().flatten())
                        .flatten()
                }),
        };
        match record {
            Some(j) => j.pipeline_run_id,
            None => {
                tracing::warn!(entity_id = %entity_id, "pipeline: job not found in projection");
                return false;
            }
        }
    };

    // Acquire per-job lock to prevent double-dispatch when concurrent events
    // (e.g. step.observed + job.completed) both try to advance the same job.
    let run_lock = {
        let mut locks = octx.run_locks.lock().await;
        locks.entry(pipeline_run_id.clone()).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))).clone()
    };
    let _guard = run_lock.lock().await;

    // Reload projection under lock so we see the latest state.
    let job = match ship_dir_override {
        Some(dir) => match load_pipeline_runs_at(dir) {
            Ok(jobs) => match jobs.get(&pipeline_run_id) {
                Some(j) => j.clone(),
                None => {
                    tracing::warn!(pipeline_run_id, "pipeline: job disappeared between lock acquire and reload");
                    return false;
                }
            },
            Err(e) => {
                tracing::warn!(entity_id = %envelope.entity_id, "pipeline: failed to reload jobs: {e}");
                return false;
            }
        },
        None => match crate::projections::pipeline_run::load_pipeline_run_by_id(&pipeline_run_id) {
            Ok(Some(j)) => j,
            Ok(None) => {
                tracing::warn!(pipeline_run_id, "pipeline: job disappeared between lock acquire and reload");
                return false;
            }
            Err(e) => {
                tracing::warn!(entity_id = %envelope.entity_id, "pipeline: failed to reload jobs: {e}");
                return false;
            }
        },
    };

    let slug = &job.slug;
    let pipeline = match &job.steps {
        Some(p) => p,
        None => {
            tracing::debug!(slug, "pipeline: no steps defined for job");
            return false;
        }
    };

    // Build the completed-steps set.
    let completed: Vec<String> = match &job.current_step {
        Some(id) => {
            if job.completed_steps.contains(id) {
                tracing::debug!(slug, step = %id, "pipeline: step already completed, skipping");
                return false;
            }
            tracing::info!(slug, step = %id, "pipeline: step completed");
            let mut c = job.completed_steps.clone();
            c.push(id.clone());
            c
        }
        None if !job.completed_steps.is_empty() => {
            tracing::info!(slug, completed_count = job.completed_steps.len(),
                "pipeline: advancing from projection-tracked completed_steps");
            job.completed_steps.clone()
        }
        None => {
            match pipeline.root_steps().first() {
                Some(root) => {
                    tracing::info!(slug, step = %root.id, "pipeline: first step completed");
                    vec![root.id.clone()]
                }
                None => {
                    tracing::warn!(slug, "pipeline: no root step found");
                    return false;
                }
            }
        }
    };

    let next_steps = pipeline.unblocked_steps(&completed);
    if next_steps.is_empty() {
        tracing::info!(slug, completed_count = completed.len(), "pipeline: all steps completed");
        return false;
    }

    let wt_root = worktree_root_override
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| octx.platform.worktrees_dir());
    let worktree_path = wt_root.join(slug);

    // Dispatch each unblocked step
    let run_vars = job.vars.clone().unwrap_or_default();
    let mut dispatched_any = false;
    for (step_idx, step) in pipeline.steps.iter().enumerate().filter_map(|(i, s)| {
        next_steps.iter().find(|ns| ns.id == s.id).map(|ns| (i, *ns))
    }) {
        // Interpolate {{ var }} references before dispatch.
        let step = match ship_pipelines::variables::interpolate_step(step, &run_vars, &Default::default()) {
            Ok(s) => s,
            Err(missing) => {
                tracing::error!(slug, step_id = step.id, "pipeline: unresolved pipeline var: {missing} in step {}", step.id);
                continue;
            }
        };
        let step = &step;
        tracing::info!(
            slug, step_id = step.id, step_type = ?step.step_type,
            agent = ?step.agent, "pipeline: dispatching next step"
        );

        // Fanout step: dispatch all lanes in parallel, aggregator handles completion.
        if let Some(fanout) = &step.fanout {
            let spawned = super::fanout::dispatch_fanout_step(
                kernel, &job, step, fanout, step_idx, ship_dir_override, octx,
            ).await;
            if spawned > 0 {
                emit_job_dispatched_with_meta(kernel, &job.pipeline_run_id, &worktree_path, slug, None, ship_dir_override).await;
                dispatched_any = true;
            }
            continue;
        }

        match step.step_type {
            StepType::Run => {
                let command = match step.command {
                    Some(ref c) => c.clone(),
                    None => {
                        tracing::error!(slug, step_id = step.id, "pipeline: run step missing command");
                        continue;
                    }
                };
                let kernel_clone = kernel.clone();
                let pipeline_run_id = job.pipeline_run_id.clone();
                let slug_clone = slug.clone();
                let step_id = step.id.clone();
                let work_dir = worktree_path.clone();
                let ship_dir_owned = ship_dir_override.map(|p| p.to_path_buf());
                tokio::spawn(async move {
                    super::run_step::execute_run_step(
                        &pipeline_run_id, &slug_clone, &step_id, &command,
                        &work_dir, &kernel_clone, ship_dir_owned.as_deref(),
                    ).await;
                });
                dispatched_any = true;
            }
            StepType::Agent | StepType::Review => {
                // Load-time validation guarantees an agent here; defensively skip
                // on the impossible path rather than substituting a role name.
                let Some(agent) = step.agent.as_deref().filter(|a| !a.is_empty()) else {
                    tracing::error!(slug, step_id = step.id, "pipeline: step missing agent");
                    continue;
                };

                if !worktree_path.exists() {
                    tracing::error!(slug, "pipeline: worktree missing");
                    continue;
                }

                // Resolve consumes before dispatch — fail fast if inputs are missing.
                let resolved_inputs = if let Some(pipeline) = &job.steps {
                    match resolve_consumes(step, pipeline, &job.pipeline_run_id, &worktree_path) {
                        Ok(inputs) => inputs,
                        Err(e) => {
                            tracing::error!(
                                slug, step_id = step.id,
                                "pipeline: consumes resolution failed: {e}"
                            );
                            emit_step_failed(
                                kernel, &job.pipeline_run_id, slug, &step.id, &e,
                            ).await;
                            continue;
                        }
                    }
                } else {
                    vec![]
                };

                octx.platform.graceful_agent_exit(slug, step.harness.as_deref()).await;

                tracing::info!(slug, agent, step_id = step.id, "pipeline: compiling agent config");
                if let Err(e) = octx.platform.compile_agent_config(&worktree_path, agent) {
                    tracing::warn!(slug, agent, "pipeline: ship use failed (non-fatal): {e}");
                }

                if let Some(pipeline) = &job.steps {
                    super::pipeline_specs::prepare_step_for_dispatch(
                        &worktree_path, slug, &job.pipeline_run_id,
                        &job.spec_path, pipeline, step, &resolved_inputs,
                    );
                }

                let persist = step.step_type != StepType::Review;
                let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
                let session_ctx = SessionContext {
                    pipeline_run_id: job.pipeline_run_id.clone(),
                    slug: slug.clone(),
                    agent: agent.to_string(),
                    branch: job.branch.clone(),
                    spec_path: job.spec_path.clone(),
                    spec_content: None,
                    work_dir: worktree_path.clone(),
                    env: [("SHIP_ACTOR_ID".to_string(), slug.clone())].into(),
                    exit_tx: Some(exit_tx),
                    launch: LaunchProfile {
                        harness: step.harness.clone(),
                        model: step.model.clone(),
                        persist_session: persist,
                        ..Default::default()
                    },
                    step: Some(crate::services::dispatch_ports::StepInfo { index: step_idx, id: step.id.clone() }),
                    invocation: super::dispatch::resolve_step_invocation(
                        step.invocation.as_deref(),
                        job.headless,
                    ),
                    // Pipeline steps are always per-turn, never keepalive.
                    keepalive: false,
                };
                match octx.executor.spawn(&session_ctx).await {
                    Ok(handle) => {
                        let exec_kind = if job.headless { "process" } else { "pty" };
                        let cmd_label = step.harness.as_deref().unwrap_or("claude");
                        let pid = handle.pid;
                        emit_process_spawned(
                            kernel, slug, &job.pipeline_run_id, pid,
                            cmd_label, &[], &[],
                            &worktree_path.to_string_lossy(),
                            exec_kind, Some(&step.id),
                        ).await;

                        // Every dispatched step also surfaces as an
                        // agent_session row so bare and pipeline-composed
                        // dispatches share one read model. See
                        // dispatch::emit_composed_step_session.
                        super::dispatch::emit_composed_step_session(
                            kernel,
                            &super::dispatch::ComposedRunCtx {
                                pipeline_run_id: &job.pipeline_run_id,
                                run_slug: &job.slug,
                                branch: &job.branch,
                                source_spec_path: job.source_spec_path.as_deref(),
                                vars: job.vars.as_ref(),
                                baseline_sha: job.baseline_sha.as_deref(),
                                run_headless: job.headless,
                            },
                            step, agent,
                            &worktree_path.to_string_lossy(), pid,
                        ).await;

                        octx.registry.register(slug, handle).await;
                        let monitor = StepMonitor::new(
                            job.pipeline_run_id.clone(),
                            step_idx,
                            slug.clone(),
                            step.id.clone(),
                            step.step_type.clone(),
                            worktree_path.clone(),
                        );
                        let (activity_tx, activity_rx) = tokio::sync::mpsc::unbounded_channel();
                        octx.done_flags.lock().await.insert(
                            slug.clone(),
                            monitor.done_flag(),
                        );
                        octx.activity_txs.lock().await.insert(slug.clone(), activity_tx);
                        start_step_monitor(
                            monitor,
                            kernel.clone(),
                            octx.registry.clone(),
                            octx.executor.clone(),
                            octx.probe.clone(),
                            exit_rx,
                            activity_rx,
                        );
                    }
                    Err(e) => {
                        tracing::error!(slug, step_id = step.id, "pipeline: spawn failed: {e}");
                        continue;
                    }
                }

                emit_job_dispatched_with_meta(kernel, &job.pipeline_run_id, &worktree_path, slug, None, ship_dir_override).await;
                emit_step_dispatched_event(kernel, &job.pipeline_run_id, &step.id, "agent", slug, ship_dir_override).await;
                dispatched_any = true;
            }
        }
    }

    dispatched_any
}
