//! Pipelines view — minijinja-rendered shell + Datastar SSE content patches.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/pipelines` — full-page HTML shell. Delivered
//!   once on load; contains the header, tab strip, agent-filter chip row,
//!   and an empty `#pipelines-content` that bootstraps an SSE
//!   subscription.
//! - `GET /api/runtime/view/pipelines/sse` — Datastar SSE stream that
//!   pushes `#pipelines-content` fragments whenever the underlying
//!   state changes (kernel `job.*` / `step.*` events).
//!
//! All rendering goes through `crate::views::templates`; all state reads
//! go through `crate::views::state` + `crate::views_pipelines_data`.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use runtime::events::ActorConfig;
use runtime::gen_ulid;
use runtime::projections::job::{JobRecord, JobStatus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

use crate::views::state::{
    Filter, PipelineDef, list_pipeline_defs, list_pipeline_runs, partition_runs, status_token,
};
use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;
use crate::views_pipelines_data::{BranchStats, branch_stats};

/// Query string for both shell + SSE endpoints. `tab` drives the content
/// dispatch; the flattened `Filter` fields drive filter chips.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct PipelinesParams {
    #[serde(default)]
    pub tab: Option<String>,
    #[serde(flatten)]
    pub filter: Filter,
}

impl PipelinesParams {
    pub fn active_tab(&self) -> String {
        self.tab
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "active".to_string())
    }

    /// Selected agent filter (empty string = "All agents").
    pub fn agent_filter(&self) -> String {
        self.filter.agent.clone().unwrap_or_default()
    }
}

#[derive(Serialize)]
pub struct Stats {
    pub definitions: usize,
    pub active: usize,
    pub completed: usize,
    pub failed: usize,
    pub history: usize,
}

#[derive(Serialize)]
pub struct StepView {
    pub id: String,
    pub step_type: String,
    pub state: &'static str,
    pub agent: Option<String>,
}

#[derive(Serialize)]
pub struct RunView {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub state: &'static str,
    pub tone: &'static str,
    pub duration: String,
    pub updated_at: String,
    pub steps: Vec<StepView>,
    /// Per-run git stats (active runs only; history gets defaults).
    pub branch_stats: BranchStats,
    /// Outcome segment tokens, one per defined step: `done | failed |
    /// pending | running`. Populated for history rows.
    pub segments: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct PipelinesCtx {
    pub active_tab: String,
    pub filter: Filter,
    pub stats: Stats,
    /// Subtitle string for the view header, e.g.
    /// `"3 definitions · 2 active · 14 in history"`.
    pub stats_subtitle: String,
    /// Currently-selected agent-filter chip (empty = "All agents").
    pub agent_filter: String,
    /// Distinct agent names observed across all runs, sorted.
    pub agents: Vec<String>,
    pub defs: Vec<PipelineDef>,
    pub active_runs: Vec<RunView>,
    pub history_runs: Vec<RunView>,
}

fn run_state_token(s: &JobStatus) -> &'static str {
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

fn tone_for(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Dispatched => "info",
        JobStatus::GatePending => "warning",
        JobStatus::Completed => "success",
        JobStatus::Failed => "destructive",
        JobStatus::Merged => "accent",
        JobStatus::Blocked => "warning",
        JobStatus::Pending => "muted",
    }
}

fn format_duration(secs: i64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Derive per-step outcome segment tokens from the projection fields we
/// actually have: `steps.len()`, `completed_steps`, and overall
/// `status`. No per-step verdict required.
pub fn derive_segments(job: &JobRecord) -> Vec<&'static str> {
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
        JobStatus::Completed | JobStatus::Merged => {
            for _ in 0..remaining {
                out.push("done");
            }
        }
        JobStatus::Failed => {
            if remaining > 0 {
                out.push("failed");
                for _ in 1..remaining {
                    out.push("pending");
                }
            }
        }
        JobStatus::Dispatched | JobStatus::GatePending => {
            if remaining > 0 {
                out.push("running");
                for _ in 1..remaining {
                    out.push("pending");
                }
            }
        }
        JobStatus::Pending | JobStatus::Blocked => {
            for _ in 0..remaining {
                out.push("pending");
            }
        }
    }
    out
}

