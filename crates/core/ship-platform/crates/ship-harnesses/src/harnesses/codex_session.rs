//! CodexSession — JSON-RPC over stdio connection to `codex app-server`.
//!
//! Codex's app-server is a long-lived process that speaks JSON-RPC 2.0
//! over stdin/stdout, designed for orchestrator daemons exactly like
//! shipd. Each pipeline step submits a `turn/start` on a shared thread,
//! streams event notifications to ship's projection, and terminates on
//! `turn/completed`. Compared to `codex exec`, there is no per-step
//! spawn cost and cancellation becomes a clean `turn/interrupt`.
//!
//! The transport is abstracted behind `AsyncRead`/`AsyncWrite` halves so
//! protocol tests can drive the session over in-memory pipes without
//! launching the real binary.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};

use agent_driver::server_session::{ServerSession, SessionEvent};

/// Boxed transport halves used inside `CodexSession`. Tests inject
/// in-memory duplex streams; `spawn` supplies the child's stdio.
pub(crate) type BoxedReader = Box<dyn AsyncRead + Send + Unpin>;
pub(crate) type BoxedWriter = Box<dyn AsyncWrite + Send + Unpin>;

pub struct CodexSession {
    writer: Arc<Mutex<BoxedWriter>>,
    subscribers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<SessionEvent>>>>,
    thread_id: Arc<Mutex<Option<String>>>,
    next_id: Arc<AtomicU64>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    child: Arc<Mutex<Option<Child>>>,
}

