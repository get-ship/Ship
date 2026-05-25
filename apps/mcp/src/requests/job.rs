use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, JsonSchema)]
pub struct UpdateJobRequest {
    /// Pipeline-run id this status update targets.
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    /// Target status: dispatched | gate_requested | gate_passed | gate_failed | blocked | completed | merged | failed
    pub status: String,
    /// Worktree path — required when status is "dispatched".
    pub worktree: Option<String>,
    /// Blocker description — required when status is "blocked".
    pub blocker: Option<String>,
    /// Error message — required when status is "gate_failed" or "failed".
    pub error: Option<String>,
    /// OS process ID — optional, used with "dispatched".
    pub pid: Option<u32>,
    /// Gate agent ID — required when status is "gate_requested".
    pub gate_agent: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListJobsRequest {
    /// Optional status filter. Omit to return all jobs.
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetJobRequest {
    /// Job ID to retrieve.
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct StepDoneRequest {
    /// What was accomplished in this step (optional summary).
    pub summary: Option<String>,
    /// Map of `produces.name → artifact_id` for outputs uploaded during this step.
    /// Required when the step declares `produces` entries; the orchestrator
    /// verifies all `required: true` outputs are present.
    #[serde(default)]
    pub outputs: BTreeMap<String, String>,
}
