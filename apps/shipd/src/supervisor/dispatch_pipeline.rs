//! Pipeline enrichment, spec parsing, and phase advancement for job dispatch.

use runtime::events::job::{JobCreatedPayload, PipelinePhase};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::projections::job::{load_jobs, load_jobs_at};
use runtime::services::dispatch_ports::JobContext;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{compile_agent_config, worktrees_dir};
use super::job_dispatch::DispatchContext;

/// Parse the spec file's YAML frontmatter and extract the pipeline field.
/// Returns the payload with pipeline set if found in the spec.
pub(crate) fn enrich_pipeline_from_spec(
    mut payload: JobCreatedPayload,
) -> JobCreatedPayload {
    if payload.pipeline.is_some() {
        return payload;
    }

    let spec_content = read_spec_content(&payload.spec_path);
    if let Some(pipeline) = parse_pipeline_from_frontmatter(&spec_content) {
        if !pipeline.is_empty() {
            tracing::info!(
                slug = payload.slug,
                phases = pipeline.len(),
                "job-dispatch: pipeline found in spec frontmatter"
            );
            if let Some(first) = pipeline.first() {
                payload.agent = first.agent.clone();
            }
            payload.pipeline = Some(pipeline);
        }
    }
    payload
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

/// Extract pipeline phases from YAML frontmatter.
fn parse_pipeline_from_frontmatter(content: &str) -> Option<Vec<PipelinePhase>> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];

    let fm: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    let pipeline_val = fm.get("pipeline")?;
    serde_json::from_value(pipeline_val.clone()).ok()
}

/// Emit a `job.dispatched` event, persisting it and routing through the kernel.
pub(crate) async fn emit_job_dispatched(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: payload.job_id.clone(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
    };
    let dispatched_envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        &payload.job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "job-dispatch: failed to build job.dispatched event: {e}");
            return;
        }
    };

    if let Ok(store) = runtime::events::SqliteEventStore::new() {
        if let Err(e) = runtime::events::EventStore::append(&store, &dispatched_envelope) {
            tracing::warn!(slug, "job-dispatch: failed to persist job.dispatched: {e}");
        }
    }

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(dispatched_envelope, &ctx).await {
        tracing::warn!(slug, "job-dispatch: job.dispatched routing failed: {e}");
    }

    tracing::info!(slug, "job-dispatch: job dispatched successfully");
}

/// Check if a completed job has a pipeline with more phases.
/// If so, dispatch the next phase in the same worktree. Returns true if advanced.
pub(crate) async fn try_advance_pipeline(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
) -> bool {
    try_advance_pipeline_inner(kernel, envelope, dctx, None, None).await
}

/// Test-friendly variant with explicit paths instead of global env vars.
pub(crate) async fn try_advance_pipeline_at(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
    ship_dir: &std::path::Path,
) -> bool {
    // Derive worktree root from ship_dir's parent (test layout: <tmp>/global -> <tmp>/worktrees)
    let worktree_root = ship_dir.parent().map(|p| p.join("worktrees"));
    try_advance_pipeline_inner(kernel, envelope, dctx, Some(ship_dir), worktree_root.as_deref()).await
}