impl CodexSession {
    /// Spawn `codex app-server` and complete the JSON-RPC handshake.
    pub async fn spawn() -> Result<Self> {
        let mut child = Command::new("codex")
            .arg("app-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to spawn codex app-server: {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("codex app-server child missing stdin handle"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("codex app-server child missing stdout handle"))?;
        if let Some(stderr) = child.stderr.take() {
            Self::drain_stderr(stderr);
        }

        let session = Self::from_io(Box::new(stdout), Box::new(stdin), Some(child));
        session.handshake().await?;
        Ok(session)
    }

    /// Construct from pre-split transport halves. Used by `spawn` (real
    /// child stdio) and tests (in-memory duplex). `child` is `None` for
    /// the test path.
    pub(crate) fn from_io(reader: BoxedReader, writer: BoxedWriter, child: Option<Child>) -> Self {
        let session = Self {
            writer: Arc::new(Mutex::new(writer)),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            thread_id: Arc::new(Mutex::new(None)),
            next_id: Arc::new(AtomicU64::new(1)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            child: Arc::new(Mutex::new(child)),
        };
        session.start_reader_loop(reader);
        session
    }

    /// Test helper: construct from transport halves and run the
    /// JSON-RPC handshake. Kept `pub(crate)` so only runtime tests call it.
    #[cfg(test)]
    pub(crate) async fn with_transport(reader: BoxedReader, writer: BoxedWriter) -> Result<Self> {
        let session = Self::from_io(reader, writer, None);
        session.handshake().await?;
        Ok(session)
    }

    fn start_reader_loop(&self, reader: BoxedReader) {
        let pending = self.pending.clone();
        let subscribers = self.subscribers.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        let msg: Value = match serde_json::from_str(&line) {
                            Ok(v) => v,
                            Err(err) => {
                                tracing::warn!(%err, %line,
                                    "codex-session: dropped non-JSON line");
                                continue;
                            }
                        };
                        dispatch_incoming(&msg, &pending, &subscribers).await;
                    }
                    Ok(None) => break,
                    Err(err) => {
                        tracing::warn!(%err, "codex-session: read error; exiting reader loop");
                        break;
                    }
                }
            }
            tracing::info!("codex-session: reader loop exited");
        });
    }

    fn drain_stderr(stderr: tokio::process::ChildStderr) {
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::warn!(target: "codex_session_stderr", "{line}");
            }
        });
    }

    async fn handshake(&self) -> Result<()> {
        let response = self
            .request(
                "initialize",
                serde_json::json!({
                    "client": {
                        "name": "shipd",
                        "version": env!("CARGO_PKG_VERSION"),
                    }
                }),
            )
            .await?;
        if response.get("error").is_some() {
            anyhow::bail!("codex initialize failed: {response}");
        }
        self.notify("initialized", serde_json::json!({})).await?;
        Ok(())
    }

    async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.write_line(&request).await?;
        rx.await.map_err(|_| anyhow!("codex request {id} dropped"))
    }

    async fn notify(&self, method: &str, params: Value) -> Result<()> {
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_line(&notif).await
    }

    async fn write_line(&self, value: &Value) -> Result<()> {
        let line = format!("{}\n", serde_json::to_string(value)?);
        let mut writer = self.writer.lock().await;
        writer.write_all(line.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn ensure_thread(&self) -> Result<String> {
        let mut tid = self.thread_id.lock().await;
        if let Some(existing) = tid.as_ref() {
            return Ok(existing.clone());
        }
        let response = self
            .request("thread/start", serde_json::json!({}))
            .await?;
        let new_thread_id = response
            .get("result")
            .and_then(|r| r.get("thread_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("thread/start response missing thread_id: {response}"))?
            .to_string();
        *tid = Some(new_thread_id.clone());
        Ok(new_thread_id)
    }
}

#[async_trait]
impl ServerSession for CodexSession {
    fn harness(&self) -> &str {
        "codex"
    }

    async fn start_turn(&self, prompt: &str) -> Result<String> {
        let thread_id = self.ensure_thread().await?;
        let response = self
            .request(
                "turn/start",
                serde_json::json!({
                    "thread_id": thread_id,
                    "input": prompt,
                }),
            )
            .await?;
        response
            .get("result")
            .and_then(|r| r.get("turn_id"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow!("turn/start response missing turn_id: {response}"))
    }

    async fn subscribe(&self, turn_id: &str) -> Result<mpsc::UnboundedReceiver<SessionEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.lock().await.insert(turn_id.to_string(), tx);
        Ok(rx)
    }

    async fn cancel(&self, turn_id: &str) -> Result<()> {
        // Best-effort: ignore errors (channel closed, turn unknown, etc.).
        let _ = self
            .request(
                "turn/interrupt",
                serde_json::json!({ "turn_id": turn_id }),
            )
            .await;
        self.subscribers.lock().await.remove(turn_id);
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.subscribers.lock().await.clear();
        let _ = self.notify("shutdown", serde_json::json!({})).await;
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

async fn dispatch_incoming(
    msg: &Value,
    pending: &Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    subscribers: &Arc<Mutex<HashMap<String, mpsc::UnboundedSender<SessionEvent>>>>,
) {
    // Response: has numeric "id" and either "result" or "error".
    if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
        if let Some(tx) = pending.lock().await.remove(&id) {
            let _ = tx.send(msg.clone());
        }
        return;
    }

    // Notification: has "method", no "id".
    let Some(method) = msg.get("method").and_then(|v| v.as_str()) else {
        return;
    };
    let Some(event) = map_notification_to_event(method, msg) else {
        return;
    };

    let turn_id = msg
        .get("params")
        .and_then(|p| p.get("turn_id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let subs = subscribers.lock().await;
    match turn_id {
        Some(tid) => {
            if let Some(tx) = subs.get(&tid) {
                let _ = tx.send(event);
            }
        }
        None => {
            for tx in subs.values() {
                let _ = tx.send(event.clone());
            }
        }
    }
}

fn map_notification_to_event(method: &str, msg: &Value) -> Option<SessionEvent> {
    let params = msg.get("params").cloned().unwrap_or_default();
    match method {
        "turn/started" => {
            let turn_id = params.get("turn_id").and_then(|v| v.as_str())?.to_string();
            Some(SessionEvent::TurnStarted { turn_id })
        }
        "turn/completed" => {
            let stop_reason = params
                .get("stop_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("completed")
                .to_string();
            Some(SessionEvent::TurnCompleted { stop_reason })
        }
        "turn/error" => {
            let message = params
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("codex turn error")
                .to_string();
            Some(SessionEvent::Error { message })
        }
        "item/agentMessage/delta" => {
            let text = params.get("delta").and_then(|v| v.as_str())?.to_string();
            Some(SessionEvent::AgentMessage { text })
        }
        "item/commandExecution/started" => {
            let cmd = params
                .get("command")
                .and_then(|v| v.as_str())?
                .to_string();
            Some(SessionEvent::CommandStarted { cmd })
        }
        "item/commandExecution/outputDelta" => {
            let stream = params
                .get("stream")
                .and_then(|v| v.as_str())
                .unwrap_or("stdout")
                .to_string();
            let line = params.get("delta").and_then(|v| v.as_str())?.to_string();
            Some(SessionEvent::CommandOutput { stream, line })
        }
        "item/toolUse/started" => {
            let tool = params.get("tool").and_then(|v| v.as_str())?.to_string();
            let input_preview = params
                .get("input")
                .map(|v| v.to_string())
                .unwrap_or_default();
            Some(SessionEvent::ToolUse { tool, input_preview })
        }
        _ => None,
    }
}

