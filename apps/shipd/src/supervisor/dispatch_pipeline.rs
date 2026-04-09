//! Pipeline DAG — step-based advancement for job dispatch.
//!
//! A pipeline is a DAG of steps. Each step declares dependencies via `after`.
//! When a step completes, unblocked steps are found and dispatched.
use runtime::events::job::StepType;
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::services::dispatch_ports::JobContext;
use runtime::projections::job::{load_jobs, load_jobs_at};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::agent_lifecycle::graceful_agent_exit;
use super::dispatch_emit::{emit_job_dispatched_with_meta, emit_step_dispatched_event};
use super::executors::run_step::execute_run_step;
use super::helpers::{compile_agent_config, worktrees_dir};
use super::job_dispatch::DispatchContext;
use super::liveness;

// Re-export for callers that import emit_job_dispatched from this module.
pub(crate) use super::dispatch_emit::emit_job_dispatched;

/// Advance the pipeline after a step completes. Returns true if a next step was dispatched.
pub(crate) async fn advance_pipeline(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
) -> bool {
    advance_pipeline_inner(kernel, envelope, dctx, None, None).await
}

/// Test-friendly variant with explicit paths.
#[cfg(test)]
pub(crate) async fn advance_pipeline_at(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
    ship_dir: &std::path::Path,
) -> bool {
    let worktree_root = ship_dir.parent().map(|p| p.join("worktrees"));
    advance_pipeline_inner(kernel, envelope, dctx, Some(ship_dir), worktree_root.as_deref()).await
}
async fn advance_pipeline_inner(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
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

    // Look up by job_id first (exact), then fall back to slug (ambiguous).
    // entity_id and payload job_id are authoritative; slug is a last resort.
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

    // Build the completed-steps set to use for finding the next step.
    //
    // Three cases:
    // (a) current_step is Some — idempotency check, then add to completed.
    // (b) current_step is None and completed_steps is non-empty — the projection
    //     already moved the finished step into completed_steps on JOB_COMPLETED.
    //     Use it as-is; no idempotency check needed.
    // (c) current_step is None and completed_steps is empty — first step trigger;
    //     treat the root step as just completed.
    let completed: Vec<String> = match &job.current_step {
        Some(id) => {
            // Idempotency: duplicate completion event for a step already recorded.
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
            // JOB_COMPLETED already moved current_step into completed_steps.
            // Advance from the recorded set without re-checking idempotency.
            tracing::info!(slug, completed_count = job.completed_steps.len(),
                "pipeline: advancing from projection-tracked completed_steps");
            job.completed_steps.clone()
        }
        None => {
            // No completed steps recorded — treat the root step as just finished.
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

    let wt_root = worktree_root_override.map(|p| p.to_path_buf()).unwrap_or_else(worktrees_dir);
    let worktree_path = wt_root.join(slug);

    // Dispatch each unblocked step
    let job_vars = job.vars.clone().unwrap_or_default();
    let mut dispatched_any = false;
    for step in next_steps {
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
            slug,
            step_id = step.id,
            step_type = ?step.step_type,
            agent = ?step.agent,
            "pipeline: dispatching next step"
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
                    execute_run_step(
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

                // Gracefully end the current agent (only needed for tmux reuse)
                graceful_agent_exit(slug, step.provider.as_deref()).await;

                // Swap agent profile
                tracing::info!(slug, agent, step_id = step.id, "pipeline: compiling agent config");
                if let Err(e) = compile_agent_config(&worktree_path, agent) {
                    tracing::warn!(slug, agent, "pipeline: ship use failed (non-fatal): {e}");
                }

                // Launch the agent; gate steps are ephemeral.
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
                    launch: runtime::services::dispatch_ports::LaunchProfile {
                        provider: step.provider.clone(),
                        model: step.model.clone(),
                        persist_session: persist,
                        ..Default::default()
                    },
                };
                match dctx.executor.spawn(&job_ctx).await {
                    Ok(handle) => {
                        dctx.registry.lock().await.register(slug, handle);
                        liveness::start_exit_handler(kernel.clone(), dctx.registry.clone(), exit_rx);
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

                // Gracefully end any running agent first
                graceful_agent_exit(slug, None).await;

                // Git merge the job branch into the current branch (v0.2.0 or whatever HEAD is)
                let merge_result = std::process::Command::new("git")
                    .args(["merge", &job.branch, "--no-edit"])
                    .current_dir(super::helpers::repo_dir())
                    .output();

                match merge_result {
                    Ok(output) if output.status.success() => {
                        tracing::info!(slug, branch = %job.branch, "pipeline: merge successful");

                        // Emit job.merged to trigger cleanup
                        let merged_payload = runtime::events::job::JobMergedPayload {
                            job_id: job.job_id.clone(),
                            slug: slug.clone(),
                        };
                        let merged_envelope = EventEnvelope::new(
                            runtime::events::job::event_types::JOB_MERGED,
                            &job.job_id,
                            &merged_payload,
                        );
                        if let Ok(envelope) = merged_envelope {
                            if let Ok(store) = runtime::events::SqliteEventStore::new() {
                                let _ = runtime::events::EventStore::append(&store, &envelope);
                            }
                            let ctx = EmitContext {
                                caller_kind: CallerKind::Runtime,
                                skill_id: None,
                                workspace_id: None,
                                session_id: None,
                            };
                            let _ = kernel.lock().await.route(envelope, &ctx).await;
                        }
                        dispatched_any = true;
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::error!(slug, branch = %job.branch, "pipeline: merge failed: {stderr}");
                    }
                    Err(e) => {
                        tracing::error!(slug, "pipeline: git merge command failed: {e}");
                    }
                }
            }
            StepType::Human => {
                tracing::info!(slug, step_id = step.id, "pipeline: human step — pausing for human action");
                // TODO: create inbox item. For now, the job stays in completed status.
            }
        }
    }

    dispatched_any
}

#[cfg(test)]
#[path = "tests_step_tracking.rs"]
mod tests_step_tracking;

#[cfg(test)]
#[path = "tests_pipeline_reliability.rs"]
mod tests_pipeline_reliability;
