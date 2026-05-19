//! TmuxWindowExecutor — executes pipeline steps as tmux windows inside a
//! shared per-slug session. Output captured via `tee` into
//! `.artifacts/pipeline/NN-<step>.log`. Use for interactive-attach
//! pipelines. Not a PTY.
//!
//! A pipeline run gets one tmux session named `ship-<slug>` (full slug,
//! sanitized for tmux-forbidden chars). Each step becomes a window whose
//! internal id we track by tmux's `@N` window id (not by name) so that
//! send-keys can never collide with leftover windows of the same name
//! from prior runs. The session outlives the run so retrospection is
//! possible; a periodic sweep (not this module's concern) handles TTL.
//!
//! Fallback: `TmuxWindowExecutor::new()` returns Err on Windows or when
//! tmux is absent, and dispatch falls back to ProcessExecutor.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ship_platform::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, JobContext, JobExecutor,
};
use ship_platform::services::harness_adapter::adapter_for_or_default;

/// Job executor that runs each pipeline step inside a tmux window.
pub(crate) struct TmuxWindowExecutor;

/// State stored on the ExecutorHandle.
///
/// `window` is a tmux window id (`@N`) — NOT a name. Targeting by id is
/// unambiguous even if other windows in the session share a name.
struct TmuxWindowState {
    session: String,
    window: String,
}

impl TmuxWindowExecutor {
    /// Construct a TmuxWindowExecutor. Returns Err when tmux is not usable.
    pub fn new() -> Result<Self> {
        if cfg!(windows) {
            return Err(anyhow!("TmuxWindowExecutor is unavailable on Windows"));
        }
        if !tmux_available() {
            return Err(anyhow!("tmux binary not found on PATH"));
        }
        Ok(Self)
    }
}

/// tmux session name derived from a run slug. tmux disallows `.` and `:`
/// in session names — sanitize to `_`. We use the full slug (no truncation)
/// so concurrent jobs with similar-looking slug prefixes don't collide.
pub(crate) fn session_name(slug: &str) -> String {
    let sanitized: String = slug
        .chars()
        .map(|c| if c == '.' || c == ':' { '_' } else { c })
        .collect();
    format!("ship-{sanitized}")
}

/// Log file path (relative to the worktree) for a given step.
/// Convention: `.artifacts/pipeline/NN-<step>.log`.
pub(crate) fn step_log_rel(index: usize, step_id: &str) -> String {
    format!(".artifacts/pipeline/{:02}-{}.log", index + 1, step_id)
}

