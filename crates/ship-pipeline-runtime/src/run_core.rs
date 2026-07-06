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
    // UNMAPPED COUPLING (Wave 4 phase A stop-rule, see
    // `loader::pipeline_loader`): pipelines arrive via the descriptor's
    // `ExtensionLibrary::pipelines`; the `dispatch_one` envelope-builder
    // does not yet carry one (a follow-up seam). The empty slice here means
    // `ensure_pipeline`'s by-name path always falls through to the
    // `payload.steps`-first contract; `payload.steps` is what populates
    // the runtime `Pipeline` until the orchestrator context learns to
    // carry the descriptor.
    let payload = crate::loader::pipeline_loader::ensure_pipeline(payload, &[])
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

/// Core logic for `POST /api/v1/pipelines/run`. Resolves a named pipeline
/// from the descriptor (TS declare-forms in the space's `ship.config.ts`)
/// and builds the `PipelineRunStartedPayload`, then delegates to
/// `dispatch_one`.
///
/// UNMAPPED COUPLING (Wave 4 phase A stop-rule): pipelines arrive via
/// the descriptor's `ExtensionLibrary::pipelines`, but the runtime-shape
/// conversion (descriptor `compiler::PipelineDef` → runtime
/// `ship_substrate::events::pipeline::Pipeline`) is a separate design pass
/// — the descriptor's `step_specs: [{agent, harness, prompt}]` shape does
/// not map 1:1 onto the runtime's `step_type/agent/goal/command` per-step
/// fields. Until that converter lands, this function surfaces the
/// coupling: the request's pipeline name cannot be resolved through the
/// deleted JSONC file path. Surfaced per the Wave 4 phase A spec
/// stop-rule; the follow-up seam is on the orchestrator context (carry
/// the descriptor in `OrchestratorContext` and route through
/// `ensure_pipeline`'s `pipelines: &[PipelineDef]` argument).
///
/// The scope gate (`Scope::CreateRuns`) must be enforced by the caller before
/// invoking this function.
pub async fn run_pipeline_core(
    _port: &dyn PipelineDispatchPort,
    req: PipelineRunRequest,
) -> Result<DispatchOneOutcome, RunPipelineCoreError> {
    Err(RunPipelineCoreError::NotFound(format!(
        "pipeline resolution unmapped: descriptor → runtime Pipeline conversion is a separate design pass (Wave 4 phase A stop-rule). Pipeline '{}' cannot be loaded via the deleted JSONC file path; populate `payload.steps` through a descriptor-aware seam.",
        req.pipeline
    )))
}

/// Resolve HEAD SHA of the active project root. Used at job-created time to
/// capture the pipeline's baseline. `None` when git fails or no project root.
fn resolve_project_head_sha() -> Option<String> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    crate::loader::git_helpers::resolve_head_sha(project_root)
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
