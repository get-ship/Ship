//! Agent driver trait — single interface between executors and any agent
//! runtime (Floki in-process, or a third-party harness CLI).
//!
//! Each runtime implements this trait once. Executors use the trait methods
//! without knowing which runtime is in front of them.
//!
//! # What lives here vs. elsewhere
//!
//! This module defines only trait/type contracts. Concrete driver structs
//! (Floki's native driver, plus ClaudeAdapter/CodexAdapter/… in
//! `ship-harnesses`) live with their implementation. The registry helpers
//! (`adapter_for`, `capabilities`) live with whichever crate owns the
//! concrete struct impls.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use crate::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::server_session::ServerSession;

/// Errors returned from `AgentDriver::build_resume_args`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResumeError {
    /// The harness has no session-resume capability. Dispatch should reject
    /// the resume request with a 400 rather than silently launching fresh.
    #[error("harness '{harness}' does not support session resume")]
    Unsupported { harness: String },
    /// Session id was rejected (empty, malformed, or not recognized).
    #[error("session id rejected by harness '{harness}': {reason}")]
    BadSession { harness: String, reason: String },
}

/// Events extracted from harness process output.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HarnessEvent {
    ToolUse { tool: String, input_preview: String },
    Message { text: String },
    SessionStarted { session_id: String },
    TurnComplete { stop_reason: String },
    /// Per-turn token usage observation. All counts absolute for this
    /// turn (not cumulative). Synthesized by adapters from their
    /// harness's structured stream — see agent_def::TurnEnded (token-usage fields) for
    /// the kernel-event shape this maps onto.
    TurnUsage {
        model: Option<String>,
        input_tokens: u64,
        output_tokens: u64,
        reasoning_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
        stop_reason: Option<String>,
        /// Cost as REPORTED by the harness, in USD. None when the
        /// harness doesn't compute its own (most don't). Source of
        /// truth for "what the harness thinks this turn cost" —
        /// captured verbatim; we never recompute. Subscription users
        /// may see harness-side values that don't reflect their plan
        /// (claude reports API-equivalent cost even on Max plans).
        harness_reported_cost_usd: Option<f64>,
    },
    Custom {
        #[serde(rename = "subtype")]
        kind: String,
        data: serde_json::Value,
    },
}

/// Declarative capability flags per harness.
/// Used by executors + pipeline-step routing to branch on what's supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessCapabilities {
    /// Harness supports `-p`-style headless one-shot execution.
    pub headless: bool,
    /// Harness supports interactive TUI mode.
    pub interactive: bool,
    /// Harness has a long-lived server/daemon mode (codex app-server,
    /// opencode serve) suitable for persistent orchestrator connections.
    pub server_mode: bool,
    /// Harness can emit JSON-lines structured output (codex --json,
    /// claude stream-json, cursor stream-json).
    pub json_stream: bool,
    /// Harness supports MCP tool injection at the config level.
    pub mcp: bool,
    /// Harness has a hook mechanism we can wire `ship hook session-end` into.
    pub hooks: bool,
    /// Harness supports resuming a prior session in headless/server mode
    /// (codex exec resume, opencode REST sessions).
    pub session_resume: bool,
    /// The harness process can hold across turns — a long-lived TUI /
    /// REPL we keep alive and feed new turns to, instead of spawning
    /// fresh per turn. All five CLI agents support this; they're TUIs.
    pub persistent_session: bool,
    /// How we deliver a new user turn to a persistent session. Only
    /// meaningful when `persistent_session = true`.
    pub push_mechanism: PushMechanism,
}

/// How a new user turn reaches a long-lived harness process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PushMechanism {
    /// Harness-specific MCP channel notification (Claude Code's
    /// `notifications/claude/channel`). Bidirectional, structured.
    Channel,
    /// Write the prompt + newline to the PTY master. Works for any TUI
    /// that's just a REPL waiting at a prompt. Harness has no inkling
    /// it's being driven programmatically.
    Stdin,
    /// Kill the process and respawn with `--resume <session_id>` plus
    /// the new prompt. Universal fallback — any harness with
    /// `session_resume = true` supports this.
    #[default]
    Respawn,
}

/// How the autostart message + Enter should reach the harness TUI.
/// See [`AgentDriver::submit_mode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SubmitMode {
    /// Type `<msg> Enter` as a single `tmux send-keys` argv burst.
    /// Fine for input-line-friendly TUIs that don't have a frame
    /// boundary between text input and the submit gesture (omp).
    #[default]
    Inline,
    /// Split the argv: type `<msg>` first, then sleep 150ms, then a
    /// separate `tmux send-keys Enter`. Required for harnesses whose
    /// input box swallows a fast trailing `Enter` as a paste-newline
    /// (claude's TUI has a frame boundary between text input and the
    /// submit gesture; the inline burst lands as a single paste-like
    /// frame and the submit never fires).
    Spaced,
}

impl HarnessCapabilities {
    /// Conservative defaults — headless + interactive only, no
    /// persistence story (every turn respawns).
    pub const MINIMAL: Self = Self {
        headless: true,
        interactive: true,
        server_mode: false,
        json_stream: false,
        mcp: false,
        hooks: false,
        session_resume: false,
        persistent_session: false,
        push_mechanism: PushMechanism::Respawn,
    };
}

/// Complete lifecycle adapter for a harness CLI.
#[async_trait]
pub trait AgentDriver: Send + Sync {
    /// Harness identifier (e.g. "claude", "codex").
    fn id(&self) -> &str;

    /// True when `id` names this harness — its canonical `id()` or an alias
    /// (e.g. claude also answers to "claude-code"). Single place an adapter
    /// declares the names it owns; `adapter_for` resolves through it.
    fn matches(&self, id: &str) -> bool {
        id == self.id()
    }

