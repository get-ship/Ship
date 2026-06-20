//! Pipeline domain read helpers for daemon-rendered views.
//!
//! Relocated from `shipd::views::state` (L2 → L1 ship-pipeline-runtime).
//!
//! Provides: `PipelineDef`, `Filter`, `RunCategory`, `list_pipeline_defs`,
//! `list_pipeline_runs`, `aggregate_pipeline_stats`, `partition_runs`,
//! `status_token`, `status_tone`, `pipeline_name_for_job_public`.

use crate::projections::pipeline_run::{PipelineRun, PipelineRunStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

/// Definition of a pipeline parsed from a `.ship/pipelines/*.jsonc` file,
/// plus usage stats aggregated from PipelineRuns referencing it.
#[derive(Clone, Debug, Serialize)]
pub struct PipelineDef {
    pub name: String,
    pub description: String,
    pub step_count: usize,
    /// Runs currently in `dispatched` / `pending` / `gate-pending`.
    #[serde(default)]
    pub active_runs: usize,
    /// Total runs we have a record of.
    #[serde(default)]
    pub total_runs: usize,
    /// Runs in `completed` or `merged`.
    #[serde(default)]
    pub success_runs: usize,
    /// Runs in `failed`.
    #[serde(default)]
    pub failed_runs: usize,
    /// Success rate, 0.0..1.0 over `total_runs`. 0 when `total_runs == 0`.
    #[serde(default)]
    pub success_rate: f32,
    /// Most recent run's updated_at (ISO 8601), regardless of status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<String>,
    /// Status token for the most recent run (e.g. "merged", "failed").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_status: Option<String>,
    /// Tone for the most recent run's status (info/success/destructive/etc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_tone: Option<String>,
    /// Slug of the most recent run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_slug: Option<String>,
    /// Outcome sparkline — last ~8 run status tokens, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_outcomes: Vec<String>,
}

/// Categorization of a run for tab bucketing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunCategory {
    Active,
    History,
}

impl RunCategory {
    pub fn of(status: &PipelineRunStatus) -> Self {
        match status {
            PipelineRunStatus::Dispatched | PipelineRunStatus::GatePending | PipelineRunStatus::Pending => {
                RunCategory::Active
            }
            PipelineRunStatus::Completed
            | PipelineRunStatus::Failed
            | PipelineRunStatus::Merged
            | PipelineRunStatus::Blocked
            | PipelineRunStatus::Cancelled => RunCategory::History,
        }
    }
}

/// A view filter, typically populated from a query string.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Filter {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub pipeline: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.q.is_none() && self.agent.is_none() && self.pipeline.is_none() && self.status.is_none()
    }

    pub fn matches(&self, job: &PipelineRun) -> bool {
        if let Some(q) = self.q.as_deref().filter(|s| !s.is_empty()) {
            let needle = q.to_lowercase();
            let haystack = format!("{} {}", job.slug, job.agent).to_lowercase();
            if !haystack.contains(&needle) {
                return false;
            }
        }
        if let Some(a) = self.agent.as_deref().filter(|s| !s.is_empty())
            && job.agent != a
        {
            return false;
        }
        if let Some(s) = self.status.as_deref().filter(|s| !s.is_empty())
            && status_token(&job.status) != s
        {
            return false;
        }
        if let Some(p) = self.pipeline.as_deref().filter(|s| !s.is_empty()) {
            match pipeline_name_for_job(job) {
                Some(ref name) if name == p => {}
                _ => return false,
            }
        }
        true
    }
}

/// Normalized snake_case token for `PipelineRunStatus` (the DOMAIN token —
/// proto convention; the GUI kebab variant was the drift we're closing).
pub fn status_token(s: &PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::Pending => "pending",
        PipelineRunStatus::Dispatched => "dispatched",
        PipelineRunStatus::GatePending => "gate_pending",
        PipelineRunStatus::Completed => "completed",
        PipelineRunStatus::Failed => "failed",
        PipelineRunStatus::Merged => "merged",
        PipelineRunStatus::Blocked => "blocked",
        PipelineRunStatus::Cancelled => "cancelled",
    }
}

/// Short tone token for ship-badge / ship-status-dot tone attributes.
pub fn status_tone(s: &PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::Dispatched => "info",
        PipelineRunStatus::GatePending => "warning",
        PipelineRunStatus::Completed => "success",
        PipelineRunStatus::Failed => "destructive",
        PipelineRunStatus::Merged => "accent",
        PipelineRunStatus::Blocked => "warning",
        PipelineRunStatus::Cancelled => "muted",
        PipelineRunStatus::Pending => "muted",
    }
}

/// Recognised pipeline file extensions. jsonc-only: the loader
/// (`pipeline_loader.rs`) reads `{name}.jsonc` and YAML pipelines failed to
/// dispatch from studio (the ext list matched; the loader did not).
const PIPELINE_EXTS: &[&str] = &["jsonc"];

