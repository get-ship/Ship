//! DispatchRequest wire type and validation helpers.

use serde::Deserialize;
use ship_substrate::events::agent_session::InvocationMode;

/// Wire shape for the Connect `AgentsService.Dispatch` request
/// (`DispatchAgentRequest` in `proto/ship/ops/v1/agents.proto`).
/// Agent-centric — `agent` is required and there is no `pipeline` field;
/// pipeline runs go through `PipelinesService.Dispatch` /
/// `POST /api/v1/pipelines/run`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchRequest {
    /// Ship-agent profile id (required).
    pub agent: String,

    /// Inline spec body. Becomes `.artifacts/job-spec.md` for the agent at launch.
    #[serde(default)]
    pub spec: Option<String>,

    /// Inline one-shot prompt. Either piped via the harness's one-shot
    /// flag (`auto` mode) or injected as the first user turn (interactive).
    #[serde(default)]
    pub prompt: Option<String>,

    /// Continue the last session for this (agent, workspace) pair. Errors
    /// with 400 when no matching session exists.
    #[serde(default)]
    pub resume: bool,

    /// Harness override for this dispatch. None → project/user default chain.
    #[serde(default)]
    pub harness: Option<String>,

    /// Model override. None → harness default.
    #[serde(default)]
    pub model: Option<String>,

    // ── The three orthogonal axes (replace the old single `auto: bool`) ──
    //
    // Each defaults to its no-surprise value so simple POSTs still work:
    //   { "agent": "rust-lane" }
    // = interactive PTY tab, keepalive. The same shape `ship use` opens.

    /// Does this dispatch open an interactive tab / attachable PTY
    /// surface? Default true. Set false for background / dispatched
    /// jobs that the operator will observe later (CLI returns the
    /// session_id + studio_url; nothing attaches to the operator's
    /// terminal).
    #[serde(default = "default_open_tab")]
    pub open_tab: bool,

    /// How the harness is invoked:
    ///   - pty (default): PTY-attached harness invocation (claude TUI,
    ///     codex TUI, ...). Subscription-billed for claude.
    ///   - print: one-shot non-interactive mode (claude -p,
    ///     codex exec). API-billed for claude post-2026-06-19.
    ///   - stream: server-mode (codex app-server, opencode REST).
    ///     Only codex/opencode today.
    #[serde(default = "default_invocation")]
    pub invocation: InvocationMode,

    /// Does the harness stay alive after the current work, awaiting
    /// the next operator push? Default true (matches the chat/worker
    /// expectation). Required for the `agent.Steer` Connect RPC push path.
    /// Mutually exclusive with `invocation = print` (print exits on
    /// completion).
    #[serde(default = "default_keepalive")]
    pub keepalive: bool,

    /// Workspace label (used for slug generation). Power-user override.
    #[serde(default)]
    pub workspace: Option<String>,

    /// Explicit slug. When `None`, derived from `workspace` + a ULID suffix.
    #[serde(default)]
    pub slug: Option<String>,

    /// Explicit branch. When `None`, defaults to `job/<slug>`.
    #[serde(default)]
    pub branch: Option<String>,

    /// Operator-facing path to the spec file (for `ship jobs` display).
    #[serde(default)]
    pub source_spec_path: Option<String>,

    /// Parent job slugs this run depends on.
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,

    /// Per-launch vars. Pipelines pass their own vars through to each
    /// dispatch via the pipeline-run endpoint.
    #[serde(default)]
    pub vars: Option<std::collections::HashMap<String, String>>,
}

pub fn default_open_tab() -> bool { true }
pub fn default_keepalive() -> bool { true }
pub fn default_invocation() -> InvocationMode { InvocationMode::Pty }

/// Validate the (open_tab × invocation × keepalive) triple. Returns
/// an operator-readable error when the combination is invalid.
///
/// The matrix is sparse because print mode is structurally one-shot
/// and has no PTY to attach to. Stream mode has no implemented
/// attach surface today.
pub fn validate_axes(req: &DispatchRequest) -> Result<(), String> {
    match req.invocation {
        InvocationMode::Print => {
            if req.open_tab {
                return Err(
                    "invocation=print is incompatible with open_tab=true \
                     (print mode has no attachable PTY). \
                     Set open_tab=false or change invocation to pty."
                        .to_string(),
                );
            }
            if req.keepalive {
                return Err(
                    "invocation=print is incompatible with keepalive=true \
                     (print mode exits on completion; cannot push more turns). \
                     Set keepalive=false or change invocation to pty."
                        .to_string(),
                );
            }
        }
        InvocationMode::Stream => {
            if req.open_tab {
                return Err(
                    "invocation=stream is incompatible with open_tab=true \
                     (server-mode attach not yet implemented). \
                     Set open_tab=false."
                        .to_string(),
                );
            }
        }
        InvocationMode::Pty => {}
    }
    Ok(())
}

/// Build the operator-facing billing warning when one applies. Today
/// the only triggering combination is `invocation=print` with claude
/// (post-2026-06-19, `claude -p` is billed at API rates regardless of
/// subscription).
pub fn billing_warning_for_axes(
    invocation: InvocationMode,
    harness: Option<&str>,
) -> Option<String> {
    if invocation != InvocationMode::Print {
        return None;
    }
    if harness.unwrap_or("claude") != "claude" {
        return None;
    }
    Some(
        "claude -p is API-billed (subscription plan does not apply). \
         Use invocation=pty for subscription billing."
            .to_string(),
    )
}
