//! Pipeline loading and parsing.
//!
//! Resolves a pipeline for a job from (in priority order):
//! 1. Already-set `steps` on the payload
//! 2. Named pipeline reference in spec frontmatter (`pipeline: <name>`)
//! 3. Inline `steps:` in spec frontmatter
//! 4. None → single `Agent` step using the job's declared agent

use ship_substrate::events::job::{JobCreatedPayload, Pipeline, PipelineStep, StepType};
use ship_jobs::pipeline::{PipelineMeta, normalize_pipeline};
// Validation moved to `ship-pipelines::validate` per modular-runtime-01.
// Re-export the load-time helper used by step_io callers.
pub(crate) use ship_pipelines::validate::validate_output_name;

mod resolve;
use resolve::resolve_steps;

/// Enrich a job with a pipeline and validate it. Resolution order:
/// 1. Already has `steps` → use it
/// 2. Spec frontmatter has `pipeline: <name>` → load from `.ship/pipelines/<name>.yaml`
/// 3. Spec frontmatter has inline `steps:` → use it
/// 4. None → single `Agent` step from `payload.agent`
///
/// Returns `Err` if the resolved pipeline fails validation — currently, any
/// agent/gate step without an `agent` declared.
pub fn ensure_pipeline(mut payload: JobCreatedPayload) -> Result<JobCreatedPayload, String> {
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
        ship_pipelines::validate::validate_pipeline(pipeline)?;
    }
    Ok(payload)
}

fn interpolate_pipeline(payload: &mut JobCreatedPayload) {
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
        render(&mut step.provider);
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

/// Load a named pipeline from `.ship/pipelines/<name>.jsonc` (preferred) or
/// `.ship/pipelines/<name>.yaml` (legacy) in the project root.
///
/// Returns:
/// - `Ok(Some(..))` — file found and parsed.
/// - `Ok(None)` — no `.ship/` dir found or no pipeline file of that name exists.
/// - `Err(msg)` — file exists but could not be read or parsed. The error
///   message includes the source path so a YAML typo surfaces with context
///   rather than being misreported as "pipeline not found".
///
/// Supports both old format (explicit id/type/agent) and new format (agents list,
/// simplified steps).
pub(crate) fn load_pipeline_file(name: &str) -> Result<Option<(Pipeline, PipelineMeta)>, String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("cwd lookup failed: {e}"))?;
    loop {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(&dir);
        if ship_dir.is_dir() {
            // Prefer .jsonc, fall back to .yaml for backwards compat
            let jsonc_path = ship_dir.join("pipelines").join(format!("{name}.jsonc"));
            let yaml_path = ship_dir.join("pipelines").join(format!("{name}.yaml"));
            let path = if jsonc_path.exists() {
                jsonc_path
            } else if yaml_path.exists() {
                yaml_path
            } else {
                return Ok(None);
            };
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

/// Parse pipeline content (JSONC or YAML) into a Pipeline and metadata.
/// Handles both old and new formats via ship_jobs normalization.
///
/// Returns a human-readable error string on any parse failure — the caller
/// is responsible for prefixing the source path.
pub(super) fn parse_pipeline_content(content: &str) -> Result<(Pipeline, PipelineMeta), String> {
    let (normalized, meta) = normalize_pipeline(content)
        .map_err(|e| format!("pipeline normalization failed: {e}"))?;
    let steps_val = normalized
        .get("steps")
        .ok_or_else(|| "pipeline content has no `steps` field after normalization".to_string())?;
    let steps: Vec<PipelineStep> = serde_json::from_value(steps_val.clone())
        .map_err(|e| format!("failed to deserialize steps: {e}"))?;
    for step in &steps {
        if let Some(gate) = &step.gate
            && let Err(e) = super::gate_rules::validate(gate)
        {
            return Err(format!("step '{}' has malformed gate block: {e}", step.id));
        }
    }
    let auto_merge = meta.auto_merge.unwrap_or(false);
    Ok((Pipeline { steps, auto_merge }, meta))
}

/// Merge pipeline-level default vars into the job payload.

pub(super) fn merge_meta_vars(payload: &mut JobCreatedPayload, meta: &PipelineMeta) {
    if let Some(pipeline_vars) = &meta.vars {
        let job_vars = payload.vars.get_or_insert_with(Default::default);
        for (k, v) in pipeline_vars {
            job_vars.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    if payload.step_specs.is_none()
        && let Some(specs) = &meta.step_specs
    {
        payload.step_specs = Some(specs.clone());
    }
}

