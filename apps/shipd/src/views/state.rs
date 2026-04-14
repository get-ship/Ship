//! Centralized read helpers for daemon-rendered views.
//!
//! View handlers should never re-walk the filesystem or issue scattered
//! runtime calls. Everything they need arrives through this module, so
//! caching, error handling, and filter semantics live in one place.

use runtime::projections::job::{JobRecord, JobStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

// Agents + Skills state lives in sibling modules to respect the 300-line
// cap on this file. Re-exports let callers use `views::state::list_agents`
// without caring about the submodule split.
pub use super::agents_state::{AgentDetail, AgentSummary, list_agents, load_agent};
pub use super::skills_state::{
    SkillDetail, SkillKind, SkillSummary, list_skills, load_skill,
};

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
            | JobStatus::Blocked => RunCategory::History,
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

/// Normalized kebab-case token for `JobStatus`.
pub fn status_token(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Pending => "pending",
        JobStatus::Dispatched => "dispatched",
        JobStatus::GatePending => "gate-pending",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Merged => "merged",
        JobStatus::Blocked => "blocked",
    }
}

/// List pipeline definitions from `<ship_dir>/pipelines/*.yaml`, sorted by name.
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
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
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

fn parse_pipeline_meta(yaml: &str) -> (String, usize) {
    let value: serde_yaml::Value = match serde_yaml::from_str(yaml) {
        Ok(v) => v,
        Err(_) => return (String::new(), 0),
    };
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let step_count = value
        .get("steps")
        .and_then(|v| v.as_sequence())
        .map(|s| s.len())
        .or_else(|| {
            value
                .get("agents")
                .and_then(|v| v.as_sequence())
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
