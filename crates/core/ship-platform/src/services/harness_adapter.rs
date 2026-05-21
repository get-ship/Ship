//! Harness adapter trait — single interface between executors and harness CLIs.
//!
//! Each harness implements this trait once. Executors use the trait methods
//! without knowing which harness they're running.

use std::path::Path;

use super::dispatch_ports::{LaunchCommand, LaunchProfile};

/// Errors returned from `HarnessAdapter::build_resume_args`.
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
    /// harness's structured stream — see HookTurnObservedPayload for
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
pub trait HarnessAdapter: Send + Sync {
    /// Harness identifier (e.g. "claude", "codex").
    fn id(&self) -> &str;

    /// True when `id` names this harness — its canonical `id()` or an alias
    /// (e.g. claude also answers to "claude-code"). Single place an adapter
    /// declares the names it owns; `adapter_for` resolves through it.
    fn matches(&self, id: &str) -> bool {
        id == self.id()
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

    /// Text to send after the CLI is ready to trigger job autostart.
    fn autostart_input(&self) -> &str;

    /// Command to send for graceful shutdown.
    fn exit_command(&self) -> &str;

    /// Parse a line of stdout from the harness process.
    fn parse_output(&self, _line: &str) -> Option<HarnessEvent> { None }

    /// Extract session metadata from the work directory after launch.
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

/// The single source of truth for which harnesses exist. `adapter_for`, the
/// `Known:` error string, and `capabilities()` all derive from this slice —
/// adding or dropping a harness is one edit here plus the adapter module, not
/// a sweep across parallel match arms and enumerations. Adapters are zero-
/// sized unit structs, so this is a `&'static` slice of const-promoted refs
/// with no allocation.
const REGISTRY: &[&dyn HarnessAdapter] = &[
    &super::harnesses::claude::ClaudeAdapter,
    &super::harnesses::codex::CodexAdapter,
    &super::harnesses::cursor::CursorAdapter,
    &super::harnesses::gemini::GeminiAdapter,
    &super::harnesses::opencode::OpenCodeAdapter,
];

/// Comma-separated list of canonical harness ids, for error messages.
fn known_ids() -> String {
    REGISTRY
        .iter()
        .map(|a| a.id())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolve a harness name to its adapter.
/// Returns Err for unknown harnesses (no silent default) so a `foo-typo`
/// config doesn't silently run as Claude with weird flags.
pub fn adapter_for(harness: &str) -> Result<&'static dyn HarnessAdapter, String> {
    REGISTRY
        .iter()
        .copied()
        .find(|a| a.matches(harness))
        .ok_or_else(|| format!("unknown harness '{harness}'. Known: {}.", known_ids()))
}

/// Convenience: resolve an Option<&str>, falling back to "claude" when None.
/// Used by executors that want a default for unconfigured pipelines.
pub fn adapter_for_or_default(
    harness: Option<&str>,
) -> Result<&'static dyn HarnessAdapter, String> {
    adapter_for(harness.unwrap_or("claude"))
}

/// Enumerate all known harnesses and their capabilities.
/// Used by `ship status --harnesses` and Studio's harness picker.
pub fn capabilities() -> Vec<(&'static str, HarnessCapabilities)> {
    REGISTRY.iter().map(|a| (a.id(), a.caps())).collect()
}

#[cfg(test)]
mod adapter_for_tests {
    use super::*;

    #[test]
    fn all_harnesses_advertise_persistent_session() {
        // Every TUI agent CLI is a long-lived REPL we can hold across
        // turns. If a new harness lands without persistent_session=true,
        // it'd quietly fall to the legacy per-turn-respawn path; this
        // test pins the parity contract.
        for name in ["claude", "codex", "cursor", "gemini", "opencode"] {
            let adapter = adapter_for(name).unwrap();
            assert!(
                adapter.caps().persistent_session,
                "harness '{name}' must support persistent_session"
            );
        }
    }

