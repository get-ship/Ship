use super::format_step_label;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn dry_run_infers_step_kind_from_body() {
    let vars = HashMap::new();
    let run_shape = json!({ "id": "build", "run": "cargo check" });
    let agent_shape = json!({ "id": "impl", "agent": "rust-lane" });
    let explicit = json!({ "id": "t", "type": "run", "run": "echo explicit" });

    assert_eq!(format_step_label(&run_shape, &vars), "[run] cargo check");
    assert_eq!(format_step_label(&agent_shape, &vars), "[agent] rust-lane");
    assert_eq!(format_step_label(&explicit, &vars), "[run] echo explicit");
}

#[test]
fn dry_run_preserves_normalized_run_and_empty_fallback() {
    let vars = HashMap::new();
    let normalized = json!({ "id": "run-1", "type": "run", "run": "echo hi" });
    let empty = json!({ "id": "x" });

    assert_eq!(format_step_label(&normalized, &vars), "[run] echo hi");
    assert_eq!(format_step_label(&empty, &vars), "[agent] ?");
}
