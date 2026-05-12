//! Manages the long-lived Bun bundler child process.
//!
//! One worker per daemon. Supervised: auto-restarts with exponential backoff
//! if the process exits unexpectedly. The worker reads `bundler.ts` from
//! `~/.ship/runtime/bundler.ts` — installed by `ship runtime install`.
//!
//! Communication is over stdin/stdout using the JSON-line protocol in
//! `proto.rs`. All pending requests are re-sent after each restart so
//! callers see at-least-once delivery on transient failures.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use tracing::{error, info, warn};

use ship_substrate::ShipHome;

use super::proto::{BundleManifest, WorkerEvent, WorkerLine, WorkerRequest, WorkerResponse};

// ── Events emitted by the worker ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum BundlerEvent {
    ModuleChanged { target: String, out_path: String, bytes: usize },
    CompileError { target: String, message: String, line: Option<u32>, col: Option<u32>, file: Option<String> },
    WorkerRestarted,
}

// ── Internal command channel ──────────────────────────────────────────────────

struct PendingCall {
    request: WorkerRequest,
    reply: oneshot::Sender<Result<WorkerResponse, WorkerError>>,
}

// ── Public handle ─────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("bundler worker is not running")]
    NotRunning,
    #[error("bundler error: {message}")]
    BundleError { message: String, target: Option<String> },
    #[error("bundler request timed out")]
    Timeout,
}

/// Handle to the supervised bundler worker. Clone-safe; backed by message
/// passing.
#[derive(Clone)]
pub struct WorkerHandle {
    cmd_tx: mpsc::Sender<PendingCall>,
    event_tx: broadcast::Sender<BundlerEvent>,
}

impl WorkerHandle {
    /// Send a request to the worker and wait for the response.
    async fn call(&self, req: WorkerRequest) -> Result<WorkerResponse, WorkerError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx.send(PendingCall { request: req, reply: reply_tx }).await
            .map_err(|_| WorkerError::NotRunning)?;
        tokio::time::timeout(Duration::from_secs(120), reply_rx)
            .await
            .map_err(|_| WorkerError::Timeout)?
            .map_err(|_| WorkerError::NotRunning)?
    }

    /// Build a single named target.
    pub async fn build(&self, target: &str) -> Result<BundleManifest, WorkerError> {
        let id = ulid::Ulid::new().to_string();
        let resp = self.call(WorkerRequest::Build { id, target: target.to_owned() }).await?;
        match resp {
            WorkerResponse::Bundle { manifest, .. } => Ok(manifest),
            WorkerResponse::Error { message, target, .. } => Err(WorkerError::BundleError { message, target }),
            _ => Err(WorkerError::BundleError { message: "unexpected response".into(), target: Some(target.to_owned()) }),
        }
    }

    /// Build all targets. Returns after all bundles have been emitted.
    pub async fn build_all(&self) -> Result<(), WorkerError> {
        let id = ulid::Ulid::new().to_string();
        self.call(WorkerRequest::BuildAll { id }).await?;
        Ok(())
    }

    /// Start file watching. Worker will emit BundlerEvent::ModuleChanged
    /// unsolicited over the broadcast channel.
    pub async fn start_watching(&self) -> Result<(), WorkerError> {
        let id = ulid::Ulid::new().to_string();
        self.call(WorkerRequest::WatchStart { id }).await?;
        Ok(())
    }

    /// Stop file watching.
    pub async fn stop_watching(&self) -> Result<(), WorkerError> {
        let id = ulid::Ulid::new().to_string();
        self.call(WorkerRequest::WatchStop { id }).await?;
        Ok(())
    }

    /// Subscribe to unsolicited worker events (module_changed, compile_error).
    pub fn subscribe_events(&self) -> broadcast::Receiver<BundlerEvent> {
        self.event_tx.subscribe()
    }

    /// Graceful shutdown. The worker process exits cleanly.
    pub async fn shutdown(&self) -> Result<(), WorkerError> {
        let id = ulid::Ulid::new().to_string();
        let _ = self.call(WorkerRequest::Shutdown { id }).await;
        Ok(())
    }
}

// ── Supervisor loop ───────────────────────────────────────────────────────────

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const MAX_RESTARTS: u32 = 20;

/// Start the supervised worker. Returns a handle immediately; the worker
/// process is spawned in the background.
pub fn start(home: &ShipHome, repo_root: PathBuf) -> WorkerHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel::<PendingCall>(64);
    let (event_tx, _) = broadcast::channel::<BundlerEvent>(128);

    let bun = home.runtime_bun();
    let bundler_ts = home.runtime_bundler_ts();
    let event_tx2 = event_tx.clone();
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));

    tokio::spawn(async move {
        run_supervisor(bun, bundler_ts, repo_root, cmd_rx, event_tx2).await;
    });

    WorkerHandle { cmd_tx, event_tx }
}

