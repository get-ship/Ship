//! Optional Yoke session Connect client — **fleet sidecar**, not the shipd default.
//!
//! Product native sessions embed Yoke in-process ([`super::embed`]). Use this
//! path when `YOKE_RPC=1` (or an explicit socket env) to drive a fleet/bridge
//! process instead. Socket: `BRIDGE_YOKE_SOCKET` → `SHIP_YOKE_SOCKET` →
//! default (`$XDG_RUNTIME_DIR/bridge-yoke.sock` or `/tmp/bridge-yoke.sock`).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent_def::dispatch_intent::DispatchRequestedPayload;
use connectrpc::client::{ClientConfig, Http2Connection};
use yoke::{
    AuthContext as YokeAuth, SessionCancelled, SessionCompleted, SessionEventSink, SessionFailed,
    SessionStarted,
    gateway::{Gateway, GatewayCallKind, GatewayError},
};
use ship_ops_rpc::yoke::v1::{
    OpResultRequest, StartSessionRequest, YokeSessionServiceClient,
    __buffa::view::oneof::session_event as view_event,
};
use super::embed::ShipdKernelSink;
use super::gateway::ShipdGateway;
use crate::ship_runtime::SharedKernel;

// ── Socket resolution ────────────────────────────────────────────────────────

/// Explicit socket from env (`BRIDGE_YOKE_SOCKET`, then `SHIP_YOKE_SOCKET`).
/// Empty values are treated as unset.
pub fn socket_path_from_env() -> Option<String> {
    if let Ok(p) = std::env::var("BRIDGE_YOKE_SOCKET") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    std::env::var_os("SHIP_YOKE_SOCKET")
        .and_then(|s| s.into_string().ok())
        .filter(|p| !p.is_empty())
}

/// Default path matching `bridge serve` / Bridge `default_socket_path`.
pub fn default_socket_path() -> PathBuf {
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime.is_empty() {
            return PathBuf::from(runtime).join("bridge-yoke.sock");
        }
    }
    PathBuf::from("/tmp/bridge-yoke.sock")
}

/// Product resolution: env override, else the Bridge default path.
pub fn resolve_socket_path() -> String {
    socket_path_from_env().unwrap_or_else(|| default_socket_path().display().to_string())
}

// ── Session outcome ──────────────────────────────────────────────────────────

/// Terminal outcome of a harness session over Connect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionOutcome {
    Completed { session_id: String },
    Failed { session_id: String, message: String },
    Cancelled { session_id: String, reason: Option<String> },
}

// ── Dispatch entry ───────────────────────────────────────────────────────────

/// Run one harness session over fleet Connect (optional; not shipd default).
///
/// Missing socket / transport → `session.failed` (no auto-fallback to embed).
pub async fn dispatch_yoke_session_rpc(
    payload: DispatchRequestedPayload,
    gateway: Arc<ShipdGateway>,
    kernel: SharedKernel,
) {
    let socket_path = resolve_socket_path();
    tracing::info!(
        agent_id = %payload.agent_id,
        slug = %payload.slug,
        session_id = %payload.session_id,
        socket = %socket_path,
        "yoke-connect: starting session",
    );

    let sink = SessionEventSink::new(Arc::new(ShipdKernelSink::new(kernel)));

    if !Path::new(&socket_path).exists() {
        let message = format!(
            "bridge yoke socket not found at {socket_path}; \
             start Bridge (`bridge serve`) or set BRIDGE_YOKE_SOCKET"
        );
        tracing::error!(
            session_id = %payload.session_id,
            %message,
            "yoke-connect: no Bridge harness server",
        );
        let _ = sink.emit_session_failed(
            &payload.session_id,
            &SessionFailed {
                session_id: payload.session_id.clone(),
                reason: message,
            },
        );
        return;
    }

    match run_session(&socket_path, &payload, gateway, &sink).await {
        Ok(SessionOutcome::Completed { session_id }) => {
            tracing::info!(%session_id, "yoke-connect: session completed");
        }
        Ok(SessionOutcome::Failed {
            session_id,
            message,
        }) => {
            tracing::error!(%session_id, %message, "yoke-connect: session failed");
        }
        Ok(SessionOutcome::Cancelled {
            session_id,
            reason,
        }) => {
            tracing::warn!(%session_id, ?reason, "yoke-connect: session cancelled");
        }
        Err(e) => {
            tracing::error!(
                session_id = %payload.session_id,
                error = %e,
                "yoke-connect: transport or protocol error",
            );
            let _ = sink.emit_session_failed(
                &payload.session_id,
                &SessionFailed {
                    session_id: payload.session_id.clone(),
                    reason: e.to_string(),
                },
            );
        }
    }
}

