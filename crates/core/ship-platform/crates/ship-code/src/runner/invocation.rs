//! Harness invocation-mode resolution.
//! Moved from ship-runner (L0) to ship-code (L1).

use ship_substrate::events::agent_session::InvocationMode;

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
        assert_eq!(resolve_step_invocation(Some("print"), false), InvocationMode::Print);
    }

    #[test]
    fn orchestrator_dispatch_invocation_explicit_stream() {
        assert_eq!(resolve_step_invocation(Some("stream"), false), InvocationMode::Stream);
    }

    #[test]
    fn orchestrator_dispatch_invocation_explicit_pty() {
        assert_eq!(resolve_step_invocation(Some("pty"), true), InvocationMode::Pty);
    }

    #[test]
    fn orchestrator_dispatch_invocation_headless_fallback() {
        assert_eq!(resolve_step_invocation(None, true), InvocationMode::Print);
    }

    #[test]
    fn orchestrator_dispatch_invocation_interactive_default() {
        assert_eq!(resolve_step_invocation(None, false), InvocationMode::Pty);
    }

    #[test]
    fn orchestrator_dispatch_invocation_unknown_value_defers_to_headless() {
        assert_eq!(resolve_step_invocation(Some("unknown-future-mode"), true), InvocationMode::Print);
        assert_eq!(resolve_step_invocation(Some("unknown-future-mode"), false), InvocationMode::Pty);
    }
}
