//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use ship_substrate::events::{ActorConfig, EventEnvelope};
use ship_substrate::events::job::{JobCreatedPayload, event_types};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

pub async fn daemon_status(State(state): State<ApiState>) -> impl IntoResponse {
    use ship_substrate::projections::job::JobStatus;

    let ship_dir = state.ship_dir();
    let jobs = ship_substrate::projections::job::load_jobs_at(ship_dir).unwrap_or_default();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for j in jobs.values() {
        let key = match j.status {
            JobStatus::Pending => "pending",
            JobStatus::Dispatched => "dispatched",
            JobStatus::Completed => "completed",
            JobStatus::GatePending => "gate_pending",
            JobStatus::Blocked => "blocked",
            JobStatus::Merged => "merged",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        };
        *counts.entry(key).or_insert(0) += 1;
    }

    // Most recent sweep_completed and the drift count it recorded.
    let sweep_events =
        super::jobs::load_recent_events_by_type(ship_dir, &["supervisor.sweep_completed"], 100)
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

/// POST /api/v1/runtime/rebuild-projections — rebuild all projections from events.
#[utoipa::path(
    operation_id = "rebuildProjections",
    post,
    path = "/api/v1/rebuild-projections",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "rebuild already running", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn rebuild_projections(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::AdminDaemon) {
        return e;
    }
    match ship_substrate::db::rebuild::rebuild_projections(state.ship_dir()) {
        Ok(report) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "events_replayed": report.events_replayed,
                    "projections_rebuilt": report.projections_rebuilt,
                    "duration_ms": report.duration_ms,
                }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}
