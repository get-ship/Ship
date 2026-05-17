//! `session_message` (user turn) and `session_reply` (agent reply)
//! handlers, plus the annotation-rendering helpers that turn structured
//! attachment metadata into a `<context>` XML prefix every harness
//! sees as part of the prompt.
//!
//! `session_message` emits `studio.chat_turn` for channel-capable
//! agents and branches per-harness on `push_mechanism` (channel /
//! stdin / respawn). `session_reply` is the canonical write path for
//! agent → user replies (emits `studio.reply` + broadcasts to SSE
//! subscribers).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::Deserialize;
use ship_substrate::events::{types::event_types, EventEnvelope};

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;

use super::super::lifecycle;
use super::{err, work_dir_for};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MessageBody {
    pub text: String,
    /// Optional context attachments. Rendered into the prompt as
    /// XML-tagged blocks so any harness's TUI / stream-json sees them
    /// as part of the user turn — harness-agnostic, no per-mechanism
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
/// Harness-agnostic XML so claude / codex / opencode all see the
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
    // (channel / stdin / respawn) sees the same prompt shape — harness-
    // agnostic.
    let text = render_message_with_annotations(&raw_text, &body.annotations);

    // Emit `studio.chat_turn` for channel-capable long-lived agents.
    // Per-turn harnesses ignore this path; they still get driven by the
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
        if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
            // Event-sourced audit gap if dropped silently — log at warn so
            // a missing studio.chat_turn shows up in operator dashboards.
            tracing::warn!(error = %e, session_id = %id, "studio.chat_turn emit failed");
        }
    }

    // Persistent-session agents live in ActorRegistry. Delivery branches
    // on the harness's push_mechanism: channel (event-only, the emit
    // above is enough), stdin (write prompt + newline to the PTY master),
    // respawn (kill + spawn fresh — not yet implemented; falls through
    // to the legacy per-turn path).
    if let Some(entry) = state.actor_sessions().get(&id) {
        use ship_platform::services::harness_adapter::{adapter_for, PushMechanism};
        let harness = entry
            .meta_snapshot()
            .harness
            .unwrap_or_default();
        let mechanism = adapter_for(&harness)
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
                // --resume internally for harnesses that support it).
                let work_dir = work_dir_for(&state, "main");
                return match lifecycle::send_agent_message(
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
        m.status = super::super::types::SessionStatus::Thinking;
        m.last_activity = chrono::Utc::now();
        return (StatusCode::ACCEPTED, ()).into_response();
    }

    let work_dir = work_dir_for(&state, "main");
    match lifecycle::send_agent_message(
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
    let ev = ship_platform::services::harness_adapter::HarnessEvent::Message { text };
    if let Some(entry) = state.actor_sessions().get(&id) {
        if let Some(tx) = entry.event_tx.as_ref() {
            let _ = tx.send(ev);
        }
    } else if let Some(session) = state.pty_sessions().get(&id) {
        let _ = session.0.event_tx.send(ev);
    }

    (StatusCode::ACCEPTED, ()).into_response()
}
