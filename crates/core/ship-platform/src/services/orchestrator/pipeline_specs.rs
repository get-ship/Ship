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
//! Resolution: per-step bodies come from the pipeline JSONC's `step_specs:`
//! block (positional, one entry per step). Steps without an entry get an
//! empty body — the dispatched agent works from the job-level spec at
//! `.artifacts/pipeline/spec.md` plus its agent profile. There is no
//! implicit "share the overall spec across every stage" behaviour.

use ship_substrate::events::pipeline::{Pipeline, PipelineStep};
use crate::services::orchestrator::step_io::ResolvedInput;
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
    _slug: &str,
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

    let bodies = resolve_step_bodies(pipeline, step_specs);
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
    resolved_inputs: &[ResolvedInput],
) -> Result<PathBuf> {
    let content = render_dispatch_prompt(pipeline_name, pipeline, current, resolved_inputs);
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
    resolved_inputs: &[ResolvedInput],
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
        worktree_path, spec_path, ctx.pipeline_name.as_deref(), pipeline, current, resolved_inputs,
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
    pipeline: &Pipeline,
    step_specs: Option<&[String]>,
) -> Vec<String> {
    match step_specs {
        Some(specs) => {
            let mut out: Vec<String> = specs.iter().cloned().collect();
            out.resize(pipeline.steps.len(), String::new());
            out
        }
        None => vec![String::new(); pipeline.steps.len()],
    }
}

fn fmt_size(bytes: u64) -> String {
    if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn render_dispatch_prompt(
    pipeline_name: Option<&str>,
    pipeline: &Pipeline,
    current: &PipelineStep,
    resolved_inputs: &[ResolvedInput],
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

    // Resolved inputs section. Lane-scoped inputs are grouped by lane.
    if !resolved_inputs.is_empty() {
        use crate::services::orchestrator::step_io::parse_lane_scoped_ref;
        use std::collections::BTreeMap;

        let (lane_inputs, plain_inputs): (Vec<_>, Vec<_>) = resolved_inputs
            .iter()
            .partition(|inp| parse_lane_scoped_ref(&inp.name).is_some());

        if !plain_inputs.is_empty() {
            out.push_str("## Inputs available\n\n");
            for inp in &plain_inputs {
                let size_kb = fmt_size(inp.size);
                out.push_str(&format!(
                    "  - **{}** — id: `{}`, path: `{}`, size: {}\n",
                    inp.name, inp.artifact_id, inp.worktree_path, size_kb
                ));
            }
            out.push('\n');
        }

        if !lane_inputs.is_empty() {
            // Determine which fanout step these came from (for the header label).
            let fanout_step_id = pipeline.steps.iter()
                .find(|s| s.fanout.is_some())
                .map(|s| s.id.as_str())
                .unwrap_or("fanout");

            // Group by lane_id (preserving insertion order via BTreeMap).
            let mut by_lane: BTreeMap<&str, Vec<_>> = BTreeMap::new();
            for inp in &lane_inputs {
                if let Some((lane_id, _)) = parse_lane_scoped_ref(&inp.name) {
                    by_lane.entry(lane_id).or_default().push(inp);
                }
            }

            out.push_str(&format!("## Inputs from fanout `{fanout_step_id}`\n\n"));
            for (lane_id, inputs) in &by_lane {
                // If this lane has a `.failed` marker, render the lane as
                // FAILED with the reason so the judge knows the lane did
                // not produce usable output.
                let failed = inputs.iter().find(|inp| {
                    parse_lane_scoped_ref(&inp.name).map(|(_, o)| o) == Some("failed")
                });
                if let Some(f) = failed {
                    let reason = f.failure_reason.as_deref().unwrap_or("lane failed");
                    out.push_str(&format!("  {lane_id}: FAILED — {reason}\n"));
                    out.push_str(&format!(
                        "    marker: `{}` (id: `{}`)\n",
                        f.worktree_path, f.artifact_id
                    ));
                    continue;
                }
                out.push_str(&format!("  {lane_id}:\n"));
                for inp in inputs {
                    let output_name = parse_lane_scoped_ref(&inp.name)
                        .map(|(_, o)| o)
                        .unwrap_or(&inp.name);
                    let size_kb = fmt_size(inp.size);
                    out.push_str(&format!(
                        "    - {} — id: `{}`, path: `{}`, size: {}\n",
                        output_name, inp.artifact_id, inp.worktree_path, size_kb
                    ));
                }
            }
            out.push('\n');
        }
    }

    // Produces contract section (populated when this step has `produces`).
    if !current.produces.is_empty() {
        out.push_str("## This step must produce the following artifact(s)\n\n");
        for output in &current.produces {
            let req = if output.required { "required" } else { "optional" };
            out.push_str(&format!(
                "  - **{}** ({} scope, {})\n",
                output.name,
                format!("{:?}", output.scope).to_lowercase(),
                req
            ));
            out.push_str(&format!(
                "    Upload: `artifact_upload(scope: \"{}\", metadata: {{ produced_name: \"{}\" }})`\n",
                format!("{:?}", output.scope).to_lowercase(),
                output.name
            ));
            out.push_str(&format!(
                "    Report: include `{{ \"{}\": \"<id>\" }}` in `step_done.outputs`.\n\n",
                output.name
            ));
        }
    }

    out.push_str(super::prompt_composer::RUNTIME_CONTRACT);
    out
}

#[cfg(test)]
#[path = "pipeline_specs_tests.rs"]
mod tests;
