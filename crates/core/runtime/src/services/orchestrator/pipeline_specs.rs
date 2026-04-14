//! Per-stage pipeline specs — pre-written once at job launch.
//!
//! At dispatch time the orchestrator writes `.artifacts/pipeline/`:
//!
//! ```text
//! .artifacts/pipeline/
//!   spec.md              # overall job spec (user body)
//!   01-<step_id>.md      # stage 1 spec
//!   02-<step_id>.md      # stage 2 spec
//!   ...
//! ```
//!
//! Each agent then receives a short dispatch prompt at `.artifacts/job-spec.md`
//! that tells it which file to execute and where to find upstream/downstream
//! context.
//!
//! Resolution order for per-step bodies:
//! 1. Explicit `step_specs:` block on the pipeline YAML.
//! 2. Split the overall spec body on agent-named `##` headers.
//! 3. Legacy fallback: share the overall spec body across every step (emits
//!    a deprecation log line).

use crate::events::job::{Pipeline, PipelineStep};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Directory (relative to the worktree root) that holds the per-stage specs.
pub const PIPELINE_DIR: &str = ".artifacts/pipeline";
/// File name for the overall spec body inside `PIPELINE_DIR`.
pub const OVERALL_SPEC_FILE: &str = "spec.md";

/// Write the overall `spec.md` and every `NN-<step_id>.md` under
/// `<worktree>/.artifacts/pipeline/`. Write-once: existing per-step files are
/// preserved so a later stage cannot overwrite an earlier stage's spec.
pub fn write_pipeline_specs(
    worktree_path: &Path,
    slug: &str,
    pipeline: &Pipeline,
    user_spec_body: &str,
    step_specs: Option<&[String]>,
) -> Result<PathBuf> {
    let dir = worktree_path.join(PIPELINE_DIR);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create pipeline spec dir {}", dir.display()))?;

    let overall = dir.join(OVERALL_SPEC_FILE);
    if !overall.exists() {
        std::fs::write(&overall, user_spec_body)
            .with_context(|| format!("write {OVERALL_SPEC_FILE}"))?;
    }

    let bodies = resolve_step_bodies(slug, pipeline, user_spec_body, step_specs);
    for (idx, step) in pipeline.steps.iter().enumerate() {
        let fname = step_spec_filename(idx, step);
        let path = dir.join(&fname);
        if path.exists() {
            continue;
        }
        let body = bodies.get(idx).cloned().unwrap_or_default();
        std::fs::write(&path, body)
            .with_context(|| format!("write {fname}"))?;
    }
    Ok(dir)
}

/// Filename for a step's per-stage spec (e.g. `01-test-writer.md`).
pub fn step_spec_filename(idx: usize, step: &PipelineStep) -> String {
    format!("{:02}-{}.md", idx + 1, step.id)
}

/// Compose and write the dispatch prompt for `current` step at
/// `<worktree>/<spec_path>` (conventionally `.artifacts/job-spec.md`).
/// This replaces the legacy single-spec overwrite — the actual step content
/// lives in `.artifacts/pipeline/`, and this prompt points the agent at it.
pub fn write_dispatch_prompt(
    worktree_path: &Path,
    spec_path: &str,
    pipeline_name: Option<&str>,
    pipeline: &Pipeline,
    current: &PipelineStep,
) -> Result<PathBuf> {
    let content = render_dispatch_prompt(pipeline_name, pipeline, current);
    let dst = worktree_path.join(spec_path);
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create spec dir {}", parent.display()))?;
    }
    std::fs::write(&dst, &content)
        .with_context(|| format!("write dispatch prompt to {}", dst.display()))?;
    Ok(dst)
}

/// Idempotent per-step preparation for the advance path. Re-writes
/// `.artifacts/pipeline/` (safe across replay) and drops a fresh dispatch
/// prompt at `spec_path`.
pub fn prepare_step_for_dispatch(
    worktree_path: &Path,
    slug: &str,
    job_id: &str,
    spec_path: &str,
    pipeline: &Pipeline,
    current: &PipelineStep,
) {
    let ctx = super::prompt_composer::load_job_context(job_id);
    let step_specs = ctx
        .pipeline_name
        .as_deref()
        .and_then(super::pipeline_loader::load_step_specs_by_name);
    if let Err(e) = write_pipeline_specs(
        worktree_path, slug, pipeline, &ctx.user_spec_body, step_specs.as_deref(),
    ) {
        tracing::error!(slug, step_id = current.id, "pipeline: failed to write pipeline specs: {e}");
    }
    match write_dispatch_prompt(
        worktree_path, spec_path, ctx.pipeline_name.as_deref(), pipeline, current,
    ) {
        Ok(path) => tracing::info!(
            slug, step_id = current.id,
            "pipeline: wrote dispatch prompt to {}", path.display()
        ),
        Err(e) => tracing::error!(
            slug, step_id = current.id,
            "pipeline: failed to write dispatch prompt: {e}"
        ),
    }
}