/// List pipeline definitions from `<ship_dir>/pipelines/`, sorted by name.
pub fn list_pipeline_defs(ship_dir: &Path) -> Vec<PipelineDef> {
    let mut walker = std::env::current_dir().ok();
    let mut project_local = None;
    while let Some(d) = walker.as_ref() {
        let ship = ship_substrate::ShipHome::project_ship_dir(d);
        let cand = ship_store::compose::composed_root_for(&ship).join("pipelines");
        if cand.is_dir() {
            project_local = Some(cand);
            break;
        }
        walker = d.parent().map(|p| p.to_path_buf());
    }
    let resolved = match project_local {
        Some(p) => p,
        None => {
            let global = ship_dir.join("pipelines");
            if global.is_dir() { global } else { return Vec::new(); }
        }
    };
    let dir = resolved;
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            tracing::error!(error = %e, dir = %dir.display(), "read pipelines dir failed");
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !PIPELINE_EXTS.contains(&ext) {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let (description, step_count) = parse_pipeline_meta(&content);
        out.push(PipelineDef {
            name,
            description,
            step_count,
            active_runs: 0,
            total_runs: 0,
            success_runs: 0,
            failed_runs: 0,
            success_rate: 0.0,
            last_run_at: None,
            last_run_status: None,
            last_run_tone: None,
            last_run_slug: None,
            recent_outcomes: Vec::new(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Aggregate per-pipeline run stats from a PipelineRun iterator.
pub fn aggregate_pipeline_stats<'a>(
    defs: &mut [PipelineDef],
    jobs: impl IntoIterator<Item = &'a PipelineRun>,
) {
    let mut by_name: HashMap<String, Vec<&PipelineRun>> = HashMap::new();
    for job in jobs {
        let Some(pipe) = pipeline_name_for_job(job) else { continue };
        by_name.entry(pipe).or_default().push(job);
    }

    for def in defs.iter_mut() {
        let Some(runs) = by_name.get_mut(&def.name) else { continue };
        runs.sort_by_key(|r| std::cmp::Reverse(r.updated_at));

        def.total_runs = runs.len();
        def.active_runs = runs
            .iter()
            .filter(|j| matches!(j.status, PipelineRunStatus::Dispatched | PipelineRunStatus::GatePending | PipelineRunStatus::Pending))
            .count();
        def.success_runs = runs
            .iter()
            .filter(|j| matches!(j.status, PipelineRunStatus::Completed | PipelineRunStatus::Merged))
            .count();
        def.failed_runs = runs
            .iter()
            .filter(|j| matches!(j.status, PipelineRunStatus::Failed))
            .count();
        def.success_rate = if def.total_runs == 0 {
            0.0
        } else {
            def.success_runs as f32 / def.total_runs as f32
        };
        if let Some(latest) = runs.first() {
            def.last_run_at = Some(latest.updated_at.to_rfc3339());
            def.last_run_status = Some(status_token(&latest.status).to_string());
            def.last_run_tone = Some(status_tone(&latest.status).to_string());
            def.last_run_slug = Some(latest.slug.clone());
        }
        let mut recent: Vec<String> = runs
            .iter()
            .take(8)
            .map(|j| status_token(&j.status).to_string())
            .collect();
        recent.reverse();
        def.recent_outcomes = recent;
    }
}

/// Public alias — history view needs to group/filter runs by pipeline.
pub fn pipeline_name_for_job_public(job: &PipelineRun) -> Option<String> {
    pipeline_name_for_job(job)
}

fn pipeline_name_for_job(job: &PipelineRun) -> Option<String> {
    let (prefix, suffix) = job.slug.rsplit_once('-')?;
    if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(prefix.to_string())
    } else {
        None
    }
}

/// Parse pipeline content (JSONC or YAML) and extract metadata.
fn parse_pipeline_meta(content: &str) -> (String, usize) {
    let value: serde_json::Value = {
        let trimmed = content.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            // Use ship_substrate::fs_util instead of compiler::jsonc (no dep on compiler here).
            let json = ship_substrate::fs_util::strip_jsonc_comments(content);
            match serde_json::from_str(&json) {
                Ok(v) => v,
                Err(_) => return (String::new(), 0),
            }
        } else {
            match serde_yaml::from_str(content) {
                Ok(v) => v,
                Err(_) => return (String::new(), 0),
            }
        }
    };
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let step_count = value
        .get("steps")
        .and_then(|v| v.as_array())
        .map(|s| s.len())
        .or_else(|| {
            value
                .get("agents")
                .and_then(|v| v.as_array())
                .map(|s| s.len())
        })
        .unwrap_or(0);
    (description, step_count)
}

/// All known job records, filtered, sorted by `updated_at` descending.
pub fn list_pipeline_runs(ship_dir: &Path, filter: &Filter) -> Vec<PipelineRun> {
    let jobs = match crate::projections::pipeline_run::load_pipeline_runs_at(ship_dir) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!(error = %e, "load_pipeline_runs_at failed");
            HashMap::new()
        }
    };
    filter_and_sort(jobs, filter)
}

pub fn filter_and_sort(jobs: HashMap<String, PipelineRun>, filter: &Filter) -> Vec<PipelineRun> {
    let mut out: Vec<PipelineRun> = jobs.into_values().filter(|j| filter.matches(j)).collect();
    out.sort_by_key(|r| std::cmp::Reverse(r.updated_at));
    out
}

/// Partition a run list into `(active, history)` by [`RunCategory`].
pub fn partition_runs(runs: Vec<PipelineRun>) -> (Vec<PipelineRun>, Vec<PipelineRun>) {
    let mut active = Vec::new();
    let mut history = Vec::new();
    for run in runs {
        match RunCategory::of(&run.status) {
            RunCategory::Active => active.push(run),
            RunCategory::History => history.push(run),
        }
    }
    (active, history)
}
