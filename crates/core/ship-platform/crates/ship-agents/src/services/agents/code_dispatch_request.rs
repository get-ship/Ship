//! `CodeDispatchRequest` — the coding-class dispatch shape.
//!
//! Mirrors `ship.code.v1.CodeDispatchRequest` (the coding-agent capability
//! surface — `proto/ship/code/v1/code.proto:22`). Lives on `CodeService`
//! which is mounted behind `#[cfg(feature = "bridge")]` at the router
//! assembly (or unconditionally on the helmd testbed). All the fields the
//! standard `DispatchRequest` reserved are here:
//! `spec / source_spec_path / resume / harness / invocation / keepalive
//! / branch / depends_on / repo_path / base_ref` plus the four standard
//! `agent / prompt / model / slug`.
//!
//! Shared plumbing: slug conventions, session creation, the
//! `dispatch.requested` event payload projection. See
//! `dispatch_core.rs::dispatch_code_session_core`.

use agent_schema::InvocationMode;
use serde::Deserialize;

/// Coding-class dispatch request — wired by `CodeService.Dispatch`.
///
/// The agent / prompt / model / slug quadruple mirrors the standard
/// `DispatchRequest` (so the agent profile resolution, prompt content
/// channel, model override, and slug convention are identical across the
/// two surfaces). The other ten fields are coding-class only: a coding
/// dispatch has to know which harness to launch, how (pty/print/stream),
/// whether to keep the session alive, which branch to worktree from,
/// which repo to fork, what spec to deliver, etc. None of those exist
/// in-process.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeDispatchRequest {
    // ── Standard four (mirror `DispatchRequest`) ───────────────────────
    /// Ship-agent profile id (required).
    pub agent: String,

    /// Inline one-shot prompt. Becomes the harness's `autostart_prompt`
    /// (the first user turn delivered into the TUI once the ready
    /// pattern matches). Mirrors the standard field name.
    #[serde(default)]
    pub prompt: Option<String>,

    /// Model override. None → harness default.
    #[serde(default)]
    pub model: Option<String>,

    /// Explicit slug. When `None`, derived from `agent` + a ULID suffix.
    #[serde(default)]
    pub slug: Option<String>,

    // ── Coding-class only ──────────────────────────────────────────────
    /// Job-spec markdown delivered into the worktree.
    #[serde(default)]
    pub spec: Option<String>,

    /// Operator-facing path to the spec file (for `ship jobs` display).
    #[serde(default)]
    pub source_spec_path: Option<String>,

    /// Resume the harness session recorded for this slug.
    #[serde(default)]
    pub resume: bool,

    /// Harness override for this dispatch. None → agent definition's
    /// harness. Empty string also means "use the agent default" (the
    /// wire's "empty = default" convention).
    #[serde(default)]
    pub harness: Option<String>,

    /// How the harness is invoked:
    ///   - pty (default): PTY-attached harness invocation (claude TUI,
    ///     codex TUI, ...). Subscription-billed for claude.
    ///   - print: one-shot non-interactive mode (claude -p, codex exec).
    ///     API-billed for claude post-2026-06-19.
    ///   - stream: server-mode (codex app-server, opencode REST).
    ///     Only codex/opencode today.
    #[serde(default = "default_invocation")]
    pub invocation: InvocationMode,

    /// Does the harness stay alive after the current work, awaiting
    /// the next operator push? Default true (matches the chat/worker
    /// expectation). Mutually exclusive with `invocation = print`
    /// (print exits on completion).
    #[serde(default = "default_keepalive")]
    pub keepalive: bool,

    /// Explicit branch. When `None`, defaults to `job/<slug>`.
    #[serde(default)]
    pub branch: Option<String>,

    /// Slugs this run depends on. Pipelines carry their dependencies
    /// through to each step's dispatch; bare single-agent dispatch is
    /// still a valid shape (an empty list means "no deps").
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
    /// Absolute path to the repo working tree this job's worktree forks
    /// from. Per-call, never persisted. Empty / None → the daemon's
    /// default repo.
    #[serde(default)]
    pub repo_path: Option<String>,

    /// Ref the job's worktree branches from. Empty / None → that
    /// repo's HEAD.
    #[serde(default)]
    pub base_ref: Option<String>,
}

pub fn default_invocation() -> InvocationMode {
    InvocationMode::Pty
}

