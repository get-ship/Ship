//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::routes::rest_api::{ApiState, MeshResponse};

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
    let sweep_events =
        ship_substrate::runtime_reads::load_recent_events_by_type(ship_dir, &["supervisor.sweep_completed"], 100)
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
