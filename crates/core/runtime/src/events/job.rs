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

/// Emitted by the step monitor after observing process exit and/or step_done.
/// This is the authoritative signal for pipeline advancement.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepObservedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_index: usize,
    pub step_name: String,
    pub succeeded: bool,
    pub exit_code: Option<i32>,
    pub duration_secs: u64,
    pub done_declared: bool,
    pub reason: Option<String>,
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
