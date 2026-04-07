//! Pipeline DAG — step-based advancement for job dispatch.
//!
//! A pipeline is a DAG of steps. Each step has a type (agent, gate, merge, human)
//! and declares dependencies via `after`. The engine walks the DAG: when a step
//! completes, it finds unblocked steps and dispatches them.

use runtime::events::job::{Pipeline, StepType};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::services::dispatch_ports::JobExecutor;
use runtime::projections::job::{load_jobs, load_jobs_at};
use runtime::services::dispatch_ports::JobContext;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{compile_agent_config, worktrees_dir};
use super::job_dispatch::DispatchContext;

/// Enrich a job with a pipeline. Resolution order:
/// 1. Already has `steps` → use it
/// 2. Spec frontmatter has `pipeline: <name>` → load from `.ship/pipelines/<name>.yaml`
/// 3. Spec frontmatter has inline `steps:` → use it
/// 4. None → inject default (implement → gate → merge)
pub(crate) fn ensure_pipeline(
    mut payload: runtime::events::job::JobCreatedPayload,
) -> runtime::events::job::JobCreatedPayload {
    if payload.steps.is_some() {
        return payload;
    }

    let spec_content = read_spec_content(&payload.spec_path);

    // Try explicit pipeline_name from the create request first
    let pipeline_ref = payload.pipeline_name.clone()
        .or_else(|| parse_pipeline_name_from_frontmatter(&spec_content));

    if let Some(name) = pipeline_ref {
        if let Some(pipeline) = load_pipeline_file(&name) {
            tracing::info!(
                slug = payload.slug,
                pipeline = name,
                steps = pipeline.steps.len(),
                "pipeline: loaded from .ship/pipelines/{name}.yaml"
            );
            if let Some(root) = pipeline.root_steps().first() {
                if let Some(ref agent) = root.agent {
                    payload.agent = agent.clone();
                }
            }
            payload.steps = Some(pipeline);
            return payload;
        }
        tracing::warn!(
            slug = payload.slug,
            pipeline = name,
            "pipeline: referenced pipeline not found, falling back"
        );
    }

    // Try inline steps from frontmatter
    if let Some(pipeline) = parse_steps_from_frontmatter(&spec_content) {
        if !pipeline.steps.is_empty() {
            tracing::info!(
                slug = payload.slug,
                steps = pipeline.steps.len(),
                "pipeline: loaded inline steps from spec frontmatter"
            );
            if let Some(root) = pipeline.root_steps().first() {
                if let Some(ref agent) = root.agent {
                    payload.agent = agent.clone();
                }
            }
            payload.steps = Some(pipeline);
            return payload;
        }
    }

    // Default pipeline
    tracing::info!(
        slug = payload.slug,
        agent = payload.agent,
        "pipeline: no steps defined, injecting default pipeline"
    );
    payload.steps = Some(Pipeline::default_for_agent(&payload.agent));
    payload
}

/// Load a named pipeline from `.ship/pipelines/<name>.yaml` in the project root.
fn load_pipeline_file(name: &str) -> Option<Pipeline> {
    // Walk up from cwd looking for .ship/ directory
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let ship_dir = dir.join(".ship");
        if ship_dir.is_dir() {
            let path = ship_dir.join("pipelines").join(format!("{name}.yaml"));
            if path.exists() {
                let content = std::fs::read_to_string(&path).ok()?;
                let val: serde_json::Value = serde_yaml::from_str(&content).ok()?;
                let steps_val = val.get("steps")?;
                let steps: Vec<runtime::events::job::PipelineStep> =
                    serde_json::from_value(steps_val.clone()).ok()?;
                return Some(Pipeline { steps });
            }
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Extract `pipeline: <name>` from YAML frontmatter (string value, not array/object).
fn parse_pipeline_name_from_frontmatter(content: &str) -> Option<String> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];
    let fm: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    fm.get("pipeline")?.as_str().map(str::to_string)
}

/// Read spec content, trying absolute path then relative to project root.
fn read_spec_content(spec_path: &str) -> String {
    let src = std::path::Path::new(spec_path);
    if src.exists() {
        return std::fs::read_to_string(src).unwrap_or_default();
    }
    if let Ok(ship_dir) = runtime::project::get_global_dir() {
        if let Some(root) = ship_dir.parent() {
            let alt = root.join(spec_path);
            if alt.exists() {
                return std::fs::read_to_string(&alt).unwrap_or_default();
            }
        }
    }
    String::new()
}

/// Parse pipeline steps from YAML frontmatter.
fn parse_steps_from_frontmatter(content: &str) -> Option<Pipeline> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];

    let fm: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    let steps_val = fm.get("steps")?;
    serde_json::from_value(steps_val.clone()).ok()
}

/// Emit a `job.dispatched` event.
pub(crate) async fn emit_job_dispatched(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
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

    // Mark current step as completed and find next steps
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
                graceful_agent_exit(slug).await;

                // Swap agent profile
                tracing::info!(slug, agent, step_id = step.id, "pipeline: compiling agent config");
                if let Err(e) = compile_agent_config(&worktree_path, agent) {
                    tracing::warn!(slug, agent, "pipeline: ship use failed (non-fatal): {e}");
                }

                // Launch the agent
                let job_ctx = JobContext {
                    job_id: job.job_id.clone(),
                    slug: slug.clone(),
                    agent: agent.to_string(),
                    branch: job.branch.clone(),
                    spec_path: job.spec_path.clone(),
                    work_dir: worktree_path.clone(),
                    env: [("SHIP_MESH_ID".to_string(), slug.clone())].into(),
                    model: step.model.clone(),
                    provider: step.provider.clone(),
                };
                // Pipeline steps use ProcessExecutor (headless, auto-exits on completion).
                // Falls back to the configured executor if ProcessExecutor fails.
                let process_executor = super::adapters::ProcessExecutor;
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
                graceful_agent_exit(slug).await;

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

/// Send /exit to the agent in a tmux session and wait for clean exit.
/// Best-effort — skips if tmux isn't available (tests).
async fn graceful_agent_exit(slug: &str) {
    let tmux_session = format!("job-{}", slug);

    let has_session = std::process::Command::new("tmux")
        .args(["has-session", "-t", &tmux_session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_session {
        tracing::debug!(slug, "pipeline: no tmux session, skipping graceful exit");
        return;
    }

    tracing::info!(slug, session = %tmux_session, "pipeline: sending /exit to current agent");
    let _ = std::process::Command::new("tmux")
        .args(["send-keys", "-t", &tmux_session, "/exit", "Enter"])
        .status();

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let output = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", &tmux_session, "-p"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.lines().rev().take(3).any(|l| l.trim_end().ends_with('$')) {
                tracing::info!(slug, "pipeline: agent exited cleanly");
                return;
            }
        }
    }
    tracing::warn!(slug, "pipeline: agent did not exit within 30s, proceeding anyway");
}
