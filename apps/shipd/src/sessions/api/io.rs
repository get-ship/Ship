//! PTY stdin write + SSE streaming handlers (raw bytes and parsed
//! ProviderEvents). The raw byte stream replays the log file from
//! disk before tailing the broadcast; the event stream is broadcast-only.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures_util::stream::{self};
use serde::Deserialize;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as TokioStreamExt;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::ApiState;

use super::err;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct InputBody {
    /// Raw bytes (UTF-8 text). Added verbatim to PTY stdin — include
    /// `\n` for enter-key behaviour.
    pub data: String,
}

#[utoipa::path(
    operation_id = "sessionInput",
    post,
    path = "/api/v1/sessions/{id}/input",
    params(("id" = String, Path, description = "Session id")),
    request_body = InputBody,
    responses(
        (status = 204, description = "Input accepted"),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn session_input(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<InputBody>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e.into_response();
    }
    // Only sessions that own a live transport (shell + persistent agent
    // sessions on ActorRegistry) accept raw stdin. Per-turn agent
    // sessions on `pty_sessions` have no live writer; clients use
    // `send_agent_message` for those.
    let Some(entry) = state.actor_sessions().get(&id) else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };
    if let Err(e) = entry.write_input(body.data.as_bytes()) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    (StatusCode::NO_CONTENT, ()).into_response()
}

#[utoipa::path(
    operation_id = "streamSessionEvents",
    get,
    path = "/api/v1/sessions/{id}/events",
    params(("id" = String, Path, description = "Session id")),
    responses(
        (status = 200, description = "SSE stream of parsed ProviderEvents", content_type = "text/event-stream"),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn stream_session_events(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ReadJobs) {
        return e.into_response();
    }
    let rx = if let Some(entry) = state.actor_sessions().get(&id) {
        entry.subscribe_events()
    } else if let Some(session) = state.pty_sessions().get(&id) {
        session.subscribe_events()
    } else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };
    let ev_stream = BroadcastStream::new(rx)
        .filter_map(|r| r.ok())
        .map(|ev| {
            let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".into());
            Ok::<_, Infallible>(Event::default().event("provider").data(data))
        });
    let sse = Sse::new(ev_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)));
    sse.into_response()
}

#[utoipa::path(
    operation_id = "streamSession",
    get,
    path = "/api/v1/sessions/{id}/stream",
    params(("id" = String, Path, description = "Session id")),
    responses(
        (status = 200, description = "SSE stream of PTY bytes + status events", content_type = "text/event-stream"),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn stream_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ManageJobs) {
        return e.into_response();
    }
    let (log_path, live) = if let Some(entry) = state.actor_sessions().get(&id) {
        (entry.log_path(), entry.subscribe())
    } else if let Some(session) = state.pty_sessions().get(&id) {
        (session.0.log_path.clone(), session.subscribe())
    } else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };

    // Replay log content first, then live tail. The log is written by
    // the reader thread; opening + reading once captures a snapshot.
    let replay = tokio::task::spawn_blocking(move || std::fs::read(&log_path).unwrap_or_default())
        .await
        .unwrap_or_default();
    let live_stream = BroadcastStream::new(live).filter_map(|r| r.ok());

    let replay_stream = stream::iter(std::iter::once(replay));
    let bytes_stream = replay_stream.chain(live_stream);

    // PTY output is raw bytes — escape sequences contain CR (\r) for
    // cursor returns, color codes, prompt redraws. SSE line-folds on
    // both \r and \n, and EventSource rejoins fields with \n only, so
    // any \r in the payload silently becomes \n on the client and the
    // terminal renders garbled output. Base64-encode each chunk so the
    // transport is byte-exact; the bridge decodes back to bytes and
    // writes them straight into xterm.
    let event_stream = bytes_stream.map(|bytes| {
        let encoded = BASE64.encode(&bytes);
        Ok::<_, Infallible>(Event::default().event("bytes").data(encoded))
    });

    let sse = Sse::new(event_stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)));
    sse.into_response()
}
