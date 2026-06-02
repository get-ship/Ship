//! Tests for `pipeline_loader::ensure_pipeline`.
//!
//! Covers the two load-time invariants restored by the platform-purge job:
//! missing agent on an agent/gate step fails loud, and jobs without any
//! declared pipeline resolve to a single-step Agent pipeline — no hardcoded
//! review/merge/sonnet.

use super::pipeline_loader::ensure_pipeline;
use ship_substrate::events::pipeline::{
    PipelineRunStartedPayload, OutputScope, Pipeline, PipelineStep, StepOutput, StepPromote, StepType,
};

fn base_payload(agent: &str) -> PipelineRunStartedPayload {
    PipelineRunStartedPayload {
        pipeline_run_id: "j-1".into(),
        slug: "slug-1".into(),
        agent: agent.into(),
        branch: "job/slug-1".into(),
        spec_path: ".ship/specs/slug-1.md".into(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        harness: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
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
            harness: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: None,
        }],
        auto_merge: false,
    });

    let err = ensure_pipeline(payload).expect_err("agent step without agent must fail");
    assert!(err.contains("orphan"), "error must name the step id: {err}");
    assert!(err.to_lowercase().contains("agent"), "error must mention agent: {err}");
}

#[test]
fn missing_agent_on_review_step_fails_at_load() {
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
                harness: None,
                command: None,
                max_retries: None,
                gate: None,
                produces: vec![],
                consumes: vec![],
                promotes: vec![],
                fanout: None,
            invocation: None,
            },
            PipelineStep {
                id: "naked-gate".into(),
                step_type: StepType::Review,
                agent: None,
                goal: None,
                after: vec!["impl".into()],
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
            },
        ],
        auto_merge: false,
    });

    let err = ensure_pipeline(payload).expect_err("review step without agent must fail");
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
            harness: None,
            command: Some("echo hi".into()),
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: None,
        }],
        auto_merge: false,
    });
    ensure_pipeline(payload).expect("run steps need no agent");
}

// ── I/O validation tests ──────────────────────────────────────────────────────

fn agent_step(id: &str, agent: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: StepType::Agent,
        agent: Some(agent.into()),
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
fn consumes_unknown_name_fails_at_load() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("impl", "coder");
    step.consumes = vec!["nonexistent".into()];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("unknown consume must fail");
    assert!(err.contains("nonexistent"), "error must name the missing output: {err}");
}

#[test]
fn consumes_own_output_fails_at_load() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("impl", "coder");
    step.produces = vec![StepOutput { name: "patch".into(), scope: OutputScope::Run, required: false }];
    step.consumes = vec!["patch".into()];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("self-consume must fail");
    assert!(err.contains("patch"), "error must mention the output name: {err}");
}

#[test]
fn consumes_with_valid_producer_passes() {
    let mut payload = base_payload("coder");
    let mut producer = agent_step("write-tests", "coder");
    producer.produces = vec![StepOutput {
        name: "failing_tests".into(),
        scope: OutputScope::Run,
        required: true,
    }];
    let mut consumer = agent_step("implement", "coder");
    consumer.after = vec!["write-tests".into()];
    consumer.consumes = vec!["failing_tests".into()];
    payload.steps = Some(Pipeline {
        steps: vec![producer, consumer],
        auto_merge: false,
    });
    ensure_pipeline(payload).expect("valid producer must pass");
}

#[test]
fn promotes_unknown_name_fails_at_load() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("gate", "reviewer");
    step.promotes = vec![StepPromote {
        name: "nonexistent".into(),
        target_scope: OutputScope::Project,
    }];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("unknown promote name must fail");
    assert!(err.contains("nonexistent"), "error must name the bad promote: {err}");
}