fn to_run_view(job: &JobRecord, include_stats: bool) -> RunView {
    let job_failed = matches!(job.status, JobStatus::Failed);
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
                        if job_failed { "failed" } else { "running" }
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
        job_id: job.job_id.clone(),
        slug: job.slug.clone(),
        agent: job.agent.clone(),
        branch: job.branch.clone(),
        state: run_state_token(&job.status),
        tone: tone_for(&job.status),
        duration: format_duration((job.updated_at - job.created_at).num_seconds()),
        updated_at: job.updated_at.to_rfc3339(),
        steps,
        branch_stats: stats,
        segments: derive_segments(job),
    }
}

fn pluralize(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn build_subtitle(stats: &Stats) -> String {
    format!(
        "{} · {} · {} in history",
        pluralize(stats.definitions, "definition"),
        pluralize(stats.active, "active"),
        stats.history
    )
}

fn build_ctx(params: &PipelinesParams, ship_dir: &std::path::Path) -> PipelinesCtx {
    let defs = list_pipeline_defs(ship_dir);
    // Unfiltered pass populates the agent-chip row; the active
    // filter only narrows rendered runs, not the chip options.
    let all_runs = list_pipeline_runs(ship_dir, &Filter::default());
    let agents: Vec<String> = all_runs
        .iter()
        .map(|j| j.agent.clone())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let runs = list_pipeline_runs(ship_dir, &params.filter);
    let (active, history) = partition_runs(runs);

    let history_runs: Vec<RunView> = history
        .iter()
        .take(50)
        .filter(|j| {
            params
                .filter
                .status
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| status_token(&j.status) == s)
                .unwrap_or(true)
        })
        .map(|j| to_run_view(j, false))
        .collect();
    let active_runs: Vec<RunView> = active.iter().map(|j| to_run_view(j, true)).collect();

    let completed = history
        .iter()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .count();
    let failed = history
        .iter()
        .filter(|j| matches!(j.status, JobStatus::Failed))
        .count();

    let stats = Stats {
        definitions: defs.len(),
        active: active.len(),
        completed,
        failed,
        history: history.len(),
    };
    let stats_subtitle = build_subtitle(&stats);

    PipelinesCtx {
        active_tab: params.active_tab(),
        filter: params.filter.clone(),
        stats,
        stats_subtitle,
        agent_filter: params.agent_filter(),
        agents,
        defs,
        active_runs,
        history_runs,
    }
}

/// GET /api/runtime/view/pipelines — full shell, minijinja-rendered.
pub async fn pipelines_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<PipelinesParams>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_ctx(&params, state.ship_dir());
    templates::render("pipelines/shell.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "pipelines shell render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/pipelines/sse — Datastar patches for the
/// `#pipelines-content` fragment, re-rendered on any `job.*`/`step.*`
/// event.
pub async fn pipelines_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<PipelinesParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();

    let actor_id = format!("pipelines-sse-{}", gen_ulid());
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => Some(mb),
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "spawn_actor failed for pipelines SSE");
            None
        }
    };
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        if let Some(html) = render_content(&params, &ship_dir) {
            if vs.patch("#pipelines-content", html).await.is_err() {
                return;
            }
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let Some(html) = render_content(&params, &ship_dir) else { continue };
            if vs.patch("#pipelines-content", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

fn render_content(params: &PipelinesParams, ship_dir: &std::path::Path) -> Option<String> {
    let ctx = build_ctx(params, ship_dir);
    match templates::render("pipelines/content.html.j2", &ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(error = %e, "pipelines content render failed");
            None
        }
    }
}

#[cfg(test)]
#[path = "views_pipelines_tests.rs"]
mod tests;
