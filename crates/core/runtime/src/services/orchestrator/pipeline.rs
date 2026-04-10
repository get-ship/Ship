//! Pipeline DAG — step-based advancement for job dispatch.
//!
//! A pipeline is a DAG of steps. Each step declares dependencies via `after`.
//! When a step completes, unblocked steps are found and dispatched.

use crate::events::job::StepType;
use crate::events::EventEnvelope;
use crate::projections::job::{load_jobs, load_jobs_at};
use crate::services::dispatch_ports::{JobContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::{emit_job_dispatched_with_meta, emit_step_dispatched_event};
use super::handlers::execute_merge_step;
use super::step_monitor::{StepMonitor, start_step_monitor};
use super::OrchestratorContext;

// Re-export for callers that import emit_job_dispatched from this module.
pub use super::emit::emit_job_dispatched;

/// Advance the pipeline after a step completes. Returns true if a next step was dispatched.
pub async fn advance_pipeline(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) -> bool {
    advance_pipeline_inner(kernel, envelope, octx, None, None).await
}

/// Test-friendly variant with explicit paths.
pub async fn advance_pipeline_at(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
    ship_dir: &std::path::Path,
) -> bool {
    let worktree_root = ship_dir.parent().map(|p| p.join("worktrees"));
    advance_pipeline_inner(kernel, envelope, octx, Some(ship_dir), worktree_root.as_deref()).await
}

async fn advance_pipeline_inner(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
    ship_dir_override: Option<&std::path::Path>,
    worktree_root_override: Option<&std::path::Path>,
) -> bool {
    let jobs = match ship_dir_override {
        Some(dir) => load_jobs_at(dir),
        None => load_jobs(),
    };
    let jobs = match jobs {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!(entity_id = %envelope.entity_id, "pipeline: failed to load jobs: {e}");
            return false;
        }
    };

    // Find the job
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
    let payload_slug = payload["slug"].as_str().unwrap_or("");
    let payload_job_id = payload["job_id"].as_str().unwrap_or("");
    let entity_id = &envelope.entity_id;

    let job = jobs.values()
        .find(|j| j.job_id == *entity_id)
        .or_else(|| jobs.values().find(|j| !payload_job_id.is_empty() && j.job_id == payload_job_id))
        .or_else(|| jobs.values().find(|j| !payload_slug.is_empty() && j.slug == payload_slug))
        .cloned();
    let job = match job {
        Some(j) => j,
        None => {
            tracing::warn!(entity_id = %entity_id, "pipeline: job not found in projection");
            return false;
        }
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
    let job_vars = job.vars.clone().unwrap_or_default();
    let mut dispatched_any = false;
    for (step_idx, step) in pipeline.steps.iter().enumerate().filter_map(|(i, s)| {
        next_steps.iter().find(|ns| ns.id == s.id).map(|ns| (i, *ns))
    }) {
        // Interpolate {{ var }} references before dispatch.
        let step = match super::pipeline_vars::interpolate_step(step, &job_vars, &Default::default()) {
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
                let job_id = job.job_id.clone();
                let slug_clone = slug.clone();
                let step_id = step.id.clone();
                let work_dir = worktree_path.clone();
                let ship_dir_owned = ship_dir_override.map(|p| p.to_path_buf());
                tokio::spawn(async move {
                    super::run_step::execute_run_step(
                        &job_id, &slug_clone, &step_id, &command,
                        &work_dir, &kernel_clone, ship_dir_owned.as_deref(),
                    ).await;
                });
                dispatched_any = true;
            }
            StepType::Agent | StepType::Gate => {
                let agent = step.agent.as_deref().unwrap_or("reviewer");

                if !worktree_path.exists() {
                    tracing::error!(slug, "pipeline: worktree missing");
                    continue;
                }

                octx.platform.graceful_agent_exit(slug, step.provider.as_deref()).await;

                tracing::info!(slug, agent, step_id = step.id, "pipeline: compiling agent config");
                if let Err(e) = octx.platform.compile_agent_config(&worktree_path, agent) {
                    tracing::warn!(slug, agent, "pipeline: ship use failed (non-fatal): {e}");
                }

                let persist = step.step_type != StepType::Gate;
                let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
                let job_ctx = JobContext {
                    job_id: job.job_id.clone(),
                    slug: slug.clone(),
                    agent: agent.to_string(),
                    branch: job.branch.clone(),
                    spec_path: job.spec_path.clone(),
                    work_dir: worktree_path.clone(),
                    env: [("SHIP_MESH_ID".to_string(), slug.clone())].into(),
                    exit_tx: Some(exit_tx),
                    launch: LaunchProfile {
                        provider: step.provider.clone(),
                        model: step.model.clone(),
                        persist_session: persist,
                        ..Default::default()
                    },
                };
                match octx.executor.spawn(&job_ctx).await {
                    Ok(handle) => {
                        octx.registry.register(slug, handle).await;
                        let monitor = StepMonitor::new(
                            job.job_id.clone(),
                            step_idx,
                            slug.clone(),
                            step.id.clone(),
                        );
                        octx.done_flags.lock().await.insert(
                            slug.clone(),
                            monitor.done_flag(),
                        );
                        start_step_monitor(
                            monitor,
                            kernel.clone(),
                            octx.registry.clone(),
                            exit_rx,
                        );
                    }
                    Err(e) => {
                        tracing::error!(slug, step_id = step.id, "pipeline: spawn failed: {e}");
                        continue;
                    }
                }

                emit_job_dispatched_with_meta(kernel, &job.job_id, &worktree_path, slug, None, ship_dir_override).await;
                emit_step_dispatched_event(kernel, &job.job_id, &step.id, "agent", slug, ship_dir_override).await;
                dispatched_any = true;
            }
            StepType::Merge => {
                tracing::info!(slug, branch = %job.branch, "pipeline: executing merge step");
                octx.platform.graceful_agent_exit(slug, None).await;
                if execute_merge_step(
                    kernel, &job.job_id, slug, &job.branch, &*octx.platform,
                ).await {
                    dispatched_any = true;
                }
            }
            StepType::Human => {
                tracing::info!(slug, step_id = step.id, "pipeline: human step — pausing for human action");
            }
        }
    }

    dispatched_any
}
