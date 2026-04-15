/// Step type in a pipeline DAG.
#[derive(Debug, Clone, PartialEq)]
pub enum StepType {
    Agent,
    Gate,
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

/// Infer the step type string from a YAML value's shape.
///
/// Returns the `type` field value that the runtime PipelineStep deserializer expects:
/// - Has `run` key → "run"
/// - Has `agent` key or is a string → "agent"
pub fn infer_step_type(val: &serde_json::Value) -> &'static str {
    if val.is_string() {
        return "agent";
    }
    if let Some(obj) = val.as_object() {
        if obj.contains_key("run") {
            return "run";
        }
        if obj.contains_key("type")
            && let Some(t) = obj["type"].as_str()
        {
            return match t {
                "agent" => "agent",
                "gate" => "gate",
                "run" => "run",
                _ => "agent",
            };
        }
    }
    "agent"
}
