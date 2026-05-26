//! `POST /api/v1/chat/send` — emit a `studio.message` kernel event.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;

use crate::ship_runtime::ApiState;
use crate::wire::MeshResponse;

#[derive(Deserialize)]
pub struct ChatSendRequest {
    pub text: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

pub async fn send_message(
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
    let mut envelope = match ship_substrate::events::EventEnvelope::new(
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
    let ctx = ship_substrate::events::EmitContext {
        caller_kind: ship_substrate::events::CallerKind::Cli,
        skill_id: None,
        project_id: req.workspace_id.clone(),
        session_id: None,
    };
    if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
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
