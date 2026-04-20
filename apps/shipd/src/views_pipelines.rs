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
    Filter, PipelineDef, aggregate_pipeline_stats, list_pipeline_defs, list_pipeline_runs,
    partition_runs, status_token,
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
pub struct StatusPill {
    pub label: &'static str,
    pub tone: &'static str,
    pub count: usize,
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
    pub status_text: &'static str,
    pub duration: String,
    pub updated_at: String,
    pub steps: Vec<StepView>,
    /// Per-run git stats (active runs only; history gets defaults).
    pub branch_stats: BranchStats,
    /// Outcome segment tokens, one per defined step: `done | failed |
    /// pending | running`. Populated for history rows.
    pub segments: Vec<&'static str>,
    /// Provider name (e.g. "anthropic").
    pub provider: String,
    /// Model name (e.g. "claude-sonnet-4-20250514").
    pub model: String,
    /// Blocker reason, if status is blocked.
    pub blocker: String,
    /// Error message, if status is failed.
    pub error: String,
    /// Job dependencies (slugs).
    pub depends_on: Vec<String>,
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
    /// Status pill row shown above tabs — one per state.
    pub status_pills: Vec<StatusPill>,
    /// Provider IDs detected on PATH. Feeds the dispatch dialog's
    /// optional provider override. Unavailable providers are excluded.
    pub providers: Vec<String>,
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
        JobStatus::Cancelled => "cancelled",
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
        JobStatus::Cancelled => "muted",
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
        JobStatus::Failed | JobStatus::Cancelled => {
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

fn status_text_label(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Pending => "pending",
        JobStatus::Dispatched => "running",
        JobStatus::GatePending => "gate",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Merged => "merged",
        JobStatus::Blocked => "blocked",
        JobStatus::Cancelled => "cancelled",
    }
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
        status_text: status_text_label(&job.status),
        duration: format_duration((job.updated_at - job.created_at).num_seconds()),
        updated_at: job.updated_at.to_rfc3339(),
        steps,
        branch_stats: stats,
        segments: derive_segments(job),
        provider: job.provider.clone().unwrap_or_default(),
        model: job.model.clone().unwrap_or_default(),
        blocker: job.blocker.clone().unwrap_or_default(),
        error: job.error.clone().unwrap_or_default(),
        depends_on: job.depends_on.clone().unwrap_or_default(),
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

pub fn build_ctx(params: &PipelinesParams, ship_dir: &std::path::Path) -> PipelinesCtx {
    let mut defs = list_pipeline_defs(ship_dir);
    // Unfiltered pass populates the agent-chip row; the active
    // filter only narrows rendered runs, not the chip options.
    let all_runs = list_pipeline_runs(ship_dir, &Filter::default());
    aggregate_pipeline_stats(&mut defs, all_runs.iter());
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

    // Per-state counts across all runs (active + history) for the
    // pill row above the tabs.
    let all_by_status = |want: JobStatus| -> usize {
        active
            .iter()
            .chain(history.iter())
            .filter(|j| std::mem::discriminant(&j.status) == std::mem::discriminant(&want))
            .count()
    };
    let status_pills = vec![
        StatusPill { label: "Pending", tone: "muted", count: all_by_status(JobStatus::Pending) },
        StatusPill { label: "Running", tone: "info", count: all_by_status(JobStatus::Dispatched) },
        StatusPill { label: "Gate", tone: "warning", count: all_by_status(JobStatus::GatePending) },
        StatusPill { label: "Done", tone: "success", count: all_by_status(JobStatus::Completed) },
        StatusPill { label: "Merged", tone: "accent", count: all_by_status(JobStatus::Merged) },
        StatusPill { label: "Blocked", tone: "warning", count: all_by_status(JobStatus::Blocked) },
        StatusPill { label: "Failed", tone: "destructive", count: all_by_status(JobStatus::Failed) },
    ];

    let providers: Vec<String> = runtime::agent_export::PROVIDERS
        .iter()
        .filter(|d| runtime::detect_binary(d.binary))
        .map(|d| d.id.to_string())
        .collect();

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
        status_pills,
        providers,
    }
}

/// Try rendering a template from `.ship/views/pipelines/<file>`.
/// Returns `None` if the view directory or file does not exist.
fn try_render_view(file: &str, ctx: &PipelinesCtx) -> Option<String> {
    let (view_dir, _source) = crate::views::resolve_view_dir("pipelines")?;
    let source = std::fs::read_to_string(view_dir.join(file)).ok()?;
    let template_name = format!("pipelines/{file}");
    templates::render_view_source(&template_name, &source, ctx).ok()
}

// ── Run detail ──────────────────────────────────────────────────────────────

/// Per-run detail context — job record + step observations + timeline.
#[derive(Serialize)]
pub struct RunDetailCtx {
    pub slug: String,
    pub run: RunView,
    /// Ordered step observations derived from step.observed events.
    /// Carries verdict, trigger, exit_code, duration, log_tail.
    pub observations: Vec<StepObservationRow>,
    /// Recent events for this job, oldest first, capped.
    pub timeline: Vec<TimelineRow>,
}

#[derive(Serialize)]
pub struct StepObservationRow {
    pub step_name: String,
    pub step_index: usize,
    pub trigger: String,
    pub verdict: String,
    pub succeeded: bool,
    pub done_declared: bool,
    pub exit_code: Option<i32>,
    pub duration_secs: u64,
    pub commits: u32,
    pub reason: Option<String>,
    pub log_tail: Option<String>,
}

#[derive(Serialize)]
pub struct TimelineRow {
    pub event_type: String,
    pub at: String,
    pub summary: String,
}

/// Build run-detail context for `slug`. Returns `None` when no job
/// record matches.
pub fn build_run_detail_ctx(slug: &str, ship_dir: &std::path::Path) -> Option<RunDetailCtx> {
    let jobs = runtime::projections::job::load_jobs_at(ship_dir).ok()?;
    let job = jobs.values().find(|j| j.slug == slug)?;
    let run = to_run_view(job, true);
    let events = runtime::db::events::query_recent_events_for_entity_at(
        &ship_dir.join("platform.db"),
        &job.job_id,
        500,
    )
    .unwrap_or_default();

    let mut observations: Vec<StepObservationRow> = events
        .iter()
        .filter(|e| e.event_type == "step.observed")
        .filter_map(|e| {
            serde_json::from_str::<runtime::events::job::StepObservedPayload>(&e.payload_json)
                .ok()
                .map(|p| StepObservationRow {
                    step_name: p.step_name,
                    step_index: p.step_index,
                    trigger: format!("{:?}", p.trigger),
                    verdict: format!("{:?}", p.verdict),
                    succeeded: p.succeeded,
                    done_declared: p.done_declared,
                    exit_code: p.exit_code,
                    duration_secs: p.duration_secs,
                    commits: p.commits_since_start,
                    reason: p.reason,
                    log_tail: p.log_tail,
                })
        })
        .collect();
    observations.sort_by_key(|o| o.step_index);

    let timeline: Vec<TimelineRow> = events
        .iter()
        .rev() // oldest first
        .map(|e| TimelineRow {
            event_type: e.event_type.clone(),
            at: e.created_at.to_rfc3339(),
            summary: event_summary(e),
        })
        .collect();

    Some(RunDetailCtx { slug: slug.to_string(), run, observations, timeline })
}

fn event_summary(env: &runtime::events::EventEnvelope) -> String {
    let v: serde_json::Value =
        serde_json::from_str(&env.payload_json).unwrap_or(serde_json::Value::Null);
    match env.event_type.as_str() {
        "job.created" => v.get("pipeline_name").and_then(|x| x.as_str())
            .map(|s| format!("pipeline: {s}")).unwrap_or_default(),
        "job.dispatched" => "job dispatched".to_string(),
        "job.completed" => "job completed".to_string(),
        "job.merged" => "job merged".to_string(),
        "job.failed" => v.get("error").and_then(|x| x.as_str())
            .map(|s| s.lines().next().unwrap_or("failed").to_string())
            .unwrap_or("failed".to_string()),
        "step.dispatched" => v.get("step_id").and_then(|x| x.as_str())
            .map(|s| format!("→ {s}")).unwrap_or_default(),
        "step.observed" => {
            let name = v.get("step_name").and_then(|x| x.as_str()).unwrap_or("?");
            let verdict = v.get("verdict").and_then(|x| x.as_str()).unwrap_or("?");
            format!("{name} — {verdict}")
        }
        "process.spawned" => v.get("cmd").and_then(|x| x.as_str())
            .map(|s| format!("spawned {s}")).unwrap_or_default(),
        "process.exited" => v.get("exit_code").and_then(|x| x.as_i64())
            .map(|c| format!("exit {c}")).unwrap_or("exited".to_string()),
        "process.signal_sent" => v.get("signal").and_then(|x| x.as_str())
            .map(|s| format!("signal {s}")).unwrap_or_default(),
        _ => String::new(),
    }
}

/// Per-pipeline focus context — full definition + runs of that pipeline.
#[derive(Serialize)]
pub struct PipelineFocusCtx {
    pub name: String,
    pub def: PipelineDef,
    /// Raw pipeline document as JSON (converted from YAML/JSONC) — the
    /// template unpacks steps, gate rules, vars, auto_merge, step_specs.
    pub doc: serde_json::Value,
    /// Verbatim file contents on disk (JSONC or YAML). Fed into
    /// <ship-code-editor> for the raw-edit mode. Writes go back to
    /// PUT /api/workspaces/{workspace_id}/pipelines/{name}.
    pub raw: String,
    /// On-disk file extension — "jsonc" (preferred) or "yaml".
    pub format: String,
    /// Workspace id (= current git branch at the project root) that
    /// the editor PUTs back to. None when the daemon can't resolve
    /// a current workspace.
    pub workspace_id: Option<String>,
    /// Runs of this pipeline, newest first.
    pub runs: Vec<RunView>,
    /// Provider IDs on PATH for the dispatch form.
    pub providers: Vec<String>,
}

fn find_pipeline_file(name: &str) -> Option<std::path::PathBuf> {
    let mut walker = std::env::current_dir().ok()?;
    loop {
        let dir = walker.join(".ship").join("pipelines");
        if dir.is_dir() {
            for ext in ["yaml", "yml", "jsonc", "json"] {
                let cand = dir.join(format!("{name}.{ext}"));
                if cand.exists() {
                    return Some(cand);
                }
            }
            return None;
        }
        if !walker.pop() {
            return None;
        }
    }
}

pub fn build_focus_ctx(name: &str, ship_dir: &std::path::Path) -> Option<PipelineFocusCtx> {
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

    // Reuse list_pipeline_defs + aggregate to get the same stats shape.
    let mut defs = list_pipeline_defs(ship_dir);
    let all_runs = list_pipeline_runs(ship_dir, &Filter::default());
    aggregate_pipeline_stats(&mut defs, all_runs.iter());
    let def = defs.into_iter().find(|d| d.name == name)?;

    let runs: Vec<RunView> = all_runs
        .iter()
        .filter(|j| {
            j.slug.rsplit_once('-')
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

    let providers: Vec<String> = runtime::agent_export::PROVIDERS
        .iter()
        .filter(|d| runtime::detect_binary(d.binary))
        .map(|d| d.id.to_string())
        .collect();

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jsonc")
        .to_string();
    let workspace_id = current_project_workspace_id();

    Some(PipelineFocusCtx {
        name: name.to_string(),
        def,
        doc,
        raw,
        format,
        workspace_id,
        runs,
        providers,
    })
}

/// The workspace id the editor PUTs to — current git branch at the
/// project root the daemon is attached to. None when the project
/// isn't a git repo or the branch can't be determined.
fn current_project_workspace_id() -> Option<String> {
    let project_ship = runtime::project::get_project_dir(None).ok()?;
    let root = project_ship.parent()?;
    let out = std::process::Command::new("git")
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

/// GET /api/runtime/view/pipelines/pipeline/{name} — per-pipeline focus shell.
pub async fn pipeline_focus_shell(
    State(state): State<crate::rest_api::ApiState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some(ctx) = build_focus_ctx(&name, state.ship_dir()) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let Some((view_dir, _)) = crate::views::resolve_view_dir("pipelines") else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let path = view_dir.join("focus").join("index.html.j2");
    let source = std::fs::read_to_string(&path).map_err(|_| StatusCode::NOT_FOUND)?;
    templates::render_view_source("pipelines/focus/index.html.j2", &source, &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "pipeline focus render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/pipelines/run/{slug} — run detail shell.
pub async fn pipelines_run_shell(
    State(state): State<crate::rest_api::ApiState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some(ctx) = build_run_detail_ctx(&slug, state.ship_dir()) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let Some((view_dir, _)) = crate::views::resolve_view_dir("pipelines") else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let path = view_dir.join("run").join("index.html.j2");
    let source = std::fs::read_to_string(&path).map_err(|_| StatusCode::NOT_FOUND)?;
    templates::render_view_source("pipelines/run/index.html.j2", &source, &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "pipelines run detail render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/pipelines — full shell, minijinja-rendered.
pub async fn pipelines_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<PipelinesParams>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_ctx(&params, state.ship_dir());
    // Try .ship/views/pipelines/index.html.j2 first (new canonical location).
    if let Some(html) = try_render_view("index.html.j2", &ctx) {
        return Ok(Html(html));
    }
    // Fallback to embedded template.
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
    // Prefer the user-level .ship/views/pipelines/partials/content.html.j2
    // (authoritative — matches what pipelines_shell renders). Fall back to
    // the embedded template only if the view dir isn't present.
    if let Some(html) = try_render_view("partials/content.html.j2", &ctx) {
        return Some(html);
    }
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