pub fn default_keepalive() -> bool {
    true
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

/// Validate the (open_tab × invocation × keepalive) triple. Returns an
/// operator-readable error when the combination is invalid.
///
/// The matrix is sparse because print mode is structurally one-shot
/// and has no PTY to attach to. Stream mode has no implemented
/// attach surface today.
pub fn validate_axes(req: &CodeDispatchRequest) -> Result<(), String> {
    match req.invocation {
        InvocationMode::Print => {
            if req.keepalive {
                return Err("invocation=print is incompatible with keepalive=true \
                     (print mode exits on completion; cannot push more turns). \
                     Set keepalive=false or change invocation to pty."
                    .to_string());
            }
        }
        InvocationMode::Stream => {
            // Stream mode has no implemented attach surface; bare
            // single-agent dispatch never opens a studio tab.
        }
        InvocationMode::Pty => {}
    }
    Ok(())
}

/// True when the optional spec/prompt inputs are both empty (None or
/// `Some("")`). The coding dispatch path needs at least one of them —
/// a job with no spec body and no first-turn prompt is unrecoverable
/// downstream and always wastes a session.
fn both_inputs_empty(spec: Option<&str>, prompt: Option<&str>) -> bool {
    fn is_blank(s: Option<&str>) -> bool {
        s.map(str::trim).map(str::is_empty).unwrap_or(true)
    }
    is_blank(spec) && is_blank(prompt)
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

/// Validate the coding-class dispatch request SHAPE before any side
/// effect. Returns an operator-readable error when the request is
/// malformed, so the wire can map it to `invalid_argument` (400) and
/// skip provisioning.
///
/// Rules (mount-split 2026-07-10):
///   * `agent` must be non-empty
///   * an explicit `slug` must match `[a-z][a-z0-9-]*` (None is OK — the
///     core derives one)
///   * `spec` and `prompt` must not BOTH be blank (mirrors the
///     pre-split behavior — a coding job with neither is meaningless)
///   * the (invocation × keepalive) triple must be consistent
///
/// The "agent is in the agent index" check lives at the wire boundary
/// (`code_wire::dispatch`) — it needs the `ship_dir` to consult the
/// index, which this transport-neutral layer doesn't have.
pub fn validate_code_dispatch_request(req: &CodeDispatchRequest) -> Result<(), String> {
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
    validate_axes(req)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CodeDispatchRequest, billing_warning_for_axes, is_valid_slug,
        validate_code_dispatch_request,
    };
    use agent_schema::InvocationMode;

    fn req_with(
        agent: &str,
        spec: Option<&str>,
        prompt: Option<&str>,
        slug: Option<&str>,
    ) -> CodeDispatchRequest {
        CodeDispatchRequest {
            agent: agent.to_string(),
            spec: spec.map(str::to_string),
            prompt: prompt.map(str::to_string),
            model: None,
            slug: slug.map(str::to_string),
            source_spec_path: None,
            resume: false,
            harness: None,
            invocation: InvocationMode::Pty,
            keepalive: true,
            branch: None,
            depends_on: None,
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
        assert!(!is_valid_slug("UpperCase"));
        assert!(!is_valid_slug("with space"));
        assert!(!is_valid_slug("with/slash"));
        assert!(!is_valid_slug("1leading-digit"));
    }

    // ── validate_code_dispatch_request: rule coverage ───────────────────

    #[test]
    fn empty_agent_rejected() {
        let req = req_with("", Some("spec body"), None, None);
        let err = validate_code_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`agent` is required"), "got: {err}");
    }

    #[test]
    fn empty_explicit_slug_rejected() {
        let req = req_with("rust-lane", Some("spec"), None, Some(""));
        let err = validate_code_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`slug` must not be empty"), "got: {err}");
    }

    #[test]
    fn bad_slug_rejected() {
        let req = req_with("rust-lane", Some("spec"), None, Some("-bad"));
        let err = validate_code_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("does not match the required pattern"),
            "got: {err}"
        );
    }

    #[test]
    fn both_spec_and_prompt_empty_rejected() {
        let req = req_with("rust-lane", None, None, None);
        let err = validate_code_dispatch_request(&req).unwrap_err();
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
        let err = validate_code_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("at least one of `spec` or `prompt`"),
            "got: {err}"
        );
    }

    #[test]
    fn spec_only_passes() {
        let req = req_with("rust-lane", Some("do the thing"), None, None);
        assert!(validate_code_dispatch_request(&req).is_ok());
    }

    #[test]
    fn prompt_only_passes() {
        let req = req_with("rust-lane", None, Some("first user turn"), None);
        assert!(validate_code_dispatch_request(&req).is_ok());
    }

    #[test]
    fn both_inputs_passes() {
        let req = req_with("rust-lane", Some("spec body"), Some("first turn"), None);
        assert!(validate_code_dispatch_request(&req).is_ok());
    }

    #[test]
    fn print_mode_with_keepalive_rejected() {
        let mut req = req_with("rust-lane", Some("spec body"), None, None);
        req.invocation = InvocationMode::Print;
        req.keepalive = true;
        let err = validate_code_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("print is incompatible with keepalive"),
            "got: {err}"
        );
    }

    #[test]
    fn print_mode_without_keepalive_passes() {
        let mut req = req_with("rust-lane", Some("spec body"), None, None);
        req.invocation = InvocationMode::Print;
        req.keepalive = false;
        assert!(validate_code_dispatch_request(&req).is_ok());
    }

    // ── billing warning ───────────────────────────────────────────────

    #[test]
    fn billing_warning_only_for_claude_print() {
        assert!(billing_warning_for_axes(InvocationMode::Print, Some("claude")).is_some());
        assert!(billing_warning_for_axes(InvocationMode::Print, None).is_some()); // default = claude
        assert!(billing_warning_for_axes(InvocationMode::Print, Some("codex")).is_none());
        assert!(billing_warning_for_axes(InvocationMode::Pty, Some("claude")).is_none());
        assert!(billing_warning_for_axes(InvocationMode::Stream, Some("claude")).is_none());
    }
}
