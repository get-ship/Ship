//! Tests for CLI-03: Actionable error messages.
//!
//! Every core CLI command error must tell the user what went wrong, where,
//! and how to fix it. Each error test asserts a remediation hint — a "try X",
//! "check Y", or "run Z" suggestion. No raw anyhow/serde traces in output.
//!
//! Each test runs the actual binary against an isolated temp directory.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn ship() -> Command {
    Command::cargo_bin("ship").unwrap()
}

fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_GLOBAL_DIR", tmp.path());
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

// ── 1. ship use <nonexistent> ────────────────────────────────────────────────

#[test]
fn use_nonexistent_agent_names_agent_and_suggests_fix() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    write(
        tmp.path(),
        ".ship/agents/valid-agent.jsonc",
        r#"{ "agent": { "name": "Valid", "id": "valid-agent", "providers": ["claude"] } }"#,
    );

    let output = ship_in(&tmp)
        .args(["use", "does-not-exist", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    // Names the agent
    assert!(stderr.contains("does-not-exist"), "must name the agent: {stderr}");
    // Points to .ship/agents/
    assert!(stderr.contains(".ship/agents/"), "must mention location: {stderr}");
    // Lists available agents as a remediation hint
    assert!(
        stderr.contains("valid-agent"),
        "must list available agents so user knows what to type: {stderr}"
    );
}

#[test]
fn use_nonexistent_agent_with_no_agents_dir_suggests_init() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();
    // Remove agents dir to simulate empty project
    let _ = fs::remove_dir_all(runtime::ShipHome::project_ship_dir(tmp.path()).join("agents"));

    let output = ship_in(&tmp)
        .args(["use", "anything", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    // When no agents exist at all, should suggest creating one
    assert!(
        stderr.contains(".ship/agents/") || stderr.contains("ship docs agents"),
        "must point user to agents dir or docs: {stderr}"
    );
}

// ── 2. ship compile with broken MCP config ──────────────────────────────────

#[test]
fn compile_broken_mcp_warns_about_file() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");
    write(tmp.path(), ".ship/mcp.jsonc", "{ invalid json {{{{");

    let output = ship()
        .args(["compile", "--provider", "claude", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Compile should still succeed but warn about the broken MCP file
    assert!(output.status.success());
    assert!(
        stderr.contains("mcp") || stderr.contains("MCP"),
        "compile must warn about broken MCP config: {stderr}"
    );
}

#[test]
fn validate_mcp_stdio_missing_command_shows_fix_hint() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/mcp.toml",
        "[[servers]]\nid = \"broken-server\"\nserver_type = \"stdio\"\n",
    );
    write(
        tmp.path(),
        ".ship/agents/test.jsonc",
        r#"{
  "agent": { "name": "Test", "id": "test", "providers": ["claude"] },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {}
}"#,
    );

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("broken-server")
                .and(predicate::str::contains("command"))
                .and(predicate::str::contains("Add")),
        );
}

// ── 3. ship compile with missing skill ──────────────────────────────────────

#[test]
fn validate_missing_skill_suggests_create_or_install() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/test-agent.jsonc",
        r#"{
  "agent": { "name": "Test", "id": "test-agent", "providers": ["claude"] },
  "skills": { "refs": ["nonexistent-skill"] },
  "mcp": { "servers": [] },
  "permissions": {}
}"#,
    );

    let output = ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success());
    assert!(stdout.contains("nonexistent-skill"), "must name the skill: {stdout}");
    // Remediation: where to create or how to install
    assert!(
        stdout.contains("create") || stdout.contains("install") || stdout.contains("skills/"),
        "must suggest creating the skill or running ship install: {stdout}"
    );
}

#[test]
fn compile_extra_skill_not_found_suggests_fix() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    write(
        tmp.path(),
        ".ship/agents/test.jsonc",
        r#"{
  "agent": { "name": "Test", "id": "test", "providers": ["claude"] },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": {}
}"#,
    );

    let output = ship_in(&tmp)
        .args(["use", "test", "--with", "ghost-skill", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("ghost-skill"), "must name the skill: {stderr}");
    assert!(
        stderr.contains("skill_paths") || stderr.contains("ship install"),
        "must suggest checking skill_paths or running ship install: {stderr}"
    );
}

// ── 4. ship install errors ──────────────────────────────────────────────────

#[test]
fn install_frozen_out_of_sync_suggests_run_without_frozen() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/ship.toml",
        "[module]\nname = \"github.com/test/repo\"\nversion = \"0.1.0\"\n\n\
         [dependencies]\n\"github.com/owner/pkg\" = \"main\"\n",
    );
    write(tmp.path(), ".ship/ship.lock", "version = 1\n");

    let output = ship()
        .args(["install", "--frozen"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("frozen") || stderr.contains("lock"),
        "must mention frozen/lockfile issue: {stderr}"
    );
    // Remediation: run without --frozen
    assert!(
        stderr.contains("ship install") || stderr.contains("without --frozen"),
        "must suggest running ship install without --frozen: {stderr}"
    );
}

#[test]
fn install_no_manifest_suggests_how_to_create() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(runtime::ShipHome::project_ship_dir(tmp.path())).unwrap();

    let output = ship()
        .args(["install"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("ship.jsonc") || stderr.contains("ship.toml"),
        "must name the missing file: {stderr}"
    );
    // Remediation: how to create the manifest
    assert!(
        stderr.contains("module") || stderr.contains("ship init"),
        "must suggest how to create a manifest: {stderr}"
    );
}

// ── 5. ship publish without auth ────────────────────────────────────────────

