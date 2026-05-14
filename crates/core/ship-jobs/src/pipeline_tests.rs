use crate::pipeline::normalize_pipeline;

#[test]
fn test_agents_short_form() {
    let src = r#"{
        "name": "tdd",
        "agents": ["test-writer", "{{ impl_agent }}", "reviewer"]
    }"#;
    let (val, meta) = normalize_pipeline(src).unwrap();
    let steps = val["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(meta.name.as_deref(), Some("tdd"));

    // First step has no after
    assert!(steps[0].get("after").is_none() || steps[0]["after"].as_array().unwrap().is_empty());
    assert_eq!(steps[0]["agent"], "test-writer");

    // Second step depends on first
    assert_eq!(steps[1]["after"][0], "step-1");
    assert_eq!(steps[1]["agent"], "{{ impl_agent }}");

    // Third step depends on second
    assert_eq!(steps[2]["after"][0], "step-2");
}

#[test]
fn test_mixed_steps() {
    let src = r#"{
        "name": "research",
        "steps": [
            "researcher",
            "writer",
            { "run": "echo review" },
            "writer"
        ]
    }"#;
    let (val, _) = normalize_pipeline(src).unwrap();
    let steps = val["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 4);

    assert_eq!(steps[0]["type"], "agent");
    assert_eq!(steps[0]["agent"], "researcher");

    assert_eq!(steps[2]["type"], "run");
    assert_eq!(steps[2]["command"], "echo review");

    // Last step chains after the run step.
    assert_eq!(steps[3]["after"][0].as_str().unwrap(), steps[2]["id"].as_str().unwrap());
}

#[test]
fn test_agent_with_overrides() {
    let src = r#"{
        "name": "careful",
        "steps": [
            "test-writer",
            { "agent": "rust-lane", "model": "opus", "max_retries": 2 },
            "reviewer"
        ]
    }"#;
    let (val, _) = normalize_pipeline(src).unwrap();
    let steps = val["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[1]["agent"], "rust-lane");
    assert_eq!(steps[1]["model"], "opus");
    assert_eq!(steps[1]["max_retries"], 2);
}

#[test]
fn test_run_step() {
    let src = r#"{
        "name": "build",
        "steps": [
            "rust-lane",
            { "run": "cargo test --workspace" }
        ]
    }"#;
    let (val, _) = normalize_pipeline(src).unwrap();
    let steps = val["steps"].as_array().unwrap();
    assert_eq!(steps[1]["type"], "run");
    assert_eq!(steps[1]["command"], "cargo test --workspace");
}

#[test]
fn test_old_format_passthrough() {
    let src = r#"{
        "name": "tdd",
        "steps": [
            { "id": "test", "type": "agent", "agent": "test-writer" },
            { "id": "impl", "type": "agent", "agent": "rust-lane", "after": ["test"] }
        ]
    }"#;
    let (val, _) = normalize_pipeline(src).unwrap();
    let steps = val["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["id"], "test");
    assert_eq!(steps[1]["after"][0], "test");
}

#[test]
fn test_parallel_group() {
    let src = r#"{
        "name": "fanout",
        "steps": [
            "planner",
            { "parallel": ["researcher", "writer"] },
            "reviewer"
        ]
    }"#;
    let (val, _) = normalize_pipeline(src).unwrap();
    let steps = val["steps"].as_array().unwrap();
    // planner + 2 parallel + reviewer = 4
    assert_eq!(steps.len(), 4);

    // Both parallel steps depend on planner
    assert_eq!(steps[1]["after"][0].as_str().unwrap(), steps[0]["id"].as_str().unwrap());
    assert_eq!(steps[2]["after"][0].as_str().unwrap(), steps[0]["id"].as_str().unwrap());

    // Reviewer depends on both parallel steps
    let reviewer_after = steps[3]["after"].as_array().unwrap();
    assert_eq!(reviewer_after.len(), 2);
}

#[test]
fn test_meta_extraction() {
    let src = r#"{
        "name": "tdd",
        "description": "Test-driven development",
        "vars": { "impl_agent": "rust-lane" },
        "timeout": "2h",
        "max_retries": 1,
        "agents": ["test-writer"]
    }"#;
    let (_, meta) = normalize_pipeline(src).unwrap();
    assert_eq!(meta.name.as_deref(), Some("tdd"));
    assert_eq!(meta.description.as_deref(), Some("Test-driven development"));
    assert_eq!(meta.vars.as_ref().unwrap().get("impl_agent").unwrap(), "rust-lane");
    assert_eq!(meta.timeout.as_deref(), Some("2h"));
    assert_eq!(meta.max_retries, Some(1));
}

#[test]
fn test_error_on_missing_steps_and_agents() {
    let src = r#"{ "name": "broken" }"#;
    assert!(normalize_pipeline(src).is_err());
}

#[test]
fn test_error_on_invalid_jsonc() {
    let src = "not jsonc {{{}}}";
    assert!(normalize_pipeline(src).is_err());
}
