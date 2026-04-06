use super::{Pipeline, PipelineStep, StepType};

fn agent_step(id: &str, after: &[&str]) -> PipelineStep {
    PipelineStep {
        id: id.to_string(),
        step_type: StepType::Agent,
        agent: Some("test-agent".to_string()),
        after: after.iter().map(|s| s.to_string()).collect(),
        model: None,
        provider: None,
    }
}

fn gate_step(id: &str, after: &[&str]) -> PipelineStep {
    PipelineStep {
        id: id.to_string(),
        step_type: StepType::Gate,
        agent: Some("reviewer".to_string()),
        after: after.iter().map(|s| s.to_string()).collect(),
        model: Some("sonnet".to_string()),
        provider: None,
    }
}

fn merge_step(id: &str, after: &[&str]) -> PipelineStep {
    PipelineStep {
        id: id.to_string(),
        step_type: StepType::Merge,
        agent: None,
        after: after.iter().map(|s| s.to_string()).collect(),
        model: None,
        provider: None,
    }
}

#[test]
fn root_steps_returns_steps_with_no_deps() {
    let pipeline = Pipeline {
        steps: vec![
            agent_step("a", &[]),
            agent_step("b", &["a"]),
            agent_step("c", &[]),
        ],
    };
    let roots: Vec<&str> = pipeline.root_steps().iter().map(|s| s.id.as_str()).collect();
    assert_eq!(roots, vec!["a", "c"]);
}

#[test]
fn root_steps_empty_pipeline() {
    let pipeline = Pipeline { steps: vec![] };
    assert!(pipeline.root_steps().is_empty());
}

#[test]
fn root_steps_all_have_deps() {
    let pipeline = Pipeline {
        steps: vec![
            agent_step("a", &["x"]),
            agent_step("b", &["y"]),
        ],
    };
    assert!(pipeline.root_steps().is_empty());
}

#[test]
fn unblocked_steps_initially_returns_roots() {
    let pipeline = Pipeline {
        steps: vec![
            agent_step("a", &[]),
            agent_step("b", &["a"]),
        ],
    };
    let unblocked: Vec<&str> = pipeline
        .unblocked_steps(&[])
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(unblocked, vec!["a"]);
}

#[test]
fn unblocked_steps_advances_after_completion() {
    let pipeline = Pipeline {
        steps: vec![
            agent_step("a", &[]),
            gate_step("b", &["a"]),
            merge_step("c", &["b"]),
        ],
    };

    let completed = vec!["a".to_string()];
    let unblocked: Vec<&str> = pipeline
        .unblocked_steps(&completed)
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(unblocked, vec!["b"]);
}

#[test]
fn unblocked_steps_requires_all_deps() {
    let pipeline = Pipeline {
        steps: vec![
            agent_step("a", &[]),
            agent_step("b", &[]),
            agent_step("c", &["a", "b"]),
        ],
    };

    // Only "a" completed — "c" still blocked on "b"
    let partial = vec!["a".to_string()];
    let unblocked: Vec<&str> = pipeline
        .unblocked_steps(&partial)
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(unblocked, vec!["b"]);

    // Both completed — "c" unblocked
    let full = vec!["a".to_string(), "b".to_string()];
    let unblocked: Vec<&str> = pipeline
        .unblocked_steps(&full)
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(unblocked, vec!["c"]);
}

#[test]
fn unblocked_steps_excludes_already_completed() {
    let pipeline = Pipeline {
        steps: vec![
            agent_step("a", &[]),
            agent_step("b", &["a"]),
        ],
    };
    let completed = vec!["a".to_string(), "b".to_string()];
    assert!(pipeline.unblocked_steps(&completed).is_empty());
}

#[test]
fn default_for_agent_produces_implement_gate_merge() {
    let pipeline = Pipeline::default_for_agent("rust-runtime");
    let ids: Vec<&str> = pipeline.steps.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["implement", "gate", "merge"]);
}

#[test]
fn default_for_agent_step_types() {
    let pipeline = Pipeline::default_for_agent("rust-runtime");
    assert_eq!(pipeline.steps[0].step_type, StepType::Agent);
    assert_eq!(pipeline.steps[1].step_type, StepType::Gate);
    assert_eq!(pipeline.steps[2].step_type, StepType::Merge);
}

#[test]
fn default_for_agent_sets_agent_on_implement() {
    let pipeline = Pipeline::default_for_agent("my-agent");
    assert_eq!(pipeline.steps[0].agent.as_deref(), Some("my-agent"));
}

#[test]
fn default_for_agent_dag_structure() {
    let pipeline = Pipeline::default_for_agent("x");

    // implement is a root
    assert!(pipeline.steps[0].after.is_empty());
    // gate depends on implement
    assert_eq!(pipeline.steps[1].after, vec!["implement"]);
    // merge depends on gate
    assert_eq!(pipeline.steps[2].after, vec!["gate"]);
}

#[test]
fn default_for_agent_unblocked_walk() {
    let pipeline = Pipeline::default_for_agent("x");

    // Initially only implement is unblocked
    let step0: Vec<&str> = pipeline.unblocked_steps(&[]).iter().map(|s| s.id.as_str()).collect();
    assert_eq!(step0, vec!["implement"]);

    // After implement, gate unblocks
    let step1: Vec<&str> = pipeline
        .unblocked_steps(&["implement".to_string()])
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(step1, vec!["gate"]);

    // After gate, merge unblocks
    let step2: Vec<&str> = pipeline
        .unblocked_steps(&["implement".to_string(), "gate".to_string()])
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(step2, vec!["merge"]);

    // All done — nothing unblocked
    let done = pipeline.unblocked_steps(&[
        "implement".to_string(),
        "gate".to_string(),
        "merge".to_string(),
    ]);
    assert!(done.is_empty());
}
