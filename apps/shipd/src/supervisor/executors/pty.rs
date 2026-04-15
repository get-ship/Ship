//! PtyExecutor — spawns pipeline steps in tmux PTY windows.
//!
//! This is the default executor for `ship run`. A pipeline run gets one
//! tmux session named `ship-<short>` (8-char hash of the slug/run id).
//! Each step becomes a window named after its step id, running the
//! provider command teed to `.artifacts/pipeline/<NN>-<step>.log`.
//!
//! Users attach with `tmux attach -t ship-<short>` to watch the agent
//! work in real time. The session outlives the run so retrospection is
//! possible; a periodic sweep (not this module's concern) cleans up
//! sessions older than a TTL.
//!
//! Fallback behavior:
//!   - Windows: `PtyExecutor::new()` returns Err — dispatch falls back
//!     to ProcessExecutor.
//!   - tmux binary missing: same — Err at construction.
//!   - Per-job `headless` flag set: caller routes to ProcessExecutor.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use runtime::services::dispatch_ports::{
    ExecutorHandle, ExitSignal, JobContext, JobExecutor,
};
use runtime::services::provider_adapter::adapter_for;

/// Job executor that runs each pipeline step inside a tmux window.
pub(crate) struct PtyExecutor;

/// State stored on the ExecutorHandle.
struct PtyState {
    session: String,
    window: String,
}

impl PtyExecutor {
    /// Construct a PtyExecutor. Returns Err when tmux is not usable
    /// (Windows host, binary missing, or non-zero version probe).
    pub fn new() -> Result<Self> {
        if cfg!(windows) {
            return Err(anyhow!("PtyExecutor is unavailable on Windows"));
        }
        if !tmux_available() {
            return Err(anyhow!("tmux binary not found on PATH"));
        }
        Ok(Self)
    }
}

/// Short tmux session name derived from a run slug. tmux disallows `.`
/// and `:` in names — we sanitize to `_` and take 8 chars for a name the
/// user can reasonably type.
pub(crate) fn short_session_name(slug: &str) -> String {
    let sanitized: String = slug
        .chars()
        .map(|c| if c == '.' || c == ':' { '_' } else { c })
        .collect();
    let short: String = sanitized.chars().take(8).collect();
    format!("ship-{short}")
}

/// Log file path (relative to the worktree) for a given step.
/// Convention: `.artifacts/pipeline/NN-<step>.log`.
pub(crate) fn step_log_rel(index: usize, step_id: &str) -> String {
    format!(".artifacts/pipeline/{:02}-{}.log", index + 1, step_id)
}

/// Is the tmux binary on PATH and responsive to `-V`?
pub(crate) fn tmux_available() -> bool {
    std::process::Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Does a tmux session by this name already exist?
fn session_exists(name: &str) -> bool {
    std::process::Command::new("tmux")
        .args(["has-session", "-t", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[async_trait]
impl JobExecutor for PtyExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let session = short_session_name(&ctx.slug);
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

        // Create the session lazily (first step) or a new window (subsequent steps).
        if !session_exists(&session) {
            let status = std::process::Command::new("tmux")
                .args([
                    "new-session", "-d",
                    "-s", &session,
                    "-n", &step_id,
                    "-c", &work_dir,
                ])
                .status()
                .map_err(|e| anyhow!("tmux new-session failed: {e}"))?;
            if !status.success() {
                return Err(anyhow!("tmux new-session returned non-zero"));
            }
        } else {
            // New window for this step. `-d` = don't switch focus.
            let status = std::process::Command::new("tmux")
                .args([
                    "new-window", "-d",
                    "-t", &session,
                    "-n", &step_id,
                    "-c", &work_dir,
                ])
                .status()
                .map_err(|e| anyhow!("tmux new-window failed: {e}"))?;
            if !status.success() {
                return Err(anyhow!("tmux new-window returned non-zero"));
            }
        }

        let window_target = format!("{session}:{step_id}");
        let adapter = adapter_for(ctx.launch.provider.as_deref());
        let cmd = adapter.build_headless(&ctx.slug, &ctx.launch);
        let shell_cmd = cmd.to_shell_string();
        let log_rel = step_log_rel(step_idx, &step_id);
        let teed = format!("{shell_cmd} 2>&1 | tee -a {log_rel}");

        let status = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &window_target, &teed, "Enter"])
            .status()
            .map_err(|e| anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow!("tmux send-keys returned non-zero"));
        }

        tracing::info!(
            slug = ctx.slug, session, window = step_id,
            log = log_rel, "pty-executor: step launched"
        );

        // Monitor the window: once the command completes, the pane closes
        // (remain-on-exit off) but tmux may keep the window if other panes
        // exist. We poll `list-windows` for the named window.
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        let session_mon = session.clone();
        let window_mon = step_id.clone();
        std::thread::spawn(move || {
            monitor_window(&session_mon, &window_mon, &slug, &job_id, exit_tx);
        });

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(PtyState { session, window: step_id }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<PtyState>() {
            Some(s) => s,
            None => return false,
        };
        window_exists(&state.session, &state.window)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<PtyState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let target = format!("{}:{}", state.session, state.window);
        let _ = std::process::Command::new("tmux")
            .args(["kill-window", "-t", &target])
            .status();
        Ok(())
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let state = match handle.inner.downcast_ref::<PtyState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let target = format!("{}:{}", state.session, state.window);
        let status = std::process::Command::new("tmux")
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
        _provider: Option<&str>,
        current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        let session = short_session_name(slug);
        // Prefer the active step's window; fall back to the slug-named
        // window used for pre-pipeline single-step jobs.
        let window = current_step.map(str::to_string).unwrap_or_else(|| slug.to_string());
        if !window_exists(&session, &window) {
            return None;
        }
        Some(ExecutorHandle {
            pid: None,
            inner: Box::new(PtyState { session, window }),
        })
    }
}

