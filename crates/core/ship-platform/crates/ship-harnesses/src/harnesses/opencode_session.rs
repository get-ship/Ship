//! OpenCodeSession — HTTP+SSE connection to `opencode serve`.
//!
//! `opencode serve` exposes a REST API for session + turn management and
//! an SSE event stream at `/event`. Designed for orchestrator daemons:
//! each pipeline step becomes a POST to `/session/{id}/message` on a
//! shared opencode session; events fan out on the single `/event` stream
//! routed to per-turn subscribers by `messageID`. Cancellation is a
//! `DELETE /session/{id}/message/{msg}`.
//!
//! Transport is abstracted behind `base_url` so protocol tests can point
//! the session at an in-process stub HTTP server. SSE framing + event
//! mapping live in the sibling `opencode_session_sse` module so this
//! file stays under the project file cap.

#[path = "opencode_session_sse.rs"]
mod sse;

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc};

use ship_substrate::services::server_session::{ServerSession, SessionEvent};

/// Default port used by `opencode serve` when `--port` is omitted.
const DEFAULT_PORT: u16 = 4096;

/// Time between readiness probes after spawning the child.
const READY_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// How long to wait for `opencode serve` to bind its listener before
/// declaring spawn failure.
const READY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct OpenCodeSession {
    child: Arc<Mutex<Option<Child>>>,
    base_url: String,
    http: reqwest::Client,
    session_id: Arc<Mutex<Option<String>>>,
    subscribers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<SessionEvent>>>>,
}

impl OpenCodeSession {
    /// Spawn `opencode serve --port <port>`, wait for the HTTP listener to
    /// accept connections, kick off the SSE reader loop, and return a
    /// ready session. `port` defaults to 4096.
    pub async fn spawn() -> Result<Self> {
        let port = DEFAULT_PORT;
        let mut child = Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to spawn opencode serve: {e}"))?;

        // Drain stdio so the child's pipes don't fill and block.
        if let Some(out) = child.stdout.take() {
            drain_stream(out, tracing::Level::DEBUG, "opencode_stdout");
        }
        if let Some(err) = child.stderr.take() {
            drain_stream(err, tracing::Level::WARN, "opencode_stderr");
        }

        let base_url = format!("http://127.0.0.1:{port}");
        wait_for_ready(&base_url, READY_TIMEOUT).await?;
        let session = Self::new(base_url, Some(child));
        session.start_event_loop();
        Ok(session)
    }

