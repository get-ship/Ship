//! Tests for `views::agents_state`. Kept in a sibling file so the module
//! itself stays under the 300-line cap.

use super::*;
use std::path::Path;
use tempfile::tempdir;

fn write_agent(ship_dir: &Path, id: &str, body: &str) {
    let d = ship_dir.join("agents");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(format!("{id}.jsonc")), body).unwrap();
}

#[test]
fn empty_dir_returns_empty_vec() {
    let tmp = tempdir().unwrap();
    assert!(list_agents(tmp.path()).is_empty());
}

#[test]
fn parses_flat_and_nested_shapes() {
    let tmp = tempdir().unwrap();
    write_agent(
        tmp.path(),
        "coder",
        r#"{
            // flat shape
            "name": "Coder",
            "description": "Writes code.",
            "skills": ["tdd", "refactor"],
            "providers": ["claude", "codex"],
            "mcp_servers": ["ship"]
        }"#,
    );
    write_agent(
        tmp.path(),
        "reviewer",
        r#"{
            "agent": { "name": "Reviewer", "description": "Reviews PRs.", "providers": ["claude"] },
            "skills": [{"id": "review"}, "style"],
            "rules": ["kind", "clear"]
        }"#,
    );
    // Project-shape config that nests collections under sub-objects.
    // Mirrors .ship/agents/commander.jsonc.
    write_agent(
        tmp.path(),
        "commander",
        r#"{
            "agent": { "name": "Commander", "providers": ["claude", "codex"] },
            "skills": { "refs": ["cmdr", "write-adr", "ship-agents"] },
            "mcp":    { "servers": ["ship", "typescript"] },
            "rules":  { "refs": ["discipline"] },
            "permissions": { "preset": "ship-autonomous" }
        }"#,
    );
    let out = list_agents(tmp.path());
    assert_eq!(out.len(), 3);
    let coder = out.iter().find(|a| a.name == "Coder").unwrap();
    assert_eq!(coder.skills_count, 2);
    assert_eq!(coder.providers_count, 2);
    assert_eq!(coder.mcp_count, 1);
    let reviewer = out.iter().find(|a| a.name == "Reviewer").unwrap();
    assert_eq!(reviewer.skills_count, 2);
    assert_eq!(reviewer.rules_count, 2);
    let commander = out.iter().find(|a| a.name == "Commander").unwrap();
    assert_eq!(commander.skills_count, 3, "nested skills.refs");
    assert_eq!(commander.mcp_count, 2, "nested mcp.servers");
    assert_eq!(commander.rules_count, 1, "nested rules.refs");
    assert_eq!(commander.providers_count, 2);

    let detail = load_agent(tmp.path(), "commander").unwrap();
    assert_eq!(detail.skills.len(), 3);
    assert_eq!(detail.mcp_servers, vec!["ship", "typescript"]);
    assert_eq!(detail.permissions_tier, "autonomous");
}

#[test]
fn load_agent_returns_none_for_missing() {
    let tmp = tempdir().unwrap();
    assert!(load_agent(tmp.path(), "nope").is_none());
}

#[test]
fn load_agent_returns_detail_with_defaults() {
    let tmp = tempdir().unwrap();
    write_agent(
        tmp.path(),
        "coder",
        r#"{ "name": "Coder", "description": "Writes code." }"#,
    );
    let d = load_agent(tmp.path(), "coder").unwrap();
    assert_eq!(d.name, "Coder");
    assert_eq!(d.permissions_tier, "standard");
    assert_eq!(d.letter, "C");
}

#[test]
fn strip_jsonc_handles_comments() {
    let src = r#"{
        // line
        /* block */
        "x": 1,
    }"#;
    let v: serde_json::Value = serde_json::from_str(&strip_jsonc(src)).unwrap();
    assert_eq!(v["x"], 1);
}

#[test]
fn strip_jsonc_preserves_multibyte_utf8() {
    // Regression: the trailing-comma pass used to walk the output as
    // bytes and push `b as char`, which mangled any codepoint above
    // U+007F (e.g. emdash 0xE2 0x80 0x94 → U+00E2 U+0080 U+0094).
    let src = r#"{
        // description with an emdash
        "desc": "orchestrator — captures intent",
        "list": [1, 2,],
    }"#;
    let stripped = strip_jsonc(src);
    assert!(
        stripped.contains("orchestrator — captures intent"),
        "emdash must survive strip_jsonc verbatim: {stripped}",
    );
    let v: serde_json::Value = serde_json::from_str(&stripped).unwrap();
    assert_eq!(v["desc"], "orchestrator — captures intent");
    assert_eq!(v["list"], serde_json::json!([1, 2]));
}
