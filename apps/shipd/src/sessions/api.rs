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
use ship_substrate::events::{types::event_types, EventEnvelope};
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
    /// Optional context attachments. Rendered into the prompt as
    /// XML-tagged blocks so any provider's TUI / stream-json sees them
    /// as part of the user turn — provider-agnostic, no per-mechanism
    /// branching. Empty / absent → no annotation prefix.
    #[serde(default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct Annotation {
    /// `"text" | "file" | "diff" | "comment"` — drives the wrapping
    /// XML tag and any per-type formatting. Unknown values pass
    /// through as `<context type="…">…</context>`.
    #[serde(rename = "type")]
    pub kind: String,
    pub content: String,
    #[serde(default)]
    pub source: Option<AnnotationSource>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AnnotationSource {
    /// e.g. "file", "selection", "diff-hunk", "comment"
    pub kind: String,
    /// e.g. "src/foo.rs:42-58", "review/comment-12"
    pub r#ref: String,
}

/// Render `annotations` into a single block prepended to `text`.
/// Provider-agnostic XML so claude / codex / opencode all see the
/// same shape; the agent's prompt template can pick the tags it
/// cares about. Empty `annotations` returns `text` unchanged.
fn render_message_with_annotations(text: &str, annotations: &[Annotation]) -> String {
    if annotations.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + annotations.len() * 64);
    out.push_str("<context>\n");
    for ann in annotations {
        let src_attr = ann
            .source
            .as_ref()
            .map(|s| format!(" source-kind=\"{}\" source-ref=\"{}\"", xml_escape(&s.kind), xml_escape(&s.r#ref)))
            .unwrap_or_default();
        out.push_str(&format!(
            "  <attachment type=\"{}\"{}>\n{}\n  </attachment>\n",
            xml_escape(&ann.kind),
            src_attr,
            ann.content,
        ));
    }
    out.push_str("</context>\n\n");
    out.push_str(text);
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
    let raw_text = body.text.trim().to_string();
    if raw_text.is_empty() {
        return err(StatusCode::BAD_REQUEST, "empty message");
    }
    // Render annotations as a <context> prefix so every push_mechanism
    // (channel / stdin / respawn) sees the same prompt shape — provider-
    // agnostic.
    let text = render_message_with_annotations(&raw_text, &body.annotations);

    // Emit `studio.chat_turn` for channel-capable long-lived agents.
    // Per-turn providers ignore this path; they still get driven by the
    // spawn + stdin prompt below. The event is scoped to `session_id` so
    // the MCP `EventRelay` can filter it to the exact Claude instance.
    let (workspace_id, agent_id) = if let Some(entry) = state.actor_sessions().get(&id) {
        let m = entry.meta_snapshot();
        (m.workspace_id, m.agent_id)
    } else {
        state
            .pty_sessions()
            .get(&id)
            .map(|s| {
                let m = s.meta();
                (m.workspace_id, m.agent_id)
            })
            .unwrap_or_default()
    };
    let payload = serde_json::json!({
        "session_id": id,
        "text": text,
        "raw_text": raw_text,
        "agent_id": agent_id,
        "annotations": body.annotations.iter().map(|a| serde_json::json!({
            "type": a.kind,
            "content": a.content,
            "source": a.source.as_ref().map(|s| serde_json::json!({
                "kind": s.kind,
                "ref": s.r#ref,
            })),
        })).collect::<Vec<_>>(),
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

    // Persistent-session agents live in ActorRegistry. Delivery branches
    // on the provider's push_mechanism: channel (event-only, the emit
    // above is enough), stdin (write prompt + newline to the PTY master),
    // respawn (kill + spawn fresh — not yet implemented; falls through
    // to the legacy per-turn path).
    if let Some(entry) = state.actor_sessions().get(&id) {
        use ship_platform::services::provider_adapter::{adapter_for, PushMechanism};
        let provider = entry
            .meta_snapshot()
            .provider
            .unwrap_or_default();
        let mechanism = adapter_for(&provider)
            .map(|a| a.caps().push_mechanism)
            .unwrap_or(PushMechanism::Respawn);

        match mechanism {
            PushMechanism::Channel => {
                // Channel push delivers via the studio.chat_turn event
                // emitted above. Just flip status.
            }
            PushMechanism::Stdin => {
                // Write the prompt directly into the PTY master. The
                // long-lived TUI sees it as if the user typed at the
                // prompt + hit enter.
                let mut payload = text.clone();
                payload.push('\n');
                if let Err(e) = entry.write_input(payload.as_bytes()) {
                    return err(StatusCode::INTERNAL_SERVER_ERROR, format!("stdin write: {e}"));
                }
            }
            PushMechanism::Respawn => {
                // Not yet implemented at this layer — fall through to
                // legacy per-turn spawn (which respawns headless with
                // --resume internally for providers that support it).
                let work_dir = work_dir_for(&state, "main");
                return match super::lifecycle::send_agent_message(
                    state.pty_sessions(),
                    &id,
                    &text,
                    work_dir,
                ) {
                    Ok(_) => (StatusCode::ACCEPTED, ()).into_response(),
                    Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                };
            }
        }

        let mut m = entry.meta.lock().unwrap();
        m.status = super::types::SessionStatus::Thinking;
        m.last_activity = chrono::Utc::now();
        return (StatusCode::ACCEPTED, ()).into_response();
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
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<MessageBody>,
) -> axum::response::Response {
    let text = body.text;
    if text.is_empty() {
        return err(StatusCode::BAD_REQUEST, "empty reply");
    }

    // Emit `studio.reply` to the event log keyed on the actor id. This
    // is the canonical reply path: every actor in the projection gets
    // their reply written to the log regardless of whether a legacy
    // PtySession also exists. Dashboards / views tail this event.
    let payload = serde_json::json!({ "actor_id": id, "text": &text });
    if let Ok(envelope) = EventEnvelope::new(event_types::STUDIO_REPLY, &id, &payload) {
        let envelope = envelope.with_target(&id);
        let ctx = auth.to_daemon_emit_context();
        if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
            tracing::warn!(actor = %id, "studio.reply emit failed: {e}");
        }
    }

    // In-memory broadcast for SSE consumers tailing the session's
    // parsed-event stream. Check both registries so reply works
    // regardless of which substrate owns the session.
    let ev = ship_platform::services::provider_adapter::ProviderEvent::Message { text };
    if let Some(entry) = state.actor_sessions().get(&id) {
        if let Some(tx) = entry.event_tx.as_ref() {
            let _ = tx.send(ev);
        }
    } else if let Some(session) = state.pty_sessions().get(&id) {
        let _ = session.0.event_tx.send(ev);
    }

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
    // workspace resolve logic lives in runtime::sessions and can
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

            // Persistent-session providers (all five CLI agents today)
            // go through the supervisor primitive + ActorRegistry. The
            // push_mechanism (channel / stdin / respawn) is decided at
            // send-time; the spawn shape is identical regardless.
            if let Ok(adapter) = ship_platform::services::provider_adapter::adapter_for(provider) {
                if adapter.caps().persistent_session {
                    let append = lifecycle::load_agent_rules(&work_dir, agent_id);
                    return match super::agent_actor::spawn_or_attach(
                        state.actor_sessions(),
                        &req.workspace_id,
                        agent_id,
                        provider,
                        work_dir,
                        log_dir,
                        append,
                    ) {
                        Ok(outcome) => {
                            let resp = SpawnResponse {
                                session_id: outcome.entry.handle.actor_id.clone(),
                                attached: outcome.attached,
                            };
                            (StatusCode::OK, Json(resp)).into_response()
                        }
                        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                    };
                }
            }

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
            // Shell sessions go through the supervisor primitive +
            // ActorRegistry. Kernel registration, PTY hold, log file,
            // and `actor.created` emission all happen inside
            // `ship_substrate::supervisor::spawn_actor` — no second registry
            // write, no second event emit.
            match super::shell_actor::spawn_shell(
                state.actor_sessions(),
                &req.workspace_id,
                work_dir,
                log_dir,
            ) {
                Ok(entry) => {
                    let resp = SpawnResponse {
                        session_id: entry.handle.actor_id.clone(),
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
    let mut sessions: Vec<SessionMetaWire> = state
        .pty_sessions()
        .list()
        .into_iter()
        .map(SessionMetaWire::from)
        .collect();
    sessions.extend(
        state
            .actor_sessions()
            .list()
            .into_iter()
            .map(SessionMetaWire::from),
    );
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
    if let Some(entry) = state.actor_sessions().get(&id) {
        let wire: SessionMetaWire = entry.meta_snapshot().into();
        return (StatusCode::OK, Json(wire)).into_response();
    }
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
    if super::shell_actor::stop(state.actor_sessions(), &id)
        || lifecycle::stop(state.pty_sessions(), &id)
    {
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
    if let Some(entry) = state.actor_sessions().get(&id) {
        if let Err(e) = entry.write_input(body.data.as_bytes()) {
            return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
        return (StatusCode::NO_CONTENT, ()).into_response();
    }
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ResizeBody {
    pub cols: u16,
    pub rows: u16,
}

#[utoipa::path(
    operation_id = "sessionResize",
    post,
    path = "/api/v1/sessions/{id}/resize",
    params(("id" = String, Path, description = "Session id")),
    request_body = ResizeBody,
    responses(
        (status = 204, description = "PTY resized"),
        (status = 404, description = "Not found"),
    ),
    tag = "sessions"
)]
pub async fn session_resize(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(body): Json<ResizeBody>,
) -> axum::response::Response {
    if let Some(entry) = state.actor_sessions().get(&id) {
        if let Err(e) = entry.handle.resize(body.cols, body.rows) {
            return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
        return (StatusCode::NO_CONTENT, ()).into_response();
    }
    // Legacy PtySession path: poke portable-pty master directly via
    // its internal handle. The master sits behind a Mutex inside
    // Inner; not currently exposed. For now legacy sessions just
    // 404 on resize — they can opt into the actor path to gain it.
    if state.pty_sessions().get(&id).is_some() {
        return err(
            StatusCode::NOT_IMPLEMENTED,
            "legacy session: resize not wired (migrate to actor_sessions)",
        );
    }
    err(StatusCode::NOT_FOUND, format!("session '{id}' not found"))
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

    let event_stream = bytes_stream.map(|bytes| {
        let text = String::from_utf8_lossy(&bytes).to_string();
        Ok::<_, Infallible>(Event::default().event("bytes").data(text))
    });

    let sse = Sse::new(event_stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)));
    sse.into_response()
}
