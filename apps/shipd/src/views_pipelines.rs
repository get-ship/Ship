//! Pipelines view — Datastar SSE for the three-tab pipelines dashboard.
//!
//! Renders: definitions (from `.ship/pipelines/*.yaml`), active runs, and history.
//! Subscribes to `job.*` / `step.*` kernel events and re-renders on change.

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use datastar::prelude::PatchElements;
use runtime::events::ActorConfig;
use runtime::projections::job::{JobRecord, JobStatus};
use std::collections::HashMap;
use std::convert::Infallible;

/// (name, description, step_count)
pub type PipelineDef = (String, String, usize);

/// GET /api/runtime/view/pipelines/sse
pub async fn pipelines_sse(
    State(state): State<crate::rest_api::ApiState>,
) -> Sse<impl futures::stream::Stream<Item = Result<Event, Infallible>>> {
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

    let stream = async_stream::stream! {
        let _guard = guard;

        // Initial full render.
        let html = build_view_html();
        yield Ok::<_, Infallible>(PatchElements::new(html).write_as_axum_sse_event());

        // Incremental: re-render on any job/step event.
        if let Some(mut mb) = mailbox {
            while let Some(_event) = mb.recv().await {
                let html = build_view_html();
                yield Ok::<_, Infallible>(PatchElements::new(html).write_as_axum_sse_event());
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

fn build_view_html() -> String {
    let jobs = runtime::projections::job::load_jobs().unwrap_or_default();
    let pipelines = load_pipeline_definitions();
    render_pipelines_view(&jobs, &pipelines)
}

/// Load pipeline definitions from `.ship/pipelines/*.yaml`.
pub fn load_pipeline_definitions() -> Vec<PipelineDef> {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let pipelines_dir = project_dir.join("pipelines");
    let entries = match std::fs::read_dir(&pipelines_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let (description, step_count) = parse_pipeline_meta(&content);
        out.push((name, description, step_count));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
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
    let step_count = if let Some(steps) = value.get("steps").and_then(|v| v.as_sequence()) {
        steps.len()
    } else if let Some(agents) = value.get("agents").and_then(|v| v.as_sequence()) {
        agents.len()
    } else {
        0
    };
    (description, step_count)
}

// ---- Rendering ----

pub fn render_pipelines_view(
    jobs: &HashMap<String, JobRecord>,
    pipelines: &[PipelineDef],
) -> String {
    let defs = render_definitions_tab(pipelines);
    let active = render_active_tab(jobs);
    let history = render_history_tab(jobs);

    let def_count = pipelines.len();
    let active_count = jobs.values().filter(|j| matches!(j.status, JobStatus::Dispatched | JobStatus::GatePending)).count();
    let completed_count = jobs.values().filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged)).count();
    let failed_count = jobs.values().filter(|j| matches!(j.status, JobStatus::Failed)).count();
    let total_runs = jobs.len();

    format!(
        r##"<div id="pipelines-content">
  <div class="stats">
    <div class="stat stat-accent-primary"><span class="stat-label">Definitions</span><span class="stat-value">{def_count}</span></div>
    <div class="stat stat-accent-info"><span class="stat-label">Active</span><span class="stat-value">{active_count}</span></div>
    <div class="stat stat-accent-success"><span class="stat-label">Completed</span><span class="stat-value">{completed_count}</span></div>
    <div class="stat stat-accent-destructive"><span class="stat-label">Failed</span><span class="stat-value">{failed_count}</span></div>
    <div class="stat"><span class="stat-label">Total runs</span><span class="stat-value">{total_runs}</span></div>
  </div>

  <div class="tabs-bar" role="tablist">
    <button class="tab-btn" role="tab" data-on:click="$tab = 'active'" data-attr-aria-selected="$tab === 'active' ? 'true' : 'false'">
      Active <span class="tab-count">{active_count}</span>
    </button>
    <button class="tab-btn" role="tab" data-on:click="$tab = 'definitions'" data-attr-aria-selected="$tab === 'definitions' ? 'true' : 'false'">
      Definitions <span class="tab-count">{def_count}</span>
    </button>
    <button class="tab-btn" role="tab" data-on:click="$tab = 'history'" data-attr-aria-selected="$tab === 'history' ? 'true' : 'false'">
      History <span class="tab-count">{h}</span>
    </button>
  </div>

  <section class="panel" data-show="$tab === 'active'">{active}</section>
  <section class="panel" data-show="$tab === 'definitions'">{defs}</section>
  <section class="panel" data-show="$tab === 'history'">{history}</section>
</div>"##,
        h = completed_count + failed_count,
    )
}

pub fn render_definitions_tab(pipelines: &[PipelineDef]) -> String {
    if pipelines.is_empty() {
        return r##"<div class="empty">
  <span class="empty-icon">⬡</span>
  <div class="empty-title">No pipelines defined</div>
  <div class="empty-desc">Add YAML files to <code>.ship/pipelines/</code> to define repeatable multi-step workflows.</div>
</div>"##
            .to_string();
    }
    let rows: String = pipelines
        .iter()
        .map(|(name, desc, n)| {
            let desc_html = if desc.is_empty() {
                r#"<span style="opacity:0.5">No description</span>"#.to_string()
            } else {
                html_escape(desc)
            };
            format!(
                r##"<article class="def-card">
  <header><span class="def-name">{name}</span><span class="def-steps">{n} step{s}</span></header>
  <p class="def-desc">{desc_html}</p>
</article>"##,
                name = html_escape(name),
                s = if *n == 1 { "" } else { "s" },
            )
        })
        .collect();
    format!(r#"<div class="def-grid">{rows}</div>"#)
}

pub fn render_active_tab(jobs: &HashMap<String, JobRecord>) -> String {
    let mut active: Vec<&JobRecord> = jobs
        .values()
        .filter(|j| matches!(j.status, JobStatus::Dispatched | JobStatus::GatePending))
        .collect();
    active.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if active.is_empty() {
        return r##"<div class="empty">
  <span class="empty-icon">◎</span>
  <div class="empty-title">No active runs</div>
  <div class="empty-desc">Kick off a pipeline with <code>ship run &lt;pipeline&gt; &lt;spec&gt;</code> or from the Specs view.</div>
</div>"##.to_string();
    }
    let rows: String = active.iter().map(|j| render_run_card(j, true)).collect();
    format!(r#"<div class="run-list">{rows}</div>"#)
}

pub fn render_history_tab(jobs: &HashMap<String, JobRecord>) -> String {
    let mut history: Vec<&JobRecord> = jobs
        .values()
        .filter(|j| {
            matches!(
                j.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Merged
            )
        })
        .collect();
    history.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    history.truncate(50);

    if history.is_empty() {
        return r##"<div class="empty">
  <span class="empty-icon">✦</span>
  <div class="empty-title">No run history yet</div>
  <div class="empty-desc">Completed, failed, and merged runs will appear here.</div>
</div>"##.to_string();
    }
    let rows: String = history.iter().map(|j| render_history_row(j)).collect();
    format!(
        r#"<div class="history-wrap"><table class="history-table"><thead><tr>
<th>Slug</th><th>Pipeline</th><th>Status</th><th>Duration</th><th>Completed</th>
</tr></thead><tbody>{rows}</tbody></table></div>"#
    )
}

fn render_run_card(job: &JobRecord, show_dag: bool) -> String {
    let status = format!("{:?}", job.status).to_lowercase();
    let dag = if show_dag { render_mini_dag(job) } else { String::new() };
    format!(
        r##"<article class="run-card" id="run-{slug}">
  <header>
    <div><span class="run-slug">{slug}</span><span class="run-agent">{agent}</span></div>
    <span class="badge badge-{status}">{status}</span>
  </header>
  {dag}
</article>"##,
        slug = html_escape(&job.slug),
        agent = html_escape(&job.agent),
    )
}

fn render_mini_dag(job: &JobRecord) -> String {
    let steps = match &job.steps {
        Some(p) => &p.steps,
        None => return String::new(),
    };
    let completed = &job.completed_steps;
    let current = job.current_step.as_deref();
    let job_status = format!("{:?}", job.status).to_lowercase();

    let steps_html: String = steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let (cls, status, icon) = if completed.contains(&step.id) {
                ("step step-completed", "completed", "&#10003;")
            } else if current == Some(step.id.as_str()) {
                if job_status == "failed" {
                    ("step step-failed", "failed", "&#10007;")
                } else {
                    ("step step-running", "running", "&#10227;")
                }
            } else {
                ("step", "pending", "&#9675;")
            };
            let agent_label = step
                .agent
                .as_deref()
                .map(|a| format!(" &middot; {}", html_escape(a)))
                .unwrap_or_default();
            let step_type = format!("{:?}", step.step_type).to_lowercase();
            let arrow = if i < steps.len() - 1 {
                r#"<span class="arrow">&rarr;</span>"#
            } else {
                ""
            };
            format!(
                r#"<div class="{cls}">
  <div class="step-name">{icon} {id}</div>
  <div class="step-type">{step_type}{agent_label}</div>
  <div class="step-status">{status}</div>
</div>{arrow}"#,
                id = html_escape(&step.id),
            )
        })
        .collect();

    format!(r#"<div class="pipeline">{steps_html}</div>"#)
}

fn render_history_row(job: &JobRecord) -> String {
    let status = format!("{:?}", job.status).to_lowercase();
    let duration_ms = (job.updated_at - job.created_at).num_seconds();
    let duration = format_duration(duration_ms);
    let pipeline_name = job
        .steps
        .as_ref()
        .map(|p| format!("{} steps", p.steps.len()))
        .unwrap_or_else(|| "-".into());
    let completed = job.updated_at.format("%Y-%m-%d %H:%M").to_string();

    format!(
        r##"<tr>
<td>{slug}</td><td>{pipeline}</td>
<td><span class="badge badge-{status}">{status}</span></td>
<td>{duration}</td><td>{completed}</td>
</tr>"##,
        slug = html_escape(&job.slug),
        pipeline = html_escape(&pipeline_name),
    )
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "views_pipelines_tests.rs"]
mod tests;
