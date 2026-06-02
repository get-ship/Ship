//! Pipeline validation — pure logic with no fs / process / network.
//!
//! Validates platform invariants on a resolved pipeline: agent declarations,
//! produces/consumes/promotes consistency, scope transitions, fanout shapes,
//! and lane-scoped reference resolution. The daemon's `pipeline_loader`
//! calls this after parsing YAML; the cloud worker calls the same function
//! over the same types.

use std::collections::{HashMap, HashSet};

use crate::types::{FanoutSpec, OutputScope, Pipeline, PipelineStep, StepType};

/// Reserved output name emitted by the fanout aggregator when a lane fails.
/// Lane-scoped consumes referencing `<lane_id>.failed` are valid regardless
/// of the fanout step's declared produces.
pub const FAILED_MARKER_NAME: &str = "failed";

/// Parse a lane-scoped consume reference like `"lane-claude.draft"` into
/// `Some(("lane-claude", "draft"))`. Returns `None` for plain names with no `.`.
pub fn parse_lane_scoped_ref(name: &str) -> Option<(&str, &str)> {
    let dot = name.find('.')?;
    Some((&name[..dot], &name[dot + 1..]))
}

/// Top-level validation: enforces every platform invariant on a resolved
/// pipeline. Each step gets its agent + produces/consumes/promotes/fanout
/// checked.
pub fn validate_pipeline(pipeline: &Pipeline) -> Result<(), String> {
    for step in &pipeline.steps {
        let needs_agent = matches!(step.step_type, StepType::Agent | StepType::Review);
        if needs_agent && step.agent.as_deref().map(str::is_empty).unwrap_or(true) {
            return Err(format!(
                "pipeline step '{}' of type {:?} requires an agent but none was declared",
                step.id, step.step_type
            ));
        }
    }
    validate_pipeline_io(pipeline)
}

/// Validate that a `produces.name` / `consumes` entry is a safe filename
/// fragment — `.artifacts/input/<name>` must not escape the worktree.
///
/// Allowed: `[a-zA-Z0-9_.-]+`, 1–128 chars, no leading dot, no `..` anywhere.
/// This is a load-time guard; runtime executors re-check at dispatch as
/// defense-in-depth.
pub fn validate_output_name(step_id: &str, name: &str, kind: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 {
        return Err(format!(
            "step '{step_id}' declares {kind} name '{name}' with invalid length \
             (must be 1–128 chars)"
        ));
    }
    if name.starts_with('.') {
        return Err(format!(
            "step '{step_id}' declares {kind} name '{name}' which starts with '.' \
             (disallowed); allowed: [a-zA-Z0-9_.-]"
        ));
    }
    if name.contains("..") {
        return Err(format!(
            "step '{step_id}' declares {kind} name '{name}' containing '..' \
             (path traversal disallowed); allowed: [a-zA-Z0-9_.-]"
        ));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-') {
        return Err(format!(
            "step '{step_id}' declares {kind} name '{name}' which contains \
             disallowed characters; allowed: [a-zA-Z0-9_.-]"
        ));
    }
    Ok(())
}

/// Validate `produces`/`consumes`/`promotes` declarations across all steps.
pub fn validate_pipeline_io(pipeline: &Pipeline) -> Result<(), String> {
    let mut producers: HashMap<&str, Vec<&str>> = HashMap::new();
    for step in &pipeline.steps {
        for output in &step.produces {
            validate_output_name(&step.id, &output.name, "produces")?;
            producers
                .entry(output.name.as_str())
                .or_default()
                .push(step.id.as_str());
        }
    }
    for (name, owners) in &producers {
        if owners.len() > 1 {
            return Err(format!(
                "produces name '{name}' is declared by multiple steps: [{}]",
                owners.join(", ")
            ));
        }
    }
    let producers: HashMap<&str, &str> = producers
        .into_iter()
        .map(|(k, v)| (k, v[0]))
        .collect();

    for step in &pipeline.steps {
        if let Some(fanout) = &step.fanout {
            validate_fanout(pipeline, step, fanout)?;
        }
    }

    for step in &pipeline.steps {
        for name in &step.consumes {
            validate_output_name(&step.id, name, "consumes")?;
            if let Some((lane_id, output_name)) = parse_lane_scoped_ref(name) {
                validate_output_name(&step.id, lane_id, "consumes (lane id)")?;
                validate_output_name(&step.id, output_name, "consumes (lane output)")?;
                validate_lane_scoped_consume(pipeline, &step.id, name, lane_id, output_name)?;
                continue;
            }
            match producers.get(name.as_str()) {
                None => return Err(format!(
                    "step '{}' consumes '{name}' but no step in the pipeline produces it",
                    step.id
                )),
                Some(&pid) if pid == step.id => return Err(format!(
                    "step '{}' cannot consume its own output '{name}'",
                    step.id
                )),
                _ => {}
            }
        }

        let produces_names: HashSet<&str> =
            step.produces.iter().map(|o| o.name.as_str()).collect();
        let consumes_names: HashSet<&str> =
            step.consumes.iter().map(|s| s.as_str()).collect();
        for promote in &step.promotes {
            let source_scope =
                if let Some(output) = step.produces.iter().find(|o| o.name == promote.name) {
                    &output.scope
                } else if consumes_names.contains(promote.name.as_str()) {
                    &OutputScope::Run
                } else if !produces_names.contains(promote.name.as_str()) {
                    return Err(format!(
                        "step '{}' promotes '{}' but this step neither produces nor consumes it",
                        step.id, promote.name
                    ));
                } else {
                    continue;
                };
            validate_scope_transition(source_scope, &promote.target_scope, &step.id, &promote.name)?;
        }
    }
    Ok(())
}

