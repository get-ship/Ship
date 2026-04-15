//! Pipeline loading and parsing.
//!
//! Resolves a pipeline for a job from (in priority order):
//! 1. Already-set `steps` on the payload
//! 2. Named pipeline reference in spec frontmatter (`pipeline: <name>`)
//! 3. Inline `steps:` in spec frontmatter
//! 4. None → single `Agent` step using the job's declared agent

use crate::events::job::{JobCreatedPayload, Pipeline, PipelineStep, StepType};
use ship_jobs::pipeline::{PipelineMeta, normalize_pipeline_yaml};

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
        validate_pipeline(pipeline)?;
    }
    Ok(payload)
}

/// Enforce platform invariants on a resolved pipeline. Currently: any step
/// whose type requires an agent (`Agent`, `Gate`) must declare one. No silent
/// fallbacks — naming the step id makes misconfigured pipelines fail loud.
fn validate_pipeline(pipeline: &Pipeline) -> Result<(), String> {
    for step in &pipeline.steps {
        let needs_agent = matches!(step.step_type, StepType::Agent | StepType::Review);
        if needs_agent && step.agent.as_deref().map(str::is_empty).unwrap_or(true) {
            return Err(format!(
                "pipeline step '{}' of type {:?} requires an agent but none was declared",
                step.id, step.step_type
            ));
        }
    }
    Ok(())
}

/// Attach a `Pipeline` to `payload.steps` without any template interpolation.
/// Interpolation is a second pass so every resolution path goes through the same
/// substitution logic.
fn resolve_steps(payload: &mut JobCreatedPayload) {
    if payload.steps.is_some() {
        return;
    }

    // Explicit pipeline name on the payload (e.g. `ship run tdd`) takes precedence
    // over anything in the spec — the caller asked for a specific template by name.
    if let Some(name) = payload.pipeline_name.clone() {
        if let Some((pipeline, meta)) = load_pipeline_file(&name) {
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
        tracing::warn!(
            slug = payload.slug,
            pipeline = name,
            "pipeline: name on payload did not resolve to a file, continuing resolution"
        );
    }

    let spec_content = spec_content_for_resolution(payload);

    // Try named pipeline reference: `pipeline: tdd` in spec frontmatter
    if let Some(name) = parse_pipeline_name_from_frontmatter(&spec_content) {
        if let Some((pipeline, meta)) = load_pipeline_file(&name) {
            tracing::info!(
                slug = payload.slug,
                pipeline = name,
                steps = pipeline.steps.len(),
                "pipeline: loaded from .ship/pipelines/{name}.yaml"
            );
            merge_meta_vars(payload, &meta);
            payload.steps = Some(pipeline);
            return;
        }
        tracing::warn!(
            slug = payload.slug,
            pipeline = name,
            "pipeline: referenced pipeline not found, falling back"
        );
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
    let (_, meta) = load_pipeline_file(name)?;
    meta.step_specs
}

/// Load a named pipeline from `.ship/pipelines/<name>.yaml` in the project root.
/// Supports both old format (explicit id/type/agent) and new format (agents list,
/// simplified steps).
fn load_pipeline_file(name: &str) -> Option<(Pipeline, PipelineMeta)> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let ship_dir = dir.join(".ship");
        if ship_dir.is_dir() {
            let path = ship_dir.join("pipelines").join(format!("{name}.yaml"));
            if path.exists() {
                let content = std::fs::read_to_string(&path).ok()?;
                return parse_pipeline_content(&content);
            }
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Parse pipeline YAML content into a Pipeline and metadata.
/// Handles both old and new formats via ship_jobs normalization.
fn parse_pipeline_content(content: &str) -> Option<(Pipeline, PipelineMeta)> {
    let (normalized, meta) = normalize_pipeline_yaml(content).ok()?;
    let steps_val = normalized.get("steps")?;
    let steps: Vec<PipelineStep> = serde_json::from_value(steps_val.clone()).ok()?;
    for step in &steps {
        if let Some(gate) = &step.gate
            && let Err(e) = super::gate_rules::validate(gate)
        {
            tracing::error!(step = step.id, "pipeline: malformed gate block: {e}");
            return None;
        }
    }
    let auto_merge = meta.auto_merge.unwrap_or(false);
    Some((Pipeline { steps, auto_merge }, meta))
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
        let (pipeline, _) = parse_pipeline_content(yaml_str)?;
        return Some(pipeline);
    }

    // Fallback: old format with bare steps array
    let steps_val = fm.get("steps")?;
    serde_json::from_value(steps_val.clone()).ok()
}
