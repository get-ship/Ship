//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/v1/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use runtime::events::{ActorConfig, EventEnvelope};
use runtime::events::job::{JobCreatedPayload, event_types};
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

// ---- Query params ----

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SessionsQuery {
    pub workspace_id: Option<String>,
}

// ---- Handlers ----

/// GET /api/v1/runtime/workspaces
///
/// Returns only active workspaces. Archived/dead workspaces are excluded
/// from the API surface — the reconciler handles cleanup internally.
#[utoipa::path(
    operation_id = "listWorkspaces",
    get,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_workspaces(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    // Source of truth: git worktrees. DB rows overlay metadata when present.
    let mut workspaces: Vec<serde_json::Value> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 1. Git worktrees (one per branch checked out).
    if let Some(wt) = git_worktrees() {
        for (branch, path) in wt {
            if seen_names.contains(&branch) { continue; }
            seen_names.insert(branch.clone());
            workspaces.push(serde_json::json!({
                "id": branch.replace('/', "-"),
                "name": branch,
                "path": path,
                "active_agent": null,
                "created_at": chrono::Utc::now().to_rfc3339(),
            }));
        }
    }

    // 2. DB-tracked workspaces not already listed (defensive — any rows remaining
    //    from a prior run that still correspond to something meaningful).
    if let Ok(db_ws) = runtime::workspace::list_workspaces(state.ship_dir()) {
        for w in db_ws {
            if seen_names.contains(&w.name) { continue; }
            seen_names.insert(w.name.clone());
            workspaces.push(serde_json::to_value(&w).unwrap_or(serde_json::json!({})));
        }
    }

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "workspaces": workspaces }),
        }),
    )
        .into_response()
}

/// Resolve HEAD SHA of the active project root. Used at `job.created` time
/// to capture the pipeline's baseline — the point every future worktree
/// branches from, and the `from` end of gate diff evaluation. `None` when
/// the project dir can't be resolved or git fails; gates fall back to the
/// step-local start SHA in that case.
fn resolve_project_head_sha() -> Option<String> {
    let project_ship = runtime::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    runtime::services::orchestrator::git_helpers::resolve_head_sha(project_root)
}

/// Parse `git worktree list --porcelain` from the active project root.
/// Returns (branch, path) pairs, skipping detached and bare entries.
fn git_worktrees() -> Option<Vec<(String, String)>> {
    let project_ship = runtime::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() { return None; }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let mut path: Option<String> = None;
        let mut branch: Option<String> = None;
        let mut skip = false;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p.trim().to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = Some(b.trim().to_string());
            } else if line == "bare" || line == "detached" {
                skip = true;
            }
        }
        if skip { continue; }
        if let (Some(p), Some(b)) = (path, branch) {
            out.push((b, p));
        }
    }
    Some(out)
}

/// GET /api/v1/runtime/sessions[?workspace_id=<id>]
#[utoipa::path(
    operation_id = "listSessions",
    get,
    path = "/api/v1/sessions",
    params(SessionsQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_sessions(
    State(_state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<SessionsQuery>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    let workspace_id = params.workspace_id.as_deref();
    match runtime::db::session::list_workspace_sessions_db(workspace_id, 100) {
        Ok(sessions) => {
            let data: Vec<serde_json::Value> = sessions
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "workspace_id": s.workspace_id,
                        "workspace_branch": s.workspace_branch,
                        "status": s.status,
                        "started_at": s.started_at,
                        "ended_at": s.ended_at,
                        "agent_id": s.agent_id,
                        "primary_provider": s.primary_provider,
                        "goal": s.goal,
                        "summary": s.summary,
                        "tool_call_count": s.tool_call_count,
                        "created_at": s.created_at,
                        "updated_at": s.updated_at,
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "sessions": data }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
        .into_response()
}

/// GET /api/v1/runtime/agents
#[utoipa::path(
    operation_id = "listAgents",
    get,
    path = "/api/v1/agents",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_agents(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    // An agent is either (a) defined on disk as a .ship/agents/<id> profile,
    // (b) live in the mesh registry (registered via REST/MCP), or both.
    // Return the union keyed by agent_id so tests that populate only the
    // mesh see live entries, and production users see their compiled profiles.
    //
    // Uses state.ship_dir() — walking up from CWD bleeds the main repo's
    // agents into isolated test harnesses or any caller running from a
    // different working directory.
    let mut by_id: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();

    let agents_dir = state.ship_dir().join("agents");
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".jsonc") && !name.ends_with(".toml") {
                continue;
            }
            let id = name.trim_end_matches(".jsonc").trim_end_matches(".toml").to_string();
            let label = std::fs::read_to_string(entry.path())
                .ok()
                .and_then(|content| {
                    let cleaned: String = content
                        .lines()
                        .map(|l| if l.trim_start().starts_with("//") { "" } else { l })
                        .collect::<Vec<_>>()
                        .join("\n");
                    serde_json::from_str::<serde_json::Value>(&cleaned).ok()
                })
                .and_then(|v| v["name"].as_str().map(str::to_string))
                .unwrap_or_else(|| id.clone());
            by_id.insert(id.clone(), serde_json::json!({ "agent_id": id, "label": label }));
        }
    }

    // Mesh registry entries. If an id already has a profile, mesh adds
    // status/capabilities to the existing entry; otherwise a fresh row.
    let mesh = state.mesh_registry().read().await;
    for (id, entry) in mesh.iter() {
        let mut merged = by_id.remove(id).unwrap_or_else(|| {
            serde_json::json!({ "agent_id": id, "label": entry.label.clone() })
        });
        merged["status"] = serde_json::json!(format!("{:?}", entry.status).to_lowercase());
        merged["capabilities"] = serde_json::json!(entry.capabilities);
        by_id.insert(id.clone(), merged);
    }

    let agents: Vec<serde_json::Value> = by_id.into_values().collect();
    (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!({ "agents": agents }) }))
        .into_response()
}

