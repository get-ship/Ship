//! Assertions over the runtime contract text and step timeout budget.
//!
//! These tests guard against regressions in the agent-facing completion
//! contract: the prompt must lead with the completion rule (not a warning),
//! must direct agents to call `step_done` immediately, and must state the
//! 10-minute timeout budget so the agent can plan its verification effort.

use super::prompt_composer::RUNTIME_CONTRACT;

#[test]
fn runtime_contract_starts_with_completion_rule() {
    // The first bold section (between the first and second `**`) is the
    // contract's lead heading — it must be the completion rule, not a warning.
    let first_bold = RUNTIME_CONTRACT.split("**").nth(1).unwrap_or("");
    assert!(
        first_bold.to_lowercase().contains("completion rule"),
        "RUNTIME_CONTRACT must lead with the completion rule, not warnings; \
         first bold section was: {first_bold:?}"
    );
}

#[test]
fn runtime_contract_states_immediate_call() {
    assert!(
        RUNTIME_CONTRACT.to_lowercase().contains("immediately"),
        "RUNTIME_CONTRACT must direct agents to call step_done immediately"
    );
}

#[test]
fn runtime_contract_states_timeout_budget() {
    assert!(
        RUNTIME_CONTRACT.contains("5 minute")
            || RUNTIME_CONTRACT.contains("5-minute")
            || RUNTIME_CONTRACT.contains("5 minutes"),
        "RUNTIME_CONTRACT must state the 5-minute idle timeout"
    );
    assert!(
        RUNTIME_CONTRACT.contains("60 minute")
            || RUNTIME_CONTRACT.contains("60-minute")
            || RUNTIME_CONTRACT.contains("60 minutes"),
        "RUNTIME_CONTRACT must state the 60-minute hard cap"
    );
}

#[test]
fn default_idle_and_hard_cap_match_contract() {
    use crate::services::orchestrator::step_monitor::{DEFAULT_HARD_CAP, DEFAULT_IDLE_TIMEOUT};
    assert_eq!(DEFAULT_IDLE_TIMEOUT.as_secs(), 5 * 60);
    assert_eq!(DEFAULT_HARD_CAP.as_secs(), 60 * 60);
}