async fn try_advance_pipeline_inner(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
    ship_dir_override: Option<&std::path::Path>,
    worktree_root_override: Option<&std::path::Path>,
) -> bool {
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                event_id = %envelope.id,
                "pipeline: malformed job.completed payload: {e}"
            );
            return false;
        }
    };

    let slug = match payload["slug"].as_str() {
        Some(s) => s.to_string(),
        None => {
            tracing::warn!(
                event_id = %envelope.id,
                "pipeline: job.completed payload missing slug field"
            );
            return false;
        }
    };

    // Load current job state to check pipeline
    let jobs = match ship_dir_override {
        Some(dir) => load_jobs_at(dir),
        None => load_jobs(),
    };
    let jobs = match jobs {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!(slug, "pipeline: failed to load jobs for pipeline check: {e}");
            return false;
        }
    };

    let job = match jobs.values().find(|j| j.slug == slug) {
        Some(j) => j.clone(),
        None => {
            tracing::warn!(slug, "pipeline: job not found in projection");
            return false;
        }
    };

    let pipeline = match &job.pipeline {
        Some(p) if p.len() > 1 => p,
        Some(_) => {
            tracing::info!(slug, "pipeline: single-phase pipeline, no advancement needed");
            return false;
        }
        None => {
            tracing::debug!(slug, "pipeline: no pipeline defined for job");
            return false;
        }
    };

    // Determine current phase (default 0 if not tracked)
    let current = job.current_phase.unwrap_or(0);
    let next = current + 1;

    if next >= pipeline.len() {
        tracing::info!(
            slug,
            phases = pipeline.len(),
            "pipeline: all phases completed"
        );
        return false;
    }

    let next_phase = &pipeline[next];
    tracing::info!(
        slug,
        phase = next,
        total_phases = pipeline.len(),
        agent = next_phase.agent,
        goal = next_phase.goal,
        "pipeline: advancing to next phase"
    );

    let wt_root = worktree_root_override.map(|p| p.to_path_buf()).unwrap_or_else(worktrees_dir);
    let worktree_path = wt_root.join(&slug);
    if !worktree_path.exists() {
        tracing::error!(slug, "pipeline: worktree missing for pipeline advance");
        return false;
    }

    // 1. Gracefully end the current agent — send /exit via tmux, wait for clean exit.
    //    Do NOT kill the tmux session — human needs scroll-back visibility.
    //    Best-effort: if tmux isn't available (tests), skip gracefully.
    graceful_agent_exit(&slug).await;

    // 2. Swap agent profile — run `ship use <next-agent>` in the worktree.
    //    Do NOT overwrite the original job-spec.md — the next agent reads it.
    //    Best-effort: if `ship` CLI isn't available (tests), skip.
    tracing::info!(slug, agent = %next_phase.agent, "pipeline: compiling agent config for next phase");
    if let Err(e) = compile_agent_config(&worktree_path, &next_phase.agent) {
        tracing::warn!(slug, agent = %next_phase.agent, "pipeline: ship use failed (non-fatal): {e}");
    }

    // 3. Relaunch agent in the same tmux session
    let job_ctx = JobContext {
        job_id: job.job_id.clone(),
        slug: slug.clone(),
        agent: next_phase.agent.clone(),
        branch: job.branch.clone(),
        spec_path: job.spec_path.clone(),
        work_dir: worktree_path.clone(),
        env: [("SHIP_MESH_ID".to_string(), slug.clone())].into(),
        model: None,
        provider: None,
    };
    if let Err(e) = dctx.executor.spawn(&job_ctx).await {
        tracing::error!(slug, phase = next, "pipeline: executor spawn failed for phase: {e}");
        return false;
    }

    // Emit job.dispatched for the new phase
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: job.job_id.clone(),
        worktree: worktree_path.to_string_lossy().to_string(),
        pid: None,
    };
    let dispatched_envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        &job.job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(slug, phase = next, "pipeline: failed to build job.dispatched event: {e}");
            return true; // phase was spawned even though event emission failed
        }
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(dispatched_envelope, &ctx).await {
        tracing::warn!(slug, phase = next, "pipeline: job.dispatched routing failed: {e}");
    }

    tracing::info!(
        slug,
        phase = next,
        total_phases = pipeline.len(),
        "pipeline: phase dispatched successfully"
    );
    true
}

/// Send /exit to the agent in a tmux session and wait for it to exit cleanly.
/// Best-effort — if tmux isn't available (e.g. in tests), returns immediately.
async fn graceful_agent_exit(slug: &str) {
    let tmux_session = format!("job-{}", slug);

    // Check if tmux session exists
    let has_session = std::process::Command::new("tmux")
        .args(["has-session", "-t", &tmux_session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_session {
        tracing::debug!(slug, "pipeline: no tmux session found, skipping graceful exit");
        return;
    }

    tracing::info!(slug, session = %tmux_session, "pipeline: sending /exit to current agent");
    let _ = std::process::Command::new("tmux")
        .args(["send-keys", "-t", &tmux_session, "/exit", "Enter"])
        .status();

    // Wait for agent to exit (up to 30s)
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
