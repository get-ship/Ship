//! Run-level and step-level payload structs for `pipeline.*` events.
//!
//! These are the only payloads written to the `events` table under the
//! `pipeline.` namespace. Structs are kept flat and individually documented
//! because the event log is the durable wire format — field names and serde
//! behavior are part of the contract.

use serde::{Deserialize, Serialize};
use specta::Type;

// DAG types re-exported at the module level; re-imported here so this
// submodule compiles without relying on `super::*` import order.
use ship_pipelines::types::{Pipeline, PipelinePhase};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunStartedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Operator-facing path to the spec file as passed to `ship run --spec`
    /// (or the dispatch.sh equivalent). Distinct from `spec_path` which is
    /// the daemon-side worktree destination. None for legacy events and
    /// for `--spec-text` / inline-spec flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_spec_path: Option<String>,
    pub plan_id: Option<String>,
    pub model: Option<String>,
    pub harness: Option<String>,
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
    /// Per-step spec bodies, indexed by step position. Populated by
    /// `ensure_pipeline` from the pipeline JSONC's `step_specs:` block when
    /// present. Written to `.artifacts/pipeline/NN-<step_id>.md` at launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_specs: Option<Vec<String>>,
    /// When true, run steps headless (ProcessExecutor). When false (default),
    /// run steps as tmux windows in a shared per-slug session
    /// (TmuxWindowExecutor) so users can `tmux attach` and watch the agent
    /// live.
    #[serde(default)]
    pub headless: bool,
    /// Long-running interactive session — not a pipeline step. Routes to
    /// ProcessExecutor with `adapter.build_interactive` (no `-p`, no
    /// `--print`). The harness stays at its TUI prompt waiting for stdin
    /// pushes or WebSocket attach. Mutually exclusive with `headless`.
    /// Drives cmdb-modal chat, hooks SessionStart, reliable PTY attach.
    #[serde(default)]
    pub keepalive: bool,
    /// Inline one-shot prompt for the dispatched agent. Distinct from
    /// `spec_content` (the latter is written to `spec_path` as a file;
    /// `prompt` is delivered as the first user turn or piped via the
    /// harness's one-shot flag in `auto` mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Continue the last session for this (agent, workspace) pair. When
    /// set, the dispatch endpoint resolved an existing session and reused
    /// its slug/branch instead of generating new ones.
    #[serde(default)]
    pub resume: bool,
    /// Harness-native session id resolved from the worktree's
    /// `.ship/session.json` when `resume: true`. Threaded through to the
    /// orchestrator so each adapter's `build_resume_args` can produce the
    /// correct CLI flags. `None` for fresh launches and legacy events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_session_id: Option<String>,
    /// HEAD SHA of the source repo when this job was created. Pipeline
    /// gates evaluate `require_changed` / `forbid_changed` against the
    /// diff from this SHA to HEAD, so the checks see work committed in
    /// any earlier pipeline step, not just the gating step. `None` for
    /// legacy events predating this field; gates fall back to the
    /// step's start SHA in that case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_sha: Option<String>,
}

/// Emitted when the job is assigned to a worktree and (optionally) a process.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunDispatchedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    /// Worktree slug for the dispatched session. Required for observers
    /// filtering by `slug` (e.g. /api/v1/events/poll?slug=) — the
    /// envelope's entity_id is the pipeline_run_id (a ULID), not the slug.
    /// Optional with serde default for forward compat with older
    /// persisted envelopes that pre-date this field.
    #[serde(default)]
    pub slug: String,
    pub worktree: String,
    pub pid: Option<u32>,
    /// Harness-specific session metadata (e.g. Claude session ID for --resume).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_meta: Option<serde_json::Value>,
}

/// Emitted when a gate review is requested for the job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineGateRequestedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub gate_agent: String,
}

/// Emitted when the gate review passes.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineGatePassedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
}

/// Emitted when the gate review fails.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineGateFailedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub reason: String,
}

/// Emitted when the job is blocked and cannot proceed without resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunBlockedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    /// Human-readable description of what is blocking progress.
    pub blocker: String,
    pub needs_human: bool,
}

/// Emitted when the agent signals the job is done but before gate/merge.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunCompletedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
}

/// Emitted when the job's branch is merged.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunMergedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
}

/// Emitted when the job terminates with an error.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunFailedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub error: String,
}

/// Emitted when an executor's spawn() returns Err — i.e. the agent
/// process never started. Distinct from PipelineRunFailedPayload because it
/// carries the *stage* of dispatch that failed (isolation prepare,
/// agent compile, executor spawn) so the operator can debug the
/// right layer.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepSpawnFailedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    /// Which dispatch phase failed: "isolation_prepare" |
    /// "agent_compile" | "executor_spawn".
    pub stage: String,
    /// Error message captured at the failure site.
    pub error: String,
}

/// Emitted when an operator (or system policy) cancels a job. Terminal.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunCancelledPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    /// Free-form reason. UI surfaces this verbatim.
    pub reason: String,
    /// Who/what initiated the cancellation: "operator", "supervisor",
    /// "policy:timeout", etc. Open-vocabulary string.
    pub requested_by: String,
}

/// Emitted by either side (human, commander, or agent) to send a mid-flight
/// update, instruction, or course correction. The daemon routes these to the
/// agent's mailbox.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRunUpdatedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub message: String,
    /// Who sent this update — e.g. "human", "commander", agent mesh ID.
    pub sender: String,
}

/// Emitted when a pipeline step begins executing.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepStartedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_id: String,
    pub step_type: String,
}

/// Emitted when a pipeline step finishes successfully.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepCompletedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_id: String,
}

/// Emitted when a pipeline step fails. Halts pipeline advancement.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepFailedPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_id: String,
    pub error: String,
}

/// Emitted by run steps to capture stdout/stderr output.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepOutputPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub step_id: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Emitted by an agent via the `step_done` MCP tool to declare step completion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepDoneDeclaredPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_id: String,
    pub summary: Option<String>,
    /// Map of `produces.name → artifact_id` for outputs the agent uploaded.
    /// Empty when the step declares no `produces` or the agent omitted outputs.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub outputs: std::collections::BTreeMap<String, String>,
}

/// Emitted when a gated step is retried after gate failure.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineStepRetryPayload {
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub slug: String,
    pub step_id: String,
    pub step_name: String,
    pub attempt: u32,
    pub reason: String,
}