/// Connect, start session, drive event loop until terminal.
pub async fn run_session(
    socket_path: &str,
    payload: &DispatchRequestedPayload,
    gateway: Arc<ShipdGateway>,
    sink: &SessionEventSink,
) -> anyhow::Result<SessionOutcome> {
    if !Path::new(socket_path).exists() {
        anyhow::bail!("yoke socket not found: {socket_path}");
    }

    let authority: http::Uri = "http://bridge.local"
        .parse()
        .map_err(|e| anyhow::anyhow!("authority: {e}"))?;
    let conn = Http2Connection::lazy_unix(socket_path, authority.clone()).shared(32);
    let config = ClientConfig::new(authority);
    let client = YokeSessionServiceClient::new(conn.clone(), config.clone());
    // Second client for concurrent OpResult unaries while Start streams.
    let op_client = YokeSessionServiceClient::new(conn, config);

    let prompt = payload
        .prompt
        .as_deref()
        .or(payload.spec.as_deref())
        .unwrap_or("(no input provided)")
        .to_string();

    let workspace = payload.repo_path.clone().unwrap_or_default();

    let mut stream = client
        .start(StartSessionRequest {
            session_id: payload.session_id.clone(),
            agent_id: payload.agent_id.clone(),
            prompt,
            model: payload.model.clone().unwrap_or_default(),
            workspace,
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow::anyhow!("YokeSessionService.Start: {e}"))?;

    let auth = YokeAuth::in_process(&payload.agent_id, vec![]);

    loop {
        let Some(ev) = stream
            .message()
            .await
            .map_err(|e| anyhow::anyhow!("Start stream: {e}"))?
        else {
            anyhow::bail!("yoke Start stream closed before terminal event");
        };

        let session_id = ev.session_id.to_string();
        match ev.event.as_ref() {
            Some(view_event::Event::Started(s)) => {
                tracing::info!(
                    %session_id,
                    agent_id = %s.agent_id,
                    "yoke-connect: session.started",
                );
                let _ = sink.emit_session_started(
                    &session_id,
                    &SessionStarted {
                        session_id: session_id.clone(),
                        agent_id: s.agent_id.to_string(),
                        harness: "yoke".into(),
                        model: payload.model.clone(),
                        slug: Some(payload.slug.clone()),
                        input: payload.prompt.clone().or(payload.spec.clone()),
                        ..Default::default()
                    },
                );
            }
            Some(view_event::Event::Output(o)) => {
                tracing::info!(
                    %session_id,
                    text_len = o.text.len(),
                    "yoke-connect: session.output"
                );
                tracing::debug!(%session_id, text = %o.text, "yoke-connect: output text");
            }
            Some(view_event::Event::Completed(_)) => {
                let _ = sink.emit_session_completed(
                    &session_id,
                    &SessionCompleted {
                        session_id: session_id.clone(),
                    },
                );
                return Ok(SessionOutcome::Completed { session_id });
            }
            Some(view_event::Event::Failed(f)) => {
                let message = f.message.to_string();
                let _ = sink.emit_session_failed(
                    &session_id,
                    &SessionFailed {
                        session_id: session_id.clone(),
                        reason: message.clone(),
                    },
                );
                return Ok(SessionOutcome::Failed {
                    session_id,
                    message,
                });
            }
            Some(view_event::Event::Cancelled(c)) => {
                let reason = if c.reason.is_empty() {
                    None
                } else {
                    Some(c.reason.to_string())
                };
                let _ = sink.emit_session_cancelled(
                    &session_id,
                    &SessionCancelled {
                        session_id: session_id.clone(),
                        reason: reason.clone(),
                    },
                );
                return Ok(SessionOutcome::Cancelled {
                    session_id,
                    reason,
                });
            }
            Some(view_event::Event::OpCall(call)) => {
                let id = call.id.to_string();
                let op = call.op.to_string();
                let args_json = call.args_json;
                tracing::info!(
                    %id,
                    %session_id,
                    %op,
                    "yoke-connect: op_call",
                );
                let result_req = answer_op_call(
                    gateway.as_ref(),
                    &auth,
                    &id,
                    &session_id,
                    &op,
                    args_json,
                )
                .await;
                if let Err(e) = op_client.op_result(result_req).await {
                    tracing::warn!(
                        %id,
                        error = %e,
                        "yoke-connect: OpResult failed",
                    );
                }
            }
            None => {
                tracing::warn!(%session_id, "yoke-connect: empty event oneof");
            }
        }
    }
}

async fn answer_op_call(
    gateway: &ShipdGateway,
    auth: &YokeAuth,
    id: &str,
    session_id: &str,
    op: &str,
    args_json: &str,
) -> OpResultRequest {
    let args: serde_json::Value = if args_json.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(args_json).unwrap_or_else(|_| serde_json::json!({}))
    };

    match gateway
        .call(op, args, auth, GatewayCallKind::First)
        .await
    {
        Ok(value) => OpResultRequest {
            id: id.to_string(),
            session_id: session_id.to_string(),
            ok: true,
            result_json: serde_json::to_string(&value).unwrap_or_else(|_| "null".into()),
            error: String::new(),
            ..Default::default()
        },
        Err(GatewayError::Denied(msg)) => OpResultRequest {
            id: id.to_string(),
            session_id: session_id.to_string(),
            ok: false,
            result_json: String::new(),
            error: msg,
            ..Default::default()
        },
        Err(GatewayError::Ask {
            op: ask_op,
            request_id,
            ..
        }) => OpResultRequest {
            id: id.to_string(),
            session_id: session_id.to_string(),
            ok: false,
            result_json: String::new(),
            error: format!(
                "ask required for `{ask_op}` (request_id={request_id}); \
                 headless Connect path cannot approve"
            ),
            ..Default::default()
        },
        Err(GatewayError::UnknownOp(name)) => OpResultRequest {
            id: id.to_string(),
            session_id: session_id.to_string(),
            ok: false,
            result_json: String::new(),
            error: format!("unknown op `{name}`"),
            ..Default::default()
        },
        Err(GatewayError::Dispatch(msg)) => OpResultRequest {
            id: id.to_string(),
            session_id: session_id.to_string(),
            ok: false,
            result_json: String::new(),
            error: msg,
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Process-global env mutation must be serialized across tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn socket_path_prefers_bridge_then_ship_fallback() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: held under ENV_LOCK; restored before unlock.
        unsafe {
            std::env::set_var("BRIDGE_YOKE_SOCKET", "/tmp/bridge-yoke.sock");
            std::env::set_var("SHIP_YOKE_SOCKET", "/tmp/ship-yoke.sock");
        }
        assert_eq!(
            socket_path_from_env().as_deref(),
            Some("/tmp/bridge-yoke.sock")
        );
        assert_eq!(resolve_socket_path(), "/tmp/bridge-yoke.sock");

        unsafe {
            std::env::remove_var("BRIDGE_YOKE_SOCKET");
        }
        assert_eq!(
            socket_path_from_env().as_deref(),
            Some("/tmp/ship-yoke.sock")
        );

        unsafe {
            std::env::remove_var("SHIP_YOKE_SOCKET");
        }
        assert_eq!(socket_path_from_env(), None);
        // No env → Bridge default path (absolute).
        let resolved = resolve_socket_path();
        assert!(
            resolved.ends_with("bridge-yoke.sock"),
            "default should be bridge-yoke.sock, got {resolved}"
        );
        assert!(Path::new(&resolved).is_absolute());
    }
}
