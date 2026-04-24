//! Unit tests for the competing-approaches fanout primitive.
//!
//! Tests cover: config parse, lane ID uniqueness, judge-step reference
//! validation, lane-scoped output naming, and lane-scoped consumes resolution.

use crate::events::job::{
    FanoutJudge, FanoutLane, FanoutSpec, Pipeline, PipelineStep, StepOutput, StepType,
    OutputScope,
};
use crate::services::orchestrator::pipeline_loader::ensure_pipeline;
use crate::services::orchestrator::step_io::parse_lane_scoped_ref;

// ── helpers ──────────────────────────────────────────────────────────────────

fn agent_step(id: &str, agent: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: StepType::Agent,
        agent: Some(agent.into()),
        goal: None,
        after: vec![],
        model: None,
        provider: None,
        command: None,
        max_retries: None,
        gate: None,
        produces: vec![],
        consumes: vec![],
        promotes: vec![],
        fanout: None,
    }
}

fn lane(id: &str, agent: &str) -> FanoutLane {
    FanoutLane {
        id: id.into(),
        agent: agent.into(),
        model: None,
        provider: None,
    }
}

fn base_payload(
    slug: &str,
    steps: Vec<PipelineStep>,
) -> crate::events::job::JobCreatedPayload {
    crate::events::job::JobCreatedPayload {
        job_id: "j-test".into(),
        slug: slug.into(),
        agent: "implementer".into(),
        branch: "job/test".into(),
        spec_path: ".artifacts/job-spec.md".into(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        provider: None,
        depends_on: None,
        pipeline: None,
        steps: Some(Pipeline { steps, auto_merge: false }),
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: true,
        baseline_sha: None,
    }
}

// ── fanout config parse ───────────────────────────────────────────────────────

#[test]
fn fanout_spec_round_trips_through_serde() {
    let spec = FanoutSpec {
        lanes: vec![
            lane("claude", "implementer"),
            lane("sonnet", "implementer"),
        ],
        judge: FanoutJudge { step: "choose".into() },
        min_successful_lanes: 1,
    };
    let json = serde_json::to_string(&spec).unwrap();
    let back: FanoutSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lanes.len(), 2);
    assert_eq!(back.judge.step, "choose");
    assert_eq!(back.min_successful_lanes, 1);
}

#[test]
fn fanout_min_successful_defaults_to_one() {
    let json = r#"{ "lanes": [], "judge": { "step": "choose" } }"#;
    let spec: FanoutSpec = serde_json::from_str(json).unwrap();
    assert_eq!(spec.min_successful_lanes, 1);
}

#[test]
fn pipeline_step_with_fanout_round_trips() {
    let yaml = r#"
id: draft
type: agent
agent: implementer
fanout:
  lanes:
    - id: lane-a
      agent: impl-a
      model: claude-opus-4-7
    - id: lane-b
      agent: impl-b
  judge:
    step: choose
produces:
  - name: output
    scope: run
"#;
    let step: PipelineStep = serde_yaml::from_str(yaml).unwrap();
    let fanout = step.fanout.expect("fanout must be present");
    assert_eq!(fanout.lanes.len(), 2);
    assert_eq!(fanout.lanes[0].id, "lane-a");
    assert_eq!(fanout.lanes[0].model.as_deref(), Some("claude-opus-4-7"));
    assert_eq!(fanout.lanes[1].id, "lane-b");
    assert_eq!(fanout.judge.step, "choose");
}

// ── lane ID uniqueness ────────────────────────────────────────────────────────

#[test]
fn duplicate_lane_ids_fail_validation() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![lane("dup", "a"), lane("dup", "b")],
            judge: FanoutJudge { step: "choose".into() },
            min_successful_lanes: 1,
        }),
        produces: vec![StepOutput { name: "output".into(), scope: OutputScope::Run, required: false }],
        ..agent_step("draft", "implementer")
    };
    let judge = PipelineStep {
        id: "choose".into(),
        after: vec!["draft".into()],
        consumes: vec!["dup.output".into()],
        ..agent_step("choose", "judge")
    };
    let payload = base_payload("slug", vec![fanout_step, judge]);
    let err = ensure_pipeline(payload).unwrap_err();
    assert!(err.contains("duplicate lane id"), "got: {err}");
}

// ── judge step reference validation ──────────────────────────────────────────

#[test]
fn judge_step_not_found_fails_validation() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![lane("l1", "impl")],
            judge: FanoutJudge { step: "nonexistent".into() },
            min_successful_lanes: 1,
        }),
        ..agent_step("draft", "implementer")
    };
    let payload = base_payload("slug", vec![fanout_step]);
    let err = ensure_pipeline(payload).unwrap_err();
    assert!(err.contains("nonexistent"), "got: {err}");
}

