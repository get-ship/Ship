//! Tests for `PipelineStepObservedPayload::compute_verdict`.
//!
//! These pin down the evidence-based verdict policy. Each test reads as a
//! sentence about the evidence, not a pile of positional arguments — the
//! `Evidence` helper and `verdict()` shorthand keep that readable. New
//! behavior should add a test that names the case, not tweak an existing
//! one.

use super::*;
use ship_pipelines::types::StepType;

/// Shorthand for compute_verdict with named args, so each test reads as
/// a sentence about the evidence, not a pile of positional arguments.
struct Evidence {
    step_type: StepType,
    done: bool,
    done_result: Option<bool>,
    exit: Option<i32>,
    commits: u32,
    uncommitted: u32,
    trigger: StepTrigger,
}

impl Default for Evidence {
    fn default() -> Self {
        Self {
            step_type: StepType::Agent,
            done: false,
            done_result: None,
            exit: None,
            commits: 0,
            uncommitted: 0,
            trigger: StepTrigger::TurnSettled,
        }
    }
}

fn verdict(e: Evidence) -> StepVerdict {
    PipelineStepObservedPayload::compute_verdict(
        &e.step_type,
        e.done,
        e.done_result,
        e.exit,
        e.commits,
        e.uncommitted,
        &e.trigger,
    )
}

// ─── The one authoritative signal: step_done ───────────────────────────

#[test]
fn done_declared_true_wins_over_everything() {
    // Even with timeout + non-zero exit + no commits, the agent said yes.
    assert_eq!(
        verdict(Evidence {
            done: true,
            done_result: Some(true),
            exit: Some(1),
            trigger: StepTrigger::Timeout,
            ..Default::default()
        }),
        StepVerdict::Advance,
    );
}

#[test]
fn done_declared_false_wins_over_commits() {
    // Agent said it failed. Trust over git state.
    assert_eq!(
        verdict(Evidence {
            done: true,
            done_result: Some(false),
            commits: 10,
            trigger: StepTrigger::DoneDeclared,
            ..Default::default()
        }),
        StepVerdict::Failed,
    );
}

#[test]
fn done_declared_without_result_is_advance() {
    assert_eq!(
        verdict(Evidence {
            done: true,
            done_result: None,
            trigger: StepTrigger::DoneDeclared,
            ..Default::default()
        }),
        StepVerdict::Advance,
    );
}

// ─── Commits are strong positive evidence ──────────────────────────────

// Regression: tmux drops exit_tx after sending one ExitSignal. The monitor
// sees the channel close next recv. With commits present, that's success —
// agent did the work then cleaned up. Pre-fix code failed here.
#[test]
fn commits_with_channel_closed_advance() {
    assert_eq!(
        verdict(Evidence {
            commits: 5,
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        }),
        StepVerdict::Advance,
    );
}

#[test]
fn commits_with_timeout_advance() {
    // Agent timed out but had produced committed work. Advance — the gate
    // can reject if what it did is wrong.
    assert_eq!(
        verdict(Evidence {
            commits: 3,
            trigger: StepTrigger::Timeout,
            ..Default::default()
        }),
        StepVerdict::Advance,
    );
}

#[test]
fn commits_with_nonzero_exit_is_inconclusive() {
    // Agent committed work then died with an error. Gate decides — maybe
    // the implementation is fine and a final test run bailed.
    assert_eq!(
        verdict(Evidence {
            commits: 5,
            exit: Some(127),
            trigger: StepTrigger::ProcessExit,
            ..Default::default()
        }),
        StepVerdict::Inconclusive,
    );
}

// ─── Uncommitted work means "in flight" — never fail ───────────────────

#[test]
fn uncommitted_files_with_timeout_is_inconclusive() {
    // Agent was doing something — we cut it off. Don't punish work in
    // progress; let a human decide.
    assert_eq!(
        verdict(Evidence {
            uncommitted: 4,
            trigger: StepTrigger::Timeout,
            ..Default::default()
        }),
        StepVerdict::Inconclusive,
    );
}

#[test]
fn uncommitted_files_with_channel_close_is_inconclusive() {
    assert_eq!(
        verdict(Evidence {
            uncommitted: 2,
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        }),
        StepVerdict::Inconclusive,
    );
}

// ─── No evidence of work: trigger + exit code decide ───────────────────

#[test]
fn timeout_with_no_evidence_fails() {
    // Nothing produced, nothing dirty, we timed out. Real failure.
    assert_eq!(
        verdict(Evidence {
            trigger: StepTrigger::Timeout,
            ..Default::default()
        }),
        StepVerdict::Failed,
    );
}

