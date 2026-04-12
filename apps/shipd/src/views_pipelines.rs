//! Pipelines view — Datastar SSE for the three-tab pipelines dashboard.
//!
//! Renders: definitions (from `.ship/pipelines/*.yaml`), active runs, and history.
//! Subscribes to `job.*` / `step.*` kernel events and re-renders on change.
//!
//! All visible UI is emitted as `<ship-*>` elements from `@ship/ui/elements`.
//! The view shell (`.ship/views/pipelines/index.html`) is responsible for
//! loading tokens + the element bundle; this module emits only content
//! fragments that plug into the shell's SSE target `#pipelines-content`.

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
    let history_count = completed_count + failed_count;

    format!(
        r##"<div id="pipelines-content">
  <div class="stats">
    <ship-stat accent="primary"><span slot="label">Definitions</span>{def_count}</ship-stat>
    <ship-stat accent="info"><span slot="label">Active</span>{active_count}</ship-stat>
    <ship-stat accent="success"><span slot="label">Completed</span>{completed_count}</ship-stat>
    <ship-stat accent="destructive"><span slot="label">Failed</span>{failed_count}</ship-stat>
    <ship-stat><span slot="label">Total runs</span>{total_runs}</ship-stat>
  </div>

  <ship-tabs sticky data-attr-active="$tab" data-on-ship-tab-change="$tab = evt.detail.name">
    <ship-tab name="active" label="Active" count="{active_count}"></ship-tab>
    <ship-tab name="definitions" label="Definitions" count="{def_count}"></ship-tab>
    <ship-tab name="history" label="History" count="{history_count}"></ship-tab>

    <ship-tab-panel name="active">{active}</ship-tab-panel>
    <ship-tab-panel name="definitions">{defs}</ship-tab-panel>
    <ship-tab-panel name="history">{history}</ship-tab-panel>
  </ship-tabs>
</div>"##,
    )
}

pub fn render_definitions_tab(pipelines: &[PipelineDef]) -> String {
    if pipelines.is_empty() {
        return r##"<ship-empty-state>
  <span slot="icon">⬡</span>
  <span slot="title">No pipelines defined</span>
  Add YAML files to <code>.ship/pipelines/</code> to define repeatable multi-step workflows.
</ship-empty-state>"##
            .to_string();
    }
    let rows: String = pipelines
        .iter()
        .map(|(name, desc, n)| {
            format!(
                r##"<ship-pipeline-definition name="{name}" steps="{n}" description="{desc}"></ship-pipeline-definition>"##,
                name = html_escape(name),
                desc = html_escape(desc),
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
        return r##"<ship-empty-state>
  <span slot="icon">◎</span>
  <span slot="title">No active runs</span>
  Kick off a pipeline with <code>ship run &lt;pipeline&gt; &lt;spec&gt;</code> or from the Specs view.
</ship-empty-state>"##.to_string();
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
        return r##"<ship-empty-state>
  <span slot="icon">✦</span>
  <span slot="title">No run history yet</span>
  Completed, failed, and merged runs will appear here.
</ship-empty-state>"##.to_string();
    }
    let rows: String = history.iter().map(|j| render_history_row(j)).collect();
    format!(
        r#"<div class="history-wrap"><table class="history-table"><thead><tr>
<th>Slug</th><th>Pipeline</th><th>Status</th><th>Duration</th><th>Completed</th>
</tr></thead><tbody>{rows}</tbody></table></div>"#
    )
}

/// Normalized run-state token for ship-pipeline-run (kebab-case).
fn job_state_token(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Dispatched => "dispatched",
        JobStatus::GatePending => "gatepending",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Merged => "merged",
        JobStatus::Blocked => "blocked",
        JobStatus::Pending => "pending",
    }
}

/// Tone mapping for badges that only show a JobStatus label.
fn job_status_tone(s: &JobStatus) -> &'static str {
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

fn render_run_card(job: &JobRecord, show_dag: bool) -> String {
    let state = job_state_token(&job.status);
    let dag = if show_dag { render_mini_dag(job) } else { String::new() };
    format!(
        r##"<ship-pipeline-run id="run-{slug}" state="{state}" slug="{slug}" agent="{agent}">{dag}</ship-pipeline-run>"##,
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
    let job_failed = matches!(job.status, JobStatus::Failed);

    let steps_html: String = steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let state = if completed.contains(&step.id) {
                "completed"
            } else if current == Some(step.id.as_str()) {
                if job_failed { "failed" } else { "running" }
            } else {
                "pending"
            };
            let agent_attr = step
                .agent
                .as_deref()
                .map(|a| format!(r#" agent="{}""#, html_escape(a)))
                .unwrap_or_default();
            let step_type = format!("{:?}", step.step_type).to_lowercase();
            let arrow = if i < steps.len() - 1 {
                r#"<span class="arrow">&rarr;</span>"#
            } else {
                ""
            };
            format!(
                r#"<ship-pipeline-step name="{id}" step-type="{step_type}" state="{state}"{agent_attr}></ship-pipeline-step>{arrow}"#,
                id = html_escape(&step.id),
            )
        })
        .collect();

    format!("<ship-pipeline-dag>{steps_html}</ship-pipeline-dag>")
}

fn render_history_row(job: &JobRecord) -> String {
    let state = job_state_token(&job.status);
    let tone = job_status_tone(&job.status);
    let duration_ms = (job.updated_at - job.created_at).num_seconds();
    let duration = format_duration(duration_ms);
    let pipeline_name = job
        .steps
        .as_ref()
        .map(|p| format!("{} steps", p.steps.len()))
        .unwrap_or_else(|| "-".into());
    let completed = job.updated_at.format("%Y-%m-%d %H:%M").to_string();

    format!(
        r##"<ship-history-row state="{state}">
<span slot="slug">{slug}</span>
<span slot="pipeline">{pipeline}</span>
<span slot="status"><ship-badge tone="{tone}">{state}</ship-badge></span>
<span slot="duration">{duration}</span>
<span slot="completed">{completed}</span>
</ship-history-row>"##,
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
