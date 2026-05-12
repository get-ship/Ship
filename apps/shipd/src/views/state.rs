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
//! - Pipeline runs (via `ship_platform::projections::job::load_jobs`)
//! - A shared [`Filter`] struct for query-string-driven view filters
//!
//! Helpers for specs, git, and settings are intentionally absent. They
//! will be added alongside the pilots that need them — not as empty
//! stubs. This keeps the module honest about what is actually wired up.

// Agents + Skills state lives in sibling modules to respect the 300-line
// cap on this file. Re-exports let callers use `views::state::list_agents`
// without knowing about the split.
pub use crate::services::agents::view_state::{
    AgentCatalogs, AgentDetail, AgentSummary, CatalogEntry, Permissions,
    ToolPatternEntry, list_agent_catalogs, list_agents, load_agent,
};
pub use ship_platform::services::skills::view_state::{
    SkillDetail, SkillKind, SkillSummary, list_skills, load_skill,
    load_skill_file,
};

use ship_platform::projections::job::{JobRecord, JobStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

/// Definition of a pipeline parsed from a `.ship/pipelines/*.yaml` file,
/// plus usage stats aggregated from JobRecords referencing it.
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
    /// Populated by `aggregate_pipeline_stats`. Renders as a tiny strip
    /// in the list tile; each status gets a tone-tinted dot or bar.
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
        if let Some(p) = self.pipeline.as_deref().filter(|s| !s.is_empty()) {
            match pipeline_name_for_job(job) {
                Some(ref name) if name == p => {}
                _ => return false,
            }
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
/// `ship_dir` passed in is typically the global `~/.ship/`, which won't
/// contain pipelines — those live per-project at `<project>/.ship/
/// pipelines/`. When the direct path doesn't exist, walk up from the
/// daemon's CWD (matches what `pipeline_loader::load_pipeline_file`
/// does at dispatch time).
///
/// Missing directories return an empty vec — having no pipelines is a
/// valid project state, not an error.
pub fn list_pipeline_defs(ship_dir: &Path) -> Vec<PipelineDef> {
    // Prefer project-local .ship/pipelines/ discovered by walking up from
    // CWD. Fall back to the daemon's global ship_dir/pipelines only when
    // no project-local dir exists. The earlier "ship_dir first" order
    // broke silently when an empty ~/.ship/pipelines/ got created by
    // some unrelated bootstrapping — the daemon would then serve zero
    // pipelines despite a populated project dir.
    let mut walker = std::env::current_dir().ok();
    let mut project_local = None;
    while let Some(d) = walker.as_ref() {
        let cand = ship_substrate::ShipHome::project_ship_dir(&d).join("pipelines");
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

/// Aggregate per-pipeline run stats from a JobRecord iterator and
/// merge them into the provided defs. Each def's counts, success
/// rate, and last-run fields get populated in place.
pub fn aggregate_pipeline_stats<'a>(
    defs: &mut [PipelineDef],
    jobs: impl IntoIterator<Item = &'a ship_platform::projections::job::JobRecord>,
) {
    let mut by_name: std::collections::HashMap<String, Vec<&JobRecord>> =
        std::collections::HashMap::new();
    for job in jobs {
        let Some(pipe) = pipeline_name_for_job(job) else { continue };
        by_name.entry(pipe).or_default().push(job);
    }

    for def in defs.iter_mut() {
        let Some(runs) = by_name.get_mut(&def.name) else { continue };
        // Most recent first.
        runs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        def.total_runs = runs.len();
        def.active_runs = runs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Dispatched | JobStatus::GatePending | JobStatus::Pending))
            .count();
        def.success_runs = runs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
            .count();
        def.failed_runs = runs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Failed))
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
        // Recent outcomes for the list-tile sparkline. `runs` is
        // most-recent-first; the tile reads left→right = old→new, so we
        // take the newest 8 and reverse. Cap at 8 so long histories
        // don't blow up the tile footer.
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
pub fn pipeline_name_for_job_public(job: &JobRecord) -> Option<String> {
    pipeline_name_for_job(job)
}

/// Resolve a job's pipeline name from its slug. `ship run` generates
/// slug as `<pipeline_name>-<8-char-ulid>`; strip the trailing 8-char
/// alphanumeric suffix after the last `-`.
fn pipeline_name_for_job(job: &JobRecord) -> Option<String> {
    let (prefix, suffix) = job.slug.rsplit_once('-')?;
    if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(prefix.to_string())
    } else {
        None
    }
}

/// Short tone token for ship-badge / ship-status-dot tone attributes.
pub fn status_tone(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Dispatched => "info",
        JobStatus::GatePending => "warning",
        JobStatus::Completed => "success",
        JobStatus::Failed => "destructive",
        JobStatus::Merged => "accent",
        JobStatus::Blocked => "warning",
        JobStatus::Cancelled => "muted",
        JobStatus::Pending => "muted",
    }
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
    let jobs = match ship_platform::projections::job::load_jobs_at(ship_dir) {
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
