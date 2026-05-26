//! `/api/v1/sandboxes/*` — sandbox registry inspection.
//!
//! Read-only surface backed by `ship_platform::sandbox::dispatcher()`.
//! Lets users (and the UI) see which sandboxes exist, what tier
//! they're at, which actors are inside, and whether the idle reap
//! clock is armed. No write endpoints today — sandbox lifecycle is
//! driven by ProcessExecutor (spawn) + the reaper background task
//! (idle reap). Manual reap can land later if there's signal for it.

use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::routes::rest_api::{ApiState, MeshResponse};
use ship_platform::sandbox::{self, RuntimeEndpoint, SandboxState};

#[derive(Serialize)]
struct SandboxView {
    id: String,
    project: String,
    name: String,
    tier: Option<String>,
    strategy: Option<String>,
    state: String,
    endpoint: Option<EndpointView>,
    actors: Vec<String>,
    created_at: String,
    idle_since: Option<String>,
    has_owner: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum EndpointView {
    InProcess,
    Remote { addr: String, scheme: String },
}

impl From<&RuntimeEndpoint> for EndpointView {
    fn from(e: &RuntimeEndpoint) -> Self {
        match e {
            RuntimeEndpoint::InProcess => Self::InProcess,
            RuntimeEndpoint::Remote { addr, scheme } => Self::Remote {
                addr: addr.clone(),
                scheme: scheme.clone(),
            },
        }
    }
}

fn state_label(s: SandboxState) -> &'static str {
    match s {
        SandboxState::Pending => "pending",
        SandboxState::Running => "running",
        SandboxState::Idle => "idle",
        SandboxState::Reaping => "reaping",
        SandboxState::Stopped => "stopped",
    }
}

pub async fn sandbox_list(State(_state): State<ApiState>) -> impl IntoResponse {
    let dispatcher = sandbox::dispatcher();
    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let mut sandboxes: Vec<SandboxView> = Vec::new();
    // SandboxRegistry doesn't expose an "iterate all" method; we
    // collect by walking known projects via the sandboxes_in_project
    // iterator after collecting unique project names. Simpler: walk
    // the public read interface by reaching into the registry's
    // own helper — projects appear via actor lookup. For v1 we just
    // use the sandboxes_in_project pattern but with all projects.
    //
    // The registry's internal map of sandboxes IS the source; expose
    // an iterator once there's a need beyond this endpoint.
    let projects = reg
        .all_projects()
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for project in projects {
        for sandbox in reg.sandboxes_in_project(&project) {
            sandboxes.push(SandboxView {
                id: sandbox.id.0.clone(),
                project: sandbox.project.clone(),
                name: sandbox.name.0.clone(),
                tier: sandbox.isolation.tier.map(|t| t.to_string()),
                strategy: sandbox.isolation.strategy.map(|s| s.to_string()),
                state: state_label(sandbox.state).into(),
                endpoint: sandbox.runtime_endpoint.as_ref().map(EndpointView::from),
                actors: sandbox.actors.clone(),
                created_at: sandbox.created_at.to_rfc3339(),
                idle_since: sandbox.idle_since.map(|t| t.to_rfc3339()),
                has_owner: sandbox.sandbox_owner.is_some(),
            });
        }
    }
    let payload = serde_json::json!({ "sandboxes": sandboxes });
    (StatusCode::OK, Json(MeshResponse { ok: true, data: payload }))
}

pub async fn sandbox_get(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dispatcher = sandbox::dispatcher();
    let registry = dispatcher.registry();
    let reg = registry.read().await;
    let sandbox_id = ship_platform::sandbox::SandboxId(id.clone());
    let Some(sandbox) = reg.get(&sandbox_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({ "error": format!("no sandbox '{id}'") }),
            }),
        );
    };
    let view = SandboxView {
        id: sandbox.id.0.clone(),
        project: sandbox.project.clone(),
        name: sandbox.name.0.clone(),
        tier: sandbox.isolation.tier.map(|t| t.to_string()),
        strategy: sandbox.isolation.strategy.map(|s| s.to_string()),
        state: state_label(sandbox.state).into(),
        endpoint: sandbox.runtime_endpoint.as_ref().map(EndpointView::from),
        actors: sandbox.actors.clone(),
        created_at: sandbox.created_at.to_rfc3339(),
        idle_since: sandbox.idle_since.map(|t| t.to_rfc3339()),
        has_owner: sandbox.sandbox_owner.is_some(),
    };
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::to_value(&view).unwrap_or_default(),
        }),
    )
}
