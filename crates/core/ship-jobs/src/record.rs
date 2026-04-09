use crate::pipeline::Pipeline;

/// Projected state of a single job.
#[derive(Debug, Clone, Default)]
pub struct JobRecord {
    pub pipeline: Option<Pipeline>,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
}