fn validate_scope_transition(
    from: &OutputScope,
    to: &OutputScope,
    step_id: &str,
    name: &str,
) -> Result<(), String> {
    let valid = matches!((from, to), (OutputScope::Run, OutputScope::Project));
    if !valid {
        return Err(format!(
            "step '{step_id}' promotes '{name}': invalid scope transition \
             (only Run→Project is supported) (requested: {from:?}→{to:?})"
        ));
    }
    Ok(())
}

fn validate_fanout(
    pipeline: &Pipeline,
    step: &PipelineStep,
    fanout: &FanoutSpec,
) -> Result<(), String> {
    let judge = pipeline
        .steps
        .iter()
        .find(|s| s.id == fanout.judge.step)
        .ok_or_else(|| format!(
            "fanout step '{}': judge.step '{}' not found in pipeline",
            step.id, fanout.judge.step
        ))?;

    if !judge.after.contains(&step.id) {
        return Err(format!(
            "fanout step '{}': judge step '{}' must list '{}' in its `after`",
            step.id, judge.id, step.id
        ));
    }

    let mut seen = HashSet::new();
    for lane in &fanout.lanes {
        if !seen.insert(lane.id.as_str()) {
            return Err(format!(
                "fanout step '{}': duplicate lane id '{}'",
                step.id, lane.id
            ));
        }
        if lane.agent.is_empty() {
            return Err(format!(
                "fanout step '{}' lane '{}': agent must be declared",
                step.id, lane.id
            ));
        }
    }

    if fanout.min_successful_lanes == 0 {
        return Err(format!(
            "fanout step '{}': min_successful_lanes must be at least 1",
            step.id
        ));
    }

    Ok(())
}

fn validate_lane_scoped_consume(
    pipeline: &Pipeline,
    consuming_step_id: &str,
    full_name: &str,
    lane_id: &str,
    output_name: &str,
) -> Result<(), String> {
    let fanout_step = pipeline
        .steps
        .iter()
        .find(|s| s.fanout.as_ref().is_some_and(|f| f.lanes.iter().any(|l| l.id == lane_id)))
        .ok_or_else(|| format!(
            "step '{consuming_step_id}' consumes '{full_name}': \
             no fanout step has lane '{lane_id}'"
        ))?;

    if output_name == FAILED_MARKER_NAME {
        return Ok(());
    }

    if !fanout_step.produces.iter().any(|o| o.name == output_name) {
        return Err(format!(
            "step '{consuming_step_id}' consumes '{full_name}': \
             fanout step '{}' does not produce '{output_name}'",
            fanout_step.id
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent_step(id: &str, agent: Option<&str>) -> PipelineStep {
        PipelineStep {
            id: id.into(),
            step_type: StepType::Agent,
            agent: agent.map(|s| s.into()),
            goal: None,
            after: vec![],
            model: None,
            harness: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: None,
        }
    }

    #[test]
    fn agent_step_without_agent_fails() {
        let p = Pipeline {
            steps: vec![agent_step("a", None)],
            auto_merge: false,
        };
        assert!(validate_pipeline(&p).is_err());
    }

    #[test]
    fn run_step_without_agent_passes() {
        let p = Pipeline {
            steps: vec![PipelineStep {
                step_type: StepType::Run,
                command: Some("echo hi".into()),
                ..agent_step("r", None)
            }],
            auto_merge: false,
        };
        assert!(validate_pipeline(&p).is_ok());
    }

    #[test]
    fn output_name_rejects_traversal() {
        assert!(validate_output_name("s", "../etc/passwd", "produces").is_err());
        assert!(validate_output_name("s", ".hidden", "produces").is_err());
        assert!(validate_output_name("s", "", "produces").is_err());
        assert!(validate_output_name("s", "ok-name_1.txt", "produces").is_ok());
    }

    #[test]
    fn parse_lane_scoped_ref_splits_on_first_dot() {
        assert_eq!(
            parse_lane_scoped_ref("lane-x.draft"),
            Some(("lane-x", "draft"))
        );
        assert_eq!(parse_lane_scoped_ref("plain"), None);
    }
}
