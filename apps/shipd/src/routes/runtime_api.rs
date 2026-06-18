//! Runtime read API — the read-side residual after the Connect migration.
//!
//! One handler remains: `daemon_status` (GET /daemon/status), consumed by
//! `ship daemon status` (apps/ship-cli/src/daemon/status.rs). Everything
//! else this module used to carry moved to typed Connect wires
//! (AgentsService / PipelinesService / agent.Dispatch / StreamService.Events)
//! or lives on the canonical read surface (`ship_substrate::runtime_reads`).
//! The former `runtime_api/` submodule dir (daemon.rs + an events re-export
//! shim) collapsed back into this single file.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::ship_runtime::ApiState;
use crate::wire::MeshResponse;

pub async fn daemon_status(State(state): State<ApiState>) -> impl IntoResponse {
    use ship_pipeline_runtime::projections::pipeline_run::PipelineRunStatus;

    let ship_dir = state.ship_dir();
    let jobs = ship_pipeline_runtime::projections::pipeline_run::load_pipeline_runs_at(ship_dir).unwrap_or_default();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for j in jobs.values() {
        let key = match j.status {
            PipelineRunStatus::Pending => "pending",
            PipelineRunStatus::Dispatched => "dispatched",
            PipelineRunStatus::Completed => "completed",
            PipelineRunStatus::GatePending => "gate_pending",
            PipelineRunStatus::Blocked => "blocked",
            PipelineRunStatus::Merged => "merged",
            PipelineRunStatus::Failed => "failed",
            PipelineRunStatus::Cancelled => "cancelled",
        };
        *counts.entry(key).or_insert(0) += 1;
    }

    // Most recent sweep_completed and the drift count it recorded.
    // `load_recent_events_by_type` lives in `ship_substrate::runtime_reads`
    // (the canonical read surface).
    let sweep_events =
        ship_substrate::runtime_reads::load_recent_events_by_type(
            ship_dir,
            &["supervisor.sweep_completed"],
            100,
        )
        .unwrap_or_default();
    let last_sweep = sweep_events.iter().max_by_key(|e| e.created_at);
    let last_sweep_at = last_sweep.map(|e| e.created_at.to_rfc3339());
    let last_sweep_drift = last_sweep
        .and_then(|e| serde_json::from_str::<serde_json::Value>(&e.payload_json).ok())
        .and_then(|p| p["drift_count"].as_u64())
        .unwrap_or(0);

    let port = state.config().port;
    let host = state.config().host.clone();
    let uptime_secs = state.uptime().as_secs();
    let pid = std::process::id();

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({
                "pid": pid,
                "version": env!("CARGO_PKG_VERSION"),
                "host": host,
                "port": port,
                "uptime_secs": uptime_secs,
                "jobs_total": jobs.len(),
                "jobs_by_status": counts,
                "last_sweep_at": last_sweep_at,
                "last_sweep_drift": last_sweep_drift,
                "tasks": state.health_registry().snapshot(),
            }),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use crate::ship_runtime::ApiState;
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
        let payload = ship_substrate::events::supervisor::SweepCompletedPayload {
            duration_ms: 12,
            drift_count,
            actions_taken: drift_count,
        };
        let env = ship_substrate::events::EventEnvelope::new(
            ship_substrate::events::supervisor::event_types::SUPERVISOR_SWEEP_COMPLETED,
            "supervisor",
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn daemon_status_returns_pid_and_uptime() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_pipeline_run_started(&state, "status-job").await;
        emit_sweep_completed(&state, 0).await;
        let response = daemon_status(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let val = collect_body(response).await;
        let d = &val["data"];
        assert!(d["pid"].as_u64().unwrap() > 0);
        assert!(d["jobs_total"].as_u64().unwrap() >= 1);
        assert_eq!(d["jobs_by_status"]["pending"], serde_json::json!(1));
        assert!(d["last_sweep_at"].is_string());
        assert_eq!(d["last_sweep_drift"], serde_json::json!(0));
    }
}
