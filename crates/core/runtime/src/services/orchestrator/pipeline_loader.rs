//! Pipeline loading and parsing.
//!
//! Resolves a pipeline for a job from (in priority order):
//! 1. Already-set `steps` on the payload
//! 2. Named pipeline reference in spec frontmatter (`pipeline: <name>`)
//! 3. Inline `steps:` in spec frontmatter
//! 4. None → single `Agent` step using the job's declared agent

use crate::events::job::{JobCreatedPayload, Pipeline, PipelineStep, StepType};
use ship_jobs::pipeline::{PipelineMeta, normalize_pipeline};
// Validation moved to `ship-pipelines::validate` per modular-runtime-01.
// Re-export the load-time helper used by step_io callers.
pub(crate) use ship_pipelines::validate::validate_output_name;

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

fn resolve_steps(payload: &mut JobCreatedPayload) {
    if payload.steps.is_some() {
        return;
    }

    // Explicit pipeline name on the payload (e.g. `ship run tdd`) takes precedence
    // over anything in the spec — the caller asked for a specific template by name.
    if let Some(name) = payload.pipeline_name.clone() {
        match load_pipeline_file(&name) {
            Ok(Some((pipeline, meta))) => {
                tracing::info!(
                    slug = payload.slug,
                    pipeline = name,
                    steps = pipeline.steps.len(),
                    "pipeline: loaded by name from payload"
                );
                merge_meta_vars(payload, &meta);
                payload.steps = Some(pipeline);
                return;
            }
            Ok(None) => tracing::warn!(
                slug = payload.slug,
                pipeline = name,
                "pipeline: name on payload did not resolve to a file, continuing resolution"
            ),
            Err(e) => tracing::error!(
                slug = payload.slug,
                pipeline = name,
                "pipeline: failed to parse referenced pipeline file: {e}"
            ),
        }
    }

    let spec_content = spec_content_for_resolution(payload);

    // Try named pipeline reference: `pipeline: tdd` in spec frontmatter
    if let Some(name) = parse_pipeline_name_from_frontmatter(&spec_content) {
        match load_pipeline_file(&name) {
            Ok(Some((pipeline, meta))) => {
                tracing::info!(
                    slug = payload.slug,
                    pipeline = name,
                    steps = pipeline.steps.len(),
                    "pipeline: loaded from .ship/pipelines/{name}"
                );
                merge_meta_vars(payload, &meta);
                payload.steps = Some(pipeline);
                return;
            }
            Ok(None) => tracing::warn!(
                slug = payload.slug,
                pipeline = name,
                "pipeline: referenced pipeline not found, falling back"
            ),
            Err(e) => tracing::error!(
                slug = payload.slug,
                pipeline = name,
                "pipeline: failed to parse referenced pipeline file: {e}"
            ),
        }
    }

    // Try inline steps from frontmatter
    if let Some(pipeline) = parse_steps_from_frontmatter(&spec_content)
        && !pipeline.steps.is_empty()
    {
        tracing::info!(
            slug = payload.slug,
            steps = pipeline.steps.len(),
            "pipeline: loaded inline steps from spec frontmatter"
        );
        payload.steps = Some(pipeline);
        return;
    }

    // No pipeline declared anywhere — construct a single Agent step from the
    // job's declared agent. No review, no merge, no hardcoded model.
    tracing::info!(
        slug = payload.slug,
        agent = payload.agent,
        "pipeline: no steps defined, constructing single-step pipeline from job agent"
    );
    payload.steps = Some(Pipeline {
        steps: vec![PipelineStep {
            id: "implement".to_string(),
            step_type: StepType::Agent,
            agent: Some(payload.agent.clone()),
            goal: None,
            after: vec![],
            model: None,
            provider: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
        }],
        auto_merge: false,
    });
}

/// Substitute `{{ var }}` references in every step field using `payload.vars`.
/// Covers agent, model, provider, goal, command — anything that can carry a template.
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
            *v = compiler::vars::resolve_template(v, &jinja_vars);
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
                *spec = compiler::vars::resolve_template(spec, &jinja_vars);
            }
        }
    }
}

/// Reload the `step_specs` block for a named pipeline from disk. Used by
/// `advance_pipeline` so every step dispatch (including replay) can re-write
/// `.artifacts/pipeline/` idempotently without mutating the event store.
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
        let ship_dir = crate::ShipHome::project_ship_dir(&dir);
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
fn parse_pipeline_content(content: &str) -> Result<(Pipeline, PipelineMeta), String> {
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
/// CLI/payload vars take precedence over pipeline defaults.
fn merge_meta_vars(payload: &mut JobCreatedPayload, meta: &PipelineMeta) {
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

/// Resolve spec content for frontmatter parsing.
/// Prefers the inline `spec_content` on the payload (authoritative, event-carried)
/// over reading from disk (which may not have the spec at all).
fn spec_content_for_resolution(payload: &JobCreatedPayload) -> String {
    if let Some(content) = &payload.spec_content {
        return content.clone();
    }
    read_spec_content(&payload.spec_path)
}

/// Read spec content, trying absolute path then relative to project root.
fn read_spec_content(spec_path: &str) -> String {
    let src = std::path::Path::new(spec_path);
    if src.exists() {
        return std::fs::read_to_string(src).unwrap_or_default();
    }
    if let Ok(ship_dir) = crate::project::get_global_dir()
        && let Some(root) = ship_dir.parent()
    {
        let alt = root.join(spec_path);
        if alt.exists() {
            return std::fs::read_to_string(&alt).unwrap_or_default();
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

    // Try new format first (has agents: or steps: with new-style entries)
    if fm.get("agents").is_some() || fm.get("steps").is_some() {
        let (pipeline, _) = parse_pipeline_content(yaml_str).ok()?;
        return Some(pipeline);
    }

    // Fallback: old format with bare steps array
    let steps_val = fm.get("steps")?;
    serde_json::from_value(steps_val.clone()).ok()
}