#[test]
fn channel_close_no_evidence_nonzero_exit_fails() {
    assert_eq!(
        verdict(Evidence {
            exit: Some(1),
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        }),
        StepVerdict::Failed,
    );
}

#[test]
fn process_exit_no_evidence_zero_exit_is_failed() {
    // Agent exited cleanly without producing anything. Before dev-pipe-01
    // this was Inconclusive (pushed to gate); that let no-op pipeline
    // runs silently proceed. Agents must now call step_done(true) for a
    // legitimate noop — bare channel closure with zero evidence fails.
    assert_eq!(
        verdict(Evidence {
            exit: Some(0),
            trigger: StepTrigger::ProcessExit,
            ..Default::default()
        }),
        StepVerdict::Failed,
    );
}

#[test]
fn turn_settled_with_no_evidence_is_inconclusive() {
    // Agent stopped talking, no commits, no dirty files. Ambiguous.
    assert_eq!(
        verdict(Evidence {
            trigger: StepTrigger::TurnSettled,
            ..Default::default()
        }),
        StepVerdict::Inconclusive,
    );
}

// ─── Exit code alone is not a verdict ──────────────────────────────────

#[test]
fn nonzero_exit_with_commits_does_not_fail() {
    // Specifically asserts: non-zero exit is a signal to weigh, not a
    // verdict. Earlier code short-circuited Failed here; that violated
    // the evidence-based principle.
    let v = verdict(Evidence {
        exit: Some(2),
        commits: 1,
        trigger: StepTrigger::ProcessExit,
        ..Default::default()
    });
    assert!(
        matches!(v, StepVerdict::Inconclusive | StepVerdict::Advance),
        "non-zero exit with commits must not short-circuit to Failed, got {v:?}",
    );
}

// ─── dev-pipe-01 Finding 3.2 regression guards ─────────────────────────

#[test]
fn agent_no_evidence_channel_closed_clean_exit_is_failed() {
    // The exact dev-pipe-01 Finding 3.2 case: the supervisor cut the
    // agent's channel, the agent produced no commits and no dirty files
    // and never called step_done. Before this fix the router advanced
    // via gate review; now it must halt as a failure.
    let v = verdict(Evidence {
        step_type: StepType::Agent,
        exit: Some(0),
        trigger: StepTrigger::ChannelClosed,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Failed);
}

#[test]
fn agent_no_evidence_channel_closed_no_exit_is_failed() {
    // tmux can't see an exit code, so this exact shape shows up a lot in
    // practice. Same verdict: no evidence + external channel cut = fail.
    let v = verdict(Evidence {
        step_type: StepType::Agent,
        trigger: StepTrigger::ChannelClosed,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Failed);
}

#[test]
fn run_step_clean_exit_advances_regardless_of_commits() {
    // `run:` step with `cargo check && cargo test` exit 0 — verify
    // commands produce no commits, and must still advance.
    let v = verdict(Evidence {
        step_type: StepType::Run,
        exit: Some(0),
        trigger: StepTrigger::ProcessExit,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Advance);
}

#[test]
fn run_step_nonzero_exit_fails_regardless_of_commits() {
    // Even with commits, a run step that exits non-zero is a failure —
    // its exit code is the authoritative signal.
    let v = verdict(Evidence {
        step_type: StepType::Run,
        exit: Some(1),
        commits: 5,
        trigger: StepTrigger::ProcessExit,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Failed);
}

#[test]
fn run_step_missing_exit_fails() {
    // Shell process gone with no exit code = failure for run steps.
    let v = verdict(Evidence {
        step_type: StepType::Run,
        trigger: StepTrigger::ChannelClosed,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Failed);
}

#[test]
fn agent_turn_settled_no_evidence_stays_inconclusive() {
    // Activity-based settlement is different from channel closure —
    // the agent might be paused mid-task, not terminated. Gate reviews.
    let v = verdict(Evidence {
        step_type: StepType::Agent,
        trigger: StepTrigger::TurnSettled,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Inconclusive);
}

#[test]
fn agent_done_declared_still_authoritative() {
    // Even with zero commits, an agent that explicitly calls
    // step_done(true) advances. Legitimate noop steps take this path.
    let v = verdict(Evidence {
        step_type: StepType::Agent,
        done: true,
        done_result: Some(true),
        exit: Some(0),
        trigger: StepTrigger::ChannelClosed,
        ..Default::default()
    });
    assert_eq!(v, StepVerdict::Advance);
}