fn window_exists(session: &str, window: &str) -> bool {
    let Ok(out) = std::process::Command::new("tmux")
        .args(["list-windows", "-t", session, "-F", "#{window_name}"])
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l == window)
}

fn monitor_window(
    session: &str,
    window: &str,
    slug: &str,
    job_id: &str,
    exit_tx: Option<tokio::sync::mpsc::UnboundedSender<ExitSignal>>,
) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        if !window_exists(session, window) {
            tracing::info!(session, window, slug, "pty-executor: window closed, signalling exit");
            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal {
                    slug: slug.to_string(),
                    job_id: job_id.to_string(),
                    exit_code: None,
                });
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_session_name_sanitizes_dots_and_colons() {
        let name = short_session_name("ab.cd:ef12345");
        assert_eq!(name, "ship-ab_cd_ef");
        assert!(!name.contains('.'));
        assert!(!name.contains(':'));
    }

    #[test]
    fn short_session_name_is_stable_for_known_slug() {
        assert_eq!(short_session_name("pipeline-abcdef12"), "ship-pipeline");
    }

    #[test]
    fn step_log_rel_is_zero_padded() {
        assert_eq!(
            step_log_rel(0, "test-writer"),
            ".artifacts/pipeline/01-test-writer.log"
        );
        assert_eq!(
            step_log_rel(9, "impl"),
            ".artifacts/pipeline/10-impl.log"
        );
    }

    #[test]
    fn pty_new_fails_when_tmux_absent_or_windows() {
        // On Windows, always fails. Elsewhere, depends on host.
        let result = PtyExecutor::new();
        if cfg!(windows) {
            assert!(result.is_err(), "PtyExecutor must fail on Windows");
        } else if !tmux_available() {
            assert!(result.is_err(), "must fail when tmux is absent");
        } else {
            assert!(result.is_ok(), "must succeed when tmux is present");
        }
    }

    #[tokio::test]
    async fn pty_spawn_creates_session_and_window() {
        if !tmux_available() {
            eprintln!("skipping: tmux not available");
            return;
        }
        use runtime::services::dispatch_ports::{LaunchProfile, StepInfo};
        let tmp = tempfile::tempdir().unwrap();
        let slug = format!("ptytest-{}", &runtime::gen_ulid()[..8].to_lowercase());
        let ctx = JobContext {
            job_id: "j1".into(),
            slug: slug.clone(),
            agent: "noop".into(),
            branch: "job/test".into(),
            spec_path: "spec.md".into(),
            spec_content: None,
            work_dir: tmp.path().to_path_buf(),
            env: Default::default(),
            exit_tx: None,
            launch: LaunchProfile {
                provider: Some("echo-only".into()),
                ..Default::default()
            },
            step: Some(StepInfo { index: 0, id: "step-a".into() }),
            headless: false,
        };
        let exec = PtyExecutor::new().unwrap();
        let handle = exec.spawn(&ctx).await.expect("spawn");
        let session = short_session_name(&slug);
        assert!(session_exists(&session), "session should exist after spawn");
        assert!(window_exists(&session, "step-a"), "window should exist");
        // Cleanup.
        let _ = std::process::Command::new("tmux")
            .args(["kill-session", "-t", &session])
            .status();
        drop(handle);
    }
}
