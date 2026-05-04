//! Pipeline-step resolution: turning a job payload's spec /
//! pipeline-name reference into a concrete `Pipeline` value. Split off
//! from the parent module per supervisor::tests_service_architecture
//! file-size enforcement.

use ship_substrate::events::job::{JobCreatedPayload, Pipeline, PipelineStep, StepType};
use super::{load_pipeline_file, merge_meta_vars, parse_pipeline_content};

pub(super) fn resolve_steps(payload: &mut JobCreatedPayload) {
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

fn spec_content_for_resolution(payload: &JobCreatedPayload) -> String {
    if let Some(content) = &payload.spec_content {
        return content.clone();
    }
    read_spec_content(&payload.spec_path)
}


fn read_spec_content(spec_path: &str) -> String {
    let src = std::path::Path::new(spec_path);
    if src.exists() {
        return std::fs::read_to_string(src).unwrap_or_default();
    }
    if let Ok(ship_dir) = ship_substrate::project::get_global_dir()
        && let Some(root) = ship_dir.parent()
    {
        let alt = root.join(spec_path);
        if alt.exists() {
            return std::fs::read_to_string(&alt).unwrap_or_default();
        }
    }
    String::new()
}


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
