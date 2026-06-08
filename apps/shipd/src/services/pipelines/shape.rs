//! Shared view-shape types + per-record derivations for the pipelines
//! view surface. All pure — no I/O beyond the `branch_stats` subprocess
//! call reused from the parent module.
//!
//! These types are serialized straight into the registry op payloads
//! that `.ship/views/pipelines/*` templates consume.

use ship_pipeline_runtime::projections::pipeline_run::{PipelineRun, PipelineRunStatus};
use serde::Serialize;
use std::path::{Path, PathBuf};

use super::view_state::{BranchStats, branch_stats};

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Stats {
    pub definitions: usize,
    pub active: usize,
    pub completed: usize,
    pub failed: usize,
    pub history: usize,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct StatusPill {
    pub label: &'static str,
    pub tone: &'static str,
    pub count: usize,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct StepView {
    pub id: String,
    pub step_type: String,
    pub state: &'static str,
    pub agent: Option<String>,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RunView {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub state: &'static str,
    pub tone: &'static str,
    pub status_text: &'static str,
    pub duration: String,
    pub updated_at: String,
    pub steps: Vec<StepView>,
    pub branch_stats: BranchStats,
    pub segments: Vec<&'static str>,
    pub harness: String,
    pub model: String,
    pub blocker: String,
    pub error: String,
    pub depends_on: Vec<String>,
}

pub fn run_state_token(s: &PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::Pending => "pending",
        PipelineRunStatus::Dispatched => "dispatched",
        PipelineRunStatus::GatePending => "gate-pending",
        PipelineRunStatus::Completed => "completed",
        PipelineRunStatus::Failed => "failed",
        PipelineRunStatus::Merged => "merged",
        PipelineRunStatus::Blocked => "blocked",
        PipelineRunStatus::Cancelled => "cancelled",
    }
}

pub fn tone_for(s: &PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::Dispatched => "info",
        PipelineRunStatus::GatePending => "warning",
        PipelineRunStatus::Completed => "success",
        PipelineRunStatus::Failed => "destructive",
        PipelineRunStatus::Merged => "accent",
        PipelineRunStatus::Blocked => "warning",
        PipelineRunStatus::Pending => "muted",
        PipelineRunStatus::Cancelled => "muted",
    }
}

pub fn status_text_label(s: &PipelineRunStatus) -> &'static str {
    match s {
        PipelineRunStatus::Pending => "pending",
        PipelineRunStatus::Dispatched => "running",
        PipelineRunStatus::GatePending => "gate",
        PipelineRunStatus::Completed => "completed",
        PipelineRunStatus::Failed => "failed",
        PipelineRunStatus::Merged => "merged",
        PipelineRunStatus::Blocked => "blocked",
        PipelineRunStatus::Cancelled => "cancelled",
    }
}

pub fn format_duration(secs: i64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

pub fn format_duration_between(
    start: &chrono::DateTime<chrono::Utc>,
    end: &chrono::DateTime<chrono::Utc>,
) -> String {
    let secs = (end.signed_duration_since(*start)).num_seconds().max(0);
    format_duration(secs)
}

/// Derive per-step outcome segment tokens from what the projection
/// exposes: `steps.len()`, `completed_steps`, overall `status`. No
/// per-step verdict is required.
pub fn derive_segments(job: &PipelineRun) -> Vec<&'static str> {
    let total = job
        .steps
        .as_ref()
        .map(|p| p.steps.len())
        .unwrap_or(job.completed_steps.len());
    if total == 0 {
        return Vec::new();
    }
    let completed = job.completed_steps.len().min(total);
    let remaining = total - completed;
    let mut out: Vec<&'static str> = (0..completed).map(|_| "done").collect();
    match job.status {
        PipelineRunStatus::Completed | PipelineRunStatus::Merged => {
            out.resize(out.len() + remaining, "done");
        }
        PipelineRunStatus::Failed | PipelineRunStatus::Cancelled => {
            if remaining > 0 {
                out.push("failed");
                out.resize(out.len() + remaining - 1, "pending");
            }
        }
        PipelineRunStatus::Dispatched | PipelineRunStatus::GatePending => {
            if remaining > 0 {
                out.push("running");
                out.resize(out.len() + remaining - 1, "pending");
            }
        }
        PipelineRunStatus::Pending | PipelineRunStatus::Blocked => {
            out.resize(out.len() + remaining, "pending");
        }
    }
    out
}

