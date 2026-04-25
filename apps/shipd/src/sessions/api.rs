//! HTTP endpoints for the PTY session registry.
//!
//! * `POST /api/v1/sessions/spawn` — agent or shell spawn
//! * `GET /api/v1/sessions` — list all (active + ended)
//! * `GET /api/v1/sessions/{id}` — single snapshot
//! * `DELETE /api/v1/sessions/{id}` — graceful stop
//! * `POST /api/v1/sessions/{id}/input` — write stdin bytes
//! * `GET /api/v1/sessions/{id}/stream` — SSE replay + live tail
//!
//! SSE is the default for subscribers because our existing asset
//! pipeline (EventSource, same-origin cookie) handles it cleanly.
//! Bidirectional streaming (for raw terminal passthrough) will move
//! to the existing WS endpoint once the registry swap-in lands.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use futures_util::stream::{self, Stream};
use runtime::events::{types::event_types, EventEnvelope};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as TokioStreamExt;

use crate::auth::AuthContext;
use crate::rest_api::ApiState;

use super::lifecycle;
use super::types::{SessionKind, SessionMeta, SpawnRequest};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SpawnBody {
    pub workspace_id: String,
    /// `"agent" | "shell"`. Pipelines register internally.
    pub kind: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SpawnResponse {
    pub session_id: String,
    pub attached: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionMetaWire>,
}

/// Wire-format session meta. Mirrors `SessionMeta` but the internal
/// type lives in `types::private` and we want to keep that module
/// from leaking schemas via utoipa.
#[derive(Serialize, utoipa::ToSchema)]
pub struct SessionMetaWire {
    pub id: String,
    pub kind: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub status: String,
    pub log_path: String,
    pub spawned_at: String,
    pub last_activity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

impl From<SessionMeta> for SessionMetaWire {
    fn from(m: SessionMeta) -> Self {
        Self {
            id: m.id,
            kind: match m.kind {
                SessionKind::Agent => "agent",
                SessionKind::Pipeline => "pipeline",
                SessionKind::Shell => "shell",
            }
            .to_string(),
            workspace_id: m.workspace_id,
            agent_id: m.agent_id,
            provider: m.provider,
            pid: m.pid,
            status: format!("{:?}", m.status).to_lowercase(),
            log_path: m.log_path,
            spawned_at: m.spawned_at.to_rfc3339(),
            last_activity: m.last_activity.to_rfc3339(),
            exit_code: m.exit_code,
        }
    }
}

/// Provider detection: which of the known providers have their CLI
/// binaries on PATH. Powers the command-bar's spawn card.
#[derive(Serialize, utoipa::ToSchema)]
pub struct InstalledProvidersResponse {
    pub providers: Vec<String>,
}

const KNOWN_PROVIDERS: &[(&str, &str)] = &[
    ("claude", "claude"),
    ("codex", "codex"),
    ("cursor", "cursor-agent"),
    ("gemini", "gemini"),
    ("opencode", "opencode"),
];

fn binary_exists(bin: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|p| {
        let candidate = p.join(bin);
        if !candidate.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(&candidate)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true
        }
    })
}

