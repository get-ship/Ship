//! Single-session sidecar state.
//!
//! One sidecar process owns one session. Within that session it can
//! spawn N execs (subprocess children) and stream their stdio out as
//! events. Closing the session terminates all execs and tears down.

use base64::Engine;
use ship_isolation::sidecar::protocol::SidecarEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, broadcast};

const EVENT_CHANNEL_CAP: usize = 1024;

pub type EventTx = broadcast::Sender<SidecarEvent>;

#[allow(dead_code)]
pub struct ExecState {
    pub exec_id: String,
    pub pid: Option<u32>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    child: Arc<Mutex<Child>>,
}

impl ExecState {
    pub async fn write_stdin(&self, data: &[u8]) -> std::io::Result<()> {
        if let Some(stdin) = &self.stdin {
            let mut guard = stdin.lock().await;
            guard.write_all(data).await?;
            guard.flush().await?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "exec has no stdin",
            ))
        }
    }

    pub async fn kill_with(&self, signal: u8) -> std::io::Result<()> {
        let mut guard = self.child.lock().await;
        if let Some(pid) = guard.id() {
            unsafe {
                libc::kill(pid as libc::pid_t, signal as i32);
            }
        }
        // tokio's kill is SIGKILL only — we use it as the floor when
        // the caller didn't ask for SIGTERM
        if signal == 9 {
            let _ = guard.kill().await;
        }
        Ok(())
    }
}

pub struct Session {
    pub label: String,
    pub events: EventTx,
    pub execs: HashMap<String, Arc<ExecState>>,
}

impl Session {
    pub fn new(label: String) -> Self {
        let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAP);
        Self {
            label,
            events: tx,
            execs: HashMap::new(),
        }
    }
}

pub type SharedSession = Arc<Mutex<Session>>;

pub fn new_shared(label: impl Into<String>) -> SharedSession {
    Arc::new(Mutex::new(Session::new(label.into())))
}

pub async fn spawn_exec(
    sess: &SharedSession,
    bin: &str,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: &str,
) -> std::io::Result<(String, Option<u32>)> {
    let exec_id = format!("exe_{}", nanoid::nanoid!(10));

    let mut cmd = Command::new(bin);
    cmd.args(&args)
        .envs(&env)
        .current_dir(cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd.spawn()?;
    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take().map(|s| Arc::new(Mutex::new(s)));

    let child = Arc::new(Mutex::new(child));
    let state = Arc::new(ExecState {
        exec_id: exec_id.clone(),
        pid,
        stdin,
        child: child.clone(),
    });

    let tx = {
        let mut s = sess.lock().await;
        s.execs.insert(exec_id.clone(), state);
        s.events.clone()
    };

    if let Some(out) = stdout {
        let tx = tx.clone();
        let id = exec_id.clone();
        tokio::spawn(stream_lines(out, tx, id, true));
    }
    if let Some(err) = stderr {
        let tx = tx.clone();
        let id = exec_id.clone();
        tokio::spawn(stream_lines(err, tx, id, false));
    }

    let watcher_tx = tx.clone();
    let watcher_id = exec_id.clone();
    let watcher_child = child.clone();
    tokio::spawn(async move {
        let status = {
            let mut guard = watcher_child.lock().await;
            guard.wait().await
        };
        let (code, signal) = match status {
            Ok(s) => (s.code(), unix_signal_of(&s)),
            Err(_) => (None, None),
        };
        let _ = watcher_tx.send(SidecarEvent::Exited {
            exec_id: watcher_id,
            code,
            signal,
        });
    });

    Ok((exec_id, pid))
}

#[cfg(unix)]
fn unix_signal_of(s: &std::process::ExitStatus) -> Option<u8> {
    use std::os::unix::process::ExitStatusExt;
    s.signal().and_then(|x| u8::try_from(x).ok())
}

#[cfg(not(unix))]
fn unix_signal_of(_: &std::process::ExitStatus) -> Option<u8> {
    None
}

async fn stream_lines<R>(reader: R, tx: EventTx, exec_id: String, is_stdout: bool)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut buf = vec![0u8; 4096];
    let mut reader = BufReader::new(reader);
    loop {
        let n = match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let data_b64 = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
        let ev = if is_stdout {
            SidecarEvent::Stdout {
                exec_id: exec_id.clone(),
                data_b64,
            }
        } else {
            SidecarEvent::Stderr {
                exec_id: exec_id.clone(),
                data_b64,
            }
        };
        let _ = tx.send(ev);
    }
}
