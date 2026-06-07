//! Pipeline run domain cores — `dispatch_one`, `run_pipeline_core`, `end_turn_core`.
//!
//! These were previously split across `routes/runtime_api/dispatch/mod.rs`,
//! `routes/runtime_api/pipeline_run.rs`, and `routes/runtime_api/jobs.rs` in
//! shipd. All three are pure pipeline-domain logic with no HTTP dependency;
//! they consume daemon state through the port traits in `pipeline_ports.rs`.
//!
//! Gates are preserved: `run_pipeline_core` enforces `Scope::CreateRuns` via
//! the caller; the write closures in the wire enforce C1 at the ConnectRPC
//! boundary before calling here.

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, event_types};

use crate::pipeline_ports::{PipelineDispatchPort, TurnSignalPort};

// ── dispatch_one ─────────────────────────────────────────────────────────────

/// Shared dispatch primitive — runs `ensure_pipeline` (DAG validation /
/// interpolation), builds the `pipeline.run.started` envelope, persists +
/// routes it, and returns the slug/branch for the caller.
///
/// Called from the `run_pipeline_core` REST path and the `pipelines.dispatch`
/// ConnectRPC write closure.
pub async fn dispatch_one(
    port: &dyn PipelineDispatchPort,
    payload: PipelineRunStartedPayload,
    pipeline_run_id: &str,
) -> Result<DispatchOneOutcome, DispatchOneError> {
    let payload = crate::orchestrator::pipeline_loader::ensure_pipeline(payload)
        .map_err(DispatchOneError::BadRequest)?;

    let slug = payload.slug.clone();
    let branch = payload.branch.clone();
    let resume = payload.resume;

    let envelope =
        EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &payload)
            .map(|e| e.with_actor_id("studio"))
            .map_err(|e| DispatchOneError::Internal(e.to_string()))?;

    port.persist_and_route(&envelope)
        .await
        .map_err(|e| DispatchOneError::Internal(e.to_string()))?;

    Ok(DispatchOneOutcome {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug,
        branch,
        resume,
    })
}

#[derive(Debug)]
pub struct DispatchOneOutcome {
    pub pipeline_run_id: String,
    pub slug: String,
    pub branch: String,
    pub resume: bool,
}

#[derive(Debug)]
pub enum DispatchOneError {
    BadRequest(String),
    Internal(String),
}

// ── run_pipeline_core ─────────────────────────────────────────────────────────

/// Request shape for `POST /api/v1/pipelines/run` (and the ConnectRPC
/// `pipelines.dispatch` op). Transport-neutral — no Axum types.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineRunRequest {
    /// Pipeline name (file in `.ship/pipelines/<name>.jsonc`).
    pub pipeline: String,

    /// Inline spec body. Written to `.artifacts/job-spec.md` for the first
    /// agent; per-step bodies come from the pipeline file's `step_specs:`.
    #[serde(default)]
    pub spec: Option<String>,

    /// Pipeline-level `{{ var }}` overrides. Merged over the file's defaults.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,

    /// Harness override applied to every step that does not declare its own.
    #[serde(default)]
    pub harness: Option<String>,

    /// Model override applied to every step that does not declare its own.
    #[serde(default)]
    pub model: Option<String>,

    /// Workspace label (used for slug generation). Defaults to the
    /// pipeline name.
    #[serde(default)]
    pub workspace: Option<String>,

    /// Explicit slug override.
    #[serde(default)]
    pub slug: Option<String>,

    /// Autonomous (headless) vs interactive. Default false (interactive).
    #[serde(default)]
    pub auto: bool,

    /// Operator-facing path to the spec file (for `ship jobs` display).
    #[serde(default)]
    pub source_spec_path: Option<String>,
}

#[derive(Debug)]
pub enum RunPipelineCoreError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

