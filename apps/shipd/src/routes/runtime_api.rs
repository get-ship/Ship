//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/v1/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.
//!
//! The legacy `list_sessions_across_projects` workbench handler was
//! removed with the workbench `session.*` family (job
//! `workbench-teardown`); the canonical session family has no cross-
//! project read yet — the workbench's runtime.db `sessions` table is
//! gone, so a re-implementation would be a different projection over
//! the canonical event log, deferred to a follow-up.

// ── mr-04 follow-up: per-handler submodules ─────────────────────────────────
//
// Each submodule owns one logical chunk of the runtime read API. The
// parent re-exports the public handler symbols + helpers so call sites
// resolve through `crate::routes::runtime_api::*`.
//
// Note: several former submodules (`agents`, `dispatch`, `jobs`,
// `jobs_status`, `pipeline_run`, `daemon`) were cut when their REST
// handlers moved to the typed Connect wires (AgentsService,
// PipelinesService, agent.Dispatch) or folded into `/api/health`
// (daemon_status). The KEEP set here is the SSE event stream, slated
// to retire once the shell moves to `StreamService.Events`.

pub mod events;

pub use events::{EventsPollQuery, event_stream};
pub use ship_pipeline_runtime::run_core::PipelineRunRequest;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship_runtime::ApiState;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;
    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        // Isolated per-test kernel — init_kernel_router uses a OnceLock that
        // pins the first test's ship_dir globally, breaking all subsequent
        // tests with "unable to open database file" when they try to write.
        let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
        Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }
    async fn collect_body(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }
    async fn emit_pipeline_run_started(state: &ApiState, slug: &str) -> String {
        let pipeline_run_id = ship_substrate::gen_ulid();
        let payload = ship_substrate::events::pipeline::PipelineRunStartedPayload {
            pipeline_run_id: pipeline_run_id.clone(),
            slug: slug.into(),
            agent: "a".into(),
            branch: format!("job/{slug}"),
            spec_path: ".artifacts/job-spec.md".into(),
            source_spec_path: None,
            plan_id: None, model: None, harness: None, depends_on: None,
            pipeline: None, steps: None, pipeline_name: None, vars: None,
            spec_content: None, step_specs: None, headless: false,
            prompt: None,
            resume: false,
            resume_session_id: None,
            baseline_sha: None,
            keepalive: false,
        };
        let env = ship_substrate::events::EventEnvelope::new(
            ship_substrate::events::pipeline::event_types::PIPELINE_RUN_STARTED,
            &pipeline_run_id,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
        pipeline_run_id
    }
    async fn emit_sweep_completed(state: &ApiState, drift_count: usize) {
        let payload = ship_pipeline_runtime::sweep_events::SweepCompletedPayload {
            duration_ms: 12,
            drift_count,
            actions_taken: drift_count,
        };
        let env = ship_substrate::events::EventEnvelope::new(
            ship_pipeline_runtime::sweep_events::event_types::SUPERVISOR_SWEEP_COMPLETED,
            "supervisor",
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }
    // The former `stop_agent`/`steer_agent`/`get_job_detail`/`cancel_job`/
    // `list_jobs` REST handlers were cut when their typed Connect wires took
    // over (AgentsService, PipelinesService, agent.Dispatch); their tests
    // went with them. `daemon_status` folded into `/api/health` — the
    // job-status fields it carried are pinned here against that handler.

    #[tokio::test(flavor = "multi_thread")]
    async fn health_check_reports_pid_and_job_status() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_pipeline_run_started(&state, "status-job").await;
        emit_sweep_completed(&state, 0).await;
        let val = crate::router::health_check(state).await.0;
        assert!(val["pid"].as_u64().unwrap() > 0);
        assert!(val["uptime_secs"].is_u64());
        assert!(val["jobs_total"].as_u64().unwrap() >= 1);
        assert_eq!(val["jobs_by_status"]["pending"], serde_json::json!(1));
        assert!(val["last_sweep_at"].is_string());
        assert_eq!(val["last_sweep_drift"], serde_json::json!(0));
    }
}