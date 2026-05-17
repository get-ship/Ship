//! Pipeline parser — JSONC pipeline definitions → normalized `{ steps: [...] }`
//! shape plus `PipelineMeta`.
//!
//! Accepts the short form (`agents: [...]`) and the long form (`steps: [...]`
//! with mixed entry types). Input format is JSONC — no YAML. Pipeline files
//! (`.ship/pipelines/<name>.jsonc`) and inline pipeline blobs both come
//! through here.
//!
//! Moved from `ship-jobs::pipeline` when ship-jobs was dissolved; this is the
//! canonical home for pipeline parsing now that `ship-pipelines` owns the
//! Pipeline / PipelineStep / StepType types.

use serde_json::{Map, Value, json};

/// Pipeline-level metadata extracted from the pipeline file.
#[derive(Debug, Clone, Default)]
pub struct PipelineMeta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub vars: Option<std::collections::HashMap<String, String>>,
    pub timeout: Option<String>,
    pub max_retries: Option<u32>,
    /// Optional per-step spec bodies, indexed by step position (0-based).
    /// When present, the orchestrator writes each into
    /// `.artifacts/pipeline/NN-<step_id>.md` at job launch instead of sharing
    /// a single spec across every step.
    pub step_specs: Option<Vec<String>>,
    /// When true, the runtime runs `git merge` on the job branch after the
    /// final step completes. Replaces the former `type: merge` step.
    pub auto_merge: Option<bool>,
}

/// Parse a pipeline definition and normalize it into a JSON `{ steps: [...] }`
/// shape the runtime can deserialize via `PipelineStep`.
pub fn normalize_pipeline(input: &str) -> Result<(Value, PipelineMeta), String> {
    let json_str = compiler::jsonc::strip_jsonc_comments(input);
    let val: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("invalid pipeline JSONC: {e}"))?;

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
        step_specs: val.get("step_specs").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .map(|entry| entry.as_str().unwrap_or_default().to_string())
                .collect()
        }),
        auto_merge: val.get("auto_merge").and_then(|v| v.as_bool()),
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
        if let Some(obj) = step_val.as_object()
            && obj.contains_key("id")
        {
            result.push(step_val.clone());
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()).map(str::to_string) {
                prev_id = Some(id);
            }
            continue;
        }

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
            prev_id = None;
            if !group_ids.is_empty() {
                prev_id = Some(group_ids.join(","));
            }
            continue;
        }

        step_counter += 1;
        let normalized = normalize_single_step(step_val, step_counter, &prev_id, i)?;
        let id = normalized
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| format!("step-{step_counter}"));
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
        Value::String(agent) => {
            let id = format!("step-{counter}");
            step.insert("id".to_string(), json!(id));
            step.insert("type".to_string(), json!("agent"));
            step.insert("agent".to_string(), json!(agent));
        }
        Value::Object(obj) => {
            if let Some(cmd) = obj.get("run").and_then(|v| v.as_str()) {
                let id = format!("run-{counter}");
                step.insert("id".to_string(), json!(id));
                step.insert("type".to_string(), json!("run"));
                step.insert("command".to_string(), json!(cmd));
            } else if let Some(agent) = obj.get("agent").and_then(|v| v.as_str()) {
                let id = format!("step-{counter}");
                step.insert("id".to_string(), json!(id));
                step.insert("type".to_string(), json!(step_type));
                step.insert("agent".to_string(), json!(agent));
                for key in ["model", "harness", "max_retries", "goal", "command"] {
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
        if let Some(obj) = normalized.as_object_mut() {
            if let Some(prev) = prev_id {
                let after: Vec<&str> = prev.split(',').collect();
                obj.insert("after".to_string(), json!(after));
            } else {
                obj.insert("after".to_string(), json!([]));
            }
        }
        let id = normalized
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| format!("step-{counter}"));
        ids.push(id);
        result.push(normalized);
    }

    Ok(ids)
}

/// Infer the step type string from a JSON value's shape.
///
/// Returns the `type` field value that the runtime PipelineStep
/// deserializer expects:
/// - Has `run` key → "run"
/// - Has `agent` key or is a string → "agent"
fn infer_step_type(val: &serde_json::Value) -> &'static str {
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
                "review" => "review",
                "run" => "run",
                _ => "agent",
            };
        }
    }
    "agent"
}

#[cfg(test)]
mod tests;
