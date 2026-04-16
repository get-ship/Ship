use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub mod pidfile;
use pidfile::{PidfileRecord, PidfileRole};

pub enum DaemonCommand {
    Start { host: String, dev: bool },
    Stop,
    Status,
    Cleanup,
    Run { host: String, port: u16, dev: bool, with_ui: bool },
    /// Internal: supervisor loop that watches and restarts the daemon.
    Supervise { host: String, dev: bool },
}

const SHIPD_PORT: u16 = 9315;

fn ship_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine home directory")?;
    Ok(home.join(".ship"))
}

fn pid_file() -> Result<PathBuf> {
    Ok(ship_dir()?.join("daemon.pid"))
}

fn log_file() -> Result<PathBuf> {
    Ok(ship_dir()?.join("shipd.log"))
}

/// Read a PID from the pid file. Returns None if the file does not exist.
pub fn read_pid(pid_path: &std::path::Path) -> Result<Option<u32>> {
    if !pid_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(pid_path)
        .with_context(|| format!("cannot read {}", pid_path.display()))?;
    let pid: u32 = content
        .trim()
        .parse()
        .with_context(|| format!("invalid PID in {}", pid_path.display()))?;
    Ok(Some(pid))
}

/// Write a PID to the pid file.
pub fn write_pid(pid_path: &std::path::Path, pid: u32) -> Result<()> {
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory {}", parent.display()))?;
    }
    std::fs::write(pid_path, format!("{}\n", pid))
        .with_context(|| format!("cannot write {}", pid_path.display()))?;
    Ok(())
}

/// Check whether a process with the given PID is alive by sending signal 0.
fn is_process_alive(pid: u32) -> bool {
    let status = std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status();
    matches!(status, Ok(s) if s.success())
}

pub fn run_daemon(cmd: DaemonCommand) -> Result<()> {
    match cmd {
        DaemonCommand::Start { host, dev } => daemon_start(host, dev),
        DaemonCommand::Stop => daemon_stop(),
        DaemonCommand::Status => daemon_status(),
        DaemonCommand::Cleanup => daemon_cleanup(),
        DaemonCommand::Run { host, port, dev, with_ui } => daemon_run(host, port, dev, with_ui),
        DaemonCommand::Supervise { host, dev } => daemon_supervise(host, dev),
    }
}

/// Maximum restart attempts within the window before giving up.
const MAX_RESTARTS: u32 = 10;
/// Time window for counting restarts.
const RESTART_WINDOW: std::time::Duration = std::time::Duration::from_secs(600);
/// Initial backoff between restarts.
const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_secs(1);
/// Maximum backoff between restarts.
const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(60);

fn daemon_start(host: String, dev: bool) -> Result<()> {
    let pid_path = pid_file()?;
    let log_path = log_file()?;
    let global_dir = ship_dir()?;
    // Dev mode forces localhost bind regardless of --host.
    let host = if dev { "127.0.0.1".to_string() } else { host };

    // Check if already running. The pidfile lock is the source of truth;
    // a held lock means a live supervisor exists. Falling back to the legacy
    // pid file keeps callers honest if the lock-based pidfile is missing.
    if let Some((record, true)) = pidfile::read(&global_dir)? {
        println!(
            "shipd already running (pid {}, port {}, version {})",
            record.pid, record.port, record.version
        );
        return Ok(());
    }
    if let Some(pid) = read_pid(&pid_path)?
        && is_process_alive(pid)
    {
        println!("shipd already running (pid {}, port {})", pid, SHIPD_PORT);
        return Ok(());
    }
    // Stale legacy pid file — remove it before spawning.
    let _ = std::fs::remove_file(&pid_path);

    let self_exe = std::env::current_exe().context("cannot determine own executable path")?;

    // Ensure ~/.ship/ and crashes dir exist.
    std::fs::create_dir_all(&global_dir).context("cannot create ~/.ship/ directory")?;
    let crashes_dir = global_dir.join("crashes");
    std::fs::create_dir_all(&crashes_dir).context("cannot create ~/.ship/crashes/ directory")?;

    // Fork the supervisor into the background.
    // The supervisor process watches the daemon child and restarts on crash.
    let log_handle = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("cannot open log file {}", log_path.display()))?;

    let stdout = log_handle.try_clone().context("cannot clone log file handle")?;
    let stderr = log_handle;

    let mut sup_args: Vec<&str> = vec!["daemon", "supervise", "--host", &host];
    if dev {
        sup_args.push("--dev");
    }
    let supervisor = std::process::Command::new(&self_exe)
        .args(&sup_args)
        .stdout(stdout)
        .stderr(stderr)
        .stdin(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("cannot spawn supervisor: {}", self_exe.display()))?;

    let supervisor_pid = supervisor.id();
    write_pid(&pid_path, supervisor_pid)?;

    if dev {
        eprintln!("\x1b[1;33m⚠ SHIPD DEV MODE ENABLED — auth relaxed, localhost only. Not for production.\x1b[0m");
    }
    println!(
        "shipd supervisor started (pid {}, http://{}:{}, log {})",
        supervisor_pid, host, SHIPD_PORT, log_path.display()
    );
    Ok(())
}

