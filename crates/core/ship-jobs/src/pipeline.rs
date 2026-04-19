use crate::step::{PipelineStep, infer_step_type};
use serde_json::{Map, Value, json};

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

/// Pipeline-level metadata extracted from the YAML file.
#[derive(Debug, Clone, Default)]
pub struct PipelineMeta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub vars: Option<std::collections::HashMap<String, String>>,
    pub timeout: Option<String>,
    pub max_retries: Option<u32>,
}

/// Parse a pipeline definition and normalize it into the old-format steps array.
///
/// Accepts both the new simplified format (agents list, string steps) and the
/// old format (explicit id/type/agent/after per step). Returns a JSON Value
/// with a `steps` array that the runtime PipelineStep deserializer can handle,
/// plus extracted metadata.
///
/// Input can be JSONC (primary) or YAML (legacy/frontmatter).
pub fn normalize_pipeline(input: &str) -> Result<(Value, PipelineMeta), String> {
    let val: Value = try_parse_jsonc_or_yaml(input)?;

    let meta = extract_meta(&val);

    // Short form: `agents: [...]` → sequential agent steps
    if let Some(agents) = val.get("agents").and_then(|v| v.as_array()) {
        let steps = agents_to_steps(agents)?;
        return Ok((json!({ "steps": steps }), meta));
    }

    // Long form: `steps: [...]` with mixed types
    if let Some(steps_val) = val.get("steps").and_then(|v| v.as_array()) {
        let steps = normalize_steps(steps_val)?;
        return Ok((json!({ "steps": steps }), meta));
    }

    Err("pipeline must have either `agents` or `steps`".to_string())
}

/// Backwards-compatible alias — callers migrating from YAML can use this name.
pub fn normalize_pipeline_yaml(input: &str) -> Result<(Value, PipelineMeta), String> {
    normalize_pipeline(input)
}

/// Try JSONC first, fall back to YAML for legacy/frontmatter content.
fn try_parse_jsonc_or_yaml(input: &str) -> Result<Value, String> {
    let trimmed = input.trim_start();
    // If it looks like JSON (starts with `{` or `[`), parse as JSONC.
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        let json_str = compiler::jsonc::strip_jsonc_comments(input);
        return serde_json::from_str(&json_str)
            .map_err(|e| format!("invalid pipeline JSONC: {e}"));
    }
    // Otherwise treat as YAML (spec frontmatter, legacy files).
    serde_yaml::from_str(input)
        .map_err(|e| format!("invalid pipeline YAML: {e}"))
}

/// Extract pipeline-level metadata from the parsed value.
fn extract_meta(val: &Value) -> PipelineMeta {
    PipelineMeta {
        name: val.get("name").and_then(|v| v.as_str()).map(str::to_string),
        description: val.get("description").and_then(|v| v.as_str()).map(str::to_string),
        vars: val.get("vars").and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
        }),
        timeout: val.get("timeout").and_then(|v| v.as_str()).map(str::to_string),
        max_retries: val.get("max_retries").and_then(|v| v.as_u64()).map(|n| n as u32),
    }
}

/// Convert `agents: ["a", "b", "c"]` into sequential step objects.
fn agents_to_steps(agents: &[Value]) -> Result<Vec<Value>, String> {
    let mut steps = Vec::with_capacity(agents.len());
    let mut prev_id: Option<String> = None;

    for (i, agent_val) in agents.iter().enumerate() {
        let agent_str = agent_val.as_str().ok_or_else(|| {
            format!("agents[{i}]: expected a string, got {agent_val}")
        })?;
        let id = format!("step-{}", i + 1);

        let mut step = Map::new();
        step.insert("id".to_string(), json!(id));
        step.insert("agent".to_string(), json!(agent_str));
        step.insert("type".to_string(), json!("agent"));
        if let Some(ref prev) = prev_id {
            step.insert("after".to_string(), json!([prev]));
        }
        prev_id = Some(id);
        steps.push(Value::Object(step));
    }

    Ok(steps)
}

