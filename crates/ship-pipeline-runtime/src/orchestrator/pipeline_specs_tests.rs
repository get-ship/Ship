use super::*;
use ship_substrate::events::pipeline::{Pipeline, PipelineStep, StepType};
use crate::orchestrator::step_io::ResolvedInput;

fn step(id: &str, agent: &str) -> PipelineStep {
    PipelineStep {
        id: id.to_string(),
        step_type: StepType::Agent,
        agent: Some(agent.to_string()),
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

fn tdd_pipeline() -> Pipeline {
    Pipeline {
        steps: vec![
            step("step-1", "test-writer"),
            step("step-2", "rust-runtime"),
            step("step-3", "reviewer"),
        ],
        auto_merge: false,
    }
}

#[test]
fn second_call_preserves_existing_step_files() {
    let tmp = tempfile::tempdir().unwrap();
    let pipeline = tdd_pipeline();
    let specs = vec![
        "original 1".to_string(),
        "original 2".to_string(),
        "original 3".to_string(),
    ];
    write_pipeline_specs(tmp.path(), "slug", &pipeline, "overall", Some(&specs)).unwrap();

    let replacements = vec![
        "replaced 1".to_string(),
        "replaced 2".to_string(),
        "replaced 3".to_string(),
    ];
    write_pipeline_specs(tmp.path(), "slug", &pipeline, "new overall", Some(&replacements))
        .unwrap();

    let dir = tmp.path().join(".artifacts/pipeline");
    assert_eq!(std::fs::read_to_string(dir.join("spec.md")).unwrap(), "overall");
    assert_eq!(std::fs::read_to_string(dir.join("01-step-1.md")).unwrap(), "original 1");
    assert_eq!(std::fs::read_to_string(dir.join("02-step-2.md")).unwrap(), "original 2");
    assert_eq!(std::fs::read_to_string(dir.join("03-step-3.md")).unwrap(), "original 3");
}

#[test]
fn step_specs_block_writes_per_stage_files() {
    let tmp = tempfile::tempdir().unwrap();
    let pipeline = tdd_pipeline();
    let specs = vec![
        "# TEST\nwrite tests".to_string(),
        "# IMPL\nmake tests pass".to_string(),
        "# REVIEW\ngrade output".to_string(),
    ];
    let dir = write_pipeline_specs(
        tmp.path(), "slug", &pipeline, "overall body", Some(&specs),
    )
    .unwrap();

    assert_eq!(dir, tmp.path().join(".artifacts/pipeline"));
    assert_eq!(
        std::fs::read_to_string(dir.join("spec.md")).unwrap(),
        "overall body"
    );
    assert!(std::fs::read_to_string(dir.join("01-step-1.md")).unwrap().contains("write tests"));
    assert!(std::fs::read_to_string(dir.join("02-step-2.md")).unwrap().contains("make tests pass"));
    assert!(std::fs::read_to_string(dir.join("03-step-3.md")).unwrap().contains("grade output"));
}

#[test]
fn empty_step_specs_writes_empty_per_stage_files() {
    // Pipelines without a step_specs block produce empty per-stage files.
    // The dispatched agent works from the overall spec + its profile.
    let tmp = tempfile::tempdir().unwrap();
    let pipeline = tdd_pipeline();
    write_pipeline_specs(tmp.path(), "slug", &pipeline, "overall only", None).unwrap();
    let a = std::fs::read_to_string(tmp.path().join(".artifacts/pipeline/01-step-1.md")).unwrap();
    let b = std::fs::read_to_string(tmp.path().join(".artifacts/pipeline/02-step-2.md")).unwrap();
    assert!(a.is_empty(), "no step_specs => empty per-stage file");
    assert!(b.is_empty(), "no step_specs => empty per-stage file");
    assert_eq!(
        std::fs::read_to_string(tmp.path().join(".artifacts/pipeline/spec.md")).unwrap(),
        "overall only"
    );
}

#[test]
fn dispatch_prompt_points_at_own_and_sibling_specs() {
    let pipeline = tdd_pipeline();
    let out = render_dispatch_prompt(Some("tdd"), &pipeline, &pipeline.steps[1], &[]);
    assert!(out.contains("Read `.artifacts/pipeline/02-step-2.md`"));
    assert!(out.contains(".artifacts/pipeline/spec.md"));
    assert!(out.contains(".artifacts/pipeline/01-step-1.md"));
    assert!(out.contains("upstream"));
    assert!(out.contains(".artifacts/pipeline/03-step-3.md"));
    assert!(out.contains("downstream"));
    assert!(out.contains("Runtime contract"));
}

fn plain_input(name: &str, artifact_id: &str) -> ResolvedInput {
    ResolvedInput {
        name: name.into(),
        artifact_id: artifact_id.into(),
        worktree_path: format!(".artifacts/input/{name}"),
        size: 512,
        mime: "text/plain".into(),
        failure_reason: None,
    }
}

fn lane_input(lane_id: &str, output: &str, artifact_id: &str) -> ResolvedInput {
    ResolvedInput {
        name: format!("{lane_id}.{output}"),
        artifact_id: artifact_id.into(),
        worktree_path: format!(".artifacts/input/{lane_id}.{output}"),
        size: 1024,
        mime: "text/markdown".into(),
        failure_reason: None,
    }
}

fn failed_lane_input(lane_id: &str, reason: &str) -> ResolvedInput {
    ResolvedInput {
        name: format!("{lane_id}.failed"),
        artifact_id: "marker-id".into(),
        worktree_path: format!(".artifacts/input/{lane_id}.failed"),
        size: 64,
        mime: "text/plain".into(),
        failure_reason: Some(reason.into()),
    }
}

/// Plain (non-lane) inputs render in the "Inputs available" section.
#[test]
fn orchestrator_pipeline_specs_plain_inputs_section() {
    let pipeline = tdd_pipeline();
    let inputs = vec![
        plain_input("patch", "art-abc"),
        plain_input("failing_tests", "art-def"),
    ];
    let out = render_dispatch_prompt(Some("tdd"), &pipeline, &pipeline.steps[2], &inputs);
    assert!(out.contains("## Inputs available"), "must have inputs section");
    assert!(out.contains("patch"), "must list patch input");
    assert!(out.contains("art-abc"), "must include artifact id");
    assert!(out.contains("failing_tests"), "must list failing_tests input");
    assert!(
        !out.contains("## Inputs from fanout"),
        "plain inputs must not produce a fanout section"
    );
}

/// Lane-scoped successful inputs render grouped by lane under the fanout section.
#[test]
fn orchestrator_pipeline_specs_fanout_inputs_section_groups_by_lane() {
    let mut fanout_step = step("draft", "implementer");
    fanout_step.fanout = Some(ship_substrate::events::pipeline::FanoutSpec {
        lanes: vec![
            ship_substrate::events::pipeline::FanoutLane {
                id: "lane-a".into(),
                agent: "impl".into(),
                model: None,
                harness: None,
            },
            ship_substrate::events::pipeline::FanoutLane {
                id: "lane-b".into(),
                agent: "impl".into(),
                model: None,
                harness: None,
            },
        ],
        judge: ship_substrate::events::pipeline::FanoutJudge { step: "choose".into() },
        min_successful_lanes: 1,
    });
    let judge = step("choose", "judge");
    let pipeline = Pipeline {
        steps: vec![fanout_step, judge.clone()],
        auto_merge: false,
    };

    let inputs = vec![
        lane_input("lane-a", "draft", "art-a"),
        lane_input("lane-b", "draft", "art-b"),
    ];
    let out = render_dispatch_prompt(Some("tdd"), &pipeline, &judge, &inputs);
    assert!(
        out.contains("## Inputs from fanout"),
        "must have fanout inputs section"
    );
    assert!(out.contains("lane-a"), "must mention lane-a");
    assert!(out.contains("lane-b"), "must mention lane-b");
    assert!(out.contains("art-a"), "must include lane-a artifact id");
    assert!(out.contains("art-b"), "must include lane-b artifact id");
    assert!(
        !out.contains("FAILED"),
        "successful lanes must not show FAILED marker"
    );
}

/// Failed lanes render with "FAILED — <reason>" instead of their output.
#[test]
fn orchestrator_pipeline_specs_fanout_failed_lane_rendered_with_reason() {
    let judge = step("choose", "judge");
    let pipeline = Pipeline {
        steps: vec![step("draft", "impl"), judge.clone()],
        auto_merge: false,
    };

    let inputs = vec![
        lane_input("lane-a", "draft", "art-a"),
        failed_lane_input("lane-c", "agent crashed after 30s"),
    ];
    let out = render_dispatch_prompt(Some("tdd"), &pipeline, &judge, &inputs);
    assert!(
        out.contains("lane-c: FAILED"),
        "failed lane must render with FAILED label: {out}"
    );
    assert!(
        out.contains("agent crashed after 30s"),
        "failure reason must appear in output: {out}"
    );
    assert!(
        out.contains("lane-a"),
        "successful lane must still appear alongside failed lane"
    );
}
