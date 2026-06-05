//! Shared helpers for step prompt composition.
//!
//! The per-stage spec material now lives under `.artifacts/pipeline/` and is
//! written by [`super::pipeline_specs::write_pipeline_specs`]. This module
//! keeps the two pieces that both the launch path and the advance path reach
//! for: the runtime contract boilerplate appended to every dispatch prompt,
//! and the job-context loader that pulls the overall spec body + pipeline
//! name from the `job.created` event.

use ship_substrate::events::{EventFilter, EventStore, SqliteEventStore};

/// Fields read from the `job.created` event that the dispatch path needs.
/// Missing fields become empty / None so callers don't have to branch.
pub struct SessionContextFromEvents {
    pub user_spec_body: String,
    pub pipeline_name: Option<String>,
}

/// Load (`user_spec_body`, `pipeline_name`) from the `job.created` event for
/// this job. Returns defaults if the event isn't found — step dispatch never
/// blocks on event-log lookups.
pub fn load_job_context(pipeline_run_id: &str) -> SessionContextFromEvents {
    let mut out = SessionContextFromEvents {
        user_spec_body: String::new(),
        pipeline_name: None,
    };
    // pipeline.run.started is Runtime-scoped → query runtime.db.
    let Ok(store) = ship_substrate::project::get_global_dir().and_then(|dir| {
        SqliteEventStore::new_for_scope(
            &dir,
            ship_types::event_registry::EventScope::Runtime,
            None,
        )
    }) else {
        return out;
    };
    let envelopes = store
        .query(&EventFilter {
            entity_id: Some(pipeline_run_id.to_string()),
            event_type: Some("pipeline.run.started".to_string()),
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
pub const RUNTIME_CONTRACT: &str = r#"## Runtime contract — completing this step

This step is part of a Ship pipeline. The next step is waiting on you.
Your job ends when you run `ship step-done`.

**The completion rule, full stop:**

When the acceptance criteria in the job body are met (tests passing once,
the requested change present, no compile errors), **run `ship step-done`
immediately and exit**. Do not iterate further. Do not re-verify. Do not
explore neighboring code. The reviewer step is what catches missed cases —
it has the diff and your summary. Trust it.

```
ship step-done --summary "added ping() + 2 tests, all green"
```

`ship step-done` reads your job slug from `SHIP_ACTOR_ID` (injected at
dispatch) and signals completion over Connect — no MCP, no slug juggling.

**If you're blocked**, stop responding. The idle-timeout monitor will
settle and the gate step picks up the partial work for review. Do not run
`ship step-done` for blocked work.

**Timeouts:** the step is killed if no subprocess activity is seen for
5 minutes (idle timeout), or if the whole step runs past 60 minutes
(hard cap). Long tool calls like `cargo test` keep the step alive
naturally — you don't need to rush. If you haven't run `ship step-done`
before those limits, the monitor terminates the step as `timed out`.
"#;