#[test]
fn judge_step_missing_after_dependency_fails_validation() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![lane("l1", "impl")],
            judge: FanoutJudge { step: "choose".into() },
            min_successful_lanes: 1,
        }),
        produces: vec![StepOutput { name: "output".into(), scope: OutputScope::Run, required: false }],
        ..agent_step("draft", "implementer")
    };
    // Judge does NOT have "draft" in `after`
    let judge = PipelineStep {
        id: "choose".into(),
        after: vec![],  // missing "draft"
        ..agent_step("choose", "judge-agent")
    };
    let payload = base_payload("slug", vec![fanout_step, judge]);
    let err = ensure_pipeline(payload).unwrap_err();
    assert!(err.contains("draft") || err.contains("after"), "got: {err}");
}

#[test]
fn valid_fanout_pipeline_passes_validation() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![lane("claude", "impl-a"), lane("sonnet", "impl-b")],
            judge: FanoutJudge { step: "choose".into() },
            min_successful_lanes: 1,
        }),
        produces: vec![StepOutput { name: "result".into(), scope: OutputScope::Run, required: false }],
        ..agent_step("draft", "implementer")
    };
    let judge = PipelineStep {
        id: "choose".into(),
        after: vec!["draft".into()],
        consumes: vec!["claude.result".into(), "sonnet.result".into()],
        ..agent_step("choose", "judge-agent")
    };
    let payload = base_payload("slug", vec![fanout_step, judge]);
    assert!(ensure_pipeline(payload).is_ok(), "valid fanout pipeline must pass");
}

// ── lane-scoped output naming ─────────────────────────────────────────────────

#[test]
fn parse_lane_scoped_ref_splits_correctly() {
    assert_eq!(parse_lane_scoped_ref("lane-claude.draft"), Some(("lane-claude", "draft")));
    assert_eq!(parse_lane_scoped_ref("sonnet.output"), Some(("sonnet", "output")));
    assert_eq!(parse_lane_scoped_ref("plain_name"), None);
    assert_eq!(parse_lane_scoped_ref(""), None);
}

#[test]
fn lane_scoped_consume_validates_lane_existence() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![lane("claude", "impl")],
            judge: FanoutJudge { step: "choose".into() },
            min_successful_lanes: 1,
        }),
        produces: vec![StepOutput { name: "output".into(), scope: OutputScope::Run, required: false }],
        ..agent_step("draft", "implementer")
    };
    // Consume references a lane_id "unknown" that doesn't exist
    let judge = PipelineStep {
        id: "choose".into(),
        after: vec!["draft".into()],
        consumes: vec!["unknown.output".into()],
        ..agent_step("choose", "judge-agent")
    };
    let payload = base_payload("slug", vec![fanout_step, judge]);
    let err = ensure_pipeline(payload).unwrap_err();
    assert!(err.contains("unknown") || err.contains("lane"), "got: {err}");
}

#[test]
fn lane_scoped_consume_validates_output_exists_on_fanout_step() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![lane("claude", "impl")],
            judge: FanoutJudge { step: "choose".into() },
            min_successful_lanes: 1,
        }),
        produces: vec![StepOutput { name: "result".into(), scope: OutputScope::Run, required: false }],
        ..agent_step("draft", "implementer")
    };
    // Consume references output "nonexistent" which isn't in fanout_step.produces
    let judge = PipelineStep {
        id: "choose".into(),
        after: vec!["draft".into()],
        consumes: vec!["claude.nonexistent".into()],
        ..agent_step("choose", "judge-agent")
    };
    let payload = base_payload("slug", vec![fanout_step, judge]);
    let err = ensure_pipeline(payload).unwrap_err();
    assert!(err.contains("nonexistent"), "got: {err}");
}

// ── lane agent required ───────────────────────────────────────────────────────

#[test]
fn lane_with_empty_agent_fails_validation() {
    let fanout_step = PipelineStep {
        id: "draft".into(),
        fanout: Some(FanoutSpec {
            lanes: vec![FanoutLane { id: "l1".into(), agent: "".into(), model: None, provider: None }],
            judge: FanoutJudge { step: "choose".into() },
            min_successful_lanes: 1,
        }),
        produces: vec![],
        ..agent_step("draft", "implementer")
    };
    let judge = PipelineStep {
        id: "choose".into(),
        after: vec!["draft".into()],
        ..agent_step("choose", "judge-agent")
    };
    let payload = base_payload("slug", vec![fanout_step, judge]);
    let err = ensure_pipeline(payload).unwrap_err();
    assert!(err.contains("agent"), "got: {err}");
}
