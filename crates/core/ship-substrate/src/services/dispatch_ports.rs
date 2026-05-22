//! Port types for job dispatch — isolation and execution abstractions.
//!
//! These types are always available (not gated behind `unstable`) because
//! the daemon's job dispatch orchestration depends on them for trait injection.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

pub use crate::events::agent_session::InvocationMode;

/// Why an executor is signalling exit. Forces each call site to declare
/// semantics: we either observed the agent's own exit, or the substrate
/// (tmux session/window, PTY, ...) went away without us seeing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCause {
    /// The agent process actually exited; `exit_code` is meaningful
    /// (`Some` = real code, `None` = process is gone but no code captured).
    AgentExited,
    /// The executor's session/window was destroyed externally — we did
    /// not observe the agent exit. `exit_code` is always `None`.
    SessionDestroyed,
}

/// Signal sent from an executor monitor when an agent's run ends.
///
/// Executors send this on `SessionContext::exit_tx` so the dispatch handler can
/// emit `job.completed` or `job.failed` without waiting for the SessionEnd hook.
///
/// `cause` is exhaustive — every executor must declare whether it observed
/// the agent exit or merely noticed the substrate went away. The liveness
/// handler routes on `cause`: only `AgentExited` with `exit_code == Some(0)`
/// is a clean completion.
#[derive(Debug)]
pub struct ExitSignal {
    pub slug: String,
    pub pipeline_run_id: String,
    pub exit_code: Option<i32>,
    pub cause: ExitCause,
}

/// Context for a job being dispatched. Carries everything an executor needs.
#[derive(Clone, Debug)]
pub struct SessionContext {
    pub pipeline_run_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Inline spec content. When `Some`, the isolation strategy writes this to
    /// `{work_dir}/{spec_path}` during `prepare`. `None` = fall back to copying
    /// from the project root (legacy manual-dispatch path).
    pub spec_content: Option<String>,
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    /// How to launch the harness CLI for this job.
    pub launch: LaunchProfile,
    /// Exit notification channel. Set by `dispatch_job_internal` so the executor's
    /// background monitor can signal exit without curling the daemon API.
    /// `None` for pipeline re-dispatches — the liveness sweep handles those.
    pub exit_tx: Option<UnboundedSender<ExitSignal>>,
    /// Step metadata used by the TmuxWindowExecutor to pick the tmux window
    /// name and per-step log path. `None` for single-step jobs — the executor
    /// falls back to using `slug` as the window name.
    pub step: Option<StepInfo>,
    /// Harness invocation mode — the source of truth for how the
    /// executor launches the harness. Replaces the old `headless: bool`
    /// (the three-axis model: see agent-primitive-inversion plan).
    ///   - `Pty`: PTY-attached (routes to TmuxWindow when available,
    ///     else ProcessExecutor with a PTY).
    ///   - `Print`: one-shot non-interactive (ProcessExecutor, `-p`).
    ///   - `Stream`: server-mode (ServerExecutor) — routed before this
    ///     field is consulted.
    /// `keepalive` (below) is orthogonal and forces ProcessExecutor +
    /// build_interactive regardless of invocation.
    pub invocation: InvocationMode,
    /// Long-running interactive session — not a pipeline step. Routes to
    /// ProcessExecutor with `adapter.build_interactive` (no `-p`, no
    /// `--print`). The harness stays at its TUI prompt waiting for stdin
    /// pushes (via `/api/v1/sessions/{slug}/message` Stdin mechanism) or
    /// WebSocket attach. Mutually exclusive with `headless` — the
    /// dispatch validator should reject both being true.
    ///
    /// Drives the cmdb-modal chat backend, the analytics SessionStart
    /// hook flow, and reliable PTY attach. See
    /// `.artifacts/agents-arc/discussions/dispatch-modes-vs-session-hooks.md`
    /// and `pty-attach-needs-keepalive-mode.md`.
    pub keepalive: bool,
}

/// Per-step metadata threaded to executors.
#[derive(Clone, Debug)]
pub struct StepInfo {
    /// 0-based index within the pipeline.
    pub index: usize,
    /// Stable step identifier (e.g. "test-writer").
    pub id: String,
}

