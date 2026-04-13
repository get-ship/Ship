//! Per-step prompt composition — the runtime's view of the pipeline as a
//! state machine.
//!
//! Each step is a state. The prompt for step N is built fresh on every
//! dispatch from:
//!
//!   1. Pipeline metadata: which step this is, the overall pipeline name,
//!      the step's role (agent profile + optional goal/scope).
//!   2. User intent: the original `JobCreated.spec_content` body.
//!   3. Predecessor summaries: `step.done_declared.summary` for every step
//!      that has completed so far on this job.
//!   4. Runtime contract: how to signal completion (injected by runtime,
//!      not by agent authors).
//!
//! The composer never reads files from disk outside the event log. The log
//! is the source of truth for what happened; the prompt is derived from it.
//! This means the same prompt is reproducible from events alone, which is
//! the property we need for replay and remote coordination.

use crate::events::job::{Pipeline, PipelineStep};
use crate::events::{EventFilter, EventStore, SqliteEventStore};
use anyhow::{Context, Result};

/// Assemble the complete job-spec markdown for a step. Returns the string to
/// write to `.artifacts/job-spec.md` in the worktree.
pub fn compose_step_spec(
    job_id: &str,
    pipeline_name: Option<&str>,
    pipeline: &Pipeline,
    current_step: &PipelineStep,
    user_spec_body: &str,
) -> Result<String> {
    let predecessors = load_predecessor_summaries(job_id)
        .context("load predecessor summaries from event log")?;
    Ok(render(
        pipeline_name.unwrap_or("pipeline"),
        pipeline,
        current_step,
        user_spec_body,
        &predecessors,
    ))
}

/// Compose + write the step spec for a job advance, logging the outcome at
/// INFO/ERROR. Encapsulates the common dispatch-site boilerplate so callers
/// don't have to reproduce the load_job_context + write_step_spec + match
/// dance inline.
pub fn compose_and_write_for_step(
    worktree_path: &std::path::Path,
    slug: &str,
    job_id: &str,
    spec_path: &str,
    pipeline: &Pipeline,
    current_step: &PipelineStep,
) {
    let ctx = load_job_context(job_id);
    match write_step_spec(
        worktree_path,
        spec_path,
        job_id,
        ctx.pipeline_name.as_deref(),
        pipeline,
        current_step,
        &ctx.user_spec_body,
    ) {
        Ok(path) => tracing::info!(
            slug, step_id = current_step.id,
            "pipeline: wrote step spec to {}", path.display()
        ),
        Err(e) => tracing::error!(
            slug, step_id = current_step.id,
            "pipeline: failed to write step spec: {e}"
        ),
    }
}

/// Compose and write `.artifacts/job-spec.md` for the given step. Creates the
/// parent directory if needed. Returns the absolute path written.
pub fn write_step_spec(
    worktree_path: &std::path::Path,
    spec_path: &str,
    job_id: &str,
    pipeline_name: Option<&str>,
    pipeline: &Pipeline,
    current_step: &PipelineStep,
    user_spec_body: &str,
) -> Result<std::path::PathBuf> {
    let content = compose_step_spec(
        job_id, pipeline_name, pipeline, current_step, user_spec_body,
    )?;
    let dst = worktree_path.join(spec_path);
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create spec dir {}", parent.display()))?;
    }
    std::fs::write(&dst, &content)
        .with_context(|| format!("write step spec to {}", dst.display()))?;
    Ok(dst)
}

/// Fields read from the `job.created` event that the composer needs.
/// Missing fields become empty / None so callers don't have to branch.
pub struct JobContextFromEvents {
    pub user_spec_body: String,
    pub pipeline_name: Option<String>,
}

/// Load (user_spec_body, pipeline_name) from the `job.created` event for this
/// job. Returns defaults if the event isn't found — the composer tolerates
/// missing data so step dispatch never blocks on event-log lookups.
pub fn load_job_context(job_id: &str) -> JobContextFromEvents {
    let mut out = JobContextFromEvents {
        user_spec_body: String::new(),
        pipeline_name: None,
    };
    let Ok(store) = SqliteEventStore::new() else {
        return out;
    };
    let envelopes = store
        .query(&EventFilter {
            entity_id: Some(job_id.to_string()),
            event_type: Some("job.created".to_string()),
            ..Default::default()
        })
        .unwrap_or_default();
    for env in envelopes {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&env.payload_json) {
            if let Some(s) = v.get("spec_content").and_then(|x| x.as_str()) {
                out.user_spec_body = s.to_string();
            }
            if let Some(s) = v.get("pipeline_name").and_then(|x| x.as_str()) {
                out.pipeline_name = Some(s.to_string());
            }
        }
    }
    out
}

