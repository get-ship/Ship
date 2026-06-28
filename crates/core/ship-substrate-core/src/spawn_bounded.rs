//! Bounded subprocess spawn helper for the daemon.
//!
//! Mirrors `scripts/spawn-bounded.mjs` for the test/build path: any
//! subprocess the daemon spawns must die cleanly under a virtual-memory
//! and wall-clock cap so a runaway compile never takes down the host.
//! A buggy `.sui` input once drove sui-compiler to 19+GB RSS before the
//! WSL VM died; sui-compiler now self-binds via setrlimit + watchdog,
//! and external callers — including this daemon — wrap their spawns
//! through the same defense-in-depth.
//!
//! Defaults: 2 GB virtual memory, 30 s wall clock — same as the JS helper.
//! Per-call sites override the timeout when their workload legitimately
//! needs longer (e.g. agent-config compilation).
//!
//! On Unix the memory cap is enforced via `pre_exec` + `libc::setrlimit`
//! on `RLIMIT_AS`; the timeout is enforced by polling `try_wait` and
//! sending `SIGKILL` to the child PID once the deadline passes (sync) or
//! via `tokio::time::timeout` for async callers. On non-Unix platforms
//! the memory cap is a no-op and only the timeout applies.

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::{CommandExt, ExitStatusExt};

pub const DEFAULT_MEM_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum BoundedError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("timed out after {}s", .0.as_secs())]
    TimedOut(Duration),
    #[error("exceeded {0}MB virtual memory cap")]
    OomCapped(u64),
}

/// Builder for a bounded subprocess. Mirrors the small subset of
/// `std::process::Command` that the daemon's spawn sites actually use:
/// args, current_dir, env, and stdio overrides. Memory and timeout
/// caps are part of the builder so callers express the full policy at
/// the spawn site.
pub struct Cmd {
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    envs: Vec<(OsString, OsString)>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    mem_bytes: u64,
    timeout: Duration,
}

impl Cmd {
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            cwd: None,
            envs: Vec::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            mem_bytes: DEFAULT_MEM_BYTES,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn arg(mut self, a: impl AsRef<OsStr>) -> Self {
        self.args.push(a.as_ref().to_owned());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.args.push(a.as_ref().to_owned());
        }
        self
    }

    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
    }

    pub fn env(mut self, k: impl AsRef<OsStr>, v: impl AsRef<OsStr>) -> Self {
        self.envs.push((k.as_ref().to_owned(), v.as_ref().to_owned()));
        self
    }

    pub fn stdin(mut self, s: Stdio) -> Self {
        self.stdin = Some(s);
        self
    }

    pub fn stdout(mut self, s: Stdio) -> Self {
        self.stdout = Some(s);
        self
    }

    pub fn stderr(mut self, s: Stdio) -> Self {
        self.stderr = Some(s);
        self
    }

    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    pub fn timeout_secs(self, s: u64) -> Self {
        self.timeout(Duration::from_secs(s))
    }

    pub fn mem_bytes(mut self, b: u64) -> Self {
        self.mem_bytes = b;
        self
    }

    fn build_consuming(mut self, stdout_default: Stdio, stderr_default: Stdio) -> Command {
        let mut c = Command::new(&self.program);
        c.args(&self.args);
        if let Some(cwd) = &self.cwd {
            c.current_dir(cwd);
        }
        for (k, v) in &self.envs {
            c.env(k, v);
        }
        c.stdin(self.stdin.take().unwrap_or(Stdio::null()));
        c.stdout(self.stdout.take().unwrap_or(stdout_default));
        c.stderr(self.stderr.take().unwrap_or(stderr_default));
        #[cfg(unix)]
        apply_mem_cap(&mut c, self.mem_bytes);
        c
    }

    /// Run to completion, capturing stdout + stderr. Killed with SIGKILL on
    /// timeout; surfaces SIGKILL exits as `OomCapped` since the only way a
    /// child gets SIGKILLed under our policy is the rlimit kick or our own
    /// timeout fallback.
    pub fn output(self) -> Result<Output, BoundedError> {
        let timeout = self.timeout;
        let mem_bytes = self.mem_bytes;
        let cmd = self.build_consuming(Stdio::piped(), Stdio::piped());
        run_to_output(cmd, timeout, mem_bytes)
    }

    /// Run to completion, returning only the exit status. Stdout/stderr
    /// inherit the daemon's by default — overrideable via `.stdout/.stderr`.
    pub fn status(self) -> Result<ExitStatus, BoundedError> {
        let timeout = self.timeout;
        let mem_bytes = self.mem_bytes;
        let cmd = self.build_consuming(Stdio::inherit(), Stdio::inherit());
        run_to_status(cmd, timeout, mem_bytes)
    }

    /// Async variant of `output` — uses `tokio::time::timeout` around
    /// `wait_with_output`. Same memory cap policy.
    pub async fn output_async(self) -> Result<Output, BoundedError> {
        let timeout = self.timeout;
        let mem_bytes = self.mem_bytes;
        let cmd = self.build_consuming(Stdio::piped(), Stdio::piped());
        run_to_output_async(cmd, timeout, mem_bytes).await
    }
}

