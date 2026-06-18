//! In-sandbox process registry + event bus.
//!
//! Owns N child processes keyed by string id. Spawns watchers that
//! push `Envelope::ProcessOutput` / `Envelope::ProcessExited` onto a
//! broadcast channel; `Events.Stream` handlers subscribe to it.
// Wire-builder style: proto envelopes are built default-then-assign at many
// sites; the struct-literal form buys no safety on these non-exhaustive
// generated types.
#![allow(clippy::field_reassign_with_default)]

use portable_pty::{CommandBuilder, MasterPty, PtySize as PortablePtySize, native_pty_system};
use ship_isolation::sidecar::proto::*;
use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, RwLock, broadcast};

const EVENT_CHANNEL_CAP: usize = 1024;
const PTY_OUTPUT_CAP: usize = 1024;
const READ_CHUNK: usize = 4096;

/// PTY-backed process resources. Held inline on `Process` when
/// `pty_backed` is true. `pty_output` is broadcast to subscribers
/// of `AttachPty`; `writer` is the master end used by `WriteInput`;
/// `master` keeps the master fd alive (drop closes the slave side).
pub struct PtyState {
    pub master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
    pub writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    pub output: broadcast::Sender<PtyChunk>,
}

pub struct Process {
    pub id: String,
    pub bin: String,
    pub args: Vec<String>,
    pub pid: Option<u32>,
    pub created_at_unix_ms: i64,
    pub pty_backed: bool,
    state: RwLock<ProcessRecord>,
    // Non-PTY path.
    stdin: Mutex<Option<ChildStdin>>,
    child: Mutex<Option<Child>>,
    // PTY path. Either both `pty_*` fields are set, or both are None.
    pub pty: Option<PtyState>,
}

struct ProcessRecord {
    state: ProcessState,
    exit_code: i32,
}

impl Process {
    pub async fn info(&self) -> ProcessInfo {
        let rec = self.state.read().await;
        let mut info = ProcessInfo::default();
        info.id = self.id.clone();
        info.bin = self.bin.clone();
        info.args = self.args.clone();
        info.pid = self.pid.unwrap_or(0);
        info.state = ::buffa::EnumValue::Known(rec.state);
        info.created_at_unix_ms = self.created_at_unix_ms;
        info.exit_code = rec.exit_code;
        info.pty_backed = self.pty_backed;
        info
    }
}

pub struct Session {
    pub events: broadcast::Sender<Envelope>,
    pub sequence: AtomicU64,
    procs: RwLock<HashMap<String, Arc<Process>>>,
}