// ---- Job request types ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "slug": "write-readme",
    "agent": "writer",
    "branch": "job/write-readme",
    "spec_path": ".ship/specs/write-readme.md",
    "source_spec_path": "/home/me/project/.ship/specs/write-readme.md",
    "spec_content": "# Write README\n\nSummarize the project.\n",
    "vars": { "topic": "ship" },
    "headless": false
}))]
pub struct CreateJobRequest {
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Operator-facing path to the spec file (absolute when resolvable).
    /// Plumbed into the JobRecord so `ship jobs` can distinguish which
    /// spec each job was dispatched from. Distinct from `spec_path`
    /// (daemon-side worktree destination). Optional: `--spec-text`
    /// dispatches and legacy callers send `None`.
    #[serde(default)]
    pub source_spec_path: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub pipeline: Option<String>,
    pub provider: Option<String>,
    /// Inline spec content. Travels in the JobCreated event; the worktree
    /// adapter writes it to `spec_path` at dispatch time.
    pub spec_content: Option<String>,
    /// Pipeline variables for `{{ var }}` interpolation in step fields.
    /// Merged over pipeline-file defaults; CLI `--var` flags land here.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,
    /// When true, force the headless (ProcessExecutor) path. When false or
    /// absent, the default TmuxWindowExecutor (tmux-window) path is used.
    #[serde(default)]
    pub headless: bool,
}

/// GET /api/v1/runtime/jobs
///
/// Returns each `JobRecord` with two operator-visibility annotations:
///
/// - `process_state`: `"alive"` when the latest process projection for the
///   slug is still Running, `"missing"` when the most recent supervisor
///   sweep flagged drift (no_process) without a coherence_restored after
///   it, `"terminal"` for jobs in a terminal status, `"none"` otherwise.
/// - `notes`: optional truncated error / blocker for the row's NOTES column.
#[utoipa::path(
    operation_id = "listJobs",
    get,
    path = "/api/v1/jobs",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_jobs(State(state): State<ApiState>) -> impl IntoResponse {
    let jobs = match runtime::projections::job::load_jobs_at(state.ship_dir()) {
        Ok(j) => j,
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
    let processes = list_processes_at(state.ship_dir(), None).unwrap_or_default();
    let supervisor_events =
        load_recent_events_by_type(state.ship_dir(), &["supervisor.drift_detected", "supervisor.coherence_restored"], 500)
            .unwrap_or_default();

    let annotated: Vec<serde_json::Value> = jobs
        .values()
        .map(|j| annotate_job(j, &processes, &supervisor_events))
        .collect();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "jobs": annotated }),
        }),
    )
}

/// Annotate a job with `process_state` + `notes` for `ship jobs`.
fn annotate_job(
    job: &runtime::projections::job::JobRecord,
    processes: &[runtime::projections::process::ProcessRecord],
    supervisor_events: &[runtime::events::EventEnvelope],
) -> serde_json::Value {
    use runtime::projections::job::JobStatus;
    use runtime::projections::process::ProcessStatus;

    let terminal = matches!(
        job.status,
        JobStatus::Failed | JobStatus::Merged | JobStatus::Cancelled | JobStatus::Completed
    );

    // Most recent process for this slug, by spawned_at.
    let latest_proc = processes
        .iter()
        .filter(|p| p.slug == job.slug)
        .max_by_key(|p| p.spawned_at);

    let process_state = if terminal {
        "terminal"
    } else if matches!(latest_proc.map(|p| p.status), Some(ProcessStatus::Running)) {
        "alive"
    } else if drift_active_for_slug(&job.slug, supervisor_events) {
        "missing"
    } else {
        "none"
    };

    let notes = job
        .error
        .as_ref()
        .or(job.blocker.as_ref())
        .map(|s| truncate(s, 60));

    let mut obj = serde_json::to_value(job).unwrap_or(serde_json::json!({}));
    if let Some(map) = obj.as_object_mut() {
        map.insert("process_state".into(), serde_json::json!(process_state));
        map.insert("notes".into(), serde_json::json!(notes));
    }
    obj
}

