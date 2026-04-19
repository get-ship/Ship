use crate::event_types;
use crate::payload::{JobCreatedPayload, StepCompletedPayload, StepDispatchedPayload};
use crate::pipeline::Pipeline;
use crate::record::JobRecord;
use crate::step::{PipelineStep, StepType};

#[test]
fn pipeline_step_run_type() {
    let step = PipelineStep {
        id: "build".into(),
        step_type: StepType::Run { command: "echo ok".into() },
        agent: None,
        goal: None,
        after: vec![],
    };
    match &step.step_type {
        StepType::Run { command } => assert_eq!(command, "echo ok"),
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn pipeline_step_agent_type() {
    let step = PipelineStep {
        id: "impl".into(),
        step_type: StepType::Agent,
        agent: Some("rust-lane".into()),
        goal: None,
        after: vec![],
    };
    assert_eq!(step.agent, Some("rust-lane".to_string()));
}

#[test]
fn pipeline_root_steps() {
    let pipeline = Pipeline {
        steps: vec![
            PipelineStep {
                id: "step1".into(),
                step_type: StepType::Agent,
                agent: None,
                goal: None,
                after: vec![],
            },
            PipelineStep {
                id: "step2".into(),
                step_type: StepType::Agent,
                agent: None,
                goal: None,
                after: vec!["step1".into()],
            },
            PipelineStep {
                id: "step3".into(),
                step_type: StepType::Agent,
                agent: None,
                goal: None,
                after: vec![],
            },
        ],
    };
    let roots = pipeline.root_steps();
    let ids: Vec<&str> = roots.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["step1", "step3"]);
}

#[test]
fn job_created_payload_roundtrips_serde() {
    let payload = JobCreatedPayload {
        job_id: "j-001".into(),
        slug: "add-auth".into(),
        agent: "rust-lane".into(),
        branch: "job/add-auth".into(),
        spec_path: ".artifacts/job-spec.md".into(),
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let back: JobCreatedPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(payload, back);
}

#[test]
fn step_dispatched_payload_fields() {
    let p = StepDispatchedPayload {
        job_id: "j-001".into(),
        step_index: 0,
        step_name: "implement".into(),
        step_type: "agent".into(),
    };
    assert_eq!(p.job_id, "j-001");
    assert_eq!(p.step_index, 0);
    assert_eq!(p.step_name, "implement");
    assert_eq!(p.step_type, "agent");
}

#[test]
fn step_completed_payload_fields() {
    let p = StepCompletedPayload {
        job_id: "j-001".into(),
        step_index: 0,
        step_name: "implement".into(),
    };
    assert_eq!(p.job_id, "j-001");
    assert_eq!(p.step_index, 0);
    assert_eq!(p.step_name, "implement");
}

#[test]
fn job_event_type_constants_exist() {
    assert_eq!(event_types::CREATED, "job.created");
    assert_eq!(event_types::STEP_DISPATCHED, "job.step_dispatched");
    assert_eq!(event_types::STEP_COMPLETED, "job.step_completed");
}

#[test]
fn job_record_default_has_no_steps() {
    let r = JobRecord::default();
    assert!(r.pipeline.is_none());
    assert!(r.current_step.is_none());
    assert!(r.completed_steps.is_empty());
}

#[test]
fn step_type_variants_exhaustive() {
    let variants = [
        StepType::Agent,
        StepType::Review,
        StepType::Run { command: "true".into() },
    ];
    for v in &variants {
        match v {
            StepType::Agent => {}
            StepType::Review => {}
            StepType::Run { .. } => {}
        }
    }
}

#[test]
fn ship_jobs_has_no_runtime_dep() {
    let manifest = include_str!("../Cargo.toml");
    assert!(
        !manifest.contains("runtime"),
        "ship-jobs must not depend on runtime"
    );
}