/// A previously-completed step on this job, with whatever summary the agent
/// declared when calling `step_done`.
#[derive(Debug, Clone)]
pub struct PredecessorSummary {
    pub step_id: String,
    pub summary: Option<String>,
}

/// Load summaries from `step.done_declared` events for this job, in
/// chronological order (ULID ordering = emission order).
fn load_predecessor_summaries(job_id: &str) -> Result<Vec<PredecessorSummary>> {
    let store = SqliteEventStore::new().context("open event store")?;
    let envelopes = store.query(&EventFilter {
        entity_id: Some(job_id.to_string()),
        event_type: Some("step.done_declared".to_string()),
        ..Default::default()
    })?;
    let mut out = Vec::with_capacity(envelopes.len());
    for env in envelopes {
        let v: serde_json::Value = serde_json::from_str(&env.payload_json)
            .unwrap_or(serde_json::Value::Null);
        out.push(PredecessorSummary {
            step_id: v
                .get("step_id")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            summary: v
                .get("summary")
                .and_then(|x| x.as_str())
                .map(str::to_string),
        });
    }
    Ok(out)
}

fn render(
    pipeline_name: &str,
    pipeline: &Pipeline,
    current: &PipelineStep,
    user_spec_body: &str,
    predecessors: &[PredecessorSummary],
) -> String {
    let step_number = pipeline
        .steps
        .iter()
        .position(|s| s.id == current.id)
        .map(|i| i + 1)
        .unwrap_or(0);
    let total = pipeline.steps.len();
    let agent = current.agent.as_deref().unwrap_or("unassigned");

    let mut out = String::new();
    out.push_str(&format!(
        "# Pipeline `{pipeline_name}` — step {step_number} of {total}\n\n"
    ));

    // Pipeline shape for situational awareness.
    out.push_str("## Pipeline shape\n\n");
    for (i, s) in pipeline.steps.iter().enumerate() {
        let marker = if s.id == current.id { "→" } else { " " };
        let role = s.agent.as_deref().unwrap_or("?");
        out.push_str(&format!(
            "{marker} step-{n} `{id}` ({role})\n",
            n = i + 1,
            id = s.id,
        ));
    }
    out.push('\n');

    // Current step role.
    out.push_str(&format!(
        "## Your role — step `{}` ({})\n\n",
        current.id, agent
    ));
    if let Some(goal) = &current.goal {
        out.push_str(goal);
        out.push_str("\n\n");
    } else {
        out.push_str(&format!(
            "You are the `{agent}` agent running step {step_number} of this pipeline. \
             Apply your agent profile to the job body below.\n\n"
        ));
    }

    // Predecessor summaries.
    if !predecessors.is_empty() {
        out.push_str("## What prior steps accomplished\n\n");
        for p in predecessors {
            match &p.summary {
                Some(s) if !s.is_empty() => {
                    out.push_str(&format!("- **{}** — {}\n", p.step_id, s));
                }
                _ => {
                    out.push_str(&format!(
                        "- **{}** — (no summary declared)\n",
                        p.step_id
                    ));
                }
            }
        }
        out.push('\n');
    }

    // Original user intent.
    out.push_str("## Job body\n\n");
    if user_spec_body.trim().is_empty() {
        out.push_str("_(no spec body provided)_\n\n");
    } else {
        out.push_str(user_spec_body.trim());
        out.push_str("\n\n");
    }

    // Runtime contract.
    out.push_str(RUNTIME_CONTRACT);
    out
}

/// The runtime's pipeline contract. Injected at dispatch, never author-written.
/// Tells the agent how to signal completion so the step monitor can observe.
pub const RUNTIME_CONTRACT: &str = r#"## Runtime contract — how to signal completion

This step is being observed by the Ship pipeline's step monitor. Signal
completion using one of:

**Preferred — explicit done.** Call `mcp__ship__step_done` with a one-line
summary. Advances the pipeline immediately. The summary is fed to the next
step as context.

```
mcp__ship__step_done({ summary: "added ping() and a passing test" })
```