/// The supervisor loop — spawns the daemon, watches it, restarts on crash.
///
/// Called via `ship daemon supervise`. Not meant to be called directly by users.
fn daemon_supervise(host: String, dev: bool) -> Result<()> {
    let self_exe = std::env::current_exe().context("cannot determine own executable path")?;
    let global_dir = ship_dir()?;
    let crashes_dir = global_dir.join("crashes");
    // Dev mode forces localhost bind.
    let host = if dev { "127.0.0.1".to_string() } else { host };

    // Acquire the OS-enforced singleton lock. Another live supervisor
    // holding the lock means we exit cleanly with its identity.
    let record = PidfileRecord {
        pid: std::process::id(),
        port: SHIPD_PORT,
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_sha: option_env!("VERGEN_GIT_SHA").unwrap_or("unknown").to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
        pidfile_path: String::new(),
        socket_path: String::new(),
        role: PidfileRole::Supervisor,
    };
    let _pidfile_guard = match pidfile::acquire(&global_dir, record)? {
        Some(guard) => guard,
        None => {
            if let Some((existing, _)) = pidfile::read(&global_dir)? {
                eprintln!(
                    "shipd already running (pid {}, port {}, version {})",
                    existing.pid, existing.port, existing.version
                );
            } else {
                eprintln!("shipd already running (pidfile lock held, no record readable)");
            }
            return Ok(());
        }
    };

    // Relay SIGTERM/SIGINT to the child, then exit cleanly.
    let child_pid: std::sync::Arc<std::sync::atomic::AtomicU32> =
        std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool> =
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Install signal handlers.
    #[cfg(unix)]
    {
        let cp = child_pid.clone();
        let sf = stop_flag.clone();
        unsafe {
            libc::signal(libc::SIGTERM, signal_handler as *const () as libc::sighandler_t);
            libc::signal(libc::SIGINT, signal_handler as *const () as libc::sighandler_t);
        }
        CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
        STOP_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
        // We use statics for signal handler since closures can't be signal handlers.
        let _ = (cp, sf); // Suppress unused warnings; we use the statics below.
    }

    let mut restart_count: u32 = 0;
    let mut window_start = std::time::Instant::now();
    let mut backoff = INITIAL_BACKOFF;

    loop {
        // Reset restart count if outside the window.
        if window_start.elapsed() > RESTART_WINDOW {
            restart_count = 0;
            window_start = std::time::Instant::now();
            backoff = INITIAL_BACKOFF;
        }

        if restart_count >= MAX_RESTARTS {
            eprintln!(
                "shipd supervisor: {} restarts in {}s — giving up",
                MAX_RESTARTS, RESTART_WINDOW.as_secs()
            );
            write_crash_report(&crashes_dir, "max restarts exceeded", None);
            break;
        }

        // Spawn the daemon child.
        let port_str = SHIPD_PORT.to_string();
        let mut child_args: Vec<&str> = vec!["daemon", "run", "--host", &host, "--port", &port_str];
        if dev {
            child_args.push("--dev");
        }
        let mut child = std::process::Command::new(&self_exe)
            .args(&child_args)
            .stdin(std::process::Stdio::null())
            .spawn()
            .with_context(|| "supervisor: cannot spawn daemon")?;

        let pid = child.id();
        eprintln!("shipd supervisor: daemon started (pid {pid})");

        #[cfg(unix)]
        CHILD_PID.store(pid, std::sync::atomic::Ordering::SeqCst);

        // Wait for child to exit.
        let status = child.wait().context("supervisor: failed to wait on child")?;

        #[cfg(unix)]
        CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);

        // Check if we were asked to stop.
        #[cfg(unix)]
        let should_stop = STOP_FLAG.load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(unix))]
        let should_stop = false;

        if should_stop {
            eprintln!("shipd supervisor: clean shutdown (child exit: {status})");
            break;
        }

        if status.success() {
            eprintln!("shipd supervisor: daemon exited cleanly");
            break;
        }

        // Crash detected.
        restart_count += 1;
        let reason = format_exit_status(&status);
        eprintln!(
            "shipd supervisor: daemon crashed ({reason}), restart {restart_count}/{MAX_RESTARTS} in {}s",
            backoff.as_secs()
        );
        write_crash_report(&crashes_dir, &reason, Some(pid));

        std::thread::sleep(backoff);
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }

    // Clean up PID file.
    let _ = std::fs::remove_file(pid_file().unwrap_or_default());
    Ok(())
}