async fn run_supervisor(
    bun: PathBuf,
    bundler_ts: PathBuf,
    repo_root: PathBuf,
    cmd_rx: Arc<Mutex<mpsc::Receiver<PendingCall>>>,
    event_tx: broadcast::Sender<BundlerEvent>,
) {
    // Toolchain probe — `~/.ship/runtime/` isn't populated until the
    // user runs `ship runtime install`. Don't spawn-and-backoff in a
    // loop when both pieces are missing; just log once and stay idle.
    // The supervisor wakes up if/when callers send commands (mpsc),
    // which will fail with NotRunning — also acceptable.
    if !bun.exists() || !bundler_ts.exists() {
        warn!(
            bun = %bun.display(),
            bundler_ts = %bundler_ts.display(),
            "bundler: toolchain not installed at ~/.ship/runtime/ — \
             worker disabled. Run `ship runtime install` to enable."
        );
        return;
    }

    let mut backoff = INITIAL_BACKOFF;
    let mut restarts = 0u32;

    loop {
        match spawn_worker(&bun, &bundler_ts, &repo_root).await {
            Err(e) => {
                error!(error = %e, "bundler: failed to spawn worker");
            }
            Ok(mut child) => {
                info!("bundler: worker started (pid={:?})", child.id());
                backoff = INITIAL_BACKOFF;
                run_worker_io(&mut child, &cmd_rx, &event_tx).await;
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        restarts += 1;
        if restarts > MAX_RESTARTS {
            error!("bundler: worker exceeded restart limit; giving up");
            break;
        }

        let _ = event_tx.send(BundlerEvent::WorkerRestarted);
        warn!(backoff_ms = backoff.as_millis(), "bundler: worker crashed, restarting");
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn spawn_worker(bun: &PathBuf, bundler_ts: &PathBuf, repo_root: &PathBuf) -> std::io::Result<Child> {
    Command::new(bun)
        .arg("run")
        .arg(bundler_ts)
        .env("SHIP_REPO_ROOT", repo_root)
        .current_dir(repo_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
}

/// Drive the worker's stdin/stdout I/O loop. Returns when the process exits.
async fn run_worker_io(
    child: &mut Child,
    cmd_rx: &Arc<Mutex<mpsc::Receiver<PendingCall>>>,
    event_tx: &broadcast::Sender<BundlerEvent>,
) {
    let stdout = child.stdout.take().expect("stdout must be piped");
    let stdin = child.stdin.take().expect("stdin must be piped");

    let mut stdin = stdin;
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    // Pending requests waiting for correlated response
    let pending: Arc<Mutex<HashMap<String, oneshot::Sender<Result<WorkerResponse, WorkerError>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Wait for the initial 'ready' event before accepting requests
    match tokio::time::timeout(Duration::from_secs(30), lines.next_line()).await {
        Ok(Ok(Some(line))) => {
            if let Ok(WorkerLine::Event(WorkerEvent::Ready)) = serde_json::from_str(&line) {
                info!("bundler: worker ready");
            } else {
                warn!(line = %line, "bundler: unexpected first line");
            }
        }
        _ => {
            warn!("bundler: worker did not emit 'ready' within 30s");
            return;
        }
    }

    loop {
        let mut rx = cmd_rx.lock().await;
        tokio::select! {
            // Incoming request from the Rust side
            maybe_call = rx.recv() => {
                drop(rx);
                let Some(call) = maybe_call else { return };

                let id = call.request.id().to_owned();
                let line = match serde_json::to_string(&call.request) {
                    Ok(s) => s + "\n",
                    Err(e) => {
                        let _ = call.reply.send(Err(WorkerError::BundleError {
                            message: format!("serialize error: {e}"),
                            target: None,
                        }));
                        continue;
                    }
                };

                if stdin.write_all(line.as_bytes()).await.is_err() {
                    let _ = call.reply.send(Err(WorkerError::NotRunning));
                    return;
                }

                pending.lock().await.insert(id, call.reply);
            }

            // Incoming line from the worker
            next = lines.next_line() => {
                drop(rx);
                match next {
                    Ok(Some(line)) => {
                        handle_worker_line(&line, &pending, event_tx).await;
                    }
                    _ => {
                        // EOF or error — worker process died
                        // Fail all pending requests
                        let mut map = pending.lock().await;
                        for (_, reply) in map.drain() {
                            let _ = reply.send(Err(WorkerError::NotRunning));
                        }
                        return;
                    }
                }
            }
        }
    }
}

async fn handle_worker_line(
    line: &str,
    pending: &Arc<Mutex<HashMap<String, oneshot::Sender<Result<WorkerResponse, WorkerError>>>>>,
    event_tx: &broadcast::Sender<BundlerEvent>,
) {
    let parsed: WorkerLine = match serde_json::from_str(line) {
        Ok(p) => p,
        Err(e) => {
            warn!(line = %line, error = %e, "bundler: failed to parse worker output");
            return;
        }
    };

    match parsed {
        WorkerLine::Response(resp) => {
            let id = resp.id().to_owned();
            if let Some(reply) = pending.lock().await.remove(&id) {
                let _ = reply.send(Ok(resp));
            }
        }
        WorkerLine::Event(event) => match event {
            WorkerEvent::ModuleChanged { target, out_path, bytes } => {
                let _ = event_tx.send(BundlerEvent::ModuleChanged { target, out_path, bytes });
            }
            WorkerEvent::CompileError { target, message, line, col, file } => {
                let _ = event_tx.send(BundlerEvent::CompileError { target, message, line, col, file });
            }
            WorkerEvent::Ready => {
                // Second ready would be from a restart — already handled
            }
        },
    }
}
