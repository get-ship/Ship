use crate::step::PipelineStep;

/// A pipeline definition — a DAG of steps.
#[derive(Debug, Clone)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
}

impl Pipeline {
    /// Find root steps (no dependencies).
    pub fn root_steps(&self) -> Vec<&PipelineStep> {
        self.steps.iter().filter(|s| s.after.is_empty()).collect()
    }
}
