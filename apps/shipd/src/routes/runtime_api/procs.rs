//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::routes::rest_api::{ApiState, MeshResponse};

// ---- Procs / events / daemon status (slice 6) ----

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ProcsQuery {
    pub slug: Option<String>,
    /// Filter by session kind. Mirrors `SessionKind`: agent / pipeline-step / shell.
    /// Mapped to the projection's `executor_kind` field.
    pub kind: Option<String>,
    pub limit: Option<usize>,
}

/// Map a `SessionKind`-style value to the set of `executor_kind` strings
/// that the orchestrator emits for that session class. Returns `None`
/// for an unrecognized kind so the caller can surface a 400.
fn kind_to_executor_kinds(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "agent" => Some(&["pty", "tmux", "tmux_window"]),
        "pipeline-step" => Some(&["process"]),
        "shell" => Some(&["shell"]),
        _ => None,
    }
}

/// GET /api/v1/runtime/procs[?slug=X][&kind=K][&limit=N]
///
/// Returns process records from the `process.*` projection. Without
/// `slug`, returns the most recent `limit` records (default 100), sorted
/// newest first. `kind` filters by session class via `executor_kind`
/// (valid values: agent, pipeline-step, shell).
#[utoipa::path(
    operation_id = "listProcs",
    get,
    path = "/api/v1/procs",
    params(ProcsQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_procs(
    State(state): State<ApiState>,
    Query(params): Query<ProcsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100);
    let slug = params.slug.as_deref();
    let kind_filter = match params.kind.as_deref() {
        Some(k) => match kind_to_executor_kinds(k) {
            Some(set) => Some(set),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(MeshResponse {
                        ok: false,
                        data: serde_json::json!(format!(
                            "invalid kind '{k}'. Valid values: agent, pipeline-step, shell"
                        )),
                    }),
                );
            }
        },
        None => None,
    };
    match super::jobs::list_processes_at(state.ship_dir(), slug) {
        Ok(mut records) => {
            if let Some(set) = kind_filter {
                records.retain(|r| set.contains(&r.executor_kind.as_str()));
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_mapping_agent_includes_pty_and_tmux() {
        let set = kind_to_executor_kinds("agent").unwrap();
        assert!(set.contains(&"pty"));
        assert!(set.contains(&"tmux"));
        assert!(set.contains(&"tmux_window"));
    }

    #[test]
    fn kind_mapping_pipeline_step_is_process() {
        let set = kind_to_executor_kinds("pipeline-step").unwrap();
        assert_eq!(set, &["process"]);
    }

    #[test]
    fn kind_mapping_shell_is_shell() {
        let set = kind_to_executor_kinds("shell").unwrap();
        assert_eq!(set, &["shell"]);
    }

    #[test]
    fn kind_mapping_unknown_returns_none() {
        assert!(kind_to_executor_kinds("widget").is_none());
        assert!(kind_to_executor_kinds("").is_none());
    }
}
