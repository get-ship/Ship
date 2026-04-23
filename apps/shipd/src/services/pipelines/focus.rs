//! `pipelines.focus(name)` — per-pipeline detail page data.
//!
//! Pre-shapes everything the focus template needs, including the
//! already-partitioned `active_for_pipeline` / `other_for_pipeline`
//! run buckets so the template doesn't need a `.append()` loop
//! (minijinja 2.x in this repo doesn't support list mutation).

use serde::Serialize;
use std::path::Path;

use crate::views::state::{
    Filter, PipelineDef, aggregate_pipeline_stats, list_pipeline_defs, list_pipeline_runs,
};

use super::shape::{
    RunView, current_project_workspace_id, find_pipeline_file, providers_on_path, to_run_view,
};

#[derive(Serialize)]
pub struct FocusCtx {
    pub name: String,
    pub def: PipelineDef,
    /// Raw pipeline doc as JSON (from YAML or JSONC) — template unpacks
    /// steps, gate rules, vars, auto_merge, step_specs.
    pub doc: serde_json::Value,
    /// Verbatim file contents for `<ship-code-editor>` raw-edit mode.
    pub raw: String,
    /// On-disk extension — "jsonc" or "yaml".
    pub format: String,
    pub workspace_id: Option<String>,
    /// All runs of this pipeline, newest first (cap 25).
    pub runs: Vec<RunView>,
    /// Active-only runs (dispatched / pending / gate-pending).
    /// Pre-computed so the template doesn't loop-append in minijinja.
    pub active_for_pipeline: Vec<RunView>,
    /// Runs not currently active — history-mode rendering.
    pub other_for_pipeline: Vec<RunView>,
    /// True when any active runs exist (template's `ops_mode` trigger).
    pub ops_mode: bool,
    pub providers: Vec<String>,
}

/// Build the focus context. Returns `None` when no pipeline file or
/// aggregated def matches `name`.
pub fn build(name: &str, ship_dir: &Path) -> Option<FocusCtx> {
    let path = find_pipeline_file(name)?;
    let raw = std::fs::read_to_string(&path).ok()?;
    let doc: serde_json::Value = {
        let trimmed = raw.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            let json = compiler::jsonc::strip_jsonc_comments(&raw);
            serde_json::from_str(&json).ok()?
        } else {
            let v: serde_yaml::Value = serde_yaml::from_str(&raw).ok()?;
            serde_json::to_value(v).ok()?
        }
    };

    let mut defs = list_pipeline_defs(ship_dir);
    let all_runs = list_pipeline_runs(ship_dir, &Filter::default());
    aggregate_pipeline_stats(&mut defs, all_runs.iter());
    let def = defs.into_iter().find(|d| d.name == name)?;

    let runs: Vec<RunView> = all_runs
        .iter()
        .filter(|j| {
            j.slug
                .rsplit_once('-')
                .map(|(prefix, suffix)| {
                    prefix == name
                        && suffix.len() == 8
                        && suffix.chars().all(|c| c.is_ascii_alphanumeric())
                })
                .unwrap_or(false)
        })
        .take(25)
        .map(|j| to_run_view(j, true))
        .collect();

    let active_for_pipeline: Vec<RunView> = runs
        .iter()
        .filter(|r| matches!(r.state, "dispatched" | "pending" | "gate-pending"))
        .cloned()
        .collect();
    let other_for_pipeline: Vec<RunView> = runs
        .iter()
        .filter(|r| !matches!(r.state, "dispatched" | "pending" | "gate-pending"))
        .cloned()
        .collect();
    let ops_mode = !active_for_pipeline.is_empty();

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jsonc")
        .to_string();

    Some(FocusCtx {
        name: name.to_string(),
        def,
        doc,
        raw,
        format,
        workspace_id: current_project_workspace_id(),
        runs,
        active_for_pipeline,
        other_for_pipeline,
        ops_mode,
        providers: providers_on_path(),
    })
}
