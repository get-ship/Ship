use serde::{Deserialize, Serialize};

/// Emitted when a job is created.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobCreatedPayload {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
}

/// Emitted when a pipeline step begins executing.
#[derive(Debug, Clone)]
pub struct StepDispatchedPayload {
    pub job_id: String,
    pub step_index: usize,
    pub step_name: String,
    pub step_type: String,
}

/// Emitted when a pipeline step finishes successfully.
#[derive(Debug, Clone)]
pub struct StepCompletedPayload {
    pub job_id: String,
    pub step_index: usize,
    pub step_name: String,
}
