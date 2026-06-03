//! Harness invocation-mode resolution — the domain-free dispatch-slot
//! plumbing the runner uses to decide how a step's process is launched.
//!
//! Mechanism, not policy: it maps a primitive per-step invocation string
//! (and the legacy run-level `headless` flag) onto the substrate
//! [`InvocationMode`] enum. No agent / pipeline / config struct is named.

use ship_substrate::events::agent_session::InvocationMode;

/// Resolve the harness invocation mode for a pipeline step.
///
/// Per-step `invocation` is the source of truth; falls back to the run-level
/// `headless` flag during the transition. `headless == true` historically
/// meant `-p` print mode, so it maps to `Print`.
pub fn resolve_step_invocation(
    step_invocation: Option<&str>,
    run_headless: bool,
) -> InvocationMode {
    match step_invocation {
        Some("print") => InvocationMode::Print,
        Some("stream") => InvocationMode::Stream,
        Some("pty") => InvocationMode::Pty,
        _ if run_headless => InvocationMode::Print,
        _ => InvocationMode::Pty,
    }
}

#[cfg(test)]
mod invocation_tests {
    use super::resolve_step_invocation;
    use ship_substrate::events::agent_session::InvocationMode;

    #[test]
    fn orchestrator_dispatch_invocation_explicit_print() {
        assert_eq!(
            resolve_step_invocation(Some("print"), false),
            InvocationMode::Print,
        );
    }

    #[test]
    fn orchestrator_dispatch_invocation_explicit_stream() {
        assert_eq!(
            resolve_step_invocation(Some("stream"), false),
            InvocationMode::Stream,
        );
    }

    #[test]
    fn orchestrator_dispatch_invocation_explicit_pty() {
        // Explicit "pty" wins even when run_headless is true.
        assert_eq!(
            resolve_step_invocation(Some("pty"), true),
            InvocationMode::Pty,
        );
    }

    #[test]
    fn orchestrator_dispatch_invocation_headless_fallback() {
        // No per-step override, run is headless → Print (the historical mapping).
        assert_eq!(
            resolve_step_invocation(None, true),
            InvocationMode::Print,
        );
    }

    #[test]
    fn orchestrator_dispatch_invocation_interactive_default() {
        // No per-step override, not headless → Pty (interactive).
        assert_eq!(
            resolve_step_invocation(None, false),
            InvocationMode::Pty,
        );
    }

    #[test]
    fn orchestrator_dispatch_invocation_unknown_value_defers_to_headless() {
        // Unrecognised step invocation string falls through to the headless branch.
        assert_eq!(
            resolve_step_invocation(Some("unknown-future-mode"), true),
            InvocationMode::Print,
        );
        assert_eq!(
            resolve_step_invocation(Some("unknown-future-mode"), false),
            InvocationMode::Pty,
        );
    }
}