#[cfg(unix)]
fn apply_mem_cap(cmd: &mut Command, mem_bytes: u64) {
    // SAFETY: pre_exec runs in the post-fork pre-exec child. We only call
    // `setrlimit`, which is async-signal-safe per POSIX.
    unsafe {
        cmd.pre_exec(move || {
            let rlim = libc::rlimit {
                rlim_cur: mem_bytes as libc::rlim_t,
                rlim_max: mem_bytes as libc::rlim_t,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &rlim) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

fn run_to_status(
    mut cmd: Command,
    timeout: Duration,
    mem_bytes: u64,
) -> Result<ExitStatus, BoundedError> {
    let mut child = cmd.spawn()?;
    let pid = child.id();
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if Instant::now() >= deadline {
            kill_pid(pid);
            let _ = child.wait();
            return Err(BoundedError::TimedOut(timeout));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    classify_status(status, mem_bytes)
}

fn run_to_output(
    mut cmd: Command,
    timeout: Duration,
    mem_bytes: u64,
) -> Result<Output, BoundedError> {
    let mut child = cmd.spawn()?;
    let pid = child.id();
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if Instant::now() >= deadline {
            kill_pid(pid);
            let _ = child.wait();
            return Err(BoundedError::TimedOut(timeout));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    use std::io::Read;
    if let Some(mut s) = child.stdout.take() {
        let _ = s.read_to_end(&mut stdout);
    }
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_end(&mut stderr);
    }
    let status = classify_status(status, mem_bytes)?;
    Ok(Output { status, stdout, stderr })
}

async fn run_to_output_async(
    cmd: Command,
    timeout: Duration,
    mem_bytes: u64,
) -> Result<Output, BoundedError> {
    let mut tcmd = tokio::process::Command::from(cmd);
    tcmd.kill_on_drop(true);
    let child = tcmd.spawn()?;
    let pid = child.id();
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(out)) => {
            // Re-route through classify so SIGKILL maps to OomCapped consistently.
            let status = classify_status(out.status, mem_bytes)?;
            Ok(Output { status, stdout: out.stdout, stderr: out.stderr })
        }
        Ok(Err(e)) => Err(BoundedError::Io(e)),
        Err(_) => {
            if let Some(p) = pid {
                kill_pid(p);
            }
            Err(BoundedError::TimedOut(timeout))
        }
    }
}

#[cfg(unix)]
fn kill_pid(pid: u32) {
    // SAFETY: best-effort kill of a child we just spawned. A racing
    // exit + pid reuse is theoretically possible but vanishingly
    // unlikely in the ms between try_wait + kill on Linux.
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn kill_pid(_pid: u32) {
    // Non-Unix: rely on Drop semantics; explicit kill not portable here.
}

#[cfg(unix)]
fn classify_status(status: ExitStatus, mem_bytes: u64) -> Result<ExitStatus, BoundedError> {
    if status.signal() == Some(libc::SIGKILL) {
        return Err(BoundedError::OomCapped(mem_bytes / (1024 * 1024)));
    }
    Ok(status)
}

#[cfg(not(unix))]
fn classify_status(status: ExitStatus, _mem_bytes: u64) -> Result<ExitStatus, BoundedError> {
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(not(unix), ignore)]
    fn output_succeeds_for_quick_command() {
        let out = Cmd::new("sh")
            .args(["-c", "printf hello"])
            .timeout_secs(5)
            .output()
            .expect("ok");
        assert!(out.status.success());
        assert_eq!(out.stdout, b"hello");
    }

    #[test]
    #[cfg_attr(not(unix), ignore)]
    fn timeout_kills_runaway_subprocess() {
        let start = Instant::now();
        let err = Cmd::new("sh")
            .args(["-c", "sleep 30"])
            .timeout(Duration::from_millis(300))
            .output()
            .expect_err("must time out");
        let elapsed = start.elapsed();
        assert!(matches!(err, BoundedError::TimedOut(_)), "got {err:?}");
        // Cleanup must happen well within a second of the deadline.
        assert!(
            elapsed < Duration::from_secs(2),
            "wrapper failed to kill child quickly: {elapsed:?}"
        );
    }

    #[tokio::test]
    #[cfg_attr(not(unix), ignore)]
    async fn timeout_kills_runaway_subprocess_async() {
        let start = Instant::now();
        let err = Cmd::new("sh")
            .args(["-c", "sleep 30"])
            .timeout(Duration::from_millis(300))
            .output_async()
            .await
            .expect_err("must time out");
        let elapsed = start.elapsed();
        assert!(matches!(err, BoundedError::TimedOut(_)), "got {err:?}");
        assert!(
            elapsed < Duration::from_secs(2),
            "wrapper failed to kill child quickly: {elapsed:?}"
        );
    }

    #[test]
    #[cfg_attr(not(unix), ignore)]
    fn memory_cap_kills_oversize_allocation() {
        // Ask perl to allocate ~3GB; capped at 64MB virt by the wrapper.
        // The allocator returns NULL → perl dies; or the kernel kills with
        // SIGKILL once an mmap blows past the limit. Either way the wrapper
        // must surface a non-success exit.
        // Some build environments lack perl; skip in that case rather than fail.
        if which_program("perl").is_none() {
            eprintln!("perl missing — skipping memory_cap test");
            return;
        }
        let out = Cmd::new("perl")
            .args(["-e", "my $s = 'a' x (3 * 1024 * 1024 * 1024); print length($s);"])
            .mem_bytes(64 * 1024 * 1024)
            .timeout_secs(10)
            .output();
        match out {
            Ok(out) => assert!(
                !out.status.success(),
                "perl 3GB allocation should fail under 64MB cap"
            ),
            Err(BoundedError::OomCapped(_)) => {}
            Err(BoundedError::TimedOut(_)) => {
                panic!("memory cap test timed out instead of OOM");
            }
            Err(BoundedError::Io(e)) => panic!("io error: {e}"),
        }
    }

    fn which_program(prog: &str) -> Option<PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(prog);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}
