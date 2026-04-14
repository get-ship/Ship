//! Shared helpers for step prompt composition.
//!
//! The per-stage spec material now lives under `.artifacts/pipeline/` and is
//! written by [`super::pipeline_specs::write_pipeline_specs`]. This module
//! keeps the two pieces that both the launch path and the advance path reach
//! for: the runtime contract boilerplate appended to every dispatch prompt,
//! and the job-context loader that pulls the overall spec body + pipeline
//! name from the `job.created` event.

use crate::events::{EventFilter, EventStore, SqliteEventStore};

/// Fields read from the `job.created` event that the dispatch path needs.
/// Missing fields become empty / None so callers don't have to branch.
pub struct JobContextFromEvents {
    pub user_spec_body: String,
    pub pipeline_name: Option<String>,
}

/// Load (`user_spec_body`, `pipeline_name`) from the `job.created` event for
/// this job. Returns defaults if the event isn't found — step dispatch never
/// blocks on event-log lookups.
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
