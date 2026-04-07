//! Pipeline DAG — step-based advancement for job dispatch.
//!
//! A pipeline is a DAG of steps. Each step has a type (agent, gate, merge, human)
//! and declares dependencies via `after`. The engine walks the DAG: when a step
//! completes, it finds unblocked steps and dispatches them.
use runtime::events::job::StepType;
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::services::dispatch_ports::{JobContext, JobExecutor};
use runtime::projections::job::{load_jobs, load_jobs_at};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::agent_lifecycle::graceful_agent_exit;
use super::helpers::{compile_agent_config, worktrees_dir};
use super::job_dispatch::DispatchContext;

/// Emit a `job.dispatched` event.
pub(crate) async fn emit_job_dispatched(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    emit_job_dispatched_with_meta(kernel, job_id, worktree_path, slug, None).await;
}

/// Emit a `job.dispatched` event with optional provider metadata.
pub(crate) async fn emit_job_dispatched_with_meta(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
    provider_meta: Option<serde_json::Value>,
) {
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
        provider_meta,
    };
    let dispatched_envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "pipeline: failed to build job.dispatched event: {e}");
            return;
        }
    };

    if let Ok(store) = runtime::events::SqliteEventStore::new() {
        if let Err(e) = runtime::events::EventStore::append(&store, &dispatched_envelope) {
            tracing::warn!(slug, "pipeline: failed to persist job.dispatched: {e}");
        }
    }

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(dispatched_envelope, &ctx).await {
        tracing::warn!(slug, "pipeline: job.dispatched routing failed: {e}");
    }

    tracing::info!(slug, "pipeline: job dispatched successfully");
}
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

    // Find the current step that just completed
    let current_step_id = match &job.current_step {
        Some(id) => id.clone(),
        None => {
            // First step just completed — find the root step
            if let Some(root) = pipeline.root_steps().first() {
                root.id.clone()
            } else {
                tracing::warn!(slug, "pipeline: no root step found");
                return false;
            }
        }
    };

    // Idempotency: if this step is already in completed_steps, skip.
    // This prevents re-dispatching the same step on duplicate completion events.
    if job.completed_steps.contains(&current_step_id) {
        tracing::debug!(slug, step = %current_step_id, "pipeline: step already completed, skipping");
        return false;
    }

    tracing::info!(slug, step = %current_step_id, "pipeline: step completed");
    let mut completed = job.completed_steps.clone();
    completed.push(current_step_id.clone());
    let next_steps = pipeline.unblocked_steps(&completed);
    if next_steps.is_empty() {
        tracing::info!(slug, completed_count = completed.len(), "pipeline: all steps completed");
        return false;
    }

    let wt_root = worktree_root_override.map(|p| p.to_path_buf()).unwrap_or_else(worktrees_dir);
    let worktree_path = wt_root.join(slug);

    // Dispatch each unblocked step
    let mut dispatched_any = false;
    for step in next_steps {
        tracing::info!(
            slug,
            step_id = step.id,
            step_type = ?step.step_type,
            agent = ?step.agent,
            "pipeline: dispatching next step"
        );

        match step.step_type {
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

                // Launch the agent
                // Gate steps are ephemeral — no session persistence needed.
                let persist = step.step_type != StepType::Gate;
                let job_ctx = JobContext {
                    job_id: job.job_id.clone(),
                    slug: slug.clone(),
                    agent: agent.to_string(),
                    branch: job.branch.clone(),
                    spec_path: job.spec_path.clone(),
                    work_dir: worktree_path.clone(),
                    env: [("SHIP_MESH_ID".to_string(), slug.clone())].into(),
                    launch: runtime::services::dispatch_ports::LaunchProfile {
                        provider: step.provider.clone(),
                        model: step.model.clone(),
                        persist_session: persist,
                        ..Default::default()
                    },
                };
                // Pipeline steps use ProcessExecutor (headless, auto-exits on completion).
                // Falls back to the configured executor if ProcessExecutor fails.
                let process_executor = super::executors::ProcessExecutor;
                let spawn_result = process_executor.spawn(&job_ctx).await;
                if let Err(e) = spawn_result {
                    tracing::warn!(slug, step_id = step.id, "pipeline: ProcessExecutor failed, falling back: {e}");
                    if let Err(e2) = dctx.executor.spawn(&job_ctx).await {
                        tracing::error!(slug, step_id = step.id, "pipeline: fallback spawn also failed: {e2}");
                        continue;
                    }
                }

                emit_job_dispatched(kernel, &job.job_id, &worktree_path, slug).await;
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