fn resolve_step_bodies(
    slug: &str,
    pipeline: &Pipeline,
    user_spec_body: &str,
    step_specs: Option<&[String]>,
) -> Vec<String> {
    if let Some(specs) = step_specs {
        let mut out: Vec<String> = specs.iter().cloned().collect();
        out.resize(pipeline.steps.len(), String::new());
        return out;
    }
    if let Some(split) = split_by_agent_headers(user_spec_body, pipeline) {
        return split;
    }
    tracing::warn!(
        slug,
        "pipeline: no step_specs or agent-named headers; sharing the overall spec across every \
         stage (deprecated — add a `step_specs:` block to the pipeline YAML)"
    );
    vec![user_spec_body.to_string(); pipeline.steps.len()]
}

/// Split `body` on `#`/`##` headers whose trimmed text matches a step's agent
/// name or step id. Returns `None` when at least one step can't be matched.
fn split_by_agent_headers(body: &str, pipeline: &Pipeline) -> Option<Vec<String>> {
    if pipeline.steps.is_empty() {
        return None;
    }
    let keys: Vec<String> = pipeline
        .steps
        .iter()
        .map(|s| s.agent.clone().unwrap_or_else(|| s.id.clone()))
        .collect();

    let mut sections: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut current_key: Option<String> = None;
    let mut current_buf = String::new();

    for line in body.lines() {
        if let Some(header) = parse_header(line) {
            let matched = keys.iter().find(|k| k.eq_ignore_ascii_case(header.trim()));
            if let Some(key) = matched {
                if let Some(prev) = current_key.take() {
                    sections.insert(prev, std::mem::take(&mut current_buf));
                }
                current_key = Some(key.clone());
                continue;
            }
        }
        if current_key.is_some() {
            current_buf.push_str(line);
            current_buf.push('\n');
        }
    }
    if let Some(prev) = current_key.take() {
        sections.insert(prev, current_buf);
    }

    if keys.iter().any(|k| !sections.contains_key(k)) {
        return None;
    }
    Some(
        keys.iter()
            .map(|k| sections.remove(k).unwrap_or_default().trim().to_string())
            .collect(),
    )
}

fn parse_header(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if let Some(rest) = line.strip_prefix("## ") {
        Some(rest)
    } else if let Some(rest) = line.strip_prefix("# ") {
        Some(rest)
    } else {
        None
    }
}

fn render_dispatch_prompt(
    pipeline_name: Option<&str>,
    pipeline: &Pipeline,
    current: &PipelineStep,
) -> String {
    let idx = pipeline
        .steps
        .iter()
        .position(|s| s.id == current.id)
        .unwrap_or(0);
    let total = pipeline.steps.len();
    let name = pipeline_name.unwrap_or("pipeline");
    let agent = current.agent.as_deref().unwrap_or("unassigned");
    let own = format!(
        "{}/{}",
        PIPELINE_DIR,
        step_spec_filename(idx, current)
    );

    let mut out = String::new();
    out.push_str(&format!(
        "# Pipeline `{name}` — step {n} of {total} ({agent})\n\n",
        n = idx + 1
    ));
    out.push_str(&format!(
        "Read `{own}` — this is YOUR spec for this step. Execute it autonomously.\n\n"
    ));

    out.push_str("## Context\n\n");
    out.push_str(&format!(
        "- `{PIPELINE_DIR}/{OVERALL_SPEC_FILE}` — the overall job spec (umbrella)\n"
    ));
    for (i, s) in pipeline.steps.iter().enumerate() {
        if i == idx {
            continue;
        }
        let path = format!("{}/{}", PIPELINE_DIR, step_spec_filename(i, s));
        let role = s.agent.as_deref().unwrap_or("?");
        let relation = if i < idx { "upstream" } else { "downstream" };
        out.push_str(&format!("- `{path}` — {relation} step ({role}); read for context, don't execute\n"));
    }
    out.push('\n');

    out.push_str("## Pipeline shape\n\n");
    for (i, s) in pipeline.steps.iter().enumerate() {
        let marker = if s.id == current.id { "→" } else { " " };
        let role = s.agent.as_deref().unwrap_or("?");
        out.push_str(&format!("{marker} step-{n} `{id}` ({role})\n", n = i + 1, id = s.id));
    }
    out.push('\n');

    out.push_str(super::prompt_composer::RUNTIME_CONTRACT);
    out
}

#[cfg(test)]
#[path = "pipeline_specs_tests.rs"]
mod tests;