    #[test]
    fn push_mechanism_per_harness() {
        // Pin which mechanism each harness uses. Drift in this map is
        // a behavior change — surface it in review.
        use PushMechanism::*;
        let expected: &[(&str, PushMechanism)] = &[
            ("claude", Channel),
            ("codex", Stdin),
            ("cursor", Stdin),
            ("gemini", Stdin),
            ("opencode", Stdin),
        ];
        for (name, want) in expected {
            assert_eq!(
                adapter_for(name).unwrap().caps().push_mechanism,
                *want,
                "harness '{name}' push_mechanism drifted"
            );
        }
    }

    #[test]
    fn adapter_for_cursor_returns_cursor_adapter() {
        assert_eq!(adapter_for("cursor").unwrap().id(), "cursor");
    }

    #[test]
    fn adapter_for_gemini_returns_gemini_adapter() {
        assert_eq!(adapter_for("gemini").unwrap().id(), "gemini");
    }

    #[test]
    fn adapter_for_codex_returns_codex_adapter() {
        assert_eq!(adapter_for("codex").unwrap().id(), "codex");
    }

    #[test]
    fn adapter_for_opencode_returns_opencode_adapter() {
        assert_eq!(adapter_for("opencode").unwrap().id(), "opencode");
    }

    #[test]
    fn adapter_for_or_default_none_returns_claude_adapter() {
        assert_eq!(adapter_for_or_default(None).unwrap().id(), "claude");
    }

    #[test]
    fn adapter_for_unknown_returns_err() {
        let result = adapter_for("nonexistent");
        assert!(result.is_err());
        let msg = result.err().unwrap();
        assert!(msg.contains("unknown harness"), "msg was: {msg}");
        assert!(msg.contains("Known: claude, codex"), "msg was: {msg}");
    }

    #[test]
    fn adapter_for_or_default_propagates_unknown_err() {
        let result = adapter_for_or_default(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn registry_is_single_source_of_truth() {
        // adapter_for, capabilities(), and the Known: error string all derive
        // from REGISTRY. Pin the invariant: every id capabilities() reports
        // resolves back through adapter_for to an adapter with that id, and
        // shows up in the unknown-harness error list. Adding a harness to the
        // slice satisfies all three at once — no parallel list to forget.
        let known = adapter_for("definitely-not-a-harness")
            .err()
            .expect("unknown harness must error");
        for (id, _caps) in capabilities() {
            assert_eq!(adapter_for(id).unwrap().id(), id, "adapter_for disagrees for {id}");
            assert!(known.contains(id), "Known: list missing {id}: {known}");
        }
    }

    #[test]
    fn capabilities_lists_all_five_harnesses() {
        let caps = capabilities();
        let ids: Vec<&str> = caps.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec!["claude", "codex", "cursor", "gemini", "opencode"]);
    }

    #[test]
    fn codex_declares_server_mode() {
        // Opencode's server_mode is currently off pending OpenCodeSession
        // rewrite to match the new `{parts:[...]}` / prompt_async API.
        // See `opencode.rs::caps` comment.
        let caps = capabilities();
        let server_capable: Vec<&str> = caps
            .iter()
            .filter(|(_, c)| c.server_mode)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(server_capable, vec!["codex"]);
    }

    #[test]
    fn claude_does_not_declare_server_mode() {
        let claude = adapter_for("claude").unwrap();
        assert!(!claude.caps().server_mode);
    }

    #[test]
    fn minimal_caps_are_conservative() {
        let m = HarnessCapabilities::MINIMAL;
        assert!(m.headless);
        assert!(m.interactive);
        assert!(!m.server_mode);
        assert!(!m.json_stream);
        assert!(!m.mcp);
        assert!(!m.hooks);
        assert!(!m.session_resume);
        assert!(!m.persistent_session);
        assert_eq!(m.push_mechanism, PushMechanism::Respawn);
    }

    #[test]
    fn only_claude_uses_channel_push() {
        let caps = capabilities();
        let channel_users: Vec<&str> = caps
            .iter()
            .filter(|(_, c)| c.push_mechanism == PushMechanism::Channel)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(channel_users, vec!["claude"]);
    }
}
