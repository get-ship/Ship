//! Concrete adapters for job isolation and execution.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! `TmuxExecutor` — spawns agents in tmux sessions, launches terminal tabs.

use anyhow::Result;
use async_trait::async_trait;
use runtime::events::job::PipelinePhase;
use runtime::services::dispatch_ports::{ExecutorHandle, IsolationStrategy, JobContext, JobExecutor};
use std::path::PathBuf;

use super::helpers::{
    compile_agent_config, create_worktree, ensure_tmux_session, worktrees_dir,
};
use super::terminal_launcher;

// ── GitWorktreeIsolation ────────────────────────────────────────────────────

/// Isolation strategy that creates git worktrees and compiles agent configs.
pub(crate) struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &JobContext) -> Result<PathBuf> {
        let worktree_path = worktrees_dir().join(&job.slug);

        // 1. Create git worktree
        create_worktree(&worktree_path, &job.branch, None)?;

        // 2. Create .ship-session and write spec with phase context
        let session_dir = worktree_path.join(".ship-session");
        std::fs::create_dir_all(&session_dir)?;
        write_phase_spec_from_context(&worktree_path, job);

        // 3. Run `ship use {agent}` in the worktree
        compile_agent_config(&worktree_path, &job.agent)?;

        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &JobContext) -> Result<()> {
        let worktree_path = worktrees_dir().join(&job.slug);
        if worktree_path.exists() {
            super::helpers::remove_worktree(&worktree_path)?;
        }
        super::helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}

/// Write the job spec into the worktree, reading from `spec_path` on the context.
fn write_phase_spec_from_context(worktree_path: &std::path::Path, job: &JobContext) {
    let dst_spec = worktree_path.join(".ship-session").join("job-spec.md");
    let src = std::path::Path::new(&job.spec_path);
    let content = if src.exists() {
        std::fs::read_to_string(src).unwrap_or_default()
    } else {
        let parent = worktree_path.parent().and_then(|p| p.parent());
        if let Some(root) = parent {
            let alt = root.join(&job.spec_path);
            std::fs::read_to_string(&alt).unwrap_or_default()
        } else {
            String::new()
        }
    };
    let _ = std::fs::write(&dst_spec, &content);
}

// ── TmuxExecutor ────────────────────────────────────────────────────────────

/// Job executor that spawns agents in tmux sessions.
pub(crate) struct TmuxExecutor;

#[async_trait]
impl JobExecutor for TmuxExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let tmux_session = format!("job-{}", ctx.slug);
        ensure_tmux_session(&tmux_session, &ctx.work_dir)?;
        spawn_agent_with_mesh_id(&tmux_session, &ctx.slug, ctx.model.as_deref());

        let (strategy, launched) = terminal_launcher::launch(&tmux_session);
        tracing::info!(
            slug = ctx.slug, strategy, launched,
            "job-dispatch: terminal launch attempted"
        );

        // Monitor tmux session in background — fire job.completed when it exits.
        let monitor_session = tmux_session.clone();
        let monitor_slug = ctx.slug.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(10));
                let alive = std::process::Command::new("tmux")
                    .args(["has-session", "-t", &monitor_session])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if !alive {
                    tracing::info!(
                        slug = monitor_slug,
                        session = monitor_session,
                        "tmux-monitor: session exited, firing stop hook"
                    );
                    let home = dirs::home_dir().unwrap_or_default().join(".ship");
                    if let (Ok(port_str), Ok(token)) = (
                        std::fs::read_to_string(home.join("daemon.port")),
                        std::fs::read_to_string(home.join("daemon.token")),
                    ) {
                        let port = port_str.trim().to_string();
                        let token = token.trim().to_string();
                        let _ = std::process::Command::new("curl")
                            .args([
                                "-sf", "-X", "POST",
                                &format!("http://127.0.0.1:{port}/api/runtime/jobs/end"),
                                "-H", "Content-Type: application/json",
                                "-H", &format!("Authorization: Bearer {token}"),
                                "-d", &format!("{{\"slug\":\"{monitor_slug}\"}}"),
                            ])
                            .status();
                    }
                    break;
                }
            }
        });

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(tmux_session),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return false,
        };
        std::process::Command::new("tmux")
            .args(["has-session", "-t", session])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return Ok(()),
        };
        super::helpers::kill_tmux_session(session)
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let status = std::process::Command::new("tmux")
            .args(["send-keys", "-t", session, message, "Enter"])
            .status()
            .map_err(|e| anyhow::anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!("tmux send-keys returned non-zero"));
        }
        Ok(())
    }
}

/// Send agent CLI command into the tmux session with SHIP_MESH_ID set.
fn spawn_agent_with_mesh_id(tmux_session: &str, slug: &str, model: Option<&str>) {
    let model_flag = match model {
        Some(m) => format!(" --model {m}"),
        None => String::new(),
    };
    let cmd = format!(
        "SHIP_MESH_ID={slug} claude --dangerously-skip-permissions \
         --dangerously-load-development-channels server:ship{model_flag}"
    );
    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", tmux_session, &cmd, "Enter"])
        .status();
    if let Err(e) = result {
        tracing::warn!(session = tmux_session, "tmux send-keys failed: {e}");
        return;
    }

    // Wait for claude to be ready, then auto-accept prompts and send "go".
    // Polls the tmux pane for the input prompt (❯) instead of blind delays.
    let session = tmux_session.to_string();
    std::thread::spawn(move || {
        // Phase 1: Wait for experimental channels prompt (up to 30s)
        if wait_for_prompt(&session, 30) {
            // Accept the experimental channels prompt
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &session, "Enter"])
                .status();
        }
        // Phase 2: Wait for the main input prompt (up to 30s)
        if wait_for_prompt(&session, 30) {
            // Send "go" to trigger job autostart
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &session, "go", "Enter"])
                .status();
        }
    });
}

