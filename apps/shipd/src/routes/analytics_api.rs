//! `/api/v1/analytics/*` — read endpoints over the AnalyticsProjection.
//!
//! Backed by the `analytics_sessions` and `analytics_tool_uses` tables
//! maintained by `ship_analytics::AnalyticsProjection`. Pure projection
//! reads — no writes, no kernel emit, no auth-elevated paths. Scope
//! `ReadWorkspace` everywhere.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

use ship_analytics::queries::{
    get_session, list_recent_sessions, list_session_turns, session_token_summary, tool_usage,
};
use ship_substrate::db::open_db;

use crate::auth::AuthContext;
use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    /// Cap at 500 server-side regardless of what's passed. Defaults to 100.
    pub limit: Option<i64>,
}

/// `GET /api/v1/analytics/sessions` — recent sessions, newest first.
pub async fn list_sessions_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    let mut conn = match open_db() {
        Ok(c) => c,
        Err(e) => return err_response(format!("open_db: {e}")),
    };
    match list_recent_sessions(&mut conn, q.limit.unwrap_or(100)) {
        Ok(rows) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: json!({ "count": rows.len(), "sessions": rows }),
            }),
        )
            .into_response(),
        Err(e) => err_response(format!("list_recent_sessions: {e}")),
    }
}

/// `GET /api/v1/analytics/sessions/{session_key}` — one session detail.
pub async fn get_session_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(session_key): Path<String>,
) -> impl IntoResponse {
    let mut conn = match open_db() {
        Ok(c) => c,
        Err(e) => return err_response(format!("open_db: {e}")),
    };
    match get_session(&mut conn, &session_key) {
        Ok(Some(s)) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: json!({ "session": s }) }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: json!({ "error": "session_key not found" }),
            }),
        )
            .into_response(),
        Err(e) => err_response(format!("get_session: {e}")),
    }
}

#[derive(Deserialize)]
pub struct ToolUsageQuery {
    pub since: Option<String>,
    pub until: Option<String>,
}

/// `GET /api/v1/analytics/tool-usage` — per-tool aggregates (calls, errors,
/// avg duration) optionally filtered by a started_at window.
pub async fn tool_usage_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ToolUsageQuery>,
) -> impl IntoResponse {
    let mut conn = match open_db() {
        Ok(c) => c,
        Err(e) => return err_response(format!("open_db: {e}")),
    };
    match tool_usage(&mut conn, q.since.as_deref(), q.until.as_deref()) {
        Ok(rows) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: json!({ "count": rows.len(), "tools": rows }),
            }),
        )
            .into_response(),
        Err(e) => err_response(format!("tool_usage: {e}")),
    }
}

/// `GET /api/v1/analytics/sessions/{session_key}/turns` — per-turn token
/// observations chronologically. Pairs with a session_token_summary
/// aggregate so the dashboard can render both views off one call.
pub async fn session_turns_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(session_key): Path<String>,
) -> impl IntoResponse {
    let mut conn = match open_db() {
        Ok(c) => c,
        Err(e) => return err_response(format!("open_db: {e}")),
    };
    let turns = match list_session_turns(&mut conn, &session_key) {
        Ok(t) => t,
        Err(e) => return err_response(format!("list_session_turns: {e}")),
    };
    let summary = session_token_summary(&mut conn, &session_key).unwrap_or(None);
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: json!({
                "session_key": session_key,
                "summary": summary,
                "count": turns.len(),
                "turns": turns,
            }),
        }),
    )
        .into_response()
}

fn err_response(msg: String) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(MeshResponse { ok: false, data: json!({ "error": msg }) }),
    )
        .into_response()
}