/// True if the most recent supervisor.drift_detected for `slug` is newer
/// than any subsequent supervisor.coherence_restored — i.e. the drift has
/// not yet been resolved by a sweep action.
fn drift_active_for_slug(slug: &str, events: &[runtime::events::EventEnvelope]) -> bool {
    let mut latest_drift: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut latest_restored: Option<chrono::DateTime<chrono::Utc>> = None;
    for e in events {
        if e.entity_id != slug {
            continue;
        }
        match e.event_type.as_str() {
            "supervisor.drift_detected" => {
                latest_drift = Some(latest_drift.map_or(e.created_at, |d| d.max(e.created_at)));
            }
            "supervisor.coherence_restored" => {
                latest_restored =
                    Some(latest_restored.map_or(e.created_at, |d| d.max(e.created_at)));
            }
            _ => {}
        }
    }
    match (latest_drift, latest_restored) {
        (Some(d), Some(r)) => d > r,
        (Some(_), None) => true,
        _ => false,
    }
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        chars[..max.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}

/// Project process events from an explicit ship_dir without going through
/// the global db_path. Mirrors `load_jobs_at` for jobs.
fn list_processes_at(
    ship_dir: &std::path::Path,
    slug: Option<&str>,
) -> anyhow::Result<Vec<runtime::projections::process::ProcessRecord>> {
    runtime::db::ensure_db_at(ship_dir)?;
    let events = runtime::db::events::list_recent_events_at(ship_dir, 50_000)?;
    let mut records = runtime::projections::process::project(&events);
    if let Some(s) = slug {
        records.retain(|r| r.slug == s);
    }
    Ok(records)
}

/// Filter recent events by event_type set, against an explicit ship_dir.
fn load_recent_events_by_type(
    ship_dir: &std::path::Path,
    types: &[&str],
    limit: usize,
) -> anyhow::Result<Vec<runtime::events::EventEnvelope>> {
    runtime::db::ensure_db_at(ship_dir)?;
    let events = runtime::db::events::list_recent_events_at(ship_dir, limit)?;
    Ok(events
        .into_iter()
        .filter(|e| types.contains(&e.event_type.as_str()))
        .collect())
}

// ---- Job detail response types ----

#[derive(serde::Serialize)]
pub struct JobDetailResponse {
    job: runtime::projections::job::JobRecord,
    step_observations: Vec<runtime::events::job::StepObservedPayload>,
    recent_events: Vec<EventSummary>,
}

#[derive(serde::Serialize)]
pub struct EventSummary {
    event_type: String,
    created_at: String,
    payload_preview: String,
}

/// GET /api/v1/runtime/jobs/{job_id}
#[utoipa::path(
    operation_id = "getJobDetail",
    get,
    path = "/api/v1/jobs/{job_id}",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "job not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn get_job_detail(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(job_id): Path<String>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ReadJobs) {
        return e.into_response();
    }
    let jobs = match runtime::projections::job::load_jobs_at(state.ship_dir()) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            ).into_response();
        }
    };

    let job = match jobs.get(&job_id) {
        Some(j) => j.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("job '{job_id}' not found")),
                }),
            ).into_response();
        }
    };

    let platform_db = runtime::db::platform_db_path(state.ship_dir());
    let step_observations = runtime::db::events::query_events_by_entity_and_type_at(
        &platform_db, &job_id, "step.observed",
    )
    .unwrap_or_default()
    .into_iter()
    .filter_map(|env| {
        serde_json::from_str::<runtime::events::job::StepObservedPayload>(&env.payload_json).ok()
    })
    .collect::<Vec<_>>();

    let recent_events = runtime::db::events::query_recent_events_for_entity_at(&platform_db, &job_id, 50)
        .unwrap_or_default()
        .into_iter()
        .map(|env| {
            let preview = if env.payload_json.len() > 200 {
                let boundary = env.payload_json
                    .char_indices()
                    .take_while(|(i, _)| *i < 200)
                    .last()
                    .map(|(i, c)| i + c.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &env.payload_json[..boundary])
            } else {
                env.payload_json.clone()
            };
            EventSummary {
                event_type: env.event_type.clone(),
                created_at: env.created_at.to_rfc3339(),
                payload_preview: preview,
            }
        })
        .collect::<Vec<_>>();

    let detail = JobDetailResponse { job, step_observations, recent_events };
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!(detail),
        }),
    )
        .into_response()
}

/// POST /api/v1/runtime/jobs — emit job.created event through the kernel.
///
/// The kernel persists the event and routes it to subscribers.
/// The job_dispatch service handles all heavy lifting (worktree creation,
/// dependency resolution, agent spawning).
#[utoipa::path(
    operation_id = "createJob",
    post,
    path = "/api/v1/jobs",
    request_body = CreateJobRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "job slug already exists", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn create_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::CreateJobs) {
        return e;
    }
    let job_id = runtime::gen_ulid();
    let baseline_sha = resolve_project_head_sha();
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: req.slug,
        agent: req.agent,
        branch: req.branch,
        spec_path: req.spec_path,
        source_spec_path: req.source_spec_path,
        plan_id: None,
        model: None,
        provider: req.provider,
        depends_on: req.depends_on,
        pipeline: None,
        steps: None,
        pipeline_name: req.pipeline,
        vars: req.vars,
        spec_content: req.spec_content,
        step_specs: None,
        headless: req.headless,
        baseline_sha,
    };

    // Enrich with pipeline BEFORE persisting — the persisted event must include steps
    // so projections can track pipeline state.
    let payload = match runtime::services::orchestrator::pipeline_loader::ensure_pipeline(payload) {
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
                StatusCode::BAD_REQUEST,
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

// ---- Cancel ----

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CancelJobRequest {
    #[serde(default = "default_cancel_reason")]
    pub reason: String,
}

fn default_cancel_reason() -> String {
    "operator-cancelled".to_string()
}

/// POST /api/v1/runtime/jobs/{slug}/cancel — emit `job.cancelled` for the
/// job with this slug. Idempotent: returns 409 with the current status
/// when the job is already in a terminal state, and emits no event.
#[utoipa::path(
    operation_id = "cancelJob",
    post,
    path = "/api/v1/jobs/{slug}/cancel",
    request_body = CancelJobRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "job not found", body = crate::openapi_types::ErrorResponse),
        (status = 409, description = "job already finished", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn cancel_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<CancelJobRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }

    let jobs = match runtime::projections::job::load_jobs_at(state.ship_dir()) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    let job = match jobs.values().find(|j| j.slug == slug) {
        Some(j) => j.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("no job with slug '{slug}'")),
                }),
            );
        }
    };

    use runtime::projections::job::JobStatus;
    if matches!(
        job.status,
        JobStatus::Failed | JobStatus::Merged | JobStatus::Cancelled | JobStatus::Completed
    ) {
        return (
            StatusCode::CONFLICT,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({
                    "slug": &slug,
                    "status": format!("{:?}", job.status).to_lowercase(),
                }),
            }),
        );
    }

    let payload = runtime::events::job::JobCancelledPayload {
        job_id: job.job_id.clone(),
        slug: slug.clone(),
        reason: req.reason,
        requested_by: "operator".to_string(),
    };

    let envelope = match EventEnvelope::new(
        event_types::JOB_CANCELLED,
        &job.job_id,
        &payload,
    )
    .map(|e| e.with_actor_id("operator"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "slug": &slug, "status": "cancelling" }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}

