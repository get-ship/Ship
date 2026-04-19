//! Centralized read helpers for daemon-rendered views.
//!
//! View handlers should never re-walk the filesystem or issue scattered
//! runtime calls. Everything they need arrives through this module, so
//! caching, error handling, and filter semantics live in one place.
//!
//! ## Scope in R0a
//!
//! R0a ships only what the pipelines pilot (R0b) will consume. That is:
//!
//! - Pipeline definitions from `.ship/pipelines/*.yaml`
//! - Pipeline runs (via `runtime::projections::job::load_jobs`)
//! - A shared [`Filter`] struct for query-string-driven view filters
//!
//! Helpers for specs, git, and settings are intentionally absent. They
//! will be added alongside the pilots that need them — not as empty
//! stubs. This keeps the module honest about what is actually wired up.

// Agents + Skills state lives in sibling modules to respect the 300-line
// cap on this file. Re-exports let callers use `views::state::list_agents`
// without knowing about the split.
pub use super::agents_state::{AgentDetail, AgentSummary, list_agents, load_agent};
pub use super::skills_state::{
    SkillDetail, SkillKind, SkillSummary, list_skills, load_skill,
    load_skill_file,
};

use runtime::projections::job::{JobRecord, JobStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

/// Definition of a pipeline parsed from a `.ship/pipelines/*.yaml` file.
#[derive(Clone, Debug, Serialize)]
pub struct PipelineDef {
    pub name: String,
    pub description: String,
    pub step_count: usize,
}

/// Categorization of a run for tab bucketing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunCategory {
    Active,
    History,
}

impl RunCategory {
    pub fn of(status: &JobStatus) -> Self {
        match status {
            JobStatus::Dispatched | JobStatus::GatePending | JobStatus::Pending => {
                RunCategory::Active
            }
            JobStatus::Completed
            | JobStatus::Failed
            | JobStatus::Merged
            | JobStatus::Blocked
            | JobStatus::Cancelled => RunCategory::History,
        }
    }
}

/// A view filter, typically populated from a query string. Every field is
/// optional so callers can pass an empty [`Filter::default`] for unfiltered
/// results.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Filter {
    /// Case-insensitive substring match applied to slug, agent, and
    /// pipeline name where relevant.
    #[serde(default)]
    pub q: Option<String>,
    /// Exact match on `JobRecord::agent`.
    #[serde(default)]
    pub agent: Option<String>,
    /// Exact match on pipeline name (derived from spec path's parent).
    #[serde(default)]
    pub pipeline: Option<String>,
    /// Restrict to a specific status, e.g. `failed`.
    #[serde(default)]
    pub status: Option<String>,
}

impl Filter {
    /// True if no filter fields are set.
    pub fn is_empty(&self) -> bool {
        self.q.is_none() && self.agent.is_none() && self.pipeline.is_none() && self.status.is_none()
    }

    fn matches(&self, job: &JobRecord) -> bool {
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
        true
    }
}

/// Normalized kebab-case token for `JobStatus`. Stable for query strings
/// and template comparisons.
pub fn status_token(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Pending => "pending",
        JobStatus::Dispatched => "dispatched",
        JobStatus::GatePending => "gate-pending",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Merged => "merged",
        JobStatus::Blocked => "blocked",
        JobStatus::Cancelled => "cancelled",
    }
}

/// Recognised pipeline file extensions, preferred first.
const PIPELINE_EXTS: &[&str] = &["jsonc", "yaml"];

/// List pipeline definitions from `<ship_dir>/pipelines/`, sorted by name.
///
/// Missing directories return an empty vec — having no pipelines is a
/// valid project state, not an error. Other IO errors are logged and
/// produce an empty vec rather than propagating to the view handler.
pub fn list_pipeline_defs(ship_dir: &Path) -> Vec<PipelineDef> {
    let dir = ship_dir.join("pipelines");
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
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Parse pipeline content (JSONC or YAML) and extract metadata.
fn parse_pipeline_meta(content: &str) -> (String, usize) {
    let value: serde_json::Value = {
        let trimmed = content.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            let json = compiler::jsonc::strip_jsonc_comments(content);
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
pub fn list_pipeline_runs(ship_dir: &Path, filter: &Filter) -> Vec<JobRecord> {
    let jobs = match runtime::projections::job::load_jobs_at(ship_dir) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!(error = %e, "load_jobs_at failed");
            HashMap::new()
        }
    };
    filter_and_sort(jobs, filter)
}

fn filter_and_sort(jobs: HashMap<String, JobRecord>, filter: &Filter) -> Vec<JobRecord> {
    let mut out: Vec<JobRecord> = jobs.into_values().filter(|j| filter.matches(j)).collect();
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    out
}

/// Partition a run list into `(active, history)` by [`RunCategory`].
/// Preserves relative ordering within each bucket.
pub fn partition_runs(runs: Vec<JobRecord>) -> (Vec<JobRecord>, Vec<JobRecord>) {
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

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