/// Everything about *how* to run the harness process.
///
/// Separated from SessionContext so launch configuration is composable —
/// harness defaults + pipeline step overrides + user preferences.
#[derive(Clone, Debug, Default)]
pub struct LaunchProfile {
    /// Harness CLI to use (e.g. "claude", "codex"). None = default (claude).
    pub harness: Option<String>,
    /// Model override for this run.
    pub model: Option<String>,
    /// Effort level (e.g. "low", "medium", "high", "max"). Harness-specific.
    pub effort: Option<String>,
    /// Resume a previous session by ID.
    pub resume_session: Option<String>,
    /// Display name for the session (shows in harness's session picker).
    pub session_name: Option<String>,
    /// Maximum spend in USD for this run.
    pub max_budget_usd: Option<f64>,
    /// Permission mode override (e.g. "plan", "auto", "bypassPermissions").
    pub permission_mode: Option<String>,
    /// Whether to persist the session (false = ephemeral, e.g. gate reviews).
    pub persist_session: bool,
    /// Request structured JSON output from the harness.
    pub structured_output: bool,
    /// Prompt handed to headless harness runs as the first instruction.
    /// Claude consumes it via `-p` after the rest of its flags; Codex takes it
    /// as the final positional argument to `codex exec`. `None` falls back to
    /// each adapter's default.
    pub autostart_prompt: Option<String>,
    /// Additional system-prompt content appended to the harness's base
    /// system prompt. Delivered via Claude's `--append-system-prompt`.
    /// Used to inject a Ship agent's `rules.inline` into the spawned
    /// subprocess so the agent knows its editing protocol (read-before-write,
    /// `chat_reply` conventions, etc.).
    pub append_system_prompt: Option<String>,
}

/// Handle returned by a `SessionExecutor` after spawning an agent.
pub struct ExecutorHandle {
    pub pid: Option<u32>,
    /// Opaque adapter-specific state.
    pub inner: Box<dyn std::any::Any + Send + Sync>,
}

/// CLI command built by a `HarnessLauncher`.
#[derive(Debug, Clone)]
pub struct LaunchCommand {
    /// Binary name (e.g. "claude", "codex").
    pub bin: String,
    /// CLI arguments.
    pub args: Vec<String>,
    /// Extra environment variables to set.
    pub env: HashMap<String, String>,
}

impl LaunchCommand {
    /// Flatten to a single shell command string (for tmux send-keys).
    /// Arguments and env values containing shell-significant characters
    /// are single-quoted so prompts like `"Read spec.md and execute it."`
    /// survive intact rather than being word-split by the shell.
    pub fn to_shell_string(&self) -> String {
        let mut parts = Vec::new();
        for (k, v) in &self.env {
            parts.push(format!("{k}={}", shell_quote(v)));
        }
        parts.push(shell_quote(&self.bin));
        parts.extend(self.args.iter().map(|a| shell_quote(a)));
        parts.join(" ")
    }
}

/// Quote a string for safe POSIX-sh consumption. Strings containing only
/// shell-safe characters pass through unchanged; everything else is
/// wrapped in single quotes with inner single quotes escaped as `'\''`.
fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '@' | '=' | '+' | ',')
        })
    {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod shell_quote_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn safe_strings_pass_through_unquoted() {
        assert_eq!(shell_quote("codex"), "codex");
        assert_eq!(shell_quote("--full-auto"), "--full-auto");
        assert_eq!(shell_quote("/tmp/foo.txt"), "/tmp/foo.txt");
        assert_eq!(shell_quote("KEY=value"), "KEY=value");
    }

    #[test]
    fn empty_string_is_quoted() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn strings_with_spaces_are_single_quoted() {
        assert_eq!(
            shell_quote("Read spec.md and execute it."),
            "'Read spec.md and execute it.'",
        );
    }

    #[test]
    fn inner_single_quotes_are_escaped() {
        assert_eq!(shell_quote("it's working"), "'it'\\''s working'");
    }

    #[test]
    fn shell_metachars_are_quoted() {
        assert_eq!(shell_quote("a && b"), "'a && b'");
        assert_eq!(shell_quote("$HOME"), "'$HOME'");
        assert_eq!(shell_quote("foo | bar"), "'foo | bar'");
        assert_eq!(shell_quote("`whoami`"), "'`whoami`'");
    }

    #[test]
    fn to_shell_string_quotes_multiword_prompt() {
        let cmd = LaunchCommand {
            bin: "codex".into(),
            args: vec![
                "exec".into(),
                "--full-auto".into(),
                "Read .artifacts/spec.md and execute it.".into(),
            ],
            env: HashMap::new(),
        };
        assert_eq!(
            cmd.to_shell_string(),
            "codex exec --full-auto 'Read .artifacts/spec.md and execute it.'",
        );
    }

    #[test]
    fn to_shell_string_quotes_env_values_with_spaces() {
        let mut env = HashMap::new();
        env.insert("MSG".to_string(), "hello world".to_string());
        let cmd = LaunchCommand {
            bin: "claude".into(),
            args: vec!["-p".into()],
            env,
        };
        assert!(cmd.to_shell_string().starts_with("MSG='hello world' "));
    }
}

/// Port: prepares and cleans up the isolated working directory for a job.
#[async_trait]
pub trait IsolationStrategy: Send + Sync {
    /// Prepare an isolated working directory. Returns the path.
    async fn prepare(&self, job: &SessionContext) -> Result<PathBuf>;
    /// Clean up after job completion or failure.
    async fn cleanup(&self, job: &SessionContext) -> Result<()>;
}