// ---- Job end (hook-triggered) ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "slug": "write-readme" }))]
pub struct JobEndRequest {
    pub slug: String,
}

/// POST /api/v1/runtime/jobs/end — signal that a provider turn ended for a job.
///
/// Called by the session Stop hook when an agent finishes a response turn.
/// For multi-step pipeline jobs, this sends a TurnEnded signal to the step
/// monitor (which accumulates evidence before deciding to advance).
/// For single-step jobs, emits job.completed directly.
#[utoipa::path(
    operation_id = "endJob",
    post,
    path = "/api/v1/jobs/end",
    request_body = JobEndRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn end_job(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<JobEndRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }
    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    let job = match jobs.values()
        .filter(|j| j.slug == req.slug)
        .find(|j| j.status == runtime::projections::job::JobStatus::Dispatched)
    {
        Some(j) => j.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("no dispatched job with slug '{}'", req.slug)) }),
            );
        }
    };

    let is_pipeline = job.steps.as_ref().is_some_and(|s| s.steps.len() > 1);

    // Multi-step pipeline: send TurnEnded to the step monitor.
    // The monitor accumulates evidence and decides when to advance.
    if is_pipeline {
        if let Some(dh) = state.dispatch_handle() {
            let txs = dh.activity_txs.lock().await;
            if let Some(tx) = txs.get(&req.slug) {
                let _ = tx.send(runtime::services::orchestrator::StepActivity::TurnEnded);
                return (
                    StatusCode::OK,
                    Json(MeshResponse { ok: true, data: serde_json::json!(format!("turn ended for pipeline step '{}'", req.slug)) }),
                );
            }
        }
        // No active step monitor — fall through to job.completed for safety.
        tracing::warn!(slug = req.slug, "end_job: pipeline step has no active step monitor, falling through to job.completed");
    }

    // Single-step job or fallback: emit job.completed directly.
    let payload = runtime::events::job::JobCompletedPayload {
        job_id: job.job_id.clone(),
        slug: job.slug.clone(),
    };

    let envelope = match EventEnvelope::new(event_types::JOB_COMPLETED, &job.job_id, &payload)
        .map(|e| e.with_actor_id("hook"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("job '{}' completed", req.slug)) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}

// ---- Agent control ----

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SteerAgentRequest {
    pub message: String,
}

/// POST /api/v1/runtime/agents/{slug}/stop — stop a running agent by slug.
///
/// Uses the process registry and executor to terminate the agent process.
#[utoipa::path(
    operation_id = "stopAgent",
    post,
    path = "/api/v1/agents/{slug}/stop",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "agent not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn stop_agent(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let handle = {
        let mut reg = dh.registry.lock().await;
        match reg.take(&slug) {
            Some(h) => h,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
                );
            }
        }
    };

    match dh.executor.stop(&handle).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("stopped: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to stop agent: {e}")) }),
        ),
    }
}

/// POST /api/v1/runtime/agents/{slug}/steer — inject a message into a running agent.
#[utoipa::path(
    operation_id = "steerAgent",
    post,
    path = "/api/v1/agents/{slug}/steer",
    request_body = SteerAgentRequest,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "agent not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn steer_agent(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<SteerAgentRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e;
    }
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let reg = dh.registry.lock().await;
    let handle = match reg.get(&slug) {
        Some(h) => h,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
            );
        }
    };

    match dh.executor.send(handle, &req.message).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("steered: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to steer agent: {e}")) }),
        ),
    }
}