/// Build a `RunView` for a single job record. When `include_stats`,
/// shells out for per-worktree `BranchStats`; otherwise zero-defaults.
pub fn to_run_view(job: &PipelineRun, include_stats: bool) -> RunView {
    let run_failed = matches!(job.status, PipelineRunStatus::Failed);
    let steps = job
        .steps
        .as_ref()
        .map(|p| {
            p.steps
                .iter()
                .map(|step| {
                    let state = if job.completed_steps.contains(&step.id) {
                        "completed"
                    } else if job.current_step.as_deref() == Some(step.id.as_str()) {
                        if run_failed { "failed" } else { "running" }
                    } else {
                        "pending"
                    };
                    StepView {
                        id: step.id.clone(),
                        step_type: format!("{:?}", step.step_type).to_lowercase(),
                        state,
                        agent: step.agent.clone(),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let stats = if include_stats {
        job.worktree
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|p| branch_stats(&PathBuf::from(p)))
            .unwrap_or_default()
    } else {
        BranchStats::default()
    };

    RunView {
        pipeline_run_id: job.pipeline_run_id.clone(),
        slug: job.slug.clone(),
        agent: job.agent.clone(),
        branch: job.branch.clone(),
        state: run_state_token(&job.status),
        tone: tone_for(&job.status),
        status_text: status_text_label(&job.status),
        duration: format_duration((job.updated_at - job.created_at).num_seconds()),
        updated_at: job.updated_at.to_rfc3339(),
        steps,
        branch_stats: stats,
        segments: derive_segments(job),
        harness: job.harness.clone().unwrap_or_default(),
        model: job.model.clone().unwrap_or_default(),
        blocker: job.blocker.clone().unwrap_or_default(),
        error: job.error.clone().unwrap_or_default(),
        depends_on: job.depends_on.clone().unwrap_or_default(),
    }
}

/// Workspace id = current git branch at the project root the daemon
/// is attached to. `None` when project isn't a git repo.
pub fn current_project_workspace_id() -> Option<String> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let root = project_ship.parent()?;
    let out = ship_substrate::spawn_bounded::Cmd::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if branch.is_empty() { None } else { Some(branch) }
}

/// Locate a pipeline file on disk. The optional `ship_dir` hint is
/// preferred when the daemon has an explicit project root (e.g. tests
/// that seed a temp `.ship/pipelines/`); falling back to walking up from
/// CWD keeps the live read path working when the daemon was started
/// against the user's worktree. JSONC / JSON / YAML — the loader walks
/// extensions in the order the runtime parser prefers.
pub fn find_pipeline_file(name: &str, ship_dir: Option<&Path>) -> Option<std::path::PathBuf> {
    let probe = |root: &Path| -> Option<std::path::PathBuf> {
        for ext in ["yaml", "yml", "jsonc", "json"] {
            let cand = root.join(format!("{name}.{ext}"));
            if cand.exists() {
                return Some(cand);
            }
        }
        None
    };
    if let Some(ship) = ship_dir {
        let composed = ship_store::compose::composed_root_for(ship).join("pipelines");
        if composed.is_dir() {
            if let Some(found) = probe(&composed) {
                return Some(found);
            }
        }
        let raw = ship.join("pipelines");
        if raw.is_dir() {
            if let Some(found) = probe(&raw) {
                return Some(found);
            }
        }
    }
    let mut walker = std::env::current_dir().ok()?;
    loop {
        let ship = ship_substrate::ShipHome::project_ship_dir(&walker);
        let dir = ship_store::compose::composed_root_for(&ship).join("pipelines");
        if dir.is_dir() {
            if let Some(found) = probe(&dir) {
                return Some(found);
            }
            return None;
        }
        if !walker.pop() {
            return None;
        }
    }
}

pub fn harnesses_on_path() -> Vec<String> {
    ship_agents::agent_domain::harnesses::HARNESSES
        .iter()
        .filter(|d| ship_agents::agent_domain::harnesses::detect_binary(d.binary))
        .map(|d| d.id.to_string())
        .collect()
}
