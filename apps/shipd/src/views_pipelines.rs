//! Pipelines view — minijinja-rendered shell + Datastar SSE content patches.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/pipelines` — full-page HTML shell. Delivered
//!   once on load; contains the tab strip, stats, and an empty
//!   `#pipelines-content` that bootstraps an SSE subscription.
//! - `GET /api/runtime/view/pipelines/sse` — Datastar SSE stream that
//!   pushes `#pipelines-content` fragments whenever the underlying
//!   state changes (kernel `job.*` / `step.*` events).
//!
//! All rendering goes through `crate::views::templates`; all state reads
//! go through `crate::views::state`. No inline HTML lives here.

use axum::extract::{Query, State};
use axum::response::Html;
use runtime::events::ActorConfig;
use runtime::projections::job::{JobRecord, JobStatus};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::views::state::{
    Filter, PipelineDef, list_pipeline_defs, list_pipeline_runs, partition_runs, status_token,
};
use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

/// Query string for both shell + SSE endpoints. `tab` drives the content
/// dispatch; the flattened `Filter` fields drive the history filter chips.
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
}

#[derive(Serialize)]
struct Stats {
    definitions: usize,
    active: usize,
    completed: usize,
    failed: usize,
    history: usize,
}

#[derive(Serialize)]
struct StepView {
    id: String,
    step_type: String,
    state: &'static str,
    agent: Option<String>,
}

#[derive(Serialize)]
struct RunView {
    job_id: String,
    slug: String,
    agent: String,
    state: &'static str,
    tone: &'static str,
    duration: String,
    updated_at: String,
    steps: Vec<StepView>,
}

#[derive(Serialize)]
struct PipelinesCtx {
    active_tab: String,
    filter: Filter,
    stats: Stats,
    defs: Vec<PipelineDef>,
    active_runs: Vec<RunView>,
    history_runs: Vec<RunView>,
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

fn to_run_view(job: &JobRecord) -> RunView {
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

    RunView {
        job_id: job.job_id.clone(),
        slug: job.slug.clone(),
        agent: job.agent.clone(),
        state: run_state_token(&job.status),
        tone: tone_for(&job.status),
        duration: format_duration((job.updated_at - job.created_at).num_seconds()),
        updated_at: job.updated_at.to_rfc3339(),
        steps,
    }
}

fn build_ctx(params: &PipelinesParams) -> PipelinesCtx {
    let defs = list_pipeline_defs();
    let runs = list_pipeline_runs(&params.filter);
    let (active, history) = partition_runs(runs);

    // History can be further narrowed by `status` if set. `list_pipeline_runs`
    // already applied the filter, but partition_runs doesn't know about it —
    // the filter restricts history to matching statuses via status_token.
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
        .map(to_run_view)
        .collect();
    let active_runs: Vec<RunView> = active.iter().map(to_run_view).collect();

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

    PipelinesCtx {
        active_tab: params.active_tab(),
        filter: params.filter.clone(),
        stats,
        defs,
        active_runs,
        history_runs,
    }
}

/// GET /api/runtime/view/pipelines — full shell, minijinja-rendered.
pub async fn pipelines_shell(
    State(_state): State<crate::rest_api::ApiState>,
    Query(params): Query<PipelinesParams>,
) -> Html<String> {
    let ctx = build_ctx(&params);
    let html = templates::render("pipelines/shell.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<pre>shell render error: {e}</pre>"));
    Html(html)
}

/// GET /api/runtime/view/pipelines/sse — Datastar patches for the
/// `#pipelines-content` fragment, re-rendered on any `job.*`/`step.*`
/// event.
pub async fn pipelines_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<PipelinesParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();

    let actor_id = format!(
        "pipelines-sse-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    };
    let mailbox = state
        .kernel()
        .lock()
        .await
        .spawn_actor(&actor_id, config)
        .ok()
        .map(|(_store, mb)| mb);
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        // Initial render.
        let html = render_content(&params);
        if vs.patch("#pipelines-content", html).await.is_err() {
            return;
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let html = render_content(&params);
            if vs.patch("#pipelines-content", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

fn render_content(params: &PipelinesParams) -> String {
    let ctx = build_ctx(params);
    templates::render("pipelines/content.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<div id=\"pipelines-content\"><pre>{e}</pre></div>"))
}

#[cfg(test)]
#[path = "views_pipelines_tests.rs"]
mod tests;
