// -- Spec-6 new events -------------------------------------------------------
//
// Additions that don't fit a single pre-existing aggregate — typed
// replies / cross-cutting event types introduced with Spec 6.

use serde::{Deserialize, Serialize};

/// `agent.task.complete` — task-creation completion in space history.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTaskComplete {
    pub task_id: String,
    pub task_type: String,
    #[serde(default)]
    pub space_id: Option<String>,
}

/// `human.response` — inbound human reply (telegram webhook) tied to space conversation.
#[derive(Debug, Serialize, Deserialize)]
pub struct HumanResponse {
    pub conversation_id: String,
    pub text: String,
    #[serde(default)]
    pub sender: Option<String>,
}

/// `pipeline.step.retry` — a step was retried after a transient failure.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineStepRetry {
    pub pipeline_run_id: String,
    pub step_id: String,
    pub retry_count: u32,
    pub reason: String,
}