#[test]
fn publish_without_auth_says_run_login() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/ship.jsonc",
        r#"{ "module": { "name": "github.com/test/pkg", "version": "0.1.0" } }"#,
    );

    let output = ship()
        .args(["publish"])
        .env("HOME", tmp.path())
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    // Already includes "ship login" — verify the full remediation
    assert!(
        stderr.contains("ship login"),
        "must tell user to run ship login: {stderr}"
    );
    assert!(
        stderr.contains("ship publish"),
        "must suggest retrying publish after login: {stderr}"
    );
}

#[test]
fn publish_dry_run_without_auth_succeeds() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/ship.jsonc",
        r#"{
  "module": { "name": "github.com/test/pkg", "version": "0.1.0" },
  "exports": { "skills": ["agents/skills/a"] }
}"#,
    );
    let skill_dir = runtime::ShipHome::project_ship_dir(tmp.path()).join("agents/skills/a");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "# Skill A").unwrap();

    ship()
        .args(["publish", "--dry-run"])
        .env("HOME", tmp.path())
        .current_dir(tmp.path())
        .assert()
        .success();
}

// ── 6. ship init in existing project ────────────────────────────────────────

#[test]
fn init_in_existing_project_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    ship().args(["init"]).current_dir(tmp.path()).assert().success();
    let original = fs::read_to_string(runtime::ShipHome::project_ship_dir(tmp.path()).join("ship.jsonc")).unwrap();

    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized .ship/"));

    let after = fs::read_to_string(runtime::ShipHome::project_ship_dir(tmp.path()).join("ship.jsonc")).unwrap();
    assert_eq!(original, after, "init must not overwrite existing ship.jsonc");
}

// ── 7. ship validate with invalid config ────────────────────────────────────

#[test]
fn validate_parse_error_suggests_check_syntax() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/broken.jsonc", "{ invalid json }}}");

    let output = ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success());
    assert!(stdout.contains("broken"), "must name the file: {stdout}");
    assert!(
        stdout.contains("parse error") || stdout.contains("config parse"),
        "must say it's a parse error: {stdout}"
    );
    // Remediation hint
    assert!(
        stdout.contains("Check") || stdout.contains("syntax") || stdout.contains("JSON"),
        "must suggest checking syntax: {stdout}"
    );
}

#[test]
fn validate_missing_skill_ref_shows_skill_and_agent() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/dev.jsonc",
        r#"{
  "agent": { "name": "Dev", "id": "dev", "providers": ["claude"] },
  "skills": { "refs": ["missing-skill-xyz"] },
  "mcp": { "servers": [] },
  "permissions": {}
}"#,
    );

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("missing-skill-xyz")
                .and(predicate::str::contains("dev")),
        );
}

#[test]
fn validate_bad_mcp_ref_shows_server_and_hint() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/dev.jsonc",
        r#"{
  "agent": { "name": "Dev", "id": "dev", "providers": ["claude"] },
  "skills": { "refs": [] },
  "mcp": { "servers": ["nonexistent-server"] },
  "permissions": {}
}"#,
    );

    let output = ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!output.status.success());
    assert!(stdout.contains("nonexistent-server"), "must name server: {stdout}");
    assert!(
        stdout.contains("mcp") || stdout.contains("add"),
        "must suggest adding the server to mcp config: {stdout}"
    );
}

#[test]
fn validate_json_mode_outputs_structured_errors() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/bad.jsonc", "not valid json {{{");

    let output = ship()
        .args(["validate", "--json", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("validate --json must emit valid JSON, got error {e}:\n{stdout}"));
    assert!(parsed.is_array(), "must emit JSON array: {stdout}");
    let errors = parsed.as_array().unwrap();
    assert!(!errors.is_empty(), "should have at least one error");
    let first = &errors[0];
    assert!(first.get("agent").is_some(), "error must have 'agent' field");
    assert!(first.get("file").is_some(), "error must have 'file' field");
    assert!(first.get("error").is_some(), "error must have 'error' field");
}

// ── Exit codes ──────────────────────────────────────────────────────────────

#[test]
fn success_exits_zero() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");
    ship()
        .args(["compile", "--provider", "claude", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .code(0);
}

#[test]
fn user_error_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["compile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(predicate::ne(0));
}

// ── No raw traces ───────────────────────────────────────────────────────────

#[test]
fn compile_error_has_no_raw_backtrace() {
    let tmp = TempDir::new().unwrap();
    let output = ship()
        .args(["compile", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("stack backtrace"), "no backtrace: {stderr}");
    assert!(!stderr.contains("thread 'main' panicked"), "no panic: {stderr}");
}

#[test]
fn use_error_has_no_raw_serde_trace() {
    let tmp = TempDir::new().unwrap();
    ship().args(["init"]).current_dir(tmp.path()).assert().success();
    write(tmp.path(), ".ship/agents/bad-agent.jsonc", "{ not valid json at all }}}");

    let output = ship_in(&tmp)
        .args(["use", "bad-agent", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("invalid agent") || stderr.contains("config") || stderr.contains("parse"),
        "must be user-friendly: {stderr}"
    );
    assert!(!stderr.contains("stack backtrace"), "no backtrace: {stderr}");
    assert!(!stderr.contains("thread 'main' panicked"), "no panic: {stderr}");
}

#[test]
fn validate_error_has_no_raw_trace() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/bad.jsonc", "{{{{");

    let output = ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!combined.contains("stack backtrace"), "no backtrace: {combined}");
    assert!(!combined.contains("thread 'main' panicked"), "no panic: {combined}");
}
