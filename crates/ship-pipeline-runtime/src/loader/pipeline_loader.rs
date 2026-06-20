//! Pipeline loading and parsing.
//!
//! Resolves a pipeline for a job from (in priority order):
//! 1. Already-set `steps` on the payload
//! 2. Explicit `pipeline_name` on the payload (CLI/HTTP dispatch)
//! 3. Named pipeline reference in spec frontmatter (`pipeline: <name>`)
//! 4. None → single `Agent` step using the job's declared agent
//!
//! Pipelines are JSONC files at `.ship/pipelines/<name>.jsonc`. Spec
//! frontmatter is YAML (markdown convention) and may carry a
//! `pipeline: <name>` reference, but inline pipeline definitions in
//! spec frontmatter are not supported.
//!
//! Lives at `crate::loader::pipeline_loader` (not under `orchestrator`)
//! so `run_core::dispatch_one` can build the dispatch envelope without
//! the `ship-code` feature. Pure spec validation; no process or session
//! types are touched here.

use ship_substrate::events::pipeline::{PipelineRunStartedPayload, Pipeline, PipelineStep};
use ship_pipelines::{PipelineMeta, normalize_pipeline};

mod resolve;
use resolve::resolve_steps;

/// Enrich a job with a pipeline and validate it. Resolution order:
/// 1. Already has `steps` → use it
/// 2. Payload `pipeline_name` → load `.ship/pipelines/<name>.jsonc`
/// 3. Spec frontmatter has `pipeline: <name>` → load `.ship/pipelines/<name>.jsonc`
/// 4. None → single `Agent` step from `payload.agent`
///
/// Returns `Err` if the resolved pipeline fails validation — currently, any
/// agent/gate step without an `agent` declared, or any malformed gate block.
pub fn ensure_pipeline(mut payload: PipelineRunStartedPayload) -> Result<PipelineRunStartedPayload, String> {
    resolve_steps(&mut payload);
    interpolate_pipeline(&mut payload);
    // Re-sync payload.agent with the (possibly now-interpolated) first step's agent.
    if let Some(pipeline) = &payload.steps
        && let Some(root) = pipeline.root_steps().first()
        && let Some(agent) = &root.agent
    {
        payload.agent = agent.clone();
    }
    if let Some(pipeline) = &payload.steps {
        ship_pipelines::validate::validate_pipeline(pipeline).map_err(|e| e.to_string())?;
    }
    Ok(payload)
}

fn interpolate_pipeline(payload: &mut PipelineRunStartedPayload) {
    let Some(pipeline) = payload.steps.as_mut() else { return };
    let Some(vars) = payload.vars.as_ref() else { return };
    if vars.is_empty() {
        return;
    }
    let jinja_vars: std::collections::HashMap<String, serde_json::Value> = vars
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    let render = |s: &mut Option<String>| {
        if let Some(v) = s
            && (v.contains("{{") || v.contains("{%"))
        {
            *v = resolve_template(v, &jinja_vars);
        }
    };
    for step in &mut pipeline.steps {
        render(&mut step.agent);
        render(&mut step.model);
        render(&mut step.harness);
        render(&mut step.goal);
        render(&mut step.command);
    }
    if let Some(step_specs) = payload.step_specs.as_mut() {
        for spec in step_specs.iter_mut() {
            if spec.contains("{{") || spec.contains("{%") {
                *spec = resolve_template(spec, &jinja_vars);
            }
        }
    }
}

fn resolve_template(
    content: &str,
    vars: &std::collections::HashMap<String, serde_json::Value>,
) -> String {
    use minijinja::{Environment, UndefinedBehavior};
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    match env.render_str(content, vars) {
        Ok(result) => result,
        Err(_) => content.to_string(),
    }
}

/// Reload the `step_specs` block for a named pipeline from disk. Used by
/// `advance_pipeline` so every step dispatch (including replay) can re-write
pub fn load_step_specs_by_name(name: &str) -> Option<Vec<String>> {
    let (_, meta) = load_pipeline_file(name).ok()??;
    meta.step_specs
}

/// Load a named pipeline from `.ship/pipelines/<name>.jsonc` in the project
/// root. JSONC is the only supported format.
///
/// Returns:
/// - `Ok(Some(..))` — file found and parsed.
/// - `Ok(None)` — no `.ship/` dir found or no pipeline file of that name exists.
/// - `Err(msg)` — file exists but could not be read or parsed. The error
///   message includes the source path so a JSONC typo surfaces with context
///   rather than being misreported as "pipeline not found".
///
/// Supports both old format (explicit id/type/agent) and new format (agents list,
/// simplified steps).
pub fn load_pipeline_file(name: &str) -> Result<Option<(Pipeline, PipelineMeta)>, String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("cwd lookup failed: {e}"))?;
    loop {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(&dir);
        if ship_dir.is_dir() {
            // Read pipelines from the composed overlay (local base + dep exports)
            // when the space has deps; the raw `.ship/` otherwise (depless spaces
            // resolve to `ship_dir` unchanged — behavior-preserving).
            let root = ship_store::compose::composed_root_for(&ship_dir);
            let path = root.join("pipelines").join(format!("{name}.jsonc"));
            if !path.exists() {
                return Ok(None);
            }
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("failed to read pipeline file '{}': {e}", path.display()))?;
            return parse_pipeline_content(&content)
                .map(Some)
                .map_err(|e| format!("failed to parse pipeline file '{}': {e}", path.display()));
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

/// Parse JSONC pipeline content into a Pipeline and metadata.
/// Handles both old and new formats via ship_jobs normalization.
///
/// Returns a human-readable error string on any parse failure — the caller
/// is responsible for prefixing the source path.
pub fn parse_pipeline_content(content: &str) -> Result<(Pipeline, PipelineMeta), String> {
    let (normalized, meta) = normalize_pipeline(content)
        .map_err(|e| format!("pipeline normalization failed: {e}"))?;
    let steps_val = normalized
        .get("steps")
        .ok_or_else(|| "pipeline content has no `steps` field after normalization".to_string())?;
    let steps: Vec<PipelineStep> = serde_json::from_value(steps_val.clone())
        .map_err(|e| format!("failed to deserialize steps: {e}"))?;
    for step in &steps {
        if let Some(gate) = &step.gate
            && let Err(e) = ship_pipelines::gate::validate(gate)
        {
            return Err(format!("step '{}' has malformed gate block: {e}", step.id));
        }
    }
    let auto_merge = meta.auto_merge.unwrap_or(false);
    Ok((Pipeline { steps, auto_merge }, meta))
}

/// Merge pipeline-level default vars into the job payload.
pub fn merge_meta_vars(payload: &mut PipelineRunStartedPayload, meta: &PipelineMeta) {
    if let Some(pipeline_vars) = &meta.vars {
        let run_vars = payload.vars.get_or_insert_with(Default::default);
        for (k, v) in pipeline_vars {
            run_vars.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    if payload.step_specs.is_none()
        && let Some(specs) = &meta.step_specs
    {
        payload.step_specs = Some(specs.clone());
    }
}
