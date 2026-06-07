//! Contract-freeze Item 1: GateRule must use the list-of-rules + list-of-globs
//! canonical shape. The legacy single-object and pipe-separated string
//! shorthands are rejected at deserialize with a pointer to the canonical
//! syntax.

use super::*;

#[test]
fn canonical_list_of_rules_with_glob_lists_round_trips() {
    let yaml = r#"
require_changed:
  - match: ["**/*.rs", "**/*.toml"]
    not_match: ["**/tests/**"]
    min: 2
forbid_changed:
  - match: [".artifacts/**"]
    min: 0
:"#;
    let g: GateSpec = serde_yaml::from_str(yaml).expect("canonical shape parses");
    assert_eq!(g.require_changed.len(), 1);
    assert_eq!(g.require_changed[0].match_patterns,
        vec!["**/*.rs".to_string(), "**/*.toml".to_string()]);
    assert_eq!(g.require_changed[0].not_match, vec!["**/tests/**".to_string()]);
    assert_eq!(g.require_changed[0].min, 2);
    assert_eq!(g.forbid_changed[0].match_patterns, vec![".artifacts/**".to_string()]);
}

#[test]
fn legacy_single_object_rule_is_rejected_with_actionable_error() {
    let yaml = r#"
require_changed:
  match: ["**/*.rs"]
  min: 1
:"#;
    let err = serde_yaml::from_str::<GateSpec>(yaml).unwrap_err().to_string();
    assert!(
        err.contains("array") && err.contains("contract freeze"),
        "error must point at canonical syntax + freeze rationale: {err}"
    );
}

#[test]
fn legacy_pipe_separated_match_string_is_rejected() {
    let yaml = r#"
require_changed:
  - match: "**/*.rs|**/*.toml"
    min: 1
:"#;
    let err = serde_yaml::from_str::<GateSpec>(yaml).unwrap_err().to_string();
    assert!(
        err.contains("list of glob strings") || err.contains("contract freeze"),
        "string match value must be rejected with canonical-syntax pointer: {err}"
    );
}

#[test]
fn pipe_inside_list_entry_is_rejected() {
    // The list shape was always canonical for the loader, but we used to
    // also split entries on '|'. That's the same dual shape one level
    // deeper — reject.
    let yaml = r#"
require_changed:
  - match: ["**/*.rs|**/*.toml"]
    min: 1
:"#;
    let err = serde_yaml::from_str::<GateSpec>(yaml).unwrap_err().to_string();
    assert!(err.contains("contains '|'"), "pipe inside list entry must be flagged: {err}");
}
