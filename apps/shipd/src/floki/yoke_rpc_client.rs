//! Yoke harness session RPC client (H3 spike).
//!
//! Dual dispatch path for native agent sessions:
//! - **RPC path** (this module): when `BRIDGE_YOKE_SOCKET` is set (fallback:
//!   `SHIP_YOKE_SOCKET`), shipd connects over a Unix domain stream and runs
//!   the session via Bridge/Yoke `bridge serve` (NDJSON v1 harness RPC).
//! - **In-process floki path**: default when the socket env is unset —
//!   [`super::floki_dispatch::dispatch_floki_session`].
//!
//! Floki is intentionally kept until RPC is proven. Do not delete it.
//!
//! Wire types live in [`super::yoke_rpc_protocol`] (hand-rolled mirrors of
//! bridge protocol — no path-dep on the bridge crate).
//!
//! Spike behaviour for `op.call`: reply with denied `op.result`. A real
//! `ShipdGateway` host answer is a follow-on.
//!
//! ## Smoke
//!
//! ```text
//! # terminal A
//! cargo run -p bridge --bin bridge -- serve --mock --socket /tmp/bridge-yoke.sock
//!
//! # terminal B
//! BRIDGE_YOKE_SOCKET=/tmp/bridge-yoke.sock shipd   # then dispatch a floki harness session
//! ```

use std::path::Path;

use agent_def::dispatch_intent::DispatchRequestedPayload;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use super::yoke_rpc_protocol::{
    ClientEnvelope, ClientMessage, OpResult, PROTOCOL_VERSION, ServerEnvelope, ServerMessage,
    SessionStart,
};

// ── Socket env ───────────────────────────────────────────────────────────────

/// Resolve the yoke serve socket path.
///
/// Prefer `BRIDGE_YOKE_SOCKET`; fall back to `SHIP_YOKE_SOCKET`. Empty values
/// are treated as unset.
pub fn socket_path_from_env() -> Option<String> {
    if let Ok(p) = std::env::var("BRIDGE_YOKE_SOCKET") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    // `var_os` avoids the raw `env::var("SHIP_…")` lint surface.
    std::env::var_os("SHIP_YOKE_SOCKET")
        .and_then(|s| s.into_string().ok())
        .filter(|p| !p.is_empty())
}

// ── Session outcome ──────────────────────────────────────────────────────────

/// Terminal outcome of a harness session over UDS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionOutcome {
    Completed { session_id: String },
    Failed { session_id: String, message: String },
    Cancelled { session_id: String, reason: Option<String> },
}

// ── Dispatch entry ───────────────────────────────────────────────────────────

/// Run one harness session over the yoke UDS RPC (spike).
///
/// Sends `session.start` from `payload`, reads events until a terminal
/// lifecycle message, logs `session.output`, and denies any `op.call`.
pub async fn dispatch_yoke_session(payload: DispatchRequestedPayload, socket_path: &str) {
    tracing::info!(
        agent_id = %payload.agent_id,
        slug = %payload.slug,
        session_id = %payload.session_id,
        socket = %socket_path,
        "yoke-rpc: starting session over UDS",
    );

    match run_session(socket_path, &payload).await {
        Ok(SessionOutcome::Completed { session_id }) => {
            tracing::info!(%session_id, "yoke-rpc: session completed");
        }
        Ok(SessionOutcome::Failed {
            session_id,
            message,
        }) => {
            tracing::error!(%session_id, %message, "yoke-rpc: session failed");
        }
        Ok(SessionOutcome::Cancelled {
            session_id,
            reason,
        }) => {
            tracing::warn!(%session_id, ?reason, "yoke-rpc: session cancelled");
        }
        Err(e) => {
            tracing::error!(
                session_id = %payload.session_id,
                error = %e,
                "yoke-rpc: transport or protocol error",
            );
        }
    }
}

/// Connect, start session, drive event loop until terminal.
pub async fn run_session(
    socket_path: &str,
    payload: &DispatchRequestedPayload,
) -> anyhow::Result<SessionOutcome> {
    if !Path::new(socket_path).exists() {
        anyhow::bail!("yoke socket not found: {socket_path}");
    }

    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let prompt = payload
        .prompt
        .as_deref()
        .or(payload.spec.as_deref())
        .unwrap_or("(no input provided)")
        .to_string();

    let start = ClientEnvelope::new(ClientMessage::SessionStart(SessionStart {
        session_id: Some(payload.session_id.clone()),
        agent_id: payload.agent_id.clone(),
        prompt,
        provider: None,
        model: payload.model.clone(),
        max_turns: None,
        workspace: payload.repo_path.clone(),
    }));
    writer
        .write_all(start.to_json_line()?.as_bytes())
        .await?;
    writer.flush().await?;

    loop {
        let line = lines
            .next_line()
            .await?
            .ok_or_else(|| anyhow::anyhow!("yoke socket closed before session terminal event"))?;
        if line.trim().is_empty() {
            continue;
        }

        let env = match ServerEnvelope::from_json_line(&line) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, raw = %line, "yoke-rpc: bad server line");
                continue;
            }
        };
        if env.v != PROTOCOL_VERSION {
            tracing::warn!(v = env.v, "yoke-rpc: unexpected protocol version");
        }

        match env.message {
            ServerMessage::SessionStarted(s) => {
                tracing::info!(
                    session_id = %s.session_id,
                    agent_id = ?s.agent_id,
                    "yoke-rpc: session.started",
                );
            }
            ServerMessage::SessionOutput(o) => {
                tracing::info!(
                    session_id = %o.session_id,
                    text_len = o.text.len(),
                    "yoke-rpc: session.output"
                );
                tracing::debug!(session_id = %o.session_id, text = %o.text, "yoke-rpc: output text");
            }
            ServerMessage::SessionCompleted(c) => {
                return Ok(SessionOutcome::Completed {
                    session_id: c.session_id,
                });
            }
            ServerMessage::SessionFailed(f) => {
                return Ok(SessionOutcome::Failed {
                    session_id: f.session_id,
                    message: f.message,
                });
            }
            ServerMessage::SessionCancelled(c) => {
                return Ok(SessionOutcome::Cancelled {
                    session_id: c.session_id,
                    reason: c.reason,
                });
            }
            ServerMessage::OpCall(call) => {
                // Spike: deny-all host gateway. Real ShipdGateway later.
                tracing::warn!(
                    id = %call.id,
                    session_id = %call.session_id,
                    op = %call.op,
                    "yoke-rpc: op.call denied (spike)",
                );
                let deny = ClientEnvelope::new(ClientMessage::OpResult(OpResult {
                    id: call.id,
                    session_id: call.session_id,
                    ok: false,
                    result: None,
                    error: Some("denied: ShipdGateway not wired on yoke-rpc spike".into()),
                }));
                writer
                    .write_all(deny.to_json_line()?.as_bytes())
                    .await?;
                writer.flush().await?;
            }
            ServerMessage::Error(err) => {
                anyhow::bail!(
                    "yoke-rpc protocol error (id={:?}): {}",
                    err.id,
                    err.message
                );
            }
        }
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
    }
}
