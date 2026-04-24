use super::*;
use crate::events::job::{Pipeline, PipelineStep, StepType};

fn step(id: &str, agent: &str) -> PipelineStep {
    PipelineStep {
        id: id.to_string(),
        step_type: StepType::Agent,
        agent: Some(agent.to_string()),
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
fn header_split_fallback_partitions_by_agent_name() {
    let pipeline = tdd_pipeline();
    let body = "\
# Overall
## test-writer
write the failing tests

## rust-runtime
implement against the tests

## reviewer
grade the implementation
";
    let split = split_by_agent_headers(body, &pipeline).expect("must split");
    assert!(split[0].contains("write the failing tests"));
    assert!(split[1].contains("implement against the tests"));
    assert!(split[2].contains("grade the implementation"));
}

#[test]
fn header_split_returns_none_when_a_step_is_missing() {
    let pipeline = tdd_pipeline();
    let body = "## test-writer\nonly one stage\n";
    assert!(split_by_agent_headers(body, &pipeline).is_none());
}

#[test]
fn legacy_fallback_shares_spec_across_stages() {
    let tmp = tempfile::tempdir().unwrap();
    let pipeline = tdd_pipeline();
    write_pipeline_specs(tmp.path(), "slug", &pipeline, "one body for all", None).unwrap();
    let a = std::fs::read_to_string(tmp.path().join(".artifacts/pipeline/01-step-1.md")).unwrap();
    let b = std::fs::read_to_string(tmp.path().join(".artifacts/pipeline/02-step-2.md")).unwrap();
    assert_eq!(a, "one body for all");
    assert_eq!(b, "one body for all");
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