    /// Executable name to look up on PATH / invoke. Defaults to the harness
    /// id — claude/codex/opencode ship binaries named exactly that.
    /// Cursor's CLI installs as `cursor-agent`, so it overrides; without this
    /// the PATH pre-flight checks for a `cursor` binary that never exists and
    /// rejects a working install.
    fn binary(&self) -> &str {
        self.id()
    }

    /// Declarative capability flags — what this harness supports.
    fn caps(&self) -> HarnessCapabilities;

    /// Build the CLI command for an interactive (terminal) session.
    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand;

    /// Build the CLI command for a headless (non-interactive) session.
    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand;

    /// Build the launch command to resume an existing session by id.
    ///
    /// Adapters whose harness has no resume capability (per
    /// `caps().session_resume`) leave the default in place and return
    /// `Err(ResumeError::Unsupported)`. The dispatch path translates
    /// that into a 400 rather than silently launching fresh.
    fn build_resume_args(
        &self,
        _slug: &str,
        _session_id: &str,
        _profile: &LaunchProfile,
    ) -> Result<LaunchCommand, ResumeError> {
        Err(ResumeError::Unsupported {
            harness: self.id().to_string(),
        })
    }

    /// Pattern that indicates the CLI is ready for input.
    fn ready_pattern(&self) -> &str;

    /// Pattern that indicates the harness is actively WORKING on a
    /// submitted turn (e.g. claude's "esc to interrupt" hint). The
    /// executor polls for this after sending the autostart submit to
    /// verify it actually landed, and re-sends the submit if the
    /// indicator never appears — closing the "submit silently
    /// swallowed, pane sits idle forever" failure
    /// (c-2026-07-16T17-14-03-982Z-c1717477).
    ///
    /// Empty (the default) means the adapter has no confirmed indicator
    /// text yet — the executor skips verification for that harness
    /// rather than polling for a guess. Declare conservatively; an
    /// unverified pattern that never matches is worse than no
    /// verification (it would fire the bounded-retry path on every
    /// launch and eventually fail a healthy dispatch).
    fn working_pattern(&self) -> &str {
        ""
    }

    /// Text to send after the CLI is ready to trigger job autostart.
    fn autostart_input(&self) -> &str;

    /// Command to send for graceful shutdown.
    fn exit_command(&self) -> &str;

    /// How the autostart message + Enter should reach the harness TUI.
    /// `Inline` (default) types `<msg>` and `Enter` as a single
    /// `tmux send-keys` argv burst — fine for input-line-friendly TUIs
    /// (omp). `Spaced` (claude) splits the argv: the message is sent
    /// first, then a 150ms sleep, then a separate `tmux send-keys
    /// Enter` call. The split is required for harnesses whose input
    /// box swallows a fast trailing `Enter` as a paste-newline
    /// (claude's TUI has a frame boundary between the message text
    /// and the submit gesture; the inline burst lands as a single
    /// paste-like frame and the submit never fires — see
    /// c-2026-07-06T15-30-00-000Z-claude-harness-autostart).
    fn submit_mode(&self) -> SubmitMode {
        SubmitMode::Inline
    }

    /// Parse a line of stdout from the harness process.
    fn parse_output(&self, _line: &str) -> Option<HarnessEvent> { None }
    /// Extract session metadata from the work directory after launch.
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }

    /// Spawn a fresh server-mode session for this harness (codex app-server,
    /// opencode serve). Returns `None` for harnesses without server mode and
    /// when the spawn fails — the caller then falls through to the one-shot
    /// Process/Tmux path. The adapter owns *how* to spawn its session; the
    /// caller (RoutingExecutor) owns caching and registry placement. This is
    /// what lets routing select server mode generically over the registry
    /// instead of branching per harness id.
    async fn spawn_server_session(&self) -> Option<Arc<dyn ServerSession>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stub adapter that does NOT override `build_resume_args`. Lets the
    /// trait's default implementation exercise its real body so we know
    /// the contract ("returns `ResumeError::Unsupported`") lives at the
    /// trait leaf and can't drift with the executor registry.
    struct StubAdapter;

    #[async_trait]
    impl AgentDriver for StubAdapter {
        fn id(&self) -> &str {
            "stub"
        }
        fn caps(&self) -> HarnessCapabilities {
            HarnessCapabilities::MINIMAL
        }
        fn build_interactive(
            &self,
            _slug: &str,
            _profile: &LaunchProfile,
        ) -> LaunchCommand {
            LaunchCommand {
                bin: String::new(),
                args: Vec::new(),
                env: std::collections::HashMap::new(),
            }
        }
        fn build_headless(
            &self,
            _slug: &str,
            _profile: &LaunchProfile,
        ) -> LaunchCommand {
            LaunchCommand {
                bin: String::new(),
                args: Vec::new(),
                env: std::collections::HashMap::new(),
            }
        }
        fn ready_pattern(&self) -> &str {
            ""
        }
        fn autostart_input(&self) -> &str {
            ""
        }
        fn exit_command(&self) -> &str {
            ""
        }
    }

    #[test]
    fn build_resume_args_default_returns_unsupported() {
        let adapter = StubAdapter;
        let profile = LaunchProfile::default();
        let err = adapter
            .build_resume_args("slug", "sess-1", &profile)
            .expect_err("default must reject resume with Unsupported");
        assert_eq!(
            err,
            ResumeError::Unsupported {
                harness: "stub".to_string(),
            },
            "default must surface Unsupported keyed to the adapter's id",
        );
        assert!(
            err.to_string().contains("does not support session resume"),
            "got: {err}",
        );
    }
}
