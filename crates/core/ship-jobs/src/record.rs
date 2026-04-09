use crate::pipeline::Pipeline;

/// Projected state of a single job.
#[derive(Debug, Clone)]
pub struct JobRecord {
    pub pipeline: Option<Pipeline>,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
}

impl Default for JobRecord {
    fn default() -> Self {
        todo!()
    }
}
