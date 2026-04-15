//! Chat view — streaming conversation + composer.
//!
//! Endpoints:
//! - `GET /api/runtime/view/chat` — full-page shell fragment, scoped to an
//!   optional `workspace_id` query param.
//! - `GET /api/runtime/view/chat/sse` — Datastar SSE stream that patches
//!   `#chat-messages` on every new event for the scoped workspace.
//! - `POST /api/runtime/view/chat/send` — emits a `studio.message` event so
//!   the composer is a single daemon round-trip with no web-side routing.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use runtime::events::ActorConfig;
use runtime::gen_ulid;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::rest_api::{ApiState, MeshResponse};
use crate::views::chat_state::{ChatMessage, DEFAULT_LIMIT, list_messages};
use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ChatParams {
    #[serde(default)]
    pub workspace_id: Option<String>,
}

#[derive(Serialize)]
pub struct ChatCtx {
    pub workspace_id: String,
    pub messages: Vec<ChatMessage>,
    pub total: usize,
}

#[derive(Deserialize)]
pub struct ChatSendRequest {
    pub text: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

fn build_ctx(params: &ChatParams, ship_dir: &std::path::Path) -> ChatCtx {
    let ws = params.workspace_id.as_deref().filter(|s| !s.is_empty());
    let messages = list_messages(ship_dir, ws, DEFAULT_LIMIT);
    ChatCtx {
        workspace_id: ws.unwrap_or_default().to_string(),
        total: messages.len(),
        messages,
    }
}

pub async fn chat_shell(
    State(state): State<ApiState>,
    Query(params): Query<ChatParams>,
) -> Result<Html<String>, StatusCode> {
    // Try .ship/views/chat/index.html.j2 first (user-authored override).
    if let Some((view_dir, _)) = crate::views::resolve_view_dir("chat") {
        let index_path = view_dir.join("index.html.j2");
        if let Ok(source) = std::fs::read_to_string(&index_path) {
            let ctx = build_ctx(&params, state.ship_dir());
            return templates::render_view_source("chat", &source, &ctx)
                .map(Html)
                .map_err(|e| {
                    tracing::error!(error = %e, "chat view render failed");
                    StatusCode::INTERNAL_SERVER_ERROR
                });
        }
    }
    // Fallback: daemon-embedded template.
    let ctx = build_ctx(&params, state.ship_dir());
    templates::render("chat/shell.html.j2", &ctx).map(Html).map_err(|e| {
        tracing::error!(error = %e, "chat shell render failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn chat_sse(
    State(state): State<ApiState>,
    Query(params): Query<ChatParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();

    let actor_id = format!("chat-view-sse-{}", gen_ulid());
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec![
            "studio.".into(),
            "session.".into(),
            "job.".into(),
        ],
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => Some(mb),
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "spawn_actor failed for chat SSE");
            None
        }
    };
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));
        if let Some(html) = render_content(&params, &ship_dir) {
            if vs.patch("#chat-messages", html).await.is_err() {
                return;
            }
        }
        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let Some(html) = render_content(&params, &ship_dir) else { continue };
            if vs.patch("#chat-messages", html).await.is_err() {
                break;
            }
        }
    });
    sse
}

fn render_content(params: &ChatParams, ship_dir: &std::path::Path) -> Option<String> {
    let ctx = build_ctx(params, ship_dir);
    match templates::render("chat/partials/messages.html.j2", &ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(error = %e, "chat content render failed");
            None
        }
    }
}

pub async fn chat_send(
    State(state): State<ApiState>,
    Json(req): Json<ChatSendRequest>,
) -> (StatusCode, Json<MeshResponse>) {
    let text = req.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({ "error": "empty message" }),
            }),
        );
    }
    let entity = req.workspace_id.clone().unwrap_or_else(|| "studio".into());
    let payload = serde_json::json!({
        "summary": text,
        "workspace_id": req.workspace_id,
    });
    let mut envelope = match runtime::events::EventEnvelope::new(
        "studio.message",
        &entity,
        &payload,
    ) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!({ "error": e.to_string() }),
                }),
            );
        }
    };
    envelope = envelope.with_actor_id("studio");
    if let Some(ws) = req.workspace_id.as_deref() {
        envelope = envelope.with_context(Some(ws), None);
    }
    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Cli,
        skill_id: None,
        workspace_id: req.workspace_id.clone(),
        session_id: None,
    };
    if let Err(e) = state.kernel().lock().await.route(envelope, &ctx).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!({ "error": e.to_string() }),
            }),
        );
    }
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "sent": true }),
        }),
    )
}

#[cfg(test)]
#[path = "views_chat_tests.rs"]
mod tests;
