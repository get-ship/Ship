//! Job event payload types.
//!
//! Job state is derived entirely from the event log. These payloads are the
//! only things written to the `events` table under the `job.` namespace.

use serde::{Deserialize, Serialize};
use specta::Type;

/// A single phase in a job pipeline (legacy linear format).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelinePhase {
    pub agent: String,
    pub goal: String,
    #[serde(default)]
    pub model: Option<String>,
}

/// Step type in a pipeline DAG.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Agent,
    Gate,
    Merge,
    Human,
    /// Execute a shell command. Exit 0 = success, non-zero = failure.
    Run,
}

/// A single step in a pipeline DAG.
///
/// Custom `Deserialize` allows YAML shorthand: `run: <command>` implies
/// `type: run` and `name:` is accepted as an alias for `id:`.
#[derive(Debug, Clone, Serialize, Type)]
pub struct PipelineStep {
    /// Step identifier, unique within the pipeline.
    pub id: String,
    /// What kind of step this is.
    #[serde(rename = "type")]
    pub step_type: StepType,
    /// Agent profile to use (required for agent/gate steps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Goal or instruction for the agent step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    /// Steps that must complete before this one runs.
    #[serde(default)]
    pub after: Vec<String>,
    /// Model override for this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Provider override for this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Shell command to run (populated for StepType::Run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Maximum number of retries when a gate fails on this step.
    /// `None` or `0` means no retries — gate failure is terminal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
}

impl<'de> serde::Deserialize<'de> for PipelineStep {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            #[serde(alias = "name")]
            id: String,
            #[serde(rename = "type", default)]
            step_type: Option<StepType>,
            #[serde(default)]
            agent: Option<String>,
            #[serde(default)]
            goal: Option<String>,
            #[serde(default)]
            after: Vec<String>,
            #[serde(default)]
            model: Option<String>,
            #[serde(default)]
            provider: Option<String>,
            /// Accept both `run:` (YAML shorthand) and `command:`.
            #[serde(alias = "run", default)]
            command: Option<String>,
            #[serde(default)]
            max_retries: Option<u32>,
        }
        let raw = Raw::deserialize(d)?;
        let step_type = raw.step_type.unwrap_or_else(|| {
            if raw.command.is_some() {
                StepType::Run
            } else {
                StepType::Agent
            }
        });
        Ok(PipelineStep {
            id: raw.id,
            step_type,
            agent: raw.agent,
            goal: raw.goal,
            after: raw.after,
            model: raw.model,
            provider: raw.provider,
            command: raw.command,
            max_retries: raw.max_retries,
        })
    }
}

/// A pipeline definition — a DAG of steps.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
}

impl Pipeline {
    /// Default pipeline: implement → gate → merge.
    pub fn default_for_agent(agent: &str) -> Self {
        Self {
            steps: vec![
                PipelineStep {
                    id: "implement".to_string(),
                    step_type: StepType::Agent,
                    agent: Some(agent.to_string()),
                    goal: None,
                    after: vec![],
                    model: None,
                    provider: None,
                    command: None,
                    max_retries: None,
                },
                PipelineStep {
                    id: "gate".to_string(),
                    step_type: StepType::Gate,
                    agent: Some("reviewer".to_string()),
                    goal: None,
                    after: vec!["implement".to_string()],
                    model: Some("sonnet".to_string()),
                    provider: None,
                    command: None,
                    max_retries: None,
                },
                PipelineStep {
                    id: "merge".to_string(),
                    step_type: StepType::Merge,
                    agent: None,
                    goal: None,
                    after: vec!["gate".to_string()],
                    model: None,
                    provider: None,
                    command: None,
                    max_retries: None,
                },
            ],
        }
    }

    /// Find root steps (no dependencies).
    pub fn root_steps(&self) -> Vec<&PipelineStep> {
        self.steps.iter().filter(|s| s.after.is_empty()).collect()
    }

    /// Find steps unblocked by completed step IDs.
    pub fn unblocked_steps(&self, completed: &[String]) -> Vec<&PipelineStep> {
        self.steps.iter()
            .filter(|s| !completed.contains(&s.id))
            .filter(|s| s.after.iter().all(|dep| completed.contains(dep)))
            .collect()
    }
}