fn format_exit_status(status: &std::process::ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal {signal}");
        }
    }
    match status.code() {
        Some(code) => format!("exit code {code}"),
        None => "unknown".to_string(),
    }
}

fn write_crash_report(crashes_dir: &Path, reason: &str, pid: Option<u32>) {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("crash-{timestamp}.json");
    let report = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "reason": reason,
        "pid": pid,
    });
    let path = crashes_dir.join(&filename);
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap_or_default()) {
        eprintln!("shipd supervisor: failed to write crash report: {e}");
    }
}

// Signal handler statics (needed because signal handlers can't capture closures).
#[cfg(unix)]
static CHILD_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[cfg(unix)]
static STOP_FLAG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn signal_handler(_sig: libc::c_int) {
    STOP_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
    let pid = CHILD_PID.load(std::sync::atomic::Ordering::SeqCst);
    if pid != 0 {
        unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM); }
    }
}

fn daemon_run(host: String, port: u16, dev: bool, with_ui: bool) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("cannot build tokio runtime")?;
    rt.block_on(shipd::run_network(host, port, dev, with_ui, None))
}

fn daemon_stop() -> Result<()> {
    let pid_path = pid_file()?;

    let pid = match read_pid(&pid_path)? {
        Some(p) => p,
        None => {
            println!("shipd not running (no PID file)");
            return Ok(());
        }
    };

    if !is_process_alive(pid) {
        println!("shipd not running (stale PID {})", pid);
        let _ = std::fs::remove_file(&pid_path);
        return Ok(());
    }

    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .context("cannot invoke `kill`")?;

    if !status.success() {
        anyhow::bail!("failed to send SIGTERM to pid {}", pid);
    }

    let _ = std::fs::remove_file(&pid_path);
    println!("shipd stopped (pid {})", pid);
    Ok(())
}

fn daemon_status() -> Result<()> {
    let global_dir = ship_dir()?;
    let pidfile_record = pidfile::read(&global_dir)?;
    let supervisor_pid = pidfile_record.as_ref().and_then(
        |(rec, lock_held)| if *lock_held { Some(rec.pid) } else { None },
    );

    // Try the live REST endpoint first — that's the source of truth for
    // real status. Fall back to the pidfile read if the daemon is
    // unreachable (no port file, stale process, etc).
    if let Some((port, token)) = daemon_info_if_healthy() {
        return print_live_status(port, &token, supervisor_pid);
    }

    match pidfile_record {
        None => println!("shipd not running"),
        Some((record, true)) => {
            // Lock held but REST endpoint unreachable — supervisor up,
            // daemon listener not ready (or wrong port).
            println!(
                "shipd supervisor running (pid {}, port {}, version {}) — daemon unreachable",
                record.pid, record.port, record.version
            );
        }
        Some((record, false)) => {
            println!(
                "shipd not running (stale pidfile from pid {})",
                record.pid
            );
        }
    }
    Ok(())
}

fn print_live_status(port: u16, token: &str, supervisor_pid: Option<u32>) -> Result<()> {
    let url = format!("http://127.0.0.1:{port}/api/runtime/daemon/status");
    let mut resp = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| anyhow::anyhow!("status request failed: {e}"))?;
    let value: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| anyhow::anyhow!("status: malformed response body: {e}"))?;

    if !value["ok"].as_bool().unwrap_or(false) {
        anyhow::bail!(
            "daemon returned error: {}",
            value["data"].as_str().unwrap_or("unknown")
        );
    }
    let d = &value["data"];
    let pid = d["pid"].as_u64().unwrap_or(0);
    let version = d["version"].as_str().unwrap_or("?");
    let uptime = format_uptime(d["uptime_secs"].as_u64().unwrap_or(0));
    let total = d["jobs_total"].as_u64().unwrap_or(0);
    let by_status = d["jobs_by_status"].as_object().cloned().unwrap_or_default();
    let by_status_summary: Vec<String> = by_status
        .iter()
        .filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n)))
        .filter(|(_, n)| *n > 0)
        .map(|(k, n)| format!("{n} {k}"))
        .collect();
    let last_sweep = d["last_sweep_at"]
        .as_str()
        .map(format_relative_time)
        .unwrap_or_else(|| "never".to_string());
    let drift = d["last_sweep_drift"].as_u64().unwrap_or(0);

    println!("shipd running");
    println!("  pid:            {pid}");
    if let Some(s) = supervisor_pid {
        if (s as u64) != pid {
            println!("  supervisor pid: {s}");
        }
    }
    println!("  version:        {version}");
    println!("  port:           {port}");
    println!("  uptime:         {uptime}");
    if by_status_summary.is_empty() {
        println!("  jobs:           {total} total");
    } else {
        println!(
            "  jobs:           {total} total ({})",
            by_status_summary.join(", ")
        );
    }
    println!("  last sweep:     {last_sweep}");
    println!("  drift detected: {drift}");
    Ok(())
}