**Fallback — commit and stop.** If `step_done` is unavailable, commit your
work with `git commit`. When you stop responding, the monitor observes a
commit + settle period and advances the pipeline. No summary is carried
forward in this path.

**Progress updates** (optional): call `mcp__ship__log_progress` with a
checkpoint. Doesn't advance; just records thinking in the event log.

**Never call step_done** until the job body's acceptance criteria are met.
If you're blocked, call `mcp__ship__log_progress` with the blocker and stop.

If you do nothing after the job body is addressed: the monitor settles ~15s
after you stop responding. With no commits and no step_done, the pipeline
router treats the step as Inconclusive and advances to the next step (a
gate step, if any, will review before proceeding).
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::job::{Pipeline, PipelineStep, StepType};

    fn step(id: &str, agent: &str) -> PipelineStep {
        PipelineStep {
            id: id.to_string(),
            step_type: StepType::Agent,
            agent: Some(agent.to_string()),
            goal: None,
            after: vec![],
            model: None,
            provider: None,
            command: None,
            max_retries: None,
        }
    }

    #[test]
    fn render_includes_pipeline_shape_with_current_marker() {
        let pipeline = Pipeline {
            steps: vec![
                step("step-1", "test-writer"),
                step("step-2", "rust-runtime"),
                step("step-3", "reviewer"),
            ],
        };
        let out = render(
            "tdd",
            &pipeline,
            &pipeline.steps[1],
            "add ping()",
            &[],
        );
        assert!(out.contains("step 2 of 3"));
        assert!(out.contains("→ step-2 `step-2` (rust-runtime)"));
        assert!(out.contains("  step-1 `step-1` (test-writer)"));
        assert!(out.contains("  step-3 `step-3` (reviewer)"));
    }

    #[test]
    fn render_inlines_user_spec_body() {
        let pipeline = Pipeline { steps: vec![step("step-1", "test-writer")] };
        let out = render("tdd", &pipeline, &pipeline.steps[0], "add ping()", &[]);
        assert!(out.contains("## Job body"));
        assert!(out.contains("add ping()"));
    }

    #[test]
    fn render_summarizes_predecessors_in_order() {
        let pipeline = Pipeline {
            steps: vec![
                step("step-1", "test-writer"),
                step("step-2", "rust-runtime"),
            ],
        };
        let preds = vec![PredecessorSummary {
            step_id: "step-1".into(),
            summary: Some("wrote ping test".into()),
        }];
        let out = render("tdd", &pipeline, &pipeline.steps[1], "body", &preds);
        assert!(out.contains("## What prior steps accomplished"));
        assert!(out.contains("**step-1** — wrote ping test"));
    }

    #[test]
    fn render_omits_predecessor_section_when_empty() {
        let pipeline = Pipeline { steps: vec![step("step-1", "test-writer")] };
        let out = render("tdd", &pipeline, &pipeline.steps[0], "body", &[]);
        assert!(!out.contains("What prior steps accomplished"));
    }

    #[test]
    fn render_uses_step_goal_when_present() {
        let mut s = step("step-2", "rust-runtime");
        s.goal = Some("Implement against the tests from step-1.".into());
        let pipeline = Pipeline {
            steps: vec![step("step-1", "test-writer"), s],
        };
        let out = render("tdd", &pipeline, &pipeline.steps[1], "body", &[]);
        assert!(out.contains("Implement against the tests from step-1."));
        assert!(!out.contains("Apply your agent profile"));
    }

    #[test]
    fn render_falls_back_to_generic_role_without_goal() {
        let pipeline = Pipeline { steps: vec![step("step-1", "test-writer")] };
        let out = render("tdd", &pipeline, &pipeline.steps[0], "body", &[]);
        assert!(out.contains("You are the `test-writer` agent"));
    }

    #[test]
    fn render_always_includes_runtime_contract() {
        let pipeline = Pipeline { steps: vec![step("step-1", "test-writer")] };
        let out = render("tdd", &pipeline, &pipeline.steps[0], "body", &[]);
        assert!(out.contains("Runtime contract"));
        assert!(out.contains("mcp__ship__step_done"));
    }

    #[test]
    fn empty_body_produces_placeholder() {
        let pipeline = Pipeline { steps: vec![step("step-1", "test-writer")] };
        let out = render("tdd", &pipeline, &pipeline.steps[0], "", &[]);
        assert!(out.contains("_(no spec body provided)_"));
    }
}
