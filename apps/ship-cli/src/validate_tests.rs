use super::*;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn valid_agent_jsonc() -> &'static str {
    r#"{
  "agent": { "id": "test", "name": "test", "harnesses": ["claude"] },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": { "preset": "ship-standard" }
}"#
}

fn setup() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let agents_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    write(
        tmp.path(),
        ".ship/permissions.jsonc",
        r#"{ "ship-standard": { "default_mode": "acceptEdits" } }"#,
    );
    (tmp, agents_dir)
}

#[test]
fn valid_agent_passes() {
    let (tmp, agents_dir) = setup();
    write(tmp.path(), ".ship/agents/test.jsonc", valid_agent_jsonc());
    let agent_path = agents_dir.join("test.jsonc");
    let report = validate_agent("test", &agent_path, &agents_dir);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn malformed_jsonc_reports_error() {
    let (tmp, agents_dir) = setup();
    write(
        tmp.path(),
        ".ship/agents/bad.jsonc",
        "this is not [[valid jsonc",
    );
    let agent_path = agents_dir.join("bad.jsonc");
    let report = validate_agent("bad", &agent_path, &agents_dir);
    assert_eq!(report.errors.len(), 1);
    assert!(
        report.errors[0].error.contains("config parse error"),
        "{:?}",
        report.errors[0].error
    );
}

#[test]
fn missing_skill_ref_reports_error() {
    let (tmp, agents_dir) = setup();
    let src = r#"{
  "agent": { "id": "test", "name": "test", "harnesses": ["claude"] },
  "skills": { "refs": ["nonexistent-skill"] }
}"#;
    write(tmp.path(), ".ship/agents/test.jsonc", src);
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.error.contains("nonexistent-skill")),
        "{:?}",
        report.errors
    );
}

#[test]
fn existing_skill_ref_at_dir_layout_passes() {
    // Canonical: `.ship/skills/<id>/SKILL.md` (the directory shape).
    let (tmp, agents_dir) = setup();
    write(
        tmp.path(),
        ".ship/skills/my-skill/SKILL.md",
        "---\nname: my-skill\nstable-id: my-skill\ndescription: Test.\n---\n\nContent.",
    );
    let src = r#"{
  "agent": { "id": "test", "name": "test", "harnesses": ["claude"] },
  "skills": { "refs": ["my-skill"] }
}"#;
    write(tmp.path(), ".ship/agents/test.jsonc", src);
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn existing_skill_ref_at_flat_layout_passes() {
    // Flat: `.ship/skills/<id>.md` — also legal, the loader accepts both.
    let (tmp, agents_dir) = setup();
    write(
        tmp.path(),
        ".ship/skills/flat-skill.md",
        "---\nname: flat-skill\nstable-id: flat-skill\ndescription: Test.\n---\n\nContent.",
    );
    let src = r#"{
  "agent": { "id": "test", "name": "test", "harnesses": ["claude"] },
  "skills": { "refs": ["flat-skill"] }
}"#;
    write(tmp.path(), ".ship/agents/test.jsonc", src);
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn skill_via_skill_paths_passes() {
    // `project.skill_paths` adds extra roots — the validator must honor
    // them (compiler does).
    let (tmp, agents_dir) = setup();
    write(
        tmp.path(),
        ".ship/ship.jsonc",
        r#"{"id":"t","project":{"skill_paths":["docs/"]}}"#,
    );
    write(
        tmp.path(),
        "docs/extra-skill/SKILL.md",
        "---\nname: extra-skill\nstable-id: extra-skill\ndescription: Test.\n---\n\nContent.",
    );
    let src = r#"{
  "agent": { "id": "test", "name": "test", "harnesses": ["claude"] },
  "skills": { "refs": ["extra-skill"] }
}"#;
    write(tmp.path(), ".ship/agents/test.jsonc", src);
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn stdio_mcp_missing_command_reports_error() {
    let (tmp, agents_dir) = setup();
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{
  "servers": [
    { "id": "bad-stdio", "server_type": "stdio" }
  ]
}"#,
    );
    write(tmp.path(), ".ship/agents/test.jsonc", valid_agent_jsonc());
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.error.contains("missing 'command'")),
        "{:?}",
        report.errors
    );
}

#[test]
fn http_mcp_missing_url_reports_error() {
    let (tmp, agents_dir) = setup();
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{
  "servers": [
    { "id": "bad-http", "server_type": "http" }
  ]
}"#,
    );
    write(tmp.path(), ".ship/agents/test.jsonc", valid_agent_jsonc());
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.error.contains("missing 'url'")),
        "{:?}",
        report.errors
    );
}

#[test]
fn unknown_permissions_preset_reports_error() {
    let (tmp, agents_dir) = setup();
    let src = r#"{
  "agent": { "id": "test", "name": "test", "harnesses": ["claude"] },
  "permissions": { "preset": "nonexistent-preset" }
}"#;
    write(tmp.path(), ".ship/agents/test.jsonc", src);
    let report = validate_agent("test", &agents_dir.join("test.jsonc"), &agents_dir);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.error.contains("nonexistent-preset")),
        "{:?}",
        report.errors
    );
}

#[test]
fn run_validate_exits_nonzero_on_errors() {
    let (tmp, _agents_dir) = setup();
    write(tmp.path(), ".ship/agents/bad.jsonc", "not valid jsonc [[");
    let result = run_validate(None, false, tmp.path());
    assert!(result.is_err());
}

#[test]
fn run_validate_json_flag_emits_array() {
    let (tmp, _agents_dir) = setup();
    write(tmp.path(), ".ship/agents/bad.jsonc", "not valid jsonc [[");
    let _ = run_validate(None, true, tmp.path());
}

#[test]
fn run_validate_passes_on_valid_config() {
    let (tmp, _agents_dir) = setup();
    write(tmp.path(), ".ship/agents/test.jsonc", valid_agent_jsonc());
    let result = run_validate(None, false, tmp.path());
    assert!(result.is_ok(), "{:?}", result);
}