/// Emitted when a job is created. `entity_id` in the envelope is the job_id.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobCreatedPayload {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    pub plan_id: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
    /// Legacy linear pipeline. Kept for backward compat.
    #[serde(default)]
    pub pipeline: Option<Vec<PipelinePhase>>,
    /// DAG pipeline. Takes precedence over `pipeline` if both are set.
    #[serde(default)]
    pub steps: Option<Pipeline>,
    /// Pipeline name reference (e.g. "tdd"). Resolved by daemon from .ship/pipelines/.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_name: Option<String>,
    /// Job-level variables for pipeline step interpolation. Override step-level vars.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vars: Option<std::collections::HashMap<String, String>>,
    /// Inline spec content. When present, the worktree adapter writes this to
    /// `spec_path` at dispatch time. Makes jobs self-contained across worktree
    /// boundaries — no reliance on gitignored files existing in the main checkout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_content: Option<String>,
}

/// Emitted when the job is assigned to a worktree and (optionally) a process.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobDispatchedPayload {
    pub job_id: String,
    pub worktree: String,
    pub pid: Option<u32>,
    /// Provider-specific session metadata (e.g. Claude session ID for --resume).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_meta: Option<serde_json::Value>,
}

/// Emitted when a gate review is requested for the job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobGateRequestedPayload {
    pub job_id: String,
    pub gate_agent: String,
}

/// Emitted when the gate review passes.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobGatePassedPayload {
    pub job_id: String,
}

/// Emitted when the gate review fails.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobGateFailedPayload {
    pub job_id: String,
    pub reason: String,
}

/// Emitted when the job is blocked and cannot proceed without resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobBlockedPayload {
    pub job_id: String,
    /// Human-readable description of what is blocking progress.
    pub blocker: String,
    pub needs_human: bool,
}

/// Emitted when the agent signals the job is done but before gate/merge.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobCompletedPayload {
    pub job_id: String,
    pub slug: String,
}

/// Emitted when the job's branch is merged.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobMergedPayload {
    pub job_id: String,
    pub slug: String,
}

/// Emitted when the job terminates with an error.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobFailedPayload {
    pub job_id: String,
    pub error: String,
}

/// Emitted by either side (human, commander, or agent) to send a mid-flight
/// update, instruction, or course correction. The daemon routes these to the
/// agent's mailbox.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobUpdatePayload {
    pub job_id: String,
    pub message: String,
    /// Who sent this update — e.g. "human", "commander", agent mesh ID.
    pub sender: String,
}

/// Emitted when a pipeline step begins executing.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepDispatchedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub step_type: String,
}

/// Emitted when a pipeline step finishes successfully.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepCompletedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
}

/// Emitted when a pipeline step fails. Halts pipeline advancement.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepFailedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub error: String,
}

/// Emitted by run steps to capture stdout/stderr output.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepOutputPayload {
    pub job_id: String,
    pub step_id: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Emitted by an agent via the `step_done` MCP tool to declare step completion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepDoneDeclaredPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub summary: Option<String>,
}

/// Emitted when a gated step is retried after gate failure.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepRetryPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub step_name: String,
    pub attempt: u32,
    pub reason: String,
}

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
pub struct StepObservedPayload {
    pub job_id: String,
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
}

impl StepObservedPayload {
    /// Derive a verdict from the collected evidence.
    pub fn compute_verdict(
        done_declared: bool,
        done_result: Option<bool>,
        exit_code: Option<i32>,
        commits: u32,
        trigger: &StepTrigger,
    ) -> StepVerdict {
        // 1. Agent explicitly declared outcome — strongest signal.
        if done_declared {
            return match done_result {
                Some(true) => StepVerdict::Advance,
                Some(false) => StepVerdict::Failed,
                None => StepVerdict::Advance, // called step_done without explicit false
            };
        }
        // 2. Non-zero exit code — unambiguous failure.
        if let Some(code) = exit_code
            && code != 0
        {
            return StepVerdict::Failed;
        }
        // 3. Evidence beats trigger. If the agent committed work, that's
        //    advancement — regardless of how the monitor found out the agent
        //    was done (exit signal, turn-settled, channel closed, timeout).
        //    The tmux executor normally drops exit_tx after sending one signal,
        //    so ChannelClosed is a routine cleanup path, not a failure mode.
        if commits > 0 {
            return StepVerdict::Advance;
        }
        // 4. No commits + timeout/channel-closed → real failure. Agent ran out
        //    of time or died without producing anything.
        if matches!(trigger, StepTrigger::Timeout | StepTrigger::ChannelClosed) {
            return StepVerdict::Failed;
        }
        // 5. No commits, no step_done, no error, clean settle — inconclusive.
        //    Pipeline router advances anyway (gate will catch problems).
        StepVerdict::Inconclusive
    }
}