/// Is the tmux binary on PATH and responsive to `-V`?
///
/// One retry on failure: under heavy parallel subprocess load
/// (workspace-wide test runs, dispatching many agents in parallel)
/// the bare `tmux -V` occasionally fails to fork / start and returns
/// non-success. This is a transient OS-level resource hiccup, not a
/// real "tmux is unavailable" signal. We retry once with a small
/// pause; if the second attempt also fails, tmux truly isn't usable.
pub(crate) fn tmux_available() -> bool {
    fn check() -> bool {
        ship_substrate::spawn_bounded::Cmd::new("tmux")
            .arg("-V")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    if check() {
        return true;
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    check()
}

/// Does a tmux session by this name already exist?
fn session_exists(name: &str) -> bool {
    ship_substrate::spawn_bounded::Cmd::new("tmux")
        .args(["has-session", "-t", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Create the session (or a new window in an existing session) for this
/// step and return the tmux window id (`@N`) of the newly created window.
///
/// We use `-P -F '#{window_id}'` to capture the id on stdout. This id is
/// globally unique within a tmux server — the safe target for subsequent
/// send-keys, list-windows, and kill-window calls.
fn create_window(session: &str, step_id: &str, work_dir: &str) -> Result<String> {
    let (args, label): (&[&str], &str) = if !session_exists(session) {
        (
            &["new-session", "-d", "-P", "-F", "#{window_id}",
              "-s", session, "-n", step_id, "-c", work_dir],
            "tmux new-session",
        )
    } else {
        (
            &["new-window", "-d", "-P", "-F", "#{window_id}",
              "-t", session, "-n", step_id, "-c", work_dir],
            "tmux new-window",
        )
    };
    let out = ship_substrate::spawn_bounded::Cmd::new("tmux")
        .args(args)
        .output()
        .map_err(|e| anyhow!("{label} failed: {e}"))?;
    if !out.status.success() {
        return Err(anyhow!("{label} returned non-zero"));
    }
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !id.starts_with('@') {
        return Err(anyhow!("{label} did not print a window id (got {id:?})"));
    }
    Ok(id)
}

/// True if `window_id` (e.g. `@29`) is still listed in `session`.
fn window_id_exists(session: &str, window_id: &str) -> bool {
    let Ok(out) = ship_substrate::spawn_bounded::Cmd::new("tmux")
        .args(["list-windows", "-t", session, "-F", "#{window_id}"])
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l == window_id)
}

/// Find a window id by name in the session. Returns `None` if there are
/// zero or multiple matches — on rehydration we would rather mark the job
/// failed than risk attaching to the wrong window.
fn find_unique_window_by_name(session: &str, name: &str) -> Option<String> {
    let out = ship_substrate::spawn_bounded::Cmd::new("tmux")
        .args(["list-windows", "-t", session,
               "-F", "#{window_id} #{window_name}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut matches = text.lines().filter_map(|l| {
        let (id, n) = l.split_once(' ')?;
        if n == name { Some(id.to_string()) } else { None }
    });
    let first = matches.next()?;
    if matches.next().is_some() {
        tracing::warn!(session, name,
            "tmux-window-executor: multiple windows share this name — refusing to rehydrate");
        return None;
    }
    Some(first)
}

#[async_trait]
impl JobExecutor for TmuxWindowExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let session = session_name(&ctx.slug);
        let (step_idx, step_id) = ctx
            .step
            .as_ref()
            .map(|s| (s.index, s.id.clone()))
            .unwrap_or((0, ctx.slug.clone()));

        let work_dir = ctx
            .work_dir
            .to_str()
            .ok_or_else(|| anyhow!("worktree path is not valid UTF-8"))?
            .to_string();

        // Ensure the pipeline log dir exists so `tee -a` doesn't race.
        let log_dir = ctx.work_dir.join(".artifacts/pipeline");
        std::fs::create_dir_all(&log_dir)?;

        let window_id = create_window(&session, &step_id, &work_dir)?;
        let window_target = format!("{session}:{window_id}");

        let adapter = adapter_for_or_default(ctx.launch.harness.as_deref())
            .map_err(|e| anyhow!("tmux-window executor: {e}"))?;
        let cmd = if let Some(ref sid) = ctx.launch.resume_session {
            adapter
                .build_resume_args(&ctx.slug, sid, &ctx.launch)
                .map_err(|e| anyhow!("tmux-window: build_resume_args failed: {e}"))?
        } else {
            adapter.build_headless(&ctx.slug, &ctx.launch)
        };
        let shell_cmd = cmd.to_shell_string();
        let log_rel = step_log_rel(step_idx, &step_id);
        let teed = format!("{shell_cmd} 2>&1 | tee -a {log_rel}");

        let status = ship_substrate::spawn_bounded::Cmd::new("tmux")
            .args(["send-keys", "-t", &window_target, &teed, "Enter"])
            .status()
            .map_err(|e| anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow!("tmux send-keys returned non-zero"));
        }

        tracing::info!(
            slug = ctx.slug, session, window = window_id,
            window_name = step_id, log = log_rel,
            "tmux-window-executor: step launched"
        );

        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        let session_mon = session.clone();
        let window_mon = window_id.clone();
        std::thread::spawn(move || {
            monitor_window(&session_mon, &window_mon, &slug, &job_id, exit_tx);
        });

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(TmuxWindowState { session, window: window_id }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<TmuxWindowState>() {
            Some(s) => s,
            None => return false,
        };
        window_id_exists(&state.session, &state.window)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxWindowState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let target = format!("{}:{}", state.session, state.window);
        let _ = ship_substrate::spawn_bounded::Cmd::new("tmux")
            .args(["kill-window", "-t", &target])
            .status();
        Ok(())
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxWindowState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let target = format!("{}:{}", state.session, state.window);
        let status = ship_substrate::spawn_bounded::Cmd::new("tmux")
            .args(["send-keys", "-t", &target, message, "Enter"])
            .status()
            .map_err(|e| anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow!("tmux send-keys returned non-zero"));
        }
        Ok(())
    }

    async fn resume_handle(
        &self,
        slug: &str,
        _harness: Option<&str>,
        current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        let session = session_name(slug);
        // After a restart the in-memory window id is gone; look it up by
        // name. Prefer the active step's window; fall back to the slug-
        // named window used for pre-pipeline single-step jobs.
        let name = current_step.map(str::to_string).unwrap_or_else(|| slug.to_string());
        let window = find_unique_window_by_name(&session, &name)?;
        Some(ExecutorHandle {
            pid: None,
            inner: Box::new(TmuxWindowState { session, window }),
        })
    }
}

fn monitor_window(
    session: &str,
    window_id: &str,
    slug: &str,
    job_id: &str,
    exit_tx: Option<tokio::sync::mpsc::UnboundedSender<ExitSignal>>,
) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        if !window_id_exists(session, window_id) {
            tracing::info!(session, window = window_id, slug,
                "tmux-window-executor: window closed, signalling exit");
            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal {
                    slug: slug.to_string(),
                    job_id: job_id.to_string(),
                    exit_code: None,
                    cause: ExitCause::SessionDestroyed,
                });
            }
            return;
        }
    }
}

#[cfg(test)]
#[path = "tmux_window_tests.rs"]
mod tests;