    /// Construct against an already-running server. `child` is `Some` in
    /// the real-spawn path and `None` for tests (which use `with_server`).
    pub fn new(base_url: impl Into<String>, child: Option<Child>) -> Self {
        Self {
            child: Arc::new(Mutex::new(child)),
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            session_id: Arc::new(Mutex::new(None)),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Test helper: `new` + `start_event_loop` in one call.
    #[cfg(test)]
    pub(crate) fn with_server(base_url: impl Into<String>) -> Self {
        let session = Self::new(base_url, None);
        session.start_event_loop();
        session
    }

    fn start_event_loop(&self) {
        let base_url = self.base_url.clone();
        let http = self.http.clone();
        let subscribers = self.subscribers.clone();
        tokio::spawn(async move {
            run_event_loop(base_url, http, subscribers).await;
        });
    }

    async fn ensure_session(&self) -> Result<String> {
        let mut guard = self.session_id.lock().await;
        if let Some(existing) = guard.as_ref() {
            return Ok(existing.clone());
        }
        let url = format!("{}/session", self.base_url);
        let response = self
            .http
            .post(&url)
            .send()
            .await
            .map_err(|e| anyhow!("opencode POST /session failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow!("opencode POST /session non-2xx: {e}"))?;
        let body: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("opencode POST /session bad JSON: {e}"))?;
        let id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("opencode POST /session missing id: {body}"))?
            .to_string();
        *guard = Some(id.clone());
        Ok(id)
    }
}

#[async_trait]
impl ServerSession for OpenCodeSession {
    fn harness(&self) -> &str {
        "opencode"
    }

    async fn start_turn(&self, prompt: &str) -> Result<String> {
        let session_id = self.ensure_session().await?;
        let url = format!("{}/session/{}/message", self.base_url, session_id);
        let response = self
            .http
            .post(&url)
            .json(&json!({ "role": "user", "content": prompt }))
            .send()
            .await
            .map_err(|e| anyhow!("opencode POST /message failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow!("opencode POST /message non-2xx: {e}"))?;
        let body: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("opencode POST /message bad JSON: {e}"))?;
        body.get("messageID")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow!("opencode POST /message missing messageID: {body}"))
    }

    async fn subscribe(&self, turn_id: &str) -> Result<mpsc::UnboundedReceiver<SessionEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.lock().await.insert(turn_id.to_string(), tx);
        Ok(rx)
    }

    async fn cancel(&self, turn_id: &str) -> Result<()> {
        // Best-effort: net errors and unknown-turn responses are dropped.
        if let Some(session_id) = self.session_id.lock().await.clone() {
            let url = format!("{}/session/{}/message/{}", self.base_url, session_id, turn_id);
            let _ = self.http.delete(&url).send().await;
        }
        self.subscribers.lock().await.remove(turn_id);
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.subscribers.lock().await.clear();
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

/// Drain a child stdio stream line-by-line to `tracing` so the pipe
/// doesn't fill and block the child.
fn drain_stream<S>(stream: S, level: tracing::Level, target: &'static str)
where
    S: tokio::io::AsyncRead + Send + Unpin + 'static,
{
    use tokio::io::{AsyncBufReadExt, BufReader};
    tokio::spawn(async move {
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if level == tracing::Level::WARN {
                tracing::warn!(target: "opencode_session", %target, "{line}");
            } else {
                tracing::debug!(target: "opencode_session", %target, "{line}");
            }
        }
    });
}

/// Poll `GET {base_url}/` until any HTTP response arrives or the timeout
/// expires. Any accept-ready state suffices for the subsequent `/session`
/// POST.
async fn wait_for_ready(base_url: &str, timeout: Duration) -> Result<()> {
    let probe = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .map_err(|e| anyhow!("building readiness probe client: {e}"))?;
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if probe.get(base_url).send().await.is_ok() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(anyhow!(
                "opencode serve did not become ready within {timeout:?} at {base_url}"
            ));
        }
        tokio::time::sleep(READY_POLL_INTERVAL).await;
    }
}

/// Connect to `/event`, read SSE frames, map to `SessionEvent`, route to
/// the right subscriber. Exits when the stream ends or errors.
async fn run_event_loop(
    base_url: String,
    http: reqwest::Client,
    subscribers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<SessionEvent>>>>,
) {
    let url = format!("{base_url}/event");
    let mut response = match http.get(&url).header("Accept", "text/event-stream").send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(%e, "opencode-session: /event connect failed; exiting");
            return;
        }
    };
    let mut buffer = String::new();
    loop {
        match response.chunk().await {
            Ok(Some(bytes)) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));
                while let Some(idx) = sse::find_frame_end(&buffer) {
                    let frame: String = buffer.drain(..idx).collect();
                    let sep = sse::frame_separator_len(&buffer);
                    buffer.drain(..sep);
                    if let Some(parsed) = sse::parse_sse_frame(&frame) {
                        dispatch_event(parsed, &subscribers).await;
                    }
                }
            }
            Ok(None) => break,
            Err(err) => {
                tracing::warn!(%err, "opencode-session: /event read error; exiting");
                break;
            }
        }
    }
    tracing::info!("opencode-session: reader loop exited");
}

/// Route a parsed SSE event. Events with a `messageID` target that
/// turn's channel; events without one fan out to every subscriber.
async fn dispatch_event(
    (event_type, props): (String, Value),
    subscribers: &Arc<Mutex<HashMap<String, mpsc::UnboundedSender<SessionEvent>>>>,
) {
    let Some(event) = sse::map_sse_event_to_session_event(&event_type, &props) else {
        return;
    };
    let message_id = props
        .get("messageID")
        .and_then(|v| v.as_str())
        .map(String::from);
    let subs = subscribers.lock().await;
    match message_id {
        Some(mid) => {
            if let Some(tx) = subs.get(&mid) {
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