/// Render seconds as `2h 14m`, `34m 12s`, or `47s` — keeps the longest
/// two units that still fit in 8 chars.
fn format_uptime(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    let s = secs % 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else if mins > 0 {
        format!("{mins}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// Render an RFC3339 timestamp as `8s ago`, `12m ago`, `2h ago`, etc.
fn format_relative_time(ts: &str) -> String {
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return ts.to_string();
    };
    let secs = (chrono::Utc::now() - dt.with_timezone(&chrono::Utc))
        .num_seconds()
        .max(0);
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

fn daemon_cleanup() -> Result<()> {
    use runtime::projections::job::{load_jobs, JobStatus};

    let jobs = load_jobs().context("failed to load job projection")?;

    let worktrees_base = {
        if let Ok(dir) = std::env::var("SHIP_WORKTREE_DIR") {
            PathBuf::from(dir)
        } else {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join("dev").join("ship-worktrees")
        }
    };

    let repo_dir = if let Ok(dir) = std::env::var("SHIP_REPO_DIR") {
        PathBuf::from(dir)
    } else {
        std::env::current_dir().context("cannot determine working directory")?
    };

    let orphans: Vec<_> = jobs
        .values()
        .filter(|j| matches!(j.status, JobStatus::Merged | JobStatus::Failed))
        .filter(|j| {
            let path = j
                .worktree
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| worktrees_base.join(&j.slug));
            path.exists()
        })
        .collect();

    if orphans.is_empty() {
        println!("No orphaned worktrees found.");
        return Ok(());
    }

    println!("Found {} orphaned worktree(s):", orphans.len());

    let mut cleaned = 0usize;
    let mut errors = 0usize;

    for job in &orphans {
        let worktree_path = job
            .worktree
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| worktrees_base.join(&job.slug));
        let branch = format!("job/{}", job.slug);
        let tmux_session = format!("job-{}", job.slug);

        println!("  {} — {}", job.slug, worktree_path.display());

        // Kill tmux session (best-effort).
        let _ = std::process::Command::new("tmux")
            .args(["kill-session", "-t", &tmux_session])
            .status();

        // Remove git worktree.
        let wt_str = match worktree_path.to_str() {
            Some(s) => s,
            None => {
                eprintln!("    error: worktree path is not valid UTF-8, skipping");
                errors += 1;
                continue;
            }
        };
        let remove_ok = std::process::Command::new("git")
            .args(["worktree", "remove", wt_str, "--force"])
            .current_dir(&repo_dir)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !remove_ok {
            eprintln!("    warning: worktree removal failed for {}", worktree_path.display());
            errors += 1;
            continue;
        }

        // Delete branch (best-effort — it may already be gone).
        let _ = std::process::Command::new("git")
            .args(["branch", "-D", &branch])
            .current_dir(&repo_dir)
            .status();

        cleaned += 1;
    }

    if errors > 0 {
        println!("Cleaned: {cleaned}, Errors: {errors}");
    } else {
        println!("Cleaned: {cleaned}");
    }
    Ok(())
}

/// Ensure the daemon is running, starting it if needed.
///
/// 1. Check if daemon.port file exists and the daemon responds to health check.
/// 2. If not, start the daemon and wait for it to become healthy.
/// 3. Returns `(port, token)` on success.
///
/// Callers should prefer this over manual daemon_info() checks.
pub fn ensure_daemon() -> Result<(u16, String)> {
    // Fast path: daemon already running and responsive.
    if let Some((port, token)) = daemon_info_if_healthy() {
        return Ok((port, token));
    }

    // Start the daemon.
    eprintln!("shipd: starting daemon...");
    daemon_start("0.0.0.0".to_string(), false)?;

    // Wait for the daemon to become healthy (up to 5 seconds).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some((port, token)) = daemon_info_if_healthy() {
            return Ok((port, token));
        }
    }

    anyhow::bail!("shipd: daemon started but did not become healthy within 5 seconds")
}

/// Try to read daemon info and verify it responds to /api/health.
fn daemon_info_if_healthy() -> Option<(u16, String)> {
    let home = dirs::home_dir()?.join(".ship");
    let port_str = std::fs::read_to_string(home.join("daemon.port")).ok()?;
    let port = port_str.trim().parse::<u16>().ok()?;
    let token = std::fs::read_to_string(home.join("daemon.token")).ok()?;
    let token = token.trim().to_string();

    // Quick health probe — 1 second timeout.
    let url = format!("http://127.0.0.1:{port}/api/health");
    let resp = reqwest::blocking::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(1))
        .send()
        .ok()?;

    if resp.status().is_success() {
        Some((port, token))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "daemon_tests.rs"]
mod tests;
