//! Pipeline loading and parsing.
//!
//! Resolves a pipeline for a job from (in priority order):
//! 1. Already-set `steps` on the payload
//! 2. Named pipeline reference in spec frontmatter (`pipeline: <name>`)
//! 3. Inline `steps:` in spec frontmatter
//! 4. Default pipeline (`implement → gate → merge`)

use crate::events::job::{JobCreatedPayload, Pipeline};

/// Enrich a job with a pipeline. Resolution order:
/// 1. Already has `steps` → use it
/// 2. Spec frontmatter has `pipeline: <name>` → load from `.ship/pipelines/<name>.yaml`
/// 3. Spec frontmatter has inline `steps:` → use it
/// 4. None → inject default (implement → gate → merge)
pub fn ensure_pipeline(mut payload: JobCreatedPayload) -> JobCreatedPayload {
    if payload.steps.is_some() {
        return payload;
    }

    let spec_content = read_spec_content(&payload.spec_path);

    // Try named pipeline reference: `pipeline: tdd`
    if let Some(name) = parse_pipeline_name_from_frontmatter(&spec_content) {
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
                let steps: Vec<crate::events::job::PipelineStep> =
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
    if let Ok(ship_dir) = crate::project::get_global_dir() {
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
