/// Step type in a pipeline DAG.
#[derive(Debug, Clone, PartialEq)]
pub enum StepType {
    Agent,
    Gate,
    Merge,
    Human,
    Run { command: String },
}

/// A single step in a pipeline DAG.
#[derive(Debug, Clone)]
pub struct PipelineStep {
    pub id: String,
    pub step_type: StepType,
    pub agent: Option<String>,
    pub goal: Option<String>,
    pub after: Vec<String>,
}