#[test]
fn promotes_valid_name_and_scope_passes() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("gate", "reviewer");
    step.produces = vec![StepOutput {
        name: "patch".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    step.promotes = vec![StepPromote {
        name: "patch".into(),
        target_scope: OutputScope::Project,
    }];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    ensure_pipeline(payload).expect("valid promote must pass");
}

#[test]
fn promotes_invalid_scope_transition_fails_at_load() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("gate", "reviewer");
    step.produces = vec![StepOutput {
        name: "patch".into(),
        scope: OutputScope::Project,
        required: false,
    }];
    step.promotes = vec![StepPromote {
        name: "patch".into(),
        target_scope: OutputScope::Run, // Project → Run is invalid
    }];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("invalid scope transition must fail");
    assert!(err.to_lowercase().contains("scope") || err.contains("transition"),
        "error must mention scope transition: {err}");
}

#[test]
fn produces_name_with_path_traversal_fails_at_load() {
    // Security: a pipeline that declares a produces.name containing "../"
    // must be rejected at load time. Otherwise `input_dir.join(name)` in
    // step_io::write_input_file would escape the worktree.
    let mut payload = base_payload("coder");
    let mut step = agent_step("evil", "coder");
    step.produces = vec![StepOutput {
        name: "../../../etc/evil".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("traversal in produces must fail");
    assert!(
        err.contains("..") || err.to_lowercase().contains("traversal") || err.contains("disallowed"),
        "error must cite traversal/disallowed chars: {err}"
    );
}

#[test]
fn consumes_with_path_traversal_fails_at_load() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("consumer", "coder");
    step.consumes = vec!["../../secret".into()];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("traversal in consumes must fail");
    assert!(
        err.contains("..") || err.to_lowercase().contains("traversal") || err.contains("disallowed"),
        "error must cite traversal: {err}"
    );
}

#[test]
fn produces_name_leading_dot_rejected() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("s1", "coder");
    step.produces = vec![StepOutput {
        name: ".hidden".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("leading dot must be rejected");
    assert!(err.to_lowercase().contains("."), "error must name the offending char: {err}");
}

#[test]
fn produces_name_with_slash_rejected() {
    let mut payload = base_payload("coder");
    let mut step = agent_step("s1", "coder");
    step.produces = vec![StepOutput {
        name: "sub/dir/file".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    payload.steps = Some(Pipeline { steps: vec![step], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("slash must be rejected");
    assert!(err.to_lowercase().contains("disallowed"), "error must flag disallowed chars: {err}");
}

#[test]
fn duplicate_produces_name_across_steps_fails_at_load() {
    // Two sibling steps both claim to produce "patch" — the consumes
    // resolver would non-deterministically pick one. Reject at load.
    let mut payload = base_payload("coder");
    let mut s1 = agent_step("lane-a", "coder");
    s1.produces = vec![StepOutput {
        name: "patch".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    let mut s2 = agent_step("lane-b", "coder");
    s2.produces = vec![StepOutput {
        name: "patch".into(),
        scope: OutputScope::Run,
        required: false,
    }];
    payload.steps = Some(Pipeline { steps: vec![s1, s2], auto_merge: false });
    let err = ensure_pipeline(payload).expect_err("duplicate produces name must fail");
    assert!(err.contains("patch"), "error must name the duplicated output: {err}");
    assert!(err.contains("lane-a") && err.contains("lane-b"),
        "error must list both offending steps: {err}");
}

#[test]
fn multiple_producers_are_indexed_correctly() {
    let mut payload = base_payload("coder");
    let mut s1 = agent_step("write-tests", "coder");
    s1.produces = vec![StepOutput { name: "tests".into(), scope: OutputScope::Run, required: true }];
    let mut s2 = agent_step("implement", "coder");
    s2.after = vec!["write-tests".into()];
    s2.consumes = vec!["tests".into()];
    s2.produces = vec![StepOutput { name: "patch".into(), scope: OutputScope::Run, required: false }];
    let mut s3 = agent_step("gate", "reviewer");
    s3.after = vec!["implement".into()];
    s3.consumes = vec!["tests".into(), "patch".into()];
    s3.promotes = vec![StepPromote { name: "patch".into(), target_scope: OutputScope::Project }];
    payload.steps = Some(Pipeline {
        steps: vec![s1, s2, s3],
        auto_merge: false,
    });
    ensure_pipeline(payload).expect("3-step TDD pipeline must pass validation");
}