#[utoipa::path(
    operation_id = "installedProviders",
    get,
    path = "/api/v1/providers/installed",
    responses(
        (status = 200, description = "Providers detected on PATH", body = InstalledProvidersResponse),
    ),
    tag = "sessions"
)]
pub async fn installed_providers() -> axum::response::Response {
    let providers = KNOWN_PROVIDERS
        .iter()
        .filter(|(_, bin)| binary_exists(bin))
        .map(|(id, _)| id.to_string())
        .collect();
    (
        StatusCode::OK,
        Json(InstalledProvidersResponse { providers }),
    )
        .into_response()
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MessageBody {
    pub text: String,
}

#[utoipa::path(
    operation_id = "sessionMessage",
    post,
    path = "/api/v1/sessions/{id}/message",
    params(("id" = String, Path, description = "Agent session id")),
    request_body = MessageBody,
    responses(
        (status = 202, description = "Accepted; response streams via /events"),
        (status = 404, description = "Not found"),
        (status = 400, description = "Not an agent session"),
    ),
    tag = "sessions"
)]
pub async fn session_message(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<MessageBody>,
) -> axum::response::Response {
    let text = body.text.trim().to_string();
    if text.is_empty() {
        return err(StatusCode::BAD_REQUEST, "empty message");
    }

    // Emit `studio.chat_turn` for channel-capable long-lived agents.
    // Per-turn providers ignore this path; they still get driven by the
    // spawn + stdin prompt below. The event is scoped to `session_id` so
    // the MCP `EventRelay` can filter it to the exact Claude instance.
    let (workspace_id, agent_id) = state
        .pty_sessions()
        .get(&id)
        .map(|s| {
            let m = s.meta();
            (m.workspace_id, m.agent_id)
        })
        .unwrap_or_default();
    let payload = serde_json::json!({
        "session_id": id,
        "text": text,
        "agent_id": agent_id,
    });
    if let Ok(envelope) = EventEnvelope::new(
        event_types::STUDIO_CHAT_TURN,
        &id,
        &payload,
    ) {
        let envelope = envelope
            .with_context(Some(&workspace_id), Some(&id))
            .with_target(&id);
        let ctx = auth.to_daemon_emit_context();
        let _ = state.kernel().lock().await.emit(envelope, &ctx).await;
    }

    let work_dir = work_dir_for(&state, "main");
    match super::lifecycle::send_agent_message(
        state.pty_sessions(),
        &id,
        &text,
        work_dir,
    ) {
        Ok(_) => (StatusCode::ACCEPTED, ()).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[utoipa::path(
    operation_id = "sessionReply",
    post,
    path = "/api/v1/sessions/{id}/reply",
    params(("id" = String, Path, description = "Agent session id")),
    request_body = MessageBody,
    responses(
        (status = 202, description = "Reply broadcast to session subscribers"),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn session_reply(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(body): Json<MessageBody>,
) -> axum::response::Response {
    let text = body.text;
    if text.is_empty() {
        return err(StatusCode::BAD_REQUEST, "empty reply");
    }
    let Some(session) = state.pty_sessions().get(&id) else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };
    let ev = runtime::services::provider_adapter::ProviderEvent::Message { text };
    let _ = session.0.event_tx.send(ev);
    (StatusCode::ACCEPTED, ()).into_response()
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct InputBody {
    /// Raw bytes (UTF-8 text). Added verbatim to PTY stdin — include
    /// `\n` for enter-key behaviour.
    pub data: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

fn work_dir_for(state: &ApiState, workspace_id: &str) -> std::path::PathBuf {
    // MVP: workspace dir is the project root unless we find a git
    // worktree entry for the branch. Good enough to unblock the UX;
    // workspace resolve logic lives in runtime::workspace and can
    // replace this when we thread the context through.
    let _ = workspace_id;
    std::env::current_dir()
        .unwrap_or_else(|_| state.ship_dir().to_path_buf())
}

#[utoipa::path(
    operation_id = "spawnSession",
    post,
    path = "/api/v1/sessions/spawn",
    request_body = SpawnBody,
    responses(
        (status = 200, description = "Session spawned or attached", body = SpawnResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Spawn failure"),
    ),
    tag = "sessions"
)]
pub async fn spawn_session(
    State(state): State<ApiState>,
    Json(req): Json<SpawnBody>,
) -> axum::response::Response {
    let registry = state.pty_sessions();
    let log_dir = state.ship_dir().join("logs").join("sessions");
    let work_dir = work_dir_for(&state, &req.workspace_id);

    match req.kind.as_str() {
        "agent" => {
            let agent_id = match req.agent_id.as_deref() {
                Some(a) if !a.is_empty() => a,
                _ => return err(StatusCode::BAD_REQUEST, "agent_id required for agent kind"),
            };
            let provider = match req.provider.as_deref() {
                Some(p) if !p.is_empty() => p,
                _ => return err(StatusCode::BAD_REQUEST, "provider required for agent kind"),
            };
            match lifecycle::spawn_agent(
                registry,
                &req.workspace_id,
                agent_id,
                provider,
                work_dir,
                log_dir,
            ) {
                Ok(outcome) => {
                    if !outcome.attached {
                        lifecycle::emit_actor_created_for(
                            &outcome.session,
                            &lifecycle::ActorSpawnExtras {
                                transport: Some("pty".to_string()),
                                persistence: Some("long-lived".to_string()),
                                ..Default::default()
                            },
                        );
                    }
                    let resp = SpawnResponse {
                        session_id: outcome.session.meta().id,
                        attached: outcome.attached,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        }
        "shell" => {
            match lifecycle::spawn_shell(registry, &req.workspace_id, work_dir, log_dir) {
                Ok(session) => {
                    lifecycle::emit_actor_created_for(
                        &session,
                        &lifecycle::ActorSpawnExtras {
                            transport: Some("pty".to_string()),
                            persistence: Some("long-lived".to_string()),
                            ..Default::default()
                        },
                    );
                    let resp = SpawnResponse {
                        session_id: session.meta().id,
                        attached: false,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        }
        other => err(StatusCode::BAD_REQUEST, format!("unknown kind '{other}'")),
    }
}

#[utoipa::path(
    operation_id = "listSessions",
    get,
    path = "/api/v1/sessions",
    responses(
        (status = 200, description = "All known sessions", body = SessionListResponse),
    ),
    tag = "sessions"
)]
pub async fn list_sessions(State(state): State<ApiState>) -> axum::response::Response {
    // Cheap housekeeping — flip thinking → idle as needed before
    // handing the snapshot to the client.
    lifecycle::sweep_idle(state.pty_sessions());
    let sessions = state
        .pty_sessions()
        .list()
        .into_iter()
        .map(SessionMetaWire::from)
        .collect();
    (StatusCode::OK, Json(SessionListResponse { sessions })).into_response()
}

#[utoipa::path(
    operation_id = "getSession",
    get,
    path = "/api/v1/sessions/{id}",
    params(("id" = String, Path, description = "Session id")),
    responses(
        (status = 200, description = "Session snapshot", body = SessionMetaWire),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn get_session(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    match state.pty_sessions().get(&id) {
        Some(session) => {
            let wire: SessionMetaWire = session.meta().into();
            (StatusCode::OK, Json(wire)).into_response()
        }
        None => err(StatusCode::NOT_FOUND, format!("session '{id}' not found")),
    }
}

#[utoipa::path(
    operation_id = "stopSession",
    delete,
    path = "/api/v1/sessions/{id}",
    params(("id" = String, Path, description = "Session id")),
    responses(
        (status = 204, description = "Stopped"),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn stop_session(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    if lifecycle::stop(state.pty_sessions(), &id) {
        (StatusCode::NO_CONTENT, ()).into_response()
    } else {
        err(StatusCode::NOT_FOUND, format!("session '{id}' not found"))
    }
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
    Path(id): Path<String>,
    Json(body): Json<InputBody>,
) -> axum::response::Response {
    let Some(session) = state.pty_sessions().get(&id) else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };
    if let Err(e) = lifecycle::write_input(&session, body.data.as_bytes()) {
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
    Path(id): Path<String>,
) -> axum::response::Response {
    let Some(session) = state.pty_sessions().get(&id) else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };
    let rx = session.subscribe_events();
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
    Path(id): Path<String>,
) -> axum::response::Response {
    let Some(session) = state.pty_sessions().get(&id) else {
        return err(StatusCode::NOT_FOUND, format!("session '{id}' not found"));
    };

    // Replay log content first, then live tail. The log is written by
    // the reader thread; opening + reading once captures a snapshot.
    let log_path = session.0.log_path.clone();
    let replay = tokio::task::spawn_blocking(move || std::fs::read(&log_path).unwrap_or_default())
        .await
        .unwrap_or_default();

    let live = session.subscribe();
    let live_stream = BroadcastStream::new(live).filter_map(|r| r.ok());

    let replay_stream = stream::iter(std::iter::once(replay));
    let bytes_stream = replay_stream.chain(live_stream);

    let event_stream = bytes_stream.map(|bytes| {
        let text = String::from_utf8_lossy(&bytes).to_string();
        Ok::<_, Infallible>(Event::default().event("bytes").data(text))
    });

    let sse = Sse::new(event_stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)));
    sse.into_response()
}
