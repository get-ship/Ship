//! Client-side error capture for Ship views.
//!
//! Stores reports in a bounded in-memory ring buffer so agents iterating on
//! views can retrieve browser-side Datastar/JS errors via an MCP tool
//! instead of asking the user to copy the console.
//!
//! Not persisted — buffer resets on daemon restart. No per-user scoping;
//! auth is the daemon bearer token.

use axum::{
    Extension, Json,
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use crate::auth::{AuthContext};

const RING_CAP: usize = 200;

/// Shared global ring buffer. OnceLock because we only ever need one.
fn buffer() -> &'static Mutex<VecDeque<StoredError>> {
    static BUF: OnceLock<Mutex<VecDeque<StoredError>>> = OnceLock::new();
    BUF.get_or_init(|| Mutex::new(VecDeque::with_capacity(RING_CAP)))
}

/// Monotonically increasing error id.
fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(1);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Epoch millis when the daemon received the report.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// An incoming error report, transport-agnostic (REST `PostErrorReq` and the
/// Connect `PostViewErrorRequest` both map onto this). The store assigns `id`
/// and `timestamp_server_ms`.
pub(crate) struct ErrorReport {
    pub view: Option<String>,
    pub url: Option<String>,
    pub kind: Option<String>,
    pub message: String,
    pub stack: Option<String>,
    pub filename: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub detail: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub timestamp_client: Option<String>,
}

/// Append a report to the ring, evicting the oldest past `RING_CAP`. Returns the
/// assigned id. The single store-write path (REST + Connect both call it).
pub(crate) fn record(r: ErrorReport) -> u64 {
    let id = next_id();
    let stored = StoredError {
        id,
        view: r.view,
        url: r.url,
        kind: r.kind.unwrap_or_else(|| "unknown".to_string()),
        message: r.message,
        stack: r.stack,
        filename: r.filename,
        line: r.line,
        column: r.column,
        detail: r.detail,
        user_agent: r.user_agent,
        timestamp_client: r.timestamp_client,
        timestamp_server_ms: now_ms(),
    };
    let mut buf = buffer().lock().expect("view-errors buffer poisoned");
    if buf.len() >= RING_CAP {
        buf.pop_front();
    }
    buf.push_back(stored);
    id
}

/// Newest-first query with `since` (id >) + `view` filters, capped at `limit`.
/// Returns `(matched, total_in_buffer)`.
pub(crate) fn query(since: Option<u64>, view: Option<&str>, limit: usize) -> (Vec<StoredError>, usize) {
    let buf = buffer().lock().expect("view-errors buffer poisoned");
    let total = buf.len();
    let matched = buf
        .iter()
        .filter(|e| since.map(|s| e.id > s).unwrap_or(true))
        .filter(|e| match (view, &e.view) {
            (Some(want), Some(have)) => want == have,
            (Some(_), None) => false,
            (None, _) => true,
        })
        .rev()
        .take(limit)
        .cloned()
        .collect();
    (matched, total)
}

/// Drain the ring; returns how many were cleared.
pub(crate) fn clear() -> usize {
    let mut buf = buffer().lock().expect("view-errors buffer poisoned");
    let n = buf.len();
    buf.clear();
    n
}

#[derive(Debug, Deserialize)]
pub struct PostErrorReq {
    pub view: Option<String>,
    pub url: Option<String>,
    /// Error kind reported by the client — js-error, unhandled-rejection,
    /// datastar-fetch, console-error, etc.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub message: String,
    pub stack: Option<String>,
    pub filename: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub detail: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub timestamp_client: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredError {
    pub id: u64,
    pub view: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
    pub stack: Option<String>,
    pub filename: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub detail: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub timestamp_client: Option<String>,
    pub timestamp_server_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// Only return errors with id > since.
    pub since: Option<u64>,
    pub view: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/v1/view-errors
pub async fn post_error(
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<PostErrorReq>,
) -> axum::response::Response {
    let id = record(ErrorReport {
        view: req.view,
        url: req.url,
        kind: req.kind,
        message: req.message,
        stack: req.stack,
        filename: req.filename,
        line: req.line,
        column: req.column,
        detail: req.detail,
        user_agent: req.user_agent,
        timestamp_client: req.timestamp_client,
    });
    (
        StatusCode::OK,
        Json(serde_json::json!({ "id": id, "accepted": true })),
    )
        .into_response()
}

/// GET /api/v1/view-errors
pub async fn list_errors(
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> axum::response::Response {
    let limit = q.limit.unwrap_or(50).min(RING_CAP);
    let (filtered, total) = query(q.since, q.view.as_deref(), limit);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "errors": filtered, "total": total })),
    )
        .into_response()
}

/// DELETE /api/v1/view-errors
pub async fn clear_errors(
    Extension(_auth): Extension<AuthContext>,
) -> axum::response::Response {
    let cleared = clear();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "cleared": cleared })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        buffer().lock().unwrap().clear();
    }

    #[test]
    fn ring_buffer_caps_at_200() {
        reset();
        for i in 0..250u32 {
            let mut buf = buffer().lock().unwrap();
            if buf.len() >= RING_CAP {
                buf.pop_front();
            }
            buf.push_back(StoredError {
                id: i as u64,
                view: None,
                url: None,
                kind: "t".into(),
                message: format!("m{i}"),
                stack: None,
                filename: None,
                line: None,
                column: None,
                detail: None,
                user_agent: None,
                timestamp_client: None,
                timestamp_server_ms: 0,
            });
        }
        let buf = buffer().lock().unwrap();
        assert_eq!(buf.len(), RING_CAP);
        // First retained should be id=50 (pushed 0..250, kept last 200).
        assert_eq!(buf.front().unwrap().id, 50);
    }
}
