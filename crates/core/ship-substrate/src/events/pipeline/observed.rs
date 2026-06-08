//! Step-observation payload and verdict derivation.
//!
//! `PipelineStepObservedPayload` is the evidence-based report emitted by the
//! step monitor after gathering signals about a step's outcome (process exit,
//! `step_done`, git state). The pipeline router consumes this — it does not
//! decide verdicts on its own. `compute_verdict` is the single source of
//! truth for translating evidence into a `StepVerdict`.

use serde::{Deserialize, Serialize};
use specta::Type;

use ship_pipelines::types::StepType;


/// What triggered the step observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum StepTrigger {
    /// Agent explicitly called step_done
    DoneDeclared,
    /// Process/session exited
    ProcessExit,
    /// Activity settled — turns ended, evidence accumulated, no new activity
    TurnSettled,
    /// Step timed out
    Timeout,
    /// Exit channel closed unexpectedly
    ChannelClosed,
}

/// Verdict derived from accumulated evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum StepVerdict {
    /// Strong evidence of success — advance pipeline
    Advance,
    /// Strong evidence of failure — halt pipeline
    Failed,
    /// Inconclusive — needs human review or gate
    Inconclusive,
}

/// Emitted by the step monitor after gathering evidence about a step's outcome.
///
/// This is evidence-based, not verdict-based. The step monitor collects all
/// available signals (process exit, step_done, git state) and the pipeline
/// router decides what to do based on the full picture.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepObservedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_index: usize,
    pub step_name: String,
    /// What triggered this observation.
    pub trigger: StepTrigger,
    // ── Evidence fields ──────────────────────────────────────────────
    /// Agent explicitly called step_done (strongest signal).
    pub done_declared: bool,
    /// Value passed to step_done, if called. True = agent says success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_result: Option<bool>,
    /// Process exit code, if available. None for tmux (no exit code exposed).
    pub exit_code: Option<i32>,
    /// Number of git commits on the branch since the step started.
    #[serde(default)]
    pub commits_since_start: u32,
    /// Number of files with uncommitted changes in the worktree.
    #[serde(default)]
    pub uncommitted_files: u32,
    /// Wall-clock duration of the step.
    pub duration_secs: u64,
    /// Human-readable reason (e.g. "timeout", error message).
    pub reason: Option<String>,
    // ── Derived ──────────────────────────────────────────────────────
    /// Verdict computed from evidence. Used by pipeline router.
    pub verdict: StepVerdict,
    /// Back-compat: true when verdict == Advance. Existing consumers and
    /// projections that read `succeeded` continue to work.
    #[serde(default)]
    pub succeeded: bool,
    /// HEAD SHA at step start — baseline for post-step diff gate evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_sha: Option<String>,
    /// Last ~30 lines of the agent's stdout/stderr log. Populated only on
    /// non-Advance verdicts so operators can see *why* a step failed
    /// without SSHing into the worktree. ANSI escapes stripped. Harness-
    /// agnostic — works for any executor that writes to
    /// `.artifacts/agent-<slug>.log` (ProcessExecutor does).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_tail: Option<String>,
}

impl PipelineStepObservedPayload {
    /// Derive a verdict from the collected evidence.
    ///
    /// Philosophy: no single signal maps to a verdict. Triggers, exit codes,
    /// commits, uncommitted files, and step_done are evidence to weigh. The
    /// verdict is Advance only when evidence is clearly positive, Failed only
    /// when clearly negative, Inconclusive otherwise — the router will push
    /// Inconclusive outcomes through a gate step where the reviewer decides.
    ///
    /// The only signal that maps 1-1 to a verdict is `step_done(bool)` —
    /// an explicit declaration from the agent itself.
    ///
    /// **Step contract for agents:** if your step produces no commits and no
    /// uncommitted changes, you must call `step_done(true)` explicitly to mark
    /// success. Channel closure or process exit with zero evidence is treated
    /// as failure — this pins down previously-implicit "noop step" semantics
    /// surfaced by dev-pipe-01 Finding 3.2.
    ///
    /// **Run steps short-circuit on exit code.** A `StepType::Run` step has no
    /// agent and no `step_done` mechanism; its shell exit code is authoritative.
    pub fn compute_verdict(
        step_type: &StepType,
        done_declared: bool,
        done_result: Option<bool>,
        exit_code: Option<i32>,
        commits: u32,
        uncommitted_files: u32,
        trigger: &StepTrigger,
    ) -> StepVerdict {
        // Run steps have no agent and no step_done mechanism. The shell exit
        // code is the whole story — route around the evidence machinery so
        // `cargo check` style verify steps don't require commits to advance.
        if matches!(step_type, StepType::Run) {
            return match exit_code {
                Some(0) => StepVerdict::Advance,
                _ => StepVerdict::Failed,
            };
        }

        // Agent explicitly declared outcome. Trust it. Only authoritative rule.
        if done_declared {
            return match done_result {
                Some(true) | None => StepVerdict::Advance,
                Some(false) => StepVerdict::Failed,
            };
        }

        let nonzero_exit = matches!(exit_code, Some(c) if c != 0);
        let has_commits = commits > 0;
        let has_uncommitted = uncommitted_files > 0;

        // Committed work present. The agent produced something durable.
        // Non-zero exit modifies — agent committed but died angry, gate reviews.
        if has_commits {
            return if nonzero_exit {
                StepVerdict::Inconclusive
            } else {
                StepVerdict::Advance
            };
        }

        // No commits. Uncommitted work means the agent was actively doing
        // something — might have been cut short. Never fail on mid-flight
        // work; let a human or gate decide.
        if has_uncommitted {
            return StepVerdict::Inconclusive;
        }

        // Nothing produced. Weigh the trigger and exit code together.
        match trigger {
            StepTrigger::Timeout => StepVerdict::Failed,
            // No commits, no uncommitted files, no done_declared, and the
            // agent's channel was cut or the process exited. The agent
            // produced zero evidence of any work. Treat as failure — the
            // agent must call step_done(true) to mark a legitimate noop.
            StepTrigger::ChannelClosed | StepTrigger::ProcessExit => StepVerdict::Failed,
            StepTrigger::DoneDeclared => {
                // done_declared == false already handled above. If we get here
                // with DoneDeclared trigger, something is off.
                StepVerdict::Inconclusive
            }
            // Activity-based settlement is different — turns ended but the
            // agent might still have pending work. Leave for gate review
            // rather than failing outright.
            StepTrigger::TurnSettled => StepVerdict::Inconclusive,
        }
    }
}
