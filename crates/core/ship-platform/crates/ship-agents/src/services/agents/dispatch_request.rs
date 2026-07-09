//! DispatchRequest wire type and validation helpers.

use agent_def::InvocationMode;
use serde::Deserialize;

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

    /// Continue the last session for this agent. Errors with 400 when no
    /// matching session exists.
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

    /// Explicit slug. When `None`, derived from `agent` + a ULID suffix.
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

    /// Absolute path to the repo working tree the job's worktree forks
    /// from. Per-call; meaningful to the coding-agent execution class
    /// only. Replaces the field-10 `workspace` slug label (a dead
    /// concept that confused "workspace" with "repo" and "project"
    /// alike). `None` falls back to the daemon's default repo.
    /// Mirrors the wire's `repo_path` (field 16).
    #[serde(default)]
    pub repo_path: Option<String>,

    /// Ref the job's worktree branches from. Per-call; same lifetime
    /// rules as `repo_path`. `None` / empty falls back to the repo's
    /// HEAD — matches `git worktree add -b <branch> <path>` with no
    /// base. Mirrors the wire's `base_ref` (field 17).
    #[serde(default)]
    pub base_ref: Option<String>,
}

pub fn default_open_tab() -> bool {
    true
}
pub fn default_keepalive() -> bool {
    true
}
pub fn default_invocation() -> InvocationMode {
    InvocationMode::Pty
}

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
                return Err("invocation=print is incompatible with open_tab=true \
                     (print mode has no attachable PTY). \
                     Set open_tab=false or change invocation to pty."
                    .to_string());
            }
            if req.keepalive {
                return Err("invocation=print is incompatible with keepalive=true \
                     (print mode exits on completion; cannot push more turns). \
                     Set keepalive=false or change invocation to pty."
                    .to_string());
            }
        }
        InvocationMode::Stream => {
            if req.open_tab {
                return Err("invocation=stream is incompatible with open_tab=true \
                     (server-mode attach not yet implemented). \
                     Set open_tab=false."
                    .to_string());
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

/// Slug shape: `[a-z][a-z0-9-]*` — must start with a lowercase ASCII letter,
/// followed by zero or more lowercase ASCII letters / digits / dashes.
/// Bare `job` (no slash) is NOT covered here; the dispatch core derives a
/// `<agent>-<ulid>` slug when the caller passes `None`, so this rule
/// only fires against an explicit, caller-supplied slug.
pub fn is_valid_slug(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// True when the optional spec/prompt inputs are both empty (None or
/// `Some("")`). The dispatch path needs at least one of them — a job with
/// no spec body and no first-turn prompt is unrecoverable downstream and
/// always wastes a session.
fn both_inputs_empty(spec: Option<&str>, prompt: Option<&str>) -> bool {
    fn is_blank(s: Option<&str>) -> bool {
        s.map(str::trim).map(str::is_empty).unwrap_or(true)
    }
    is_blank(spec) && is_blank(prompt)
}

/// Validate the dispatch request SHAPE before any side effect. Returns an
/// operator-readable error when the request is malformed, so the wire can
/// map it to `invalid_argument` (400) and skip provisioning.
///
/// Rules (c-2026-07-05T12-33-00-000Z-a1b2c3d4):
///   * `agent` must be non-empty
///   * an explicit `slug` must match `[a-z][a-z0-9-]*` (None is OK — the
///     core derives one)
///   * `spec` and `prompt` must not BOTH be blank
///
/// The "agent is in the agent index" check lives at the wire boundary
/// (rpc_wire::dispatch) — it needs the `ship_dir` to consult the index,
/// which this transport-neutral layer doesn't have.
pub fn validate_dispatch_request(req: &DispatchRequest) -> Result<(), String> {
    if req.agent.trim().is_empty() {
        return Err("`agent` is required".to_string());
    }
    if let Some(slug) = req.slug.as_deref() {
        if slug.is_empty() {
            return Err("`slug` must not be empty when provided".to_string());
        }
        if !is_valid_slug(slug) {
            return Err(format!(
                "`slug` {slug:?} does not match the required pattern [a-z][a-z0-9-]*"
            ));
        }
    }
    if both_inputs_empty(req.spec.as_deref(), req.prompt.as_deref()) {
        return Err("at least one of `spec` or `prompt` is required (both empty)".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DispatchRequest, is_valid_slug, validate_dispatch_request};

    fn req_with(
        agent: &str,
        spec: Option<&str>,
        prompt: Option<&str>,
        slug: Option<&str>,
    ) -> DispatchRequest {
        DispatchRequest {
            agent: agent.to_string(),
            spec: spec.map(str::to_string),
            prompt: prompt.map(str::to_string),
            resume: false,
            harness: None,
            model: None,
            open_tab: true,
            invocation: agent_def::InvocationMode::Pty,
            keepalive: true,
            slug: slug.map(str::to_string),
            branch: None,
            source_spec_path: None,
            depends_on: None,
            vars: None,
            repo_path: None,
            base_ref: None,
        }
    }

    // ── slug regex ───────────────────────────────────────────────────────

    #[test]
    fn slug_accepts_canonical_shapes() {
        assert!(is_valid_slug("a"));
        assert!(is_valid_slug("dispatch-consolidation"));
        assert!(is_valid_slug("tdd-01kp9abd"));
        assert!(is_valid_slug("rust-lane"));
    }

    #[test]
    fn slug_rejects_bad_shapes() {
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("-leading-dash"));
        // Trailing-dash IS valid per the [a-z][a-z0-9-]* regex (the spec
        // doesn't forbid it; downstream consumers can sanitize if they need to).
        assert!(is_valid_slug("trailing-dash-"));
        assert!(!is_valid_slug("UpperCase"));
        assert!(!is_valid_slug("with space"));
        assert!(!is_valid_slug("with/slash"));
        assert!(!is_valid_slug("1leading-digit"));
        assert!(!is_valid_slug("with_underscore"));
    }

    // ── validate_dispatch_request: rule coverage ─────────────────────────

    #[test]
    fn empty_agent_rejected() {
        let req = req_with("", Some("spec body"), None, None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`agent` is required"), "got: {err}");
    }

    #[test]
    fn whitespace_agent_rejected() {
        let req = req_with("   ", Some("spec body"), None, None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`agent` is required"), "got: {err}");
    }

    #[test]
    fn empty_explicit_slug_rejected() {
        let req = req_with("rust-lane", Some("spec"), None, Some(""));
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`slug` must not be empty"), "got: {err}");
    }

    #[test]
    fn bad_slug_rejected() {
        let req = req_with("rust-lane", Some("spec"), None, Some("-bad"));
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("does not match the required pattern"),
            "got: {err}"
        );
    }

    #[test]
    fn both_spec_and_prompt_empty_rejected() {
        let req = req_with("rust-lane", None, None, None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("at least one of `spec` or `prompt`"),
            "got: {err}"
        );
    }

    #[test]
    fn both_spec_and_prompt_blank_rejected() {
        // Whitespace-only inputs are also "blank" — the operator didn't actually
        // write a body. Cheap to defend against here, expensive to debug later.
        let req = req_with("rust-lane", Some("   "), Some("\n"), None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("at least one of `spec` or `prompt`"),
            "got: {err}"
        );
    }

    #[test]
    fn spec_only_passes() {
        let req = req_with("rust-lane", Some("do the thing"), None, None);
        assert!(validate_dispatch_request(&req).is_ok());
    }

    #[test]
    fn prompt_only_passes() {
        let req = req_with("rust-lane", None, Some("first user turn"), None);
        assert!(validate_dispatch_request(&req).is_ok());
    }

    #[test]
    fn both_inputs_passes() {
        let req = req_with("rust-lane", Some("spec body"), Some("first turn"), None);
        assert!(validate_dispatch_request(&req).is_ok());
    }

    #[test]
    fn valid_explicit_slug_passes() {
        let req = req_with(
            "rust-lane",
            Some("spec"),
            None,
            Some("dispatch-consolidation"),
        );
        assert!(validate_dispatch_request(&req).is_ok());
    }

    #[test]
    fn auto_slug_omitted_passes() {
        // The core derives the slug when None — the slug validator must NOT
        // reject the None case (only the explicit-bad case).
        let req = req_with("rust-lane", Some("spec"), None, None);
        assert!(validate_dispatch_request(&req).is_ok());
    }
}
