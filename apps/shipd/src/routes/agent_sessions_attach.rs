//! WebSocket attach for PTY-backed agent sessions.
//!
//! Route: `GET /api/v1/agent_sessions/{slug}/attach`. Upgrades to WS and
//! bridges the substrate `ActorHandle`'s output broadcast to the client,
//! relaying client→server frames back as `write_input`. The underlying
//! process outlives any single attach — closing the socket only drops the
//! broadcast subscriber. Multiple concurrent attaches are allowed (one
//! receiver per attach; input is last-writer-wins, same as `tmux attach`).
//!
//! Only PTY-backed sessions are attachable. Tmux-window and server-mode
//! sessions return 409 — they don't carry a substrate handle for us to
//! subscribe to.

use axum::{
    Json,
    extract::{
        Path, Request, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequestParts,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tokio::sync::broadcast::error::RecvError;

use crate::auth::{AuthContext, Scope};
use crate::ship_runtime::ApiState;
use ship_substrate::supervisor::ActorHandle;

/// GET `/api/v1/agent_sessions/{slug}/attach`
///
/// Upgrades to WebSocket and streams the PTY-backed agent session's bytes
/// in both directions. The handler resolves the running session via the
/// dispatch process registry and produces:
///
/// - 401/403 if the request lacks `WriteWorkspace` scope
/// - 404 if no session is registered for `slug`
/// - 409 if the session is registered but not PTY-backed
/// - 503 if the dispatch subsystem has not initialized
/// - 101 on successful upgrade
#[utoipa::path(
    operation_id = "attachAgentSession",
    get,
    path = "/api/v1/agent_sessions/{slug}/attach",
    params(("slug" = String, Path, description = "Agent session slug")),
    responses(
        (status = 101, description = "WebSocket upgrade; bidirectional byte stream"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::wire::ErrorResponse),
        (status = 404, description = "No agent session for slug", body = crate::wire::ErrorResponse),
        (status = 409, description = "Session is not PTY-backed", body = crate::wire::ErrorResponse),
        (status = 503, description = "Dispatch subsystem unavailable", body = crate::wire::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn attach_agent_session(
    Path(slug): Path<String>,
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    request: Request,
) -> Response {
    // Attaching means reading and writing the agent's TTY — same blast
    // radius as a shell. Gate behind WriteWorkspace so a pure read scope
    // can't smuggle keystrokes in.
    if !auth.has_scope(Scope::WriteWorkspace) && !auth.has_scope(Scope::Full) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "insufficient scope: WriteWorkspace required"})),
        )
            .into_response();
    }

    let Some(dh) = state.dispatch_handle() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "dispatch subsystem not initialized"})),
        )
            .into_response();
    };

    let actor_handle = {
        let registry = dh.registry.lock().await;
        let Some(exec_handle) = registry.get(&slug) else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("no agent session '{slug}'")})),
            )
                .into_response();
        };
        match ship_supervisor::pty_actor_handle(exec_handle) {
            Some(h) => h,
            None => {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": format!(
                            "agent session '{slug}' is not PTY-backed; attach is unavailable"
                        )
                    })),
                )
                    .into_response();
            }
        }
    };

    // Defer the WS upgrade extraction until after validation so error
    // responses come back as plain JSON instead of an upgrade.
    let (mut parts, body) = request.into_parts();
    let ws = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
        Ok(ws) => ws,
        Err(e) => return e.into_response(),
    };
    drop(body);

    ws.on_upgrade(move |socket| async move {
        handle_attach_socket(socket, actor_handle).await;
    })
    .into_response()
}

/// Pump bytes between the WebSocket and the `ActorHandle`. Exits when
/// either side closes. Does NOT stop the underlying process — drop is the
/// only side-effect.
async fn handle_attach_socket(mut socket: WebSocket, handle: ActorHandle) {
    let mut rx = handle.subscribe_output();
    loop {
        tokio::select! {
            output = rx.recv() => {
                match output {
                    Ok(bytes) => {
                        if socket.send(Message::Binary(bytes.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Binary(data))) => {
                        if handle.write_input(&data).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        if handle.write_input(text.as_bytes()).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) | None => break,
                }
            }
        }
    }
}
