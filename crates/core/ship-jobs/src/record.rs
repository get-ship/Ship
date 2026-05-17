use crate::pipeline::Pipeline;

/// Projected state of a single pipeline run.
#[derive(Debug, Clone, Default)]
pub struct PipelineRun {
    pub pipeline: Option<Pipeline>,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
}