/// Normalize a mixed `steps` array into old-format step objects.
///
/// Each step can be:
/// 1. A string → agent step
/// 2. `{ run: <cmd> }` → run step
/// 3. `{ human: <prompt> }` → human step
/// 4. `{ agent: <id>, ...overrides }` → agent step with config
/// 5. `{ parallel: [...] }` → fan-out group
/// 6. Old format (has `id` key) → pass through unchanged
fn normalize_steps(steps: &[Value]) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    let mut prev_id: Option<String> = None;
    let mut step_counter: usize = 0;

    for (i, step_val) in steps.iter().enumerate() {
        // Old format detection: has `id` field → pass through
        if let Some(obj) = step_val.as_object()
            && (obj.contains_key("id") || obj.contains_key("name"))
        {
            result.push(step_val.clone());
            // Track ID for sequential chaining of subsequent new-format steps
            let id = obj.get("id").or_else(|| obj.get("name"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            if let Some(id) = id {
                prev_id = Some(id);
            }
            continue;
        }

        // Parallel group
        if let Some(obj) = step_val.as_object()
            && let Some(parallel_val) = obj.get("parallel")
        {
            let group = parallel_val.as_array().ok_or_else(|| {
                format!("steps[{i}].parallel: expected array")
            })?;
            let group_ids = normalize_parallel_group(
                group,
                &prev_id,
                &mut step_counter,
                &mut result,
            )?;
            // All parallel steps share the same "after", and the next
            // sequential step depends on all of them.
            prev_id = None; // Cleared — next step sets after to all group IDs
            // Store group IDs for the next step's `after`
            if !group_ids.is_empty() {
                prev_id = Some(group_ids.join(","));
            }
            continue;
        }

        step_counter += 1;
        let normalized = normalize_single_step(step_val, step_counter, &prev_id, i)?;
        let id = normalized.get("id").and_then(|v| v.as_str()).unwrap().to_string();
        prev_id = Some(id);
        result.push(normalized);
    }

    Ok(result)
}

/// Normalize a single non-parallel step into old-format object.
fn normalize_single_step(
    val: &Value,
    counter: usize,
    prev_id: &Option<String>,
    index: usize,
) -> Result<Value, String> {
    let mut step = Map::new();
    let step_type = infer_step_type(val);

    match val {
        // String → agent step
        Value::String(agent) => {
            let id = format!("step-{counter}");
            step.insert("id".to_string(), json!(id));
            step.insert("type".to_string(), json!("agent"));
            step.insert("agent".to_string(), json!(agent));
        }
        Value::Object(obj) => {
            if let Some(cmd) = obj.get("run").and_then(|v| v.as_str()) {
                // run: <command>
                let id = format!("run-{counter}");
                step.insert("id".to_string(), json!(id));
                step.insert("type".to_string(), json!("run"));
                step.insert("command".to_string(), json!(cmd));
            } else if let Some(prompt) = obj.get("human").and_then(|v| v.as_str()) {
                // human: <prompt>
                let id = format!("human-{counter}");
                step.insert("id".to_string(), json!(id));
                step.insert("type".to_string(), json!("human"));
                step.insert("goal".to_string(), json!(prompt));
            } else if let Some(agent) = obj.get("agent").and_then(|v| v.as_str()) {
                // agent: <id> with overrides
                let id = format!("step-{counter}");
                step.insert("id".to_string(), json!(id));
                step.insert("type".to_string(), json!(step_type));
                step.insert("agent".to_string(), json!(agent));
                // Copy overrides
                for key in ["model", "provider", "max_retries", "goal", "command"] {
                    if let Some(v) = obj.get(key) {
                        step.insert(key.to_string(), v.clone());
                    }
                }
            } else {
                return Err(format!(
                    "steps[{index}]: cannot infer step type from keys: {:?}",
                    obj.keys().collect::<Vec<_>>()
                ));
            }
        }
        _ => {
            return Err(format!("steps[{index}]: expected string or object"));
        }
    }

    // Wire sequential dependency
    if let Some(prev) = prev_id
        && !step.contains_key("after")
    {
        let after: Vec<&str> = prev.split(',').collect();
        step.insert("after".to_string(), json!(after));
    }

    Ok(Value::Object(step))
}

/// Expand a parallel group into individual steps, all sharing the same `after`.
fn normalize_parallel_group(
    group: &[Value],
    prev_id: &Option<String>,
    counter: &mut usize,
    result: &mut Vec<Value>,
) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    for (j, item) in group.iter().enumerate() {
        *counter += 1;
        let mut normalized = normalize_single_step(item, *counter, prev_id, j)?;
        // Override after to point to prev_id (not chained within the group)
        if let Some(obj) = normalized.as_object_mut() {
            if let Some(prev) = prev_id {
                let after: Vec<&str> = prev.split(',').collect();
                obj.insert("after".to_string(), json!(after));
            } else {
                obj.insert("after".to_string(), json!([]));
            }
        }
        let id = normalized.get("id").and_then(|v| v.as_str()).unwrap().to_string();
        ids.push(id);
        result.push(normalized);
    }

    Ok(ids)
}