/// Poll tmux pane until the Claude input prompt (❯) appears.
/// Returns true if found, false on timeout.
fn wait_for_prompt(session: &str, timeout_secs: u64) -> bool {
    for _ in 0..timeout_secs {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let output = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", session, "-p"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            // Claude shows ❯ when ready for input
            if text.contains('❯') {
                return true;
            }
        }
    }
    tracing::warn!(session, "agent: timed out waiting for prompt");
    false
}

// ── ProcessExecutor ─────────────────────────────────────────────────────────

/// Job executor that spawns agents as direct child processes.
/// No tmux dependency. The runtime owns the PID and detects exit.
/// Output is logged to `.ship-session/agent.log` in the worktree.
/// The agent runs non-interactively via `--print -p "go"`.
pub(crate) struct ProcessExecutor;

/// State held in the ExecutorHandle for process-based execution.
struct ProcessState {
    pid: u32,
    log_path: PathBuf,
}

#[async_trait]
impl JobExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let log_dir = ctx.work_dir.join(".ship-session");
        std::fs::create_dir_all(&log_dir)?;
        let log_path = log_dir.join("agent.log");
        let log_file = std::fs::File::create(&log_path)?;
        let err_file = log_file.try_clone()?;

        let mut args = vec![
            "--dangerously-skip-permissions",
            "--dangerously-load-development-channels", "server:ship",
            "--print",
            "-p", "go",
        ];
        if let Some(ref m) = ctx.model {
            args.push("--model");
            args.push(m);
        }
        let child = std::process::Command::new("claude")
            .args(&args)
            .env("SHIP_MESH_ID", &ctx.slug)
            .current_dir(&ctx.work_dir)
            .stdout(log_file)
            .stderr(err_file)
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn claude: {e}"))?;

        let pid = child.id();
        tracing::info!(
            slug = ctx.slug,
            pid,
            log = %log_path.display(),
            "process-executor: agent spawned"
        );

        // Monitor the process in a background task — when it exits, fire the stop hook.
        let slug = ctx.slug.clone();
        let log_path_clone = log_path.clone();
        std::thread::spawn(move || {
            let mut child = child;
            let status = child.wait();
            match status {
                Ok(s) => {
                    tracing::info!(slug, pid, exit_code = ?s.code(), "process-executor: agent exited");
                }
                Err(e) => {
                    tracing::warn!(slug, pid, "process-executor: wait failed: {e}");
                }
            }
            // Fire the stop hook — emit job.completed via daemon API
            let home = dirs::home_dir().unwrap_or_default().join(".ship");
            if let (Ok(port_str), Ok(token)) = (
                std::fs::read_to_string(home.join("daemon.port")),
                std::fs::read_to_string(home.join("daemon.token")),
            ) {
                let port = port_str.trim();
                let token = token.trim();
                let _ = std::process::Command::new("curl")
                    .args([
                        "-sf", "-X", "POST",
                        &format!("http://127.0.0.1:{port}/api/runtime/jobs/end"),
                        "-H", "Content-Type: application/json",
                        "-H", &format!("Authorization: Bearer {token}"),
                        "-d", &format!("{{\"slug\":\"{slug}\"}}"),
                    ])
                    .status();
            }
        });

        Ok(ExecutorHandle {
            pid: Some(pid),
            inner: Box::new(ProcessState { pid, log_path }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<ProcessState>() {
            Some(s) => s,
            None => return false,
        };
        // Check if process exists (signal 0 = existence check)
        unsafe { libc::kill(state.pid as i32, 0) == 0 }
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<ProcessState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        unsafe { libc::kill(state.pid as i32, libc::SIGTERM); }
        Ok(())
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Non-interactive process — messages not supported.
        // Use mesh events for agent communication instead.
        Err(anyhow::anyhow!("ProcessExecutor does not support interactive messages"))
    }
}

/// Write phase spec into an existing worktree (used by pipeline advancement).
pub(crate) fn write_phase_spec(
    worktree_path: &std::path::Path,
    spec_path: &str,
    pipeline: &Option<Vec<PipelinePhase>>,
    phase_idx: Option<usize>,
) {
    let dst_spec = worktree_path.join(".ship-session").join("job-spec.md");
    let src = std::path::Path::new(spec_path);
    let mut content = if src.exists() {
        std::fs::read_to_string(src).unwrap_or_default()
    } else {
        let parent = worktree_path.parent().and_then(|p| p.parent());
        if let Some(root) = parent {
            let alt = root.join(spec_path);
            std::fs::read_to_string(&alt).unwrap_or_default()
        } else {
            String::new()
        }
    };

    if let (Some(pipeline), Some(idx)) = (pipeline, phase_idx) {
        let total = pipeline.len();
        let current = &pipeline[idx];
        let mut phase_header = format!(
            "## Current Phase\n\nPhase {} of {}: {}\nAgent: {}\nGoal: {}\n",
            idx + 1, total, current.goal, current.agent, current.goal,
        );
        if idx > 0 {
            phase_header.push_str("\nPrior phases completed:\n");
            for (i, p) in pipeline[..idx].iter().enumerate() {
                phase_header.push_str(&format!("- Phase {} ({}): {}\n", i + 1, p.agent, p.goal));
            }
        }
        phase_header.push('\n');
        if let Some(end) = content.find("---\n").and_then(|first| {
            content[first + 4..].find("---\n").map(|second| first + 4 + second + 4)
        }) {
            content.insert_str(end, &phase_header);
        } else {
            content = format!("{phase_header}\n{content}");
        }
    }

    if let Err(e) = std::fs::write(&dst_spec, &content) {
        tracing::error!("job-dispatch: failed to write job-spec.md: {e}");
    }
}
