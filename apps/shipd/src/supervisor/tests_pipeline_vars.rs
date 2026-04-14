//! Tests for `{{ var }}` interpolation in pipeline step fields.
//!
//! Defines the contract for variable resolution:
//!   1. Job-level vars (highest priority)
//!   2. Step-level vars (fallback defaults)
//!   3. Unresolved → Err containing the missing var name
//!
//! All tests call `interpolate_step` from the runtime orchestrator.

use runtime::events::job::{PipelineStep, StepType};
use runtime::services::orchestrator::pipeline_vars::interpolate_step;
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn agent_step(agent: &str) -> PipelineStep {
    PipelineStep {
        id: "test".to_string(),
        step_type: StepType::Agent,
        agent: Some(agent.to_string()),
        goal: None,
        after: vec![],
        model: None,
        provider: None,
        command: None,
        max_retries: None,
        gate: None,
    }
}

fn agent_step_with_model(agent: &str, model: &str) -> PipelineStep {
    PipelineStep {
        id: "test".to_string(),
        step_type: StepType::Agent,
        agent: Some(agent.to_string()),
        goal: None,
        after: vec![],
        model: Some(model.to_string()),
        provider: None,
        command: None,
        max_retries: None,
        gate: None,
    }
}

fn run_step(command: &str) -> PipelineStep {
    PipelineStep {
        id: "test".to_string(),
        step_type: StepType::Run,
        agent: None,
        goal: None,
        after: vec![],
        model: None,
        provider: None,
        command: Some(command.to_string()),
        max_retries: None,
        gate: None,
    }
}

fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Job var replaces `{{ impl_agent }}` in the agent field.
#[test]
fn step_agent_interpolated_from_job_vars() {
    let step = agent_step("{{ impl_agent }}");
    let job_vars = vars(&[("impl_agent", "rust-lane")]);
    let resolved = interpolate_step(&step, &job_vars, &HashMap::new())
        .expect("interpolation must succeed when var is provided");
    assert_eq!(resolved.agent.as_deref(), Some("rust-lane"));
}

/// Job var replaces `{{ feature_name }}` in the goal field.
/// NOTE: Requires `PipelineStep` to gain a `goal: Option<String>` field.
#[test]
fn step_goal_interpolated_from_job_vars() {
    let step = PipelineStep {
        id: "test".to_string(),
        step_type: StepType::Agent,
        agent: Some("rust-lane".to_string()),
        goal: Some("Implement {{ feature_name }}".to_string()),
        after: vec![],
        model: None,
        provider: None,
        command: None,
        max_retries: None,
        gate: None,
    };
    let job_vars = vars(&[("feature_name", "config split")]);
    let resolved = interpolate_step(&step, &job_vars, &HashMap::new())
        .expect("interpolation must succeed when var is provided");
    assert_eq!(resolved.goal.as_deref(), Some("Implement config split"));
}

/// Job var replaces `{{ model }}` in the model field.
#[test]
fn step_model_interpolated_from_job_vars() {
    let step = agent_step_with_model("rust-lane", "{{ model }}");
    let job_vars = vars(&[("model", "opus")]);
    let resolved = interpolate_step(&step, &job_vars, &HashMap::new())
        .expect("interpolation must succeed when var is provided");
    assert_eq!(resolved.model.as_deref(), Some("opus"));
}

/// Job var replaces `{{ test_filter }}` in the command field of a run step.
#[test]
fn step_command_interpolated_from_job_vars() {
    let step = run_step("cargo test {{ test_filter }}");
    let job_vars = vars(&[("test_filter", "config::")]);
    let resolved = interpolate_step(&step, &job_vars, &HashMap::new())
        .expect("interpolation must succeed when var is provided");
    assert_eq!(resolved.command.as_deref(), Some("cargo test config::"));
}

/// Step-level vars provide defaults when no job var is set.
#[test]
fn step_vars_provide_defaults() {
    let step = agent_step("{{ impl_agent }}");
    let step_vars = vars(&[("impl_agent", "default")]);
    let resolved = interpolate_step(&step, &HashMap::new(), &step_vars)
        .expect("interpolation must succeed when step var provides the default");
    assert_eq!(resolved.agent.as_deref(), Some("default"));
}

/// Job vars take priority over step vars for the same key.
#[test]
fn job_vars_override_step_vars() {
    let step = agent_step("{{ impl_agent }}");
    let job_vars = vars(&[("impl_agent", "rust-lane")]);
    let step_vars = vars(&[("impl_agent", "default")]);
    let resolved = interpolate_step(&step, &job_vars, &step_vars)
        .expect("interpolation must succeed when job var is provided");
    assert_eq!(resolved.agent.as_deref(), Some("rust-lane"), "job var must win over step var");
}

/// Unresolved template variable returns Err naming the missing var.
#[test]
fn missing_var_returns_error() {
    let step = agent_step("{{ impl_agent }}");
    let result = interpolate_step(&step, &HashMap::new(), &HashMap::new());
    let err = result.expect_err("interpolation must fail when var is missing");
    assert!(
        err.contains("impl_agent"),
        "error must name the missing variable, got: {err}"
    );
}

/// Steps without template syntax pass through without modification.
#[test]
fn no_vars_passthrough() {
    let step = agent_step("rust-lane");
    let resolved = interpolate_step(&step, &HashMap::new(), &HashMap::new())
        .expect("passthrough step must not fail");
    assert_eq!(resolved.agent.as_deref(), Some("rust-lane"));
    assert_eq!(resolved.model, None);
    assert_eq!(resolved.command, None);
}