// ---- Procs / events / daemon status (slice 6) ----

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ProcsQuery {
    pub slug: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/v1/runtime/procs[?slug=X][&limit=N]
///
/// Returns process records from the `process.*` projection. Without
/// `slug`, returns the most recent `limit` records (default 100), sorted
/// newest first.
#[utoipa::path(
    operation_id = "listProcs",
    get,
    path = "/api/v1/procs",
    params(ProcsQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_procs(
    State(state): State<ApiState>,
    Query(params): Query<ProcsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100);
    let slug = params.slug.as_deref();
    match list_processes_at(state.ship_dir(), slug) {
        Ok(mut records) => {
            // Newest first; cap to limit.
            records.sort_by(|a, b| b.spawned_at.cmp(&a.spawned_at));
            if records.len() > limit {
                records.truncate(limit);
            }
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "procs": records }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct EventsPollQuery {
    /// ULID cursor — return events with id > since_id, ordered ASC.
    pub since_id: Option<String>,
    /// If set, only return events whose envelope `entity_id` matches OR
    /// whose payload contains `"slug": "<slug>"`.
    pub slug: Option<String>,
    /// Optional event-namespace prefix filter ("job", "process", "supervisor").
    pub namespace: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/v1/runtime/events/poll
///
/// Polling fallback for `ship tail`. SSE replays only future events;
/// poll allows the CLI to walk the log forward from a known cursor.
/// Use `since_id` (ULID) for incremental polling. Limit defaults to 200.
#[utoipa::path(
    operation_id = "pollEvents",
    get,
    path = "/api/v1/events/poll",
    params(EventsPollQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn poll_events(
    State(state): State<ApiState>,
    Query(params): Query<EventsPollQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(200).min(2000);
    let ship_dir = state.ship_dir();
    if let Err(e) = runtime::db::ensure_db_at(ship_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        );
    }
    // Read the most recent N events ascending; we filter in memory.
    let events = match runtime::db::events::list_recent_events_at(ship_dir, 5_000) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
            );
        }
    };

    let cursor = params.since_id.as_deref();
    let slug = params.slug.as_deref();
    let ns = params.namespace.as_deref();

    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| match cursor {
            Some(c) => e.id.as_str() > c,
            None => true,
        })
        .filter(|e| match ns {
            Some(prefix) => e.event_type.starts_with(&format!("{prefix}.")),
            None => true,
        })
        .filter(|e| match slug {
            Some(s) => {
                if e.entity_id == s {
                    return true;
                }
                serde_json::from_str::<serde_json::Value>(&e.payload_json)
                    .ok()
                    .and_then(|v| v["slug"].as_str().map(str::to_string))
                    .map(|p| p == s)
                    .unwrap_or(false)
            }
            None => true,
        })
        .take(limit)
        .collect();

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "events": filtered }),
        }),
    )
}

