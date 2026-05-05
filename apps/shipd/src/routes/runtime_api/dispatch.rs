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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DispatchRequest {
    /// Pipeline name (file in `.ship/pipelines/<name>.{yaml,yml,jsonc,json}`).
    pub pipeline: String,
    /// Inline spec body describing what the run should do.
    #[serde(default)]
    pub spec: Option<String>,
    /// Provider override; when `None`, each step uses whatever the
    /// pipeline YAML declared (or falls back to claude).
    #[serde(default)]
    pub provider: Option<String>,
    /// Model override (e.g. "sonnet"). When `None`, step YAML decides.
    #[serde(default)]
    pub model: Option<String>,
    /// Pipeline `{{ var }}` overrides.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,
    /// When true, force the headless (ProcessExecutor) path.
    #[serde(default)]
    pub headless: bool,
    /// Optional workspace name; defaults to the pipeline name. Only
    /// used when `slug` isn't supplied.
    #[serde(default)]
    pub workspace: Option<String>,

    // ── Power-user overrides (CLI + programmatic callers) ──
    /// Explicit slug (e.g. `ship-dev-01kpx…`). When `None`, the daemon
    /// generates one from `workspace` + a random suffix.
    #[serde(default)]
    pub slug: Option<String>,
    /// Explicit branch. When `None`, defaults to `job/<slug>`.
    #[serde(default)]
    pub branch: Option<String>,
    /// Agent override. When `None`, falls back to the pipeline's
    /// first-step `agent`, then `"default"`.
    #[serde(default)]
    pub agent: Option<String>,
    /// Operator-facing path to the spec file (for `ship jobs` display).
    #[serde(default)]
    pub source_spec_path: Option<String>,
    /// Parent job slugs this run depends on.
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
}

/// POST /api/v1/jobs/dispatch
///
/// UI-facing dispatch — mirrors `ship run` on the CLI. Loads the
/// named pipeline, generates slug/branch/agent, and emits
/// `job.created`. The orchestrator picks it up from there.
#[utoipa::path(
    post,
    path = "/api/v1/jobs/dispatch",
    request_body = DispatchRequest,
    responses((status = 200, body = MeshResponse)),
    tag = "runtime"
)]
pub async fn dispatch_pipeline(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<DispatchRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::CreateJobs) {
        return e;
    }

    // Walk up from the daemon's CWD to find `.ship/pipelines/<name>.{yaml,yml,jsonc,json}`.
    // Matches ship_primitives::services::orchestrator::pipeline_loader::load_pipeline_file's
    // resolution strategy so ensure_pipeline (called below) will find the same file.
    let pipeline_path = {
        let mut dir = std::env::current_dir().ok();
        let mut found = None;
        while let Some(d) = dir.as_ref() {
            let pipelines_dir = ship_substrate::ShipHome::project_ship_dir(&d).join("pipelines");
            if pipelines_dir.is_dir() {
                for ext in ["yaml", "yml", "jsonc", "json"] {
                    let candidate = pipelines_dir.join(format!("{}.{ext}", req.pipeline));
                    if candidate.exists() {
                        found = Some(candidate);
                        break;
                    }
                }
                break;
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
        found
    };
    let Some(pipeline_path) = pipeline_path else {
        return (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(format!("pipeline '{}' not found", req.pipeline)),
            }),
        );
    };
    let raw = match std::fs::read_to_string(&pipeline_path) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("read pipeline: {e}")),
                }),
            );
        }
    };
    let (normalized, meta) = match ship_jobs::pipeline::normalize_pipeline_yaml(&raw) {
        Ok(n) => n,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("pipeline parse: {e}")),
                }),
            );
        }
    };

    let steps = match normalized.get("steps").and_then(|v| v.as_array()) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!("pipeline has no steps"),
                }),
            );
        }
    };

    let first_agent = req.agent.clone().unwrap_or_else(|| {
        steps
            .first()
            .and_then(|s| s.get("agent"))
            .and_then(|a| a.as_str())
            .unwrap_or("default")
            .to_string()
    });

    let pipeline_name = meta.name.clone().unwrap_or_else(|| req.pipeline.clone());
    let workspace_name = req.workspace.unwrap_or_else(|| pipeline_name.clone());
    let slug = req.slug.unwrap_or_else(|| {
        format!("{}-{}", workspace_name, &ship_substrate::gen_ulid()[..8].to_lowercase())
    });
    let branch = req.branch.unwrap_or_else(|| format!("job/{slug}"));
    let job_id = ship_substrate::gen_ulid();

    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug,
        agent: first_agent,
        branch,
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: req.source_spec_path,
        plan_id: None,
        model: req.model,
        provider: req.provider,
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: Some(pipeline_name),
        vars: req.vars,
        spec_content: req.spec,
        step_specs: None,
        headless: req.headless,
        baseline_sha: super::resolve_project_head_sha(),
    };

    let payload = match ship_primitives::services::orchestrator::pipeline_loader::ensure_pipeline(payload) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse { ok: false, data: serde_json::json!(e) }),
            );
        }
    };

    let envelope = match EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
        .map(|e| e.with_actor_id("studio"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            );
        }
    };
    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "job_id": &job_id }),
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
