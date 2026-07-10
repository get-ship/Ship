//! `DispatchRequest` — the in-process (floki-class) dispatch shape.
//!
//! Mirrors the slim `DispatchAgentRequest` proto (`agents.proto:232`):
//! `{agent, prompt, model, slug}`. The coding fields (spec / resume /
//! harness / invocation / keepalive / branch / source_spec_path /
//! depends_on / repo_path / base_ref) live on `CodeDispatchRequest` in
//! `code_dispatch_request.rs` and are served by `CodeService.Dispatch` —
//! the in-process path serves floki-class agents only and is simply not
//! reachable for coding-class requests (their harness is not in the
//! resolvable set; see R-A redesign, proposal-ship-code-package.md).
//!
//! Shared plumbing: slug conventions, session creation, the
//! `dispatch.requested` event payload projection. See
//! `dispatch_core.rs::dispatch_agent_session_core`.

use serde::Deserialize;

/// In-process dispatch request — floki-class only. Agent-centric; no
/// pipeline field. Pipeline runs go through `PipelinesService.Dispatch`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchRequest {
    /// Ship-agent profile id (required).
    pub agent: String,

    /// Inline one-shot prompt. The floki runtime injects this as the
    /// first user turn (no spec body concept in-process).
    #[serde(default)]
    pub prompt: Option<String>,

    /// Model override. None → harness default.
    #[serde(default)]
    pub model: Option<String>,

    /// Explicit slug. When `None`, derived from `agent` + a ULID suffix.
    #[serde(default)]
    pub slug: Option<String>,
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

/// True when the prompt input is blank (None or `Some("")`). The
/// in-process dispatch needs at least a non-empty first turn — a job with
/// no prompt is unrecoverable downstream and always wastes a session.
fn prompt_blank(prompt: Option<&str>) -> bool {
    prompt.map(str::trim).map(str::is_empty).unwrap_or(true)
}

/// Validate the in-process dispatch request SHAPE before any side effect.
/// Returns an operator-readable error when the request is malformed, so
/// the wire can map it to `invalid_argument` (400) and skip provisioning.
///
/// Rules (mount-split 2026-07-10):
///   * `agent` must be non-empty
///   * an explicit `slug` must match `[a-z][a-z0-9-]*` (None is OK — the
///     core derives one)
///   * `prompt` must not be blank
///
/// The "agent is in the agent index" check lives at the wire boundary
/// (`rpc_wire::dispatch`) — it needs the `ship_dir` to consult the index,
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
    if prompt_blank(req.prompt.as_deref()) {
        return Err("`prompt` is required (cannot be empty)".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DispatchRequest, is_valid_slug, validate_dispatch_request};

    fn req_with(agent: &str, prompt: Option<&str>, slug: Option<&str>) -> DispatchRequest {
        DispatchRequest {
            agent: agent.to_string(),
            prompt: prompt.map(str::to_string),
            model: None,
            slug: slug.map(str::to_string),
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
        let req = req_with("", Some("do the thing"), None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`agent` is required"), "got: {err}");
    }

    #[test]
    fn whitespace_agent_rejected() {
        let req = req_with("   ", Some("do the thing"), None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`agent` is required"), "got: {err}");
    }

    #[test]
    fn empty_explicit_slug_rejected() {
        let req = req_with("rust-lane", Some("do the thing"), Some(""));
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`slug` must not be empty"), "got: {err}");
    }

    #[test]
    fn bad_slug_rejected() {
        let req = req_with("rust-lane", Some("do the thing"), Some("-bad"));
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(
            err.contains("does not match the required pattern"),
            "got: {err}"
        );
    }

    #[test]
    fn blank_prompt_rejected() {
        let req = req_with("rust-lane", None, None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`prompt` is required"), "got: {err}");
    }

    #[test]
    fn whitespace_only_prompt_rejected() {
        let req = req_with("rust-lane", Some("   \n  "), None);
        let err = validate_dispatch_request(&req).unwrap_err();
        assert!(err.contains("`prompt` is required"), "got: {err}");
    }

    #[test]
    fn prompt_passes() {
        let req = req_with("rust-lane", Some("do the thing"), None);
        assert!(validate_dispatch_request(&req).is_ok());
    }

    #[test]
    fn valid_explicit_slug_passes() {
        let req = req_with(
            "rust-lane",
            Some("do the thing"),
            Some("dispatch-consolidation"),
        );
        assert!(validate_dispatch_request(&req).is_ok());
    }

    #[test]
    fn auto_slug_omitted_passes() {
        // The core derives the slug when None — the slug validator must NOT
        // reject the None case (only the explicit-bad case).
        let req = req_with("rust-lane", Some("do the thing"), None);
        assert!(validate_dispatch_request(&req).is_ok());
    }
}
