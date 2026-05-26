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

use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;

use super::super::lifecycle;
use super::{err, work_dir_for};

#[derive(Deserialize)]
pub struct MessageBody {
    pub text: String,
    /// Optional context attachments. Rendered into the prompt as
    /// XML-tagged blocks so any harness's TUI / stream-json sees them
    /// as part of the user turn — harness-agnostic, no per-mechanism
    /// branching. Empty / absent → no annotation prefix.
    #[serde(default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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
    } else if let Some(s) = state.pty_sessions().get(&id) {
        let m = s.meta();
        (m.workspace_id, m.agent_id)
    } else {
        // Dispatched sessions (POST /api/v1/agent_sessions/dispatch) land
        // in neither legacy registry — they live in the orchestrator's
        // ProcessRegistry and the actor projection. Read metadata from
        // the projection so studio.chat_turn carries the right context
        // even for sessions the legacy registries never saw.
        ship_substrate::projections::actor::get_actor(&id)
            .ok()
            .flatten()
            .map(|a| (a.workspace_id.unwrap_or_default(), a.agent_id))
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

    // Dispatched sessions (POST /api/v1/agent_sessions/dispatch) live in
    // the orchestrator's ProcessRegistry, NOT actor_sessions / pty_sessions.
    // Look them up via the dispatch handle — the same lookup the WS attach
    // endpoint uses — and route by the harness's push_mechanism.
    if let Some(dh) = state.dispatch_handle() {
        // Hold the lock just long enough to copy out the substrate
        // ActorHandle (Clone) — ExecutorHandle itself isn't Clone
        // (its inner is a Box<dyn Any>). actor_handle is None when
        // the session isn't PTY-backed (tmux / server-mode) OR when
        // no session is registered for this slug.
        let actor_handle = {
            let registry = dh.registry.lock().await;
            registry
                .get(&id)
                .and_then(ship_supervisor::pty_actor_handle)
        };
        // Distinguish "no session in registry" from "session present
        // but not PTY" by a second pass.
        let session_in_registry = { dh.registry.lock().await.get(&id).is_some() };

        if session_in_registry {
            // Harness comes from the actor projection — the dispatch
            // payload carries it through to the actor.created event.
            let harness = ship_substrate::projections::actor::get_actor(&id)
                .ok()
                .flatten()
                .and_then(|a| a.harness)
                .unwrap_or_default();
            use ship_platform::services::harness_adapter::{adapter_for, PushMechanism};
            let mechanism = adapter_for(&harness)
                .map(|a| a.caps().push_mechanism)
                .unwrap_or(PushMechanism::Respawn);

            match mechanism {
                PushMechanism::Channel => {
                    // Channel push delivers via the studio.chat_turn event
                    // emitted above. Nothing further needed here.
                    return (StatusCode::ACCEPTED, ()).into_response();
                }
                PushMechanism::Stdin => {
                    let Some(actor_handle) = actor_handle else {
                        return err(
                            StatusCode::CONFLICT,
                            format!(
                                "session '{id}' is not PTY-backed; stdin push unavailable (harness={harness})"
                            ),
                        );
                    };
                    let mut payload = text.clone();
                    payload.push('\n');
                    if let Err(e) = actor_handle.write_input(payload.as_bytes()) {
                        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("stdin write: {e}"));
                    }
                    return (StatusCode::ACCEPTED, ()).into_response();
                }
                PushMechanism::Respawn => {
                    // Respawn for dispatched sessions isn't wired yet —
                    // would require coordinating with the orchestrator
                    // to kill + relaunch with --resume. Surface a clear
                    // 501 instead of silently doing nothing.
                    return err(
                        StatusCode::NOT_IMPLEMENTED,
                        format!(
                            "respawn push not implemented for dispatched session '{id}' (harness={harness})"
                        ),
                    );
                }
            }
        }
    }

    // Fallback: legacy spawn path (sessions/spawn with kind=agent on a
    // non-persistent harness). 500s with "not found" if no registry
    // anywhere has the session — that's a true unknown id.
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
