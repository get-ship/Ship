//! Tests for `pipeline_loader::ensure_pipeline`.
//!
//! Covers the two load-time invariants restored by the platform-purge job:
//! missing agent on an agent/gate step fails loud, and jobs without any
//! declared pipeline resolve to a single-step Agent pipeline — no hardcoded
//! review/merge/sonnet.

use super::pipeline_loader::ensure_pipeline;
use crate::events::job::{JobCreatedPayload, Pipeline, PipelineStep, StepType};

fn base_payload(agent: &str) -> JobCreatedPayload {
    JobCreatedPayload {
        job_id: "j-1".into(),
        slug: "slug-1".into(),
        agent: agent.into(),
        branch: "job/slug-1".into(),
        spec_path: ".ship/specs/slug-1.md".into(),
        plan_id: None,
        model: None,
        provider: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
    }
}

#[test]
fn missing_agent_on_agent_step_fails_at_load() {
    let mut payload = base_payload("coder");
    payload.steps = Some(Pipeline {
        steps: vec![PipelineStep {
            id: "orphan".into(),
            step_type: StepType::Agent,
            agent: None,
            goal: None,
            after: vec![],
            model: None,
            provider: None,
            command: None,
            max_retries: None,
            gate: None,
        }],
        auto_merge: false,
    });

    let err = ensure_pipeline(payload).expect_err("agent step without agent must fail");
    assert!(err.contains("orphan"), "error must name the step id: {err}");
    assert!(err.to_lowercase().contains("agent"), "error must mention agent: {err}");
}

#[test]
fn missing_agent_on_gate_step_fails_at_load() {
    let mut payload = base_payload("coder");
    payload.steps = Some(Pipeline {
        steps: vec![
            PipelineStep {
                id: "impl".into(),
                step_type: StepType::Agent,
                agent: Some("coder".into()),
                goal: None,
                after: vec![],
                model: None,
                provider: None,
                command: None,
                max_retries: None,
                gate: None,
            },
            PipelineStep {
                id: "naked-gate".into(),
                step_type: StepType::Gate,
                agent: None,
                goal: None,
                after: vec!["impl".into()],
                model: None,
                provider: None,
                command: None,
                max_retries: None,
                gate: None,
            },
        ],
        auto_merge: false,
    });

    let err = ensure_pipeline(payload).expect_err("gate step without agent must fail");
    assert!(err.contains("naked-gate"), "error must name the step id: {err}");
}

#[test]
fn no_pipeline_declared_yields_single_agent_step() {
    let payload = base_payload("coder");
    let resolved = ensure_pipeline(payload).expect("single-step default must validate");
    let pipeline = resolved.steps.expect("steps must be populated");

    assert_eq!(pipeline.steps.len(), 1, "exactly one step — no review, no merge");
    let step = &pipeline.steps[0];
    assert_eq!(step.step_type, StepType::Agent);
    assert_eq!(step.agent.as_deref(), Some("coder"));
    // The three purged opinions: no hardcoded model, no reviewer, no merge.
    assert!(step.model.is_none(), "no hardcoded model on default step");
    assert!(
        pipeline.steps.iter().all(|s| s.agent.as_deref() != Some("reviewer")),
        "default pipeline must not inject a reviewer step",
    );
    assert!(
        !pipeline.auto_merge,
        "default pipeline must not auto-merge",
    );
}

#[test]
fn run_step_without_agent_is_fine() {
    let mut payload = base_payload("coder");
    payload.steps = Some(Pipeline {
        steps: vec![PipelineStep {
            id: "shell".into(),
            step_type: StepType::Run,
            agent: None,
            goal: None,
            after: vec![],
            model: None,
            provider: None,
            command: Some("echo hi".into()),
            max_retries: None,
            gate: None,
        }],
        auto_merge: false,
    });
    ensure_pipeline(payload).expect("run steps need no agent");
}