/// Event type constants for the `job.` namespace.
pub mod event_types {
    pub const JOB_CREATED: &str = "job.created";
    pub const JOB_DISPATCHED: &str = "job.dispatched";
    pub const JOB_GATE_REQUESTED: &str = "job.gate_requested";
    pub const JOB_GATE_PASSED: &str = "job.gate_passed";
    pub const JOB_GATE_FAILED: &str = "job.gate_failed";
    pub const JOB_BLOCKED: &str = "job.blocked";
    pub const JOB_COMPLETED: &str = "job.completed";
    pub const JOB_MERGED: &str = "job.merged";
    pub const JOB_FAILED: &str = "job.failed";
    pub const JOB_UPDATE: &str = "job.update";
    pub const STEP_DISPATCHED: &str = "job.step_dispatched";
    pub const STEP_COMPLETED: &str = "job.step_completed";
    pub const STEP_FAILED: &str = "job.step_failed";
    pub const STEP_OUTPUT: &str = "job.step_output";
    pub const STEP_RETRY: &str = "step.retry";
    pub const STEP_DONE_DECLARED: &str = "step.done_declared";
    pub const STEP_OBSERVED: &str = "step.observed";
}

#[cfg(test)]
mod verdict_tests {
    use super::*;

    fn verdict(
        done: bool,
        done_result: Option<bool>,
        exit: Option<i32>,
        commits: u32,
        trigger: StepTrigger,
    ) -> StepVerdict {
        StepObservedPayload::compute_verdict(done, done_result, exit, commits, &trigger)
    }

    // Regression test for the original bug: tmux executor drops exit_tx after
    // sending one ExitSignal. The step monitor sees the channel close and
    // treats it as `ChannelClosed`. With commits present, that is SUCCESS —
    // the agent did the work then cleaned up. Previous code returned Failed
    // here, breaking every tmux-backed pipeline.
    #[test]
    fn channel_closed_with_commits_advances() {
        assert_eq!(
            verdict(false, None, None, 5, StepTrigger::ChannelClosed),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn channel_closed_without_commits_fails() {
        assert_eq!(
            verdict(false, None, None, 0, StepTrigger::ChannelClosed),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn timeout_with_commits_advances() {
        assert_eq!(
            verdict(false, None, None, 3, StepTrigger::Timeout),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn timeout_without_commits_fails() {
        assert_eq!(
            verdict(false, None, None, 0, StepTrigger::Timeout),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn done_declared_true_wins_over_everything() {
        // Even with 0 commits and a timeout, explicit success is success.
        assert_eq!(
            verdict(true, Some(true), Some(1), 0, StepTrigger::Timeout),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn done_declared_false_wins_over_commits() {
        // Agent said it failed. Trust the agent over the git state.
        assert_eq!(
            verdict(true, Some(false), None, 10, StepTrigger::DoneDeclared),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn nonzero_exit_code_fails() {
        assert_eq!(
            verdict(false, None, Some(127), 5, StepTrigger::ProcessExit),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn turn_settled_with_commits_advances() {
        assert_eq!(
            verdict(false, None, None, 2, StepTrigger::TurnSettled),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn clean_settle_no_evidence_is_inconclusive() {
        // Agent exited 0 with no commits and didn't call step_done — we have
        // no evidence either way. Router will advance through gate.
        assert_eq!(
            verdict(false, None, Some(0), 0, StepTrigger::ProcessExit),
            StepVerdict::Inconclusive,
        );
    }
}