/// Core logic for `POST /api/v1/pipelines/run`. Locates and parses the pipeline
/// JSONC, builds the `PipelineRunStartedPayload`, then delegates to `dispatch_one`.
///
/// The scope gate (`Scope::CreateRuns`) must be enforced by the caller before
/// invoking this function.
pub async fn run_pipeline_core(
    port: &dyn PipelineDispatchPort,
    req: PipelineRunRequest,
) -> Result<DispatchOneOutcome, RunPipelineCoreError> {
    // Locate the pipeline JSONC. Matches the resolution strategy used by
    // `pipeline_loader::load_pipeline_file` so `ensure_pipeline` will find
    // the same file.
    let pipeline_path = locate_pipeline_file(&req.pipeline);
    let Some(pipeline_path) = pipeline_path else {
        return Err(RunPipelineCoreError::NotFound(format!(
            "pipeline '{}' not found",
            req.pipeline
        )));
    };

    let raw = std::fs::read_to_string(&pipeline_path)
        .map_err(|e| RunPipelineCoreError::Internal(format!("read pipeline: {e}")))?;

    let (normalized, meta) = ship_pipelines::normalize_pipeline(&raw)
        .map_err(|e| RunPipelineCoreError::BadRequest(format!("pipeline parse: {e}")))?;

    let steps = normalized
        .get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| RunPipelineCoreError::BadRequest("pipeline has no steps".into()))?;

    // Run-level default agent = first step's agent. Pure-shell (run) first
    // steps have none; leave it empty — worktree prep skips agent compilation
    // for empty agents (a real agent step always names one).
    let first_agent = steps
        .first()
        .and_then(|s| s.get("agent"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();

    let pipeline_name = meta.name.clone().unwrap_or_else(|| req.pipeline.clone());
    let workspace_label = req.workspace.unwrap_or_else(|| pipeline_name.clone());
    let slug = req.slug.unwrap_or_else(|| {
        format!("{}-{}", workspace_label, ship_substrate::gen_slug_suffix())
    });
    let branch = format!("job/{slug}");
    let pipeline_run_id = ship_substrate::gen_ulid();

    let payload = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.clone(),
        slug,
        agent: first_agent,
        branch,
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: req.source_spec_path,
        plan_id: None,
        model: req.model,
        harness: req.harness,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: Some(pipeline_name),
        vars: req.vars,
        spec_content: req.spec,
        step_specs: None,
        headless: req.auto,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: resolve_project_head_sha(),
    };

    dispatch_one(port, payload, &pipeline_run_id)
        .await
        .map_err(|e| match e {
            DispatchOneError::BadRequest(m) => RunPipelineCoreError::BadRequest(m),
            DispatchOneError::Internal(m) => RunPipelineCoreError::Internal(m),
        })
}

/// Locate the pipeline JSONC by walking upward from cwd. Matches
/// `pipeline_loader::load_pipeline_file` resolution so `ensure_pipeline`
/// finds the same file.
fn locate_pipeline_file(name: &str) -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok();
    while let Some(d) = dir.as_ref() {
        let ship = ship_substrate::ShipHome::project_ship_dir(d);
        let pipelines_dir = ship_store::compose::composed_root_for(&ship).join("pipelines");
        if pipelines_dir.is_dir() {
            let candidate = pipelines_dir.join(format!("{name}.jsonc"));
            if candidate.exists() {
                return Some(candidate);
            }
            break;
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Resolve HEAD SHA of the active project root. Used at job-created time to
/// capture the pipeline's baseline. `None` when git fails or no project root.
fn resolve_project_head_sha() -> Option<String> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    crate::orchestrator::git_helpers::resolve_head_sha(project_root)
}

// ── end_turn_core ─────────────────────────────────────────────────────────────

/// Request shape for the end-turn hook (transport-neutral).
pub struct EndTurnRequest {
    pub slug: String,
}

#[derive(Debug)]
pub enum EndTurnCoreError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

/// Core logic for the harness stop-hook turn-end signal.
///
/// For multi-step pipeline jobs: sends `TurnEnded` to the step monitor via
/// `signal_port`. For single-step jobs or when no monitor is active: emits
/// `pipeline.run.completed` directly.
///
/// Slug isn't unique across job lifetimes; the projection returns the
/// most-recently-updated row — that's the live one when a dispatched job
/// exists.
pub async fn end_turn_core(
    dispatch_port: &dyn PipelineDispatchPort,
    signal_port: &dyn TurnSignalPort,
    req: EndTurnRequest,
) -> Result<String, EndTurnCoreError> {
    use crate::projections::pipeline_run::{PipelineRunStatus, load_pipeline_run_by_slug};

    let job = match load_pipeline_run_by_slug(&req.slug) {
        Ok(Some(j)) if j.status == PipelineRunStatus::Dispatched => j,
        Ok(_) => {
            return Err(EndTurnCoreError::NotFound(format!(
                "no dispatched job with slug '{}'",
                req.slug
            )));
        }
        Err(e) => {
            return Err(EndTurnCoreError::Internal(e.to_string()));
        }
    };

    let is_pipeline = job.steps.as_ref().is_some_and(|s| s.steps.len() > 1);

    if is_pipeline && signal_port.signal_turn_ended(&req.slug).await {
        return Ok(format!("turn ended for pipeline step '{}'", req.slug));
    }

    if is_pipeline {
        tracing::warn!(
            slug = req.slug,
            "end_turn_core: pipeline step has no active step monitor, falling through to job.completed"
        );
    }

    // Single-step job or fallback: emit pipeline.run.completed directly.
    let payload = ship_substrate::events::pipeline::PipelineRunCompletedPayload {
        pipeline_run_id: job.pipeline_run_id.clone(),
        slug: job.slug.clone(),
    };

    let envelope =
        EventEnvelope::new(event_types::PIPELINE_RUN_COMPLETED, &job.pipeline_run_id, &payload)
            .map(|e| e.with_actor_id("hook"))
            .map_err(|e| EndTurnCoreError::BadRequest(e.to_string()))?;

    dispatch_port
        .persist_and_route(&envelope)
        .await
        .map_err(|e| EndTurnCoreError::Internal(e.to_string()))?;

    Ok(format!("job '{}' completed", req.slug))
}
