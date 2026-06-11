//! HTTP endpoints for the PTY session registry. Three concerns split
//! per file:
//!
//! - [`crud`] — spawn / list / get / stop / resize + `installed_harnesses`.
//! - [`io`] — stdin write, SSE byte stream, SSE event stream.
//! - [`messages`] — `session_message` (user turn) + `session_reply`
//!   (agent reply) + annotation rendering.
//!
//! Routes (mounted in `router.rs`):
//! * `POST /api/v1/sessions/spawn` — agent or shell spawn
//! * `GET  /api/v1/pty-sessions` — list all (active + ended)
//! * `GET  /api/v1/sessions/{id}` — single snapshot
//! * `DELETE /api/v1/sessions/{id}` — graceful stop
//! * `POST /api/v1/sessions/{id}/input` — write stdin bytes
//! * `POST /api/v1/sessions/{id}/resize` — PTY resize
//! * `POST /api/v1/sessions/{id}/message` — chat turn
//! * `POST /api/v1/sessions/{id}/reply` — agent reply
//! * `GET  /api/v1/sessions/{id}/stream` — SSE replay + live tail (bytes)
//! * `GET  /api/v1/sessions/{id}/events` — SSE parsed HarnessEvents

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::ship_runtime::ApiState;

pub mod crud;
pub mod io;
pub mod messages;

pub use crud::{
    InstalledProvidersResponse, ResizeBody, SessionListResponse, SessionMetaWire, SpawnBody,
    SpawnResponse, get_session, installed_harnesses, list_pty_sessions, session_resize,
    spawn_session, stop_session,
};
pub use io::{InputBody, session_input, stream_session, stream_session_events};
pub use messages::{Annotation, AnnotationSource, MessageBody, session_message, session_reply};

/// Shared `{ok: false, error: msg}` body — returned by every handler in
/// this surface to keep error shape uniform.
pub(super) fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

/// Resolve a workspace's working directory for a spawned session.
///
/// MVP: workspace dir is the project root unless we find a git worktree
/// entry for the branch. Good enough to unblock the UX; workspace
/// resolve logic lives in `runtime::sessions` and can replace this when
/// we thread the context through.
pub(super) fn work_dir_for(state: &ApiState, workspace_id: &str) -> std::path::PathBuf {
    let _ = workspace_id;
    std::env::current_dir()
        .unwrap_or_else(|_| state.ship_dir().to_path_buf())
}