impl Session {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAP);
        Self {
            events: tx,
            sequence: AtomicU64::new(1),
            procs: RwLock::new(HashMap::new()),
        }
    }

    pub fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn get(&self, id: &str) -> Option<Arc<Process>> {
        self.procs.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<Arc<Process>> {
        self.procs.read().await.values().cloned().collect()
    }

    /// Spawn a long-lived child and register it. Watchers stream
    /// stdout/stderr lines and exit status onto the event bus.
    pub async fn create(
        self: &Arc<Self>,
        bin: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: String,
        id_hint: String,
    ) -> std::io::Result<ProcessInfo> {
        let id = if id_hint.is_empty() {
            format!("proc_{}", nanoid::nanoid!(10))
        } else {
            id_hint
        };

        let mut cmd = Command::new(&bin);
        cmd.args(&args)
            .envs(&env)
            .current_dir(if cwd.is_empty() { "." } else { cwd.as_str() })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;
        let pid = child.id();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdin = child.stdin.take();

        let proc = Arc::new(Process {
            id: id.clone(),
            bin,
            args,
            pid,
            created_at_unix_ms: now_unix_ms(),
            pty_backed: false,
            state: RwLock::new(ProcessRecord {
                state: ProcessState::PROCESS_STATE_RUNNING,
                exit_code: 0,
            }),
            stdin: Mutex::new(stdin),
            child: Mutex::new(Some(child)),
            pty: None,
        });

        self.procs.write().await.insert(id.clone(), proc.clone());

        if let Some(out) = stdout {
            let s = self.clone();
            let pid = id.clone();
            tokio::spawn(forward_stream(out, s, pid, 1));
        }
        if let Some(err) = stderr {
            let s = self.clone();
            let pid = id.clone();
            tokio::spawn(forward_stream(err, s, pid, 2));
        }

        let watcher_self = self.clone();
        let watcher_proc = proc.clone();
        let watcher_id = id.clone();
        tokio::spawn(async move {
            let exit = {
                let mut guard = watcher_proc.child.lock().await;
                match guard.as_mut() {
                    Some(c) => c.wait().await,
                    None => return,
                }
            };
            let (code, signal) = match exit {
                Ok(s) => (s.code().unwrap_or(0), unix_signal_name(&s)),
                Err(_) => (-1, String::new()),
            };
            {
                let mut rec = watcher_proc.state.write().await;
                rec.exit_code = code;
                rec.state = if signal.is_empty() {
                    ProcessState::PROCESS_STATE_EXITED
                } else {
                    ProcessState::PROCESS_STATE_KILLED
                };
            }
            let mut exited = ProcessExited::default();
            exited.process_id = watcher_id.clone();
            exited.exit_code = code;
            exited.signal = signal;
            let mut env = Envelope::default();
            env.sequence = watcher_self.next_sequence();
            env.payload = Some(envelope::Payload::ProcessExited(Box::new(exited)));
            let _ = watcher_self.events.send(env);
        });

        Ok(proc.info().await)
    }

    /// Spawn a long-lived PTY-backed child. The slave's output is
    /// fanned out to subscribers of `Events::AttachPty`; writes from
    /// `Processes.WriteInput` go to the master end. Resize requests
    /// from `Processes.ResizeTerminal` flow through to the kernel
    /// via `MasterPty::resize`.
    pub async fn create_pty(
        self: &Arc<Self>,
        bin: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: String,
        id_hint: String,
        size: PortablePtySize,
    ) -> std::io::Result<ProcessInfo> {
        let id = if id_hint.is_empty() {
            format!("proc_{}", nanoid::nanoid!(10))
        } else {
            id_hint
        };

        // portable_pty is synchronous — set up off the runtime to keep
        // the openpty + spawn syscalls off any executor thread.
        let bin_for_thread = bin.clone();
        let args_for_thread = args.clone();
        let cwd_for_thread = if cwd.is_empty() { ".".to_string() } else { cwd.clone() };
        let env_for_thread = env.clone();
        let setup = tokio::task::spawn_blocking(move || -> std::io::Result<_> {
            let pty = native_pty_system();
            let pair = pty
                .openpty(size)
                .map_err(|e| std::io::Error::other(format!("openpty: {e}")))?;

            let mut cmd = CommandBuilder::new(&bin_for_thread);
            for a in &args_for_thread {
                cmd.arg(a);
            }
            cmd.cwd(&cwd_for_thread);
            for (k, v) in &env_for_thread {
                cmd.env(k, v);
            }

            let child = pair
                .slave
                .spawn_command(cmd)
                .map_err(|e| std::io::Error::other(format!("spawn_command: {e}")))?;
            drop(pair.slave);

            let reader = pair
                .master
                .try_clone_reader()
                .map_err(|e| std::io::Error::other(format!("try_clone_reader: {e}")))?;
            let writer = pair
                .master
                .take_writer()
                .map_err(|e| std::io::Error::other(format!("take_writer: {e}")))?;

            Ok((pair.master, reader, writer, child))
        })
        .await
        .map_err(|e| std::io::Error::other(format!("blocking task: {e}")))??;
        let (master, reader, writer, mut child) = setup;
        let pid = child.process_id();

        let (pty_output_tx, _) = broadcast::channel(PTY_OUTPUT_CAP);
        let proc = Arc::new(Process {
            id: id.clone(),
            bin,
            args,
            pid,
            created_at_unix_ms: now_unix_ms(),
            pty_backed: true,
            state: RwLock::new(ProcessRecord {
                state: ProcessState::PROCESS_STATE_RUNNING,
                exit_code: 0,
            }),
            stdin: Mutex::new(None),
            child: Mutex::new(None),
            pty: Some(PtyState {
                master: Arc::new(Mutex::new(Some(master))),
                writer: Arc::new(Mutex::new(Some(writer))),
                output: pty_output_tx.clone(),
            }),
        });

        self.procs.write().await.insert(id.clone(), proc.clone());

        // Reader thread — blocking std::io::Read from the PTY master.
        // Each chunk lands as a PtyChunk on the per-process broadcast
        // channel that AttachPty subscribers read from.
        let reader_id = id.clone();
        let reader_output = pty_output_tx.clone();
        std::thread::spawn(move || {
            pty_reader_loop(reader, reader_output, reader_id);
        });

        // Watcher thread — blocking std::sync wait on the portable_pty
        // child handle. Fires ProcessExited on the session event bus
        // when the child reaps; matches the tokio path's behavior.
        let watcher_self = self.clone();
        let watcher_proc = proc.clone();
        let watcher_id = id.clone();
        std::thread::spawn(move || {
            let status = child.wait();
            let (code, signal) = match status {
                Ok(s) if s.success() => (0i32, String::new()),
                Ok(s) => {
                    let sig = s.signal().map(|n| n.to_string()).unwrap_or_default();
                    (s.exit_code() as i32, sig)
                }
                Err(_) => (-1, String::new()),
            };
            // Block-on the tokio runtime via spawning a small task —
            // we're on a std::thread so we can't use .await directly.
            // Use a tokio::sync::oneshot semaphore via Handle::current()
            // doesn't work cross-runtime; just use blocking_lock paths
            // available on tokio::sync::Mutex/RwLock.
            let mut rec = watcher_proc.state.blocking_write();
            rec.exit_code = code;
            rec.state = if signal.is_empty() {
                ProcessState::PROCESS_STATE_EXITED
            } else {
                ProcessState::PROCESS_STATE_KILLED
            };
            drop(rec);
            let mut exited = ProcessExited::default();
            exited.process_id = watcher_id;
            exited.exit_code = code;
            exited.signal = signal;
            let mut envp = Envelope::default();
            envp.sequence = watcher_self.next_sequence();
            envp.payload = Some(envelope::Payload::ProcessExited(Box::new(exited)));
            let _ = watcher_self.events.send(envp);
        });

        Ok(proc.info().await)
    }

    /// Run a one-shot. Drains stdout/stderr in full, optionally feeds
    /// `stdin`, honors `timeout_ms`, returns ProcessRunResponse at exit.
    pub async fn run(
        self: &Arc<Self>,
        bin: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: String,
        stdin: Vec<u8>,
        timeout_ms: u64,
    ) -> std::io::Result<ProcessRunResponse> {
        let start = std::time::Instant::now();
        let mut cmd = Command::new(&bin);
        cmd.args(&args)
            .envs(&env)
            .current_dir(if cwd.is_empty() { "." } else { cwd.as_str() })
            .stdin(if stdin.is_empty() {
                Stdio::null()
            } else {
                Stdio::piped()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;
        if !stdin.is_empty() {
            if let Some(mut s) = child.stdin.take() {
                let _ = s.write_all(&stdin).await;
                let _ = s.shutdown().await;
            }
        }
        let mut stdout = child.stdout.take();
        let mut stderr = child.stderr.take();

        let collect = async move {
            let mut out = Vec::new();
            let mut err = Vec::new();
            if let Some(o) = stdout.as_mut() {
                let _ = o.read_to_end(&mut out).await;
            }
            if let Some(e) = stderr.as_mut() {
                let _ = e.read_to_end(&mut err).await;
            }
            let status = child.wait().await?;
            Ok::<_, std::io::Error>((status, out, err))
        };

        let (status, out, err) = if timeout_ms > 0 {
            match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), collect).await
            {
                Ok(v) => v?,
                Err(_) => {
                    let mut resp = ProcessRunResponse::default();
                    resp.exit_code = -1;
                    resp.duration_ms = start.elapsed().as_millis() as u64;
                    return Ok(resp);
                }
            }
        } else {
            collect.await?
        };

        let mut resp = ProcessRunResponse::default();
        resp.exit_code = status.code().unwrap_or(-1);
        resp.stdout = out;
        resp.stderr = err;
        resp.duration_ms = start.elapsed().as_millis() as u64;
        Ok(resp)
    }

    pub async fn write_input(&self, id: &str, data: &[u8]) -> std::io::Result<()> {
        let proc = self
            .get(id)
            .await
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such process"))?;

        if let Some(pty) = &proc.pty {
            let writer = pty.writer.clone();
            let data = data.to_vec();
            tokio::task::spawn_blocking(move || -> std::io::Result<()> {
                let mut guard = writer.blocking_lock();
                match guard.as_mut() {
                    Some(w) => {
                        w.write_all(&data)?;
                        w.flush()?;
                        Ok(())
                    }
                    None => Err(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "pty writer is closed",
                    )),
                }
            })
            .await
            .map_err(|e| std::io::Error::other(format!("blocking task: {e}")))??;
            return Ok(());
        }

        let mut guard = proc.stdin.lock().await;
        match guard.as_mut() {
            Some(stdin) => {
                stdin.write_all(data).await?;
                stdin.flush().await?;
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "process stdin is closed",
            )),
        }
    }

    /// Apply a window-size update to a PTY-backed process. No-op (with
    /// an error code) on non-PTY processes — callers should check
    /// `pty_backed` first.
    pub async fn resize_pty(&self, id: &str, size: PortablePtySize) -> std::io::Result<()> {
        let proc = self
            .get(id)
            .await
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such process"))?;
        let pty = proc.pty.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "process is not pty-backed",
            )
        })?;
        let master = pty.master.clone();
        tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            let guard = master.blocking_lock();
            if let Some(m) = guard.as_ref() {
                m.resize(size)
                    .map_err(|e| std::io::Error::other(format!("pty resize: {e}")))?;
            }
            Ok(())
        })
        .await
        .map_err(|e| std::io::Error::other(format!("blocking task: {e}")))?
    }

    /// Hard-kill via tokio. Used as the escalation step in Stop's
    /// grace timer; callers that want signal semantics call `signal`.
    pub async fn force_kill(&self, id: &str) -> std::io::Result<()> {
        let proc = self
            .get(id)
            .await
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such process"))?;
        let mut guard = proc.child.lock().await;
        if let Some(c) = guard.as_mut() {
            let _ = c.start_kill();
        }
        Ok(())
    }

    pub async fn signal(&self, id: &str, signum: i32) -> std::io::Result<()> {
        let proc = self
            .get(id)
            .await
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such process"))?;
        if let Some(pid) = proc.pid {
            // SAFETY: passing a valid pid + integer signal number to libc::kill;
            // failure surfaces through errno-via-io::Error::last_os_error.
            #[allow(unsafe_code)]
            let rc = unsafe { libc::kill(pid as libc::pid_t, signum) };
            if rc != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

fn pty_reader_loop(
    mut reader: Box<dyn std::io::Read + Send>,
    output: broadcast::Sender<PtyChunk>,
    process_id: String,
) {
    let mut buf = vec![0u8; READ_CHUNK];
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let mut chunk = PtyChunk::default();
        chunk.data = buf[..n].to_vec();
        if output.send(chunk).is_err() {
            // No subscribers right now — still keep draining the master
            // so the kernel doesn't block on a full pipe.
            continue;
        }
    }
    let _ = process_id; // reserved for future per-stream telemetry
}

async fn forward_stream<R>(reader: R, session: Arc<Session>, process_id: String, stream: u32)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut reader = reader;
    let mut buf = vec![0u8; READ_CHUNK];
    loop {
        let n = match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let mut out = ProcessOutput::default();
        out.process_id = process_id.clone();
        out.stream = stream;
        out.data = buf[..n].to_vec();
        let mut env = Envelope::default();
        env.sequence = session.next_sequence();
        env.payload = Some(envelope::Payload::ProcessOutput(Box::new(out)));
        let _ = session.events.send(env);
    }
}

#[cfg(unix)]
fn unix_signal_name(status: &std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(s) => format!("SIG{}", s),
        None => String::new(),
    }
}

#[cfg(not(unix))]
fn unix_signal_name(_: &std::process::ExitStatus) -> String {
    String::new()
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