/// Port: spawns and manages the agent process lifecycle.
#[async_trait]
pub trait SessionExecutor: Send + Sync {
    /// Spawn the agent process in the prepared work directory.
    async fn spawn(&self, ctx: &SessionContext) -> Result<ExecutorHandle>;
    /// Check if the agent process is still running.
    async fn is_alive(&self, handle: &ExecutorHandle) -> bool;
    /// Terminate the agent process.
    async fn stop(&self, handle: &ExecutorHandle) -> Result<()>;
    /// Inject a message into the running agent.
    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()>;

    /// Rehydrate a handle for a still-running process after a daemon
    /// restart. Returns `Some(handle)` if the executor can locate a live
    /// process for the given `slug` (and optionally `current_step` for
    /// executors that spawn per-step processes), `None` otherwise.
    /// Default: `None` — conservative; callers treat that as "no live
    /// process" and the job gets marked failed on reconciliation.
    ///
    /// Replaces the historic pattern of constructing a fake `pid: None`
    /// handle and calling `is_alive` on it, which always returned false
    /// because the inner opaque state couldn't be downcast.
    async fn resume_handle(
        &self,
        _slug: &str,
        _harness: Option<&str>,
        _current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        None
    }
}

// ── Orchestrator port traits ────────────────────────────────────────────────

/// Port: platform-specific agent operations that the orchestrator delegates
/// to the host process (e.g. shipd).
///
/// These methods abstract over tmux, git worktree, and shell execution so
/// the orchestration logic in runtime is testable without a real supervisor.
#[async_trait]
pub trait AgentPlatform: Send + Sync {
    /// Base directory where job worktrees are created.
    fn worktrees_dir(&self) -> PathBuf;

    /// Compile agent configuration via `ship use <agent_id>`.
    fn compile_agent_config(&self, worktree_path: &std::path::Path, agent_id: &str) -> Result<()>;

    /// Gracefully exit the current agent in a tmux session.
    async fn graceful_agent_exit(&self, slug: &str, harness: Option<&str>);

    /// Clean up job resources (tmux session, worktree, branch).
    async fn cleanup_job(&self, slug: &str, branch: &str);

    /// Execute `git merge <branch>` in the repo directory. Returns true on success.
    fn git_merge(&self, branch: &str) -> Result<bool>;

    /// Clean up orphaned worktrees from previous daemon runs.
    async fn cleanup_orphaned_worktrees(&self);
}

/// Port: process handle registry for tracking spawned agents.
///
/// The orchestrator registers/unregisters handles as jobs are dispatched
/// and completed. The host process owns the actual registry data structure.
#[async_trait]
pub trait ProcessRegistryPort: Send + Sync {
    /// Store an executor handle for a dispatched job.
    async fn register(&self, slug: &str, handle: ExecutorHandle);
    /// Remove a handle (e.g. when a job completes).
    async fn unregister(&self, slug: &str);
    /// Remove and return a handle so the caller can stop the process.
    /// Default `None` — tests and registries that don't track handles
    /// can leave this unimplemented.
    async fn take(&self, _slug: &str) -> Option<ExecutorHandle> { None }
    /// Return all currently registered slug keys.
    /// Used by the coherence sweep to detect stray processes.
    async fn live_slugs(&self) -> Vec<String> { vec![] }
    /// Read the current PID for a slug without removing the handle.
    /// Returns `None` if no live handle is registered, or if the handle
    /// has no captured PID. Used by the step monitor's subprocess-tree
    /// heartbeat probe — it needs a read-only root PID to walk children.
    async fn get_pid(&self, _slug: &str) -> Option<u32> { None }
    /// Probe each registered handle via `executor.is_alive()`; remove and
    /// return the slugs whose handles are no longer alive. Used by the
    /// coherence sweep to evict dead handles before the drift checks run.
    /// Default `vec![]` for registries that don't track handles.
    async fn drain_dead(&self, _executor: &dyn SessionExecutor) -> Vec<String> { vec![] }
}

/// Sample of the liveness state of a dispatched process tree.
///
/// `cpu_ticks` is a monotonic counter whose unit is implementation-defined
/// — only deltas between successive samples matter. The monitor reads
/// "tree is idle" from a zero delta and resets its idle timer on any
/// positive delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSample {
    /// Monotonic CPU counter for the tree (root + descendants). Unit is
    /// impl-defined; only deltas are meaningful.
    pub cpu_ticks: u64,
    /// Number of live processes in the tree at sample time — observability
    /// only, the monitor doesn't gate decisions on it.
    pub process_count: u32,
}

/// Port: observes liveness of a dispatched process tree.
///
/// Implementations walk descendants of the root PID and return the total
/// accumulated CPU time across the tree. The monitor compares successive
/// samples: delta == 0 → tree is idle; delta > 0 → heartbeat.
///
/// `sample` returning `None` means the root PID is gone; callers treat
/// that as "tree dead" and defer to the executor's exit signal for the
/// actual termination path.
#[async_trait]
pub trait ProcessTreeProbe: Send + Sync {
    async fn sample(&self, root_pid: u32) -> Option<TreeSample>;
}