/// GET /api/v1/runtime/daemon/status
///
/// Real, live status for `ship daemon status` — pid, version, uptime,
/// job counts by status, last sweep timestamp, drift count over the
/// last 50 sweep events.
#[utoipa::path(
    operation_id = "daemonStatus",
    get,
    path = "/api/v1/daemon/status",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn daemon_status(State(state): State<ApiState>) -> impl IntoResponse {
    use runtime::projections::job::JobStatus;

    let ship_dir = state.ship_dir();
    let jobs = runtime::projections::job::load_jobs_at(ship_dir).unwrap_or_default();
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
        load_recent_events_by_type(ship_dir, &["supervisor.sweep_completed"], 100)
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
    match runtime::db::rebuild::rebuild_projections(state.ship_dir()) {
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

/// Stops an actor when dropped — used to clean up the SSE stream actor.
pub(crate) struct ActorGuard {
    pub(crate) actor_id: String,
    pub(crate) kernel: crate::rest_api::SharedKernel,
}

impl Drop for ActorGuard {
    fn drop(&mut self) {
        let actor_id = self.actor_id.clone();
        let kernel = self.kernel.clone();
        tokio::spawn(async move {
            let _ = kernel.lock().await.stop_actor(&actor_id);
        });
    }
}

/// GET /api/v1/runtime/events — SSE stream of all kernel events.
#[utoipa::path(
    operation_id = "eventStream",
    get,
    path = "/api/v1/events",
    responses(
        (status = 200, description = "SSE stream of kernel events"),
        (status = 401, description = "unauthorized", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn event_stream(
    State(state): State<ApiState>,
) -> Result<Sse<impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    let actor_id = format!(
        "studio-events-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["".to_string()],
    };

    let mailbox = {
        let mut kernel = state.kernel().lock().await;
        kernel.spawn_actor(&actor_id, config).map(|(_store, mb)| mb).map_err(|e| {
            tracing::warn!("event_stream: actor spawn failed: {e}");
            let status = match &e {
                runtime::KernelError::ActorAlreadyExists { .. } => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })?
    };

    let guard = ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;
        let mut mb = mailbox;
        while let Some(event) = mb.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default().event("ship.event").data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Extension;
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        runtime::db::ensure_db_at(&ship_dir).unwrap();
        // Isolated per-test kernel — init_kernel_router uses a OnceLock that
        // pins the first test's ship_dir globally, breaking all subsequent
        // tests with "unable to open database file" when they try to write.
        let kernel = runtime::events::new_isolated_for_test(ship_dir.clone()).unwrap();
        Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }

    fn test_auth() -> Extension<AuthContext> {
        Extension(AuthContext {
            caller_kind: runtime::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        })
    }

    async fn collect_body(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_ok_with_array() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = list_workspaces(State(state), test_auth()).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["workspaces"].is_array(), "expected workspaces array");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_ok_no_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let query = Query(SessionsQuery { workspace_id: None });
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["sessions"].is_array(), "expected sessions array");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_ok_with_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let query = Query(SessionsQuery {
            workspace_id: Some("nonexistent-workspace".to_string()),
        });
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["sessions"].is_array());
        assert_eq!(val["data"]["sessions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_agents_returns_ok_empty_registry() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = list_agents(State(state), test_auth()).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["agents"].is_array());
        assert_eq!(val["data"]["agents"].as_array().unwrap().len(), 0);
    }

    fn create_test_workspace(state: &ApiState, branch: &str) {
        runtime::workspace::create_workspace(
            state.ship_dir(),
            runtime::workspace::CreateWorkspaceRequest {
                branch: branch.into(),
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_created_workspace() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/rt-ws-test");

        let response = list_workspaces(State(state), test_auth()).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();
        assert!(
            workspaces.iter().any(|w| w["name"] == "feature/rt-ws-test"),
            "created workspace should appear in list"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_agents_returns_populated_registry() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);

        {
            let mut reg = state.mesh_registry().write().await;
            reg.insert(
                "agent.test".into(),
                runtime::services::mesh::MeshEntry {
                    agent_id: "agent.test".into(),
                    label: "Test Agent".into(),
                    capabilities: vec!["rust".into()],
                    registered_at: chrono::Utc::now(),
                    status: runtime::services::mesh::AgentStatus::Active,
                },
            );
        }

        let response = list_agents(State(state), test_auth()).await.into_response();
        let val = collect_body(response).await;
        let agents = val["data"]["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0]["agent_id"], "agent.test");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_created_session() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/rt-sess-test");

        let ws = runtime::workspace::get_workspace(state.ship_dir(), "feature/rt-sess-test")
            .unwrap()
            .unwrap();
        let payload = runtime::events::types::SessionStarted {
            goal: Some("test".into()),
            workspace_id: ws.id.clone(),
            workspace_branch: ws.name.clone(),
            ..Default::default()
        };
        runtime::db::session_events::insert_session_with_started_event(
            "rt-test-session",
            &ws.id,
            &payload,
        )
        .unwrap();

        let query = Query(SessionsQuery { workspace_id: None });
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
        let val = collect_body(response).await;
        let sessions = val["data"]["sessions"].as_array().unwrap();
        assert!(sessions.len() >= 1, "created session should appear");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_filters_by_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/ws-filter-a");
        create_test_workspace(&state, "feature/ws-filter-b");

        let ws_a = runtime::workspace::get_workspace(state.ship_dir(), "feature/ws-filter-a")
            .unwrap()
            .unwrap();
        let payload = runtime::events::types::SessionStarted {
            goal: Some("a".into()),
            workspace_id: ws_a.id.clone(),
            workspace_branch: ws_a.name.clone(),
            ..Default::default()
        };
        runtime::db::session_events::insert_session_with_started_event(
            "rt-filter-sess",
            &ws_a.id,
            &payload,
        )
        .unwrap();

        // Filter by workspace B — should return empty.
        let ws_b = runtime::workspace::get_workspace(state.ship_dir(), "feature/ws-filter-b")
            .unwrap()
            .unwrap();
        let query = Query(SessionsQuery {
            workspace_id: Some(ws_b.id),
        });
        let response = list_sessions(State(state), test_auth(), query).await.into_response();
        let val = collect_body(response).await;
        let sessions = val["data"]["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 0, "no sessions for workspace B");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_multiple() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/multi-a");
        create_test_workspace(&state, "feature/multi-b");

        let response = list_workspaces(State(state), test_auth()).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();
        assert!(workspaces.len() >= 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_all() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        create_test_workspace(&state, "feature/ws-a");
        create_test_workspace(&state, "feature/ws-b");

        let response = list_workspaces(State(state), test_auth()).await.into_response();
        let val = collect_body(response).await;
        let workspaces = val["data"]["workspaces"].as_array().unwrap();

        assert!(
            workspaces.iter().any(|w| w["name"] == "feature/ws-a"),
            "workspace a should appear"
        );
        assert!(
            workspaces.iter().any(|w| w["name"] == "feature/ws-b"),
            "workspace b should appear"
        );
    }

    fn full_auth() -> AuthContext {
        AuthContext {
            caller_kind: runtime::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn stop_agent_no_dispatch_handle_returns_503() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = stop_agent(State(state), Extension(full_auth()), Path("test-slug".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_job_detail_not_found() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = get_job_detail(State(state), test_auth(), Path("nonexistent-job-id".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(false));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_job_detail_returns_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);

        // Emit a job.created event so load_jobs() finds it.
        let job_id = runtime::gen_ulid();
        let payload = runtime::events::job::JobCreatedPayload {
            job_id: job_id.clone(),
            slug: "test-detail".into(),
            agent: "agent-1".into(),
            branch: "feature/detail".into(),
            spec_path: ".artifacts/job-spec.md".into(),
            source_spec_path: None,
            plan_id: None,
            model: None,
            provider: None,
            depends_on: None,
            pipeline: None,
            steps: None,
            pipeline_name: None,
            vars: None,
            spec_content: None,
            step_specs: None,
            headless: false,
            baseline_sha: None,
        };
        let envelope = runtime::events::EventEnvelope::new(
            runtime::events::job::event_types::JOB_CREATED,
            &job_id,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&envelope).await.unwrap();

        let response = get_job_detail(State(state), test_auth(), Path(job_id.clone()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert_eq!(val["data"]["job"]["job_id"], job_id);
        assert_eq!(val["data"]["job"]["slug"], "test-detail");
        assert!(val["data"]["step_observations"].is_array());
        assert!(val["data"]["recent_events"].is_array());
        // The job.created event should appear in recent_events
        assert!(val["data"]["recent_events"].as_array().unwrap().len() >= 1);
    }

    async fn emit_job_created(state: &ApiState, slug: &str) -> String {
        let job_id = runtime::gen_ulid();
        let payload = runtime::events::job::JobCreatedPayload {
            job_id: job_id.clone(),
            slug: slug.into(),
            agent: "a".into(),
            branch: format!("job/{slug}"),
            spec_path: ".artifacts/job-spec.md".into(),
            source_spec_path: None,
            plan_id: None, model: None, provider: None, depends_on: None,
            pipeline: None, steps: None, pipeline_name: None, vars: None,
            spec_content: None, step_specs: None, headless: false,
            baseline_sha: None,
        };
        let env = EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
            .unwrap()
            .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
        job_id
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_emits_job_cancelled_for_pending_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let job_id = emit_job_created(&state, "cancel-ok").await;

        let req = CancelJobRequest { reason: "smoke".into() };
        let response = cancel_job(
            State(state.clone()),
            Extension(full_auth()),
            Path("cancel-ok".to_string()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let jobs = runtime::projections::job::load_jobs_at(state.ship_dir()).unwrap();
        let job = jobs.get(&job_id).unwrap();
        assert_eq!(job.status, runtime::projections::job::JobStatus::Cancelled);

        // Confirm the cancelled event actually landed.
        let events = runtime::db::events::query_recent_events_for_entity_at(
            &runtime::db::platform_db_path(state.ship_dir()),
            &job_id, 20,
        )
        .unwrap();
        assert!(
            events.iter().any(|e| e.event_type == event_types::JOB_CANCELLED),
            "expected job.cancelled in event log"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_returns_409_for_already_terminal_job() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let job_id = emit_job_created(&state, "cancel-term").await;

        // Seed a terminal state first.
        let fail_payload = runtime::events::job::JobFailedPayload {
            job_id: job_id.clone(),
            error: "boom".into(),
        };
        let env = EventEnvelope::new(event_types::JOB_FAILED, &job_id, &fail_payload)
            .unwrap()
            .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();

        let req = CancelJobRequest { reason: "after-fail".into() };
        let response = cancel_job(
            State(state.clone()),
            Extension(full_auth()),
            Path("cancel-term".to_string()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        // Make sure no job.cancelled was emitted (idempotent no-op).
        let events = runtime::db::events::query_recent_events_for_entity_at(
            &runtime::db::platform_db_path(state.ship_dir()),
            &job_id, 20,
        )
        .unwrap();
        assert!(
            events.iter().all(|e| e.event_type != event_types::JOB_CANCELLED),
            "409 path must emit no job.cancelled event"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_job_returns_404_for_unknown_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let req = CancelJobRequest { reason: "x".into() };
        let response = cancel_job(
            State(state),
            Extension(full_auth()),
            Path("nope".to_string()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn steer_agent_no_dispatch_handle_returns_503() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let req = SteerAgentRequest { message: "hello".to_string() };
        let response = steer_agent(State(state), Extension(full_auth()), Path("test-slug".to_string()), Json(req))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    // ---- Slice 6 endpoints ----

    async fn emit_process_spawned(state: &ApiState, slug: &str, pid: u32) {
        let payload = runtime::events::process::ProcessSpawnedPayload {
            slug: slug.to_string(),
            job_id: format!("job-{slug}"),
            pid: Some(pid),
            cmd: "claude".to_string(),
            argv: vec![],
            env_keys: vec![],
            cwd: "/tmp".to_string(),
            executor_kind: "process".to_string(),
            step_id: None,
        };
        let env = EventEnvelope::new(
            runtime::events::process::event_types::PROCESS_SPAWNED,
            slug,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    async fn emit_drift_detected(state: &ApiState, slug: &str) {
        let payload = runtime::events::supervisor::DriftDetectedPayload {
            slug: slug.into(),
            job_id: format!("job-{slug}"),
            expected_status: "Dispatched".into(),
            actual_state: "no_process".into(),
            detail: "test".into(),
        };
        let env = EventEnvelope::new(
            runtime::events::supervisor::event_types::SUPERVISOR_DRIFT_DETECTED,
            slug,
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    async fn emit_sweep_completed(state: &ApiState, drift_count: usize) {
        let payload = runtime::events::supervisor::SweepCompletedPayload {
            duration_ms: 12,
            drift_count,
            actions_taken: drift_count,
        };
        let env = EventEnvelope::new(
            runtime::events::supervisor::event_types::SUPERVISOR_SWEEP_COMPLETED,
            "supervisor",
            &payload,
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_jobs_annotates_terminal_jobs_as_terminal() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let job_id = emit_job_created(&state, "term-anno").await;

        // Drive into Failed.
        let env = EventEnvelope::new(
            event_types::JOB_FAILED,
            &job_id,
            &runtime::events::job::JobFailedPayload {
                job_id: job_id.clone(),
                error: "boom".into(),
            },
        )
        .unwrap()
        .with_actor_id("test");
        state.persist_and_route(&env).await.unwrap();

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "term-anno").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("terminal"));
        assert_eq!(row["notes"], serde_json::json!("boom"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_jobs_marks_drift_as_missing_when_unresolved() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _job_id = emit_job_created(&state, "drift-job").await;
        emit_drift_detected(&state, "drift-job").await;

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "drift-job").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("missing"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_jobs_marks_alive_for_running_process() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _job_id = emit_job_created(&state, "alive-job").await;
        emit_process_spawned(&state, "alive-job", 1234).await;

        let response = list_jobs(State(state)).await.into_response();
        let val = collect_body(response).await;
        let jobs = val["data"]["jobs"].as_array().unwrap();
        let row = jobs.iter().find(|j| j["slug"] == "alive-job").unwrap();
        assert_eq!(row["process_state"], serde_json::json!("alive"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_procs_filters_by_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        emit_process_spawned(&state, "procs-a", 1).await;
        emit_process_spawned(&state, "procs-b", 2).await;

        let q = Query(ProcsQuery { slug: Some("procs-a".into()), limit: None });
        let response = list_procs(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let procs = val["data"]["procs"].as_array().unwrap();
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0]["slug"], "procs-a");
        assert_eq!(procs[0]["pid"], 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_procs_returns_all_when_no_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        emit_process_spawned(&state, "all-a", 10).await;
        emit_process_spawned(&state, "all-b", 11).await;

        let q = Query(ProcsQuery { slug: None, limit: None });
        let response = list_procs(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let procs = val["data"]["procs"].as_array().unwrap();
        assert!(procs.len() >= 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn poll_events_filters_by_slug() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "tail-x").await;
        let _ = emit_job_created(&state, "tail-y").await;

        let q = Query(EventsPollQuery {
            since_id: None,
            slug: Some("tail-x".into()),
            namespace: None,
            limit: None,
        });
        let response = poll_events(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let events = val["data"]["events"].as_array().unwrap();
        assert!(!events.is_empty(), "should have at least one event for slug");
        for e in events {
            let payload: serde_json::Value =
                serde_json::from_str(e["payload_json"].as_str().unwrap_or("{}")).unwrap();
            assert_eq!(payload["slug"], "tail-x");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn poll_events_namespace_filter() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "ns-test").await;
        emit_process_spawned(&state, "ns-test", 99).await;

        let q = Query(EventsPollQuery {
            since_id: None,
            slug: Some("ns-test".into()),
            namespace: Some("process".into()),
            limit: None,
        });
        let response = poll_events(State(state), q).await.into_response();
        let val = collect_body(response).await;
        let events = val["data"]["events"].as_array().unwrap();
        assert!(!events.is_empty());
        for e in events {
            assert!(
                e["event_type"].as_str().unwrap().starts_with("process."),
                "namespace filter must restrict to process.*"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn poll_events_respects_since_id_cursor() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "cursor-test").await;

        let first = Query(EventsPollQuery {
            since_id: None,
            slug: Some("cursor-test".into()),
            namespace: None,
            limit: None,
        });
        let response = poll_events(State(state.clone()), first).await.into_response();
        let val = collect_body(response).await;
        let events = val["data"]["events"].as_array().unwrap();
        let last_id = events.last().unwrap()["id"].as_str().unwrap().to_string();

        let second = Query(EventsPollQuery {
            since_id: Some(last_id),
            slug: Some("cursor-test".into()),
            namespace: None,
            limit: None,
        });
        let response = poll_events(State(state), second).await.into_response();
        let val = collect_body(response).await;
        assert_eq!(val["data"]["events"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn daemon_status_returns_pid_and_uptime() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let _ = emit_job_created(&state, "status-job").await;
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

    #[tokio::test(flavor = "multi_thread")]
    async fn drift_active_for_slug_resolved_returns_false() {
        let drift_ts = chrono::Utc::now() - chrono::Duration::seconds(10);
        let restored_ts = chrono::Utc::now();
        let mk = |t: &str, ts: chrono::DateTime<chrono::Utc>| EventEnvelope {
            id: "x".into(),
            event_type: t.into(),
            entity_id: "slug".into(),
            actor: "test".into(),
            payload_json: "{}".into(),
            version: 1,
            causation_id: None,
            workspace_id: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: ts,
        };
        let events = vec![
            mk("supervisor.drift_detected", drift_ts),
            mk("supervisor.coherence_restored", restored_ts),
        ];
        assert!(!drift_active_for_slug("slug", &events));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drift_active_for_slug_unresolved_returns_true() {
        let mk = |t: &str| EventEnvelope {
            id: "x".into(),
            event_type: t.into(),
            entity_id: "slug".into(),
            actor: "test".into(),
            payload_json: "{}".into(),
            version: 1,
            causation_id: None,
            workspace_id: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: chrono::Utc::now(),
        };
        let events = vec![mk("supervisor.drift_detected")];
        assert!(drift_active_for_slug("slug", &events));
    }
}

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
    // Matches runtime::services::orchestrator::pipeline_loader::load_pipeline_file's
    // resolution strategy so ensure_pipeline (called below) will find the same file.
    let pipeline_path = {
        let mut dir = std::env::current_dir().ok();
        let mut found = None;
        while let Some(d) = dir.as_ref() {
            let pipelines_dir = runtime::ShipHome::project_ship_dir(d).join("pipelines");
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
        format!("{}-{}", workspace_name, &runtime::gen_ulid()[..8].to_lowercase())
    });
    let branch = req.branch.unwrap_or_else(|| format!("job/{slug}"));
    let job_id = runtime::gen_ulid();

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
        baseline_sha: resolve_project_head_sha(),
    };

    let payload = match runtime::services::orchestrator::pipeline_loader::ensure_pipeline(payload) {
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
