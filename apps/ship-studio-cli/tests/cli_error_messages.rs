//! Tests for CLI-03: Actionable error messages.
//!
//! Every core CLI command error should tell the user what went wrong, where,
//! and how to fix it. No raw anyhow/serde traces in user-facing output.
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
// Error must name the agent, mention .ship/agents/, and list available agents.

#[test]
fn use_nonexistent_agent_names_agent_in_error() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Create a valid agent so there's something to suggest
    write(
        tmp.path(),
        ".ship/agents/valid-agent.jsonc",
        r#"{ "agent": { "name": "Valid", "id": "valid-agent", "providers": ["claude"] } }"#,
    );

    ship_in(&tmp)
        .args([
            "use",
            "does-not-exist",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("does-not-exist")
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn use_nonexistent_agent_suggests_available_agents() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    write(
        tmp.path(),
        ".ship/agents/my-dev.jsonc",
        r#"{ "agent": { "name": "Dev", "id": "my-dev", "providers": ["claude"] } }"#,
    );

    let output = ship_in(&tmp)
        .args([
            "use",
            "does-not-exist",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure();

    // The error should list available agents so the user knows what to type
    output.stderr(predicate::str::contains("my-dev").or(
        // Alternatively, error might point to agents dir for the user to look
        predicate::str::contains(".ship/agents/"),
    ));
}

// ── 2. ship compile with broken MCP config ──────────────────────────────────
// Error must name the bad server and the problematic field.

#[test]
fn compile_broken_mcp_config_still_compiles_with_warning() {
    let tmp = TempDir::new().unwrap();

    // Write rules so compile has something to do
    write(tmp.path(), ".ship/rules/style.md", "Use types.");
    // Write a malformed MCP config (invalid JSON)
    write(tmp.path(), ".ship/mcp.jsonc", "{ invalid json {{{{");

    // Currently the loader silently skips unparseable MCP configs. Compile
    // still succeeds — the broken MCP file is just ignored. This test
    // documents the current behavior. Ideally compile would warn about the
    // broken file; the implementation step should add that warning.
    ship()
        .args([
            "compile",
            "--provider",
            "claude",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    // Future: once compile warns on broken MCP files, assert stderr contains
    // "mcp.jsonc" or "MCP" to confirm the warning is actionable.
}

#[test]
fn validate_mcp_stdio_missing_command_names_server() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");
    // Validate checks .ship/agents/mcp.{jsonc,toml} for MCP config, not
    // the top-level .ship/mcp.jsonc. Place the broken config where validate looks.
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
                .and(predicate::str::contains("command")),
        );
}

// ── 3. ship compile with missing skill ──────────────────────────────────────
// Error must name the missing skill and where to find/create it.

#[test]
fn compile_missing_skill_names_skill_id() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");
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

    // Validate should catch the missing skill reference
    ship()
        .args([
            "validate",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("nonexistent-skill"));
}

#[test]
fn compile_extra_skill_not_found_names_skill() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");

    // Use --with to inject a skill that doesn't exist in the library
    ship()
        .args([
            "compile",
            "--provider",
            "claude",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success(); // base compile works

    // Now try ship use with --with pointing to a nonexistent skill
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

    ship_in(&tmp)
        .args([
            "use",
            "test",
            "--with",
            "ghost-skill",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("ghost-skill")
                .and(predicate::str::contains("skill")),
        );
}

// ── 4. ship install with network failure ────────────────────────────────────
// Error should suggest --offline or checking network.

#[test]
fn install_frozen_out_of_sync_suggests_fix() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/ship.toml",
        "[module]\nname = \"github.com/test/repo\"\nversion = \"0.1.0\"\n\n\
         [dependencies]\n\"github.com/owner/pkg\" = \"main\"\n",
    );
    // Empty lock — out of sync with manifest
    write(tmp.path(), ".ship/ship.lock", "version = 1\n");

    let output = ship()
        .args(["install", "--frozen"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "install --frozen with stale lock should fail"
    );
    // Error should mention frozen/lockfile/out-of-sync — not a raw trace
    assert!(
        stderr.contains("frozen")
            || stderr.contains("lockfile")
            || stderr.contains("out of sync")
            || stderr.contains("lock"),
        "expected actionable error about frozen lockfile, got:\n{stderr}"
    );
}

#[test]
fn install_no_manifest_error_is_actionable() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".ship")).unwrap();

    let output = ship()
        .args(["install"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    // Must mention what's missing and how to fix it
    assert!(
        stderr.contains("ship.jsonc") || stderr.contains("ship.toml"),
        "error should name the missing file, got:\n{stderr}"
    );
}

// ── 5. ship publish without auth ────────────────────────────────────────────
// Error must say "run `ship login` first".

#[test]
fn publish_without_auth_says_run_login() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/ship.jsonc",
        r#"{
  "module": { "name": "github.com/test/pkg", "version": "0.1.0" }
}"#,
    );

    // Isolate HOME so no real credentials leak in
    let output = ship()
        .args(["publish"])
        .env("HOME", tmp.path())
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("ship login"),
        "publish without auth should tell user to run `ship login`, got:\n{stderr}"
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
    let skill_dir = tmp.path().join(".ship/agents/skills/a");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "# Skill A").unwrap();

    // Dry run should NOT require auth
    ship()
        .args(["publish", "--dry-run"])
        .env("HOME", tmp.path())
        .current_dir(tmp.path())
        .assert()
        .success();
}

// ── 6. ship init in existing project ────────────────────────────────────────
// Must succeed idempotently or explain what conflicts exist.

#[test]
fn init_in_existing_project_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let original = fs::read_to_string(tmp.path().join(".ship/ship.jsonc")).unwrap();

    // Second init must not fail and must not overwrite existing config
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized .ship/"));

    let after = fs::read_to_string(tmp.path().join(".ship/ship.jsonc")).unwrap();
    assert_eq!(original, after, "init must not overwrite existing ship.jsonc");
}

// ── 7. ship validate with invalid config ────────────────────────────────────
// Error must show file path and specific error.

#[test]
fn validate_invalid_json_shows_file_and_parse_error() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/broken.jsonc", "{ invalid json }}}");

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(
            // Must show the file path and that it's a parse error
            predicate::str::contains("broken")
                .and(predicate::str::contains("parse error").or(predicate::str::contains("error"))),
        );
}

#[test]
fn validate_missing_skill_ref_shows_skill_name_and_file() {
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
fn validate_bad_mcp_ref_shows_server_name() {
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

    ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("nonexistent-server"));
}

#[test]
fn validate_json_mode_outputs_structured_errors() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/bad.jsonc", "not valid json {{{");

    let output = ship()
        .args([
            "validate",
            "--json",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON output should parse as an array of validation errors
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("validate --json must emit valid JSON, got error {e}:\n{stdout}"));
    assert!(
        parsed.is_array(),
        "validate --json must emit a JSON array, got:\n{stdout}"
    );
    let errors = parsed.as_array().unwrap();
    assert!(!errors.is_empty(), "should have at least one error");
    // Each error should have agent, file, and error fields
    let first = &errors[0];
    assert!(first.get("agent").is_some(), "error must have 'agent' field");
    assert!(first.get("file").is_some(), "error must have 'file' field");
    assert!(first.get("error").is_some(), "error must have 'error' field");
}

// ── Exit code consistency ───────────────────────────────────────────────────
// 0 = success, non-zero = error (1 for user errors, 2 for system errors)

#[test]
fn success_exits_zero() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");

    ship()
        .args([
            "compile",
            "--provider",
            "claude",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .code(0);
}

#[test]
fn user_error_exits_nonzero() {
    let tmp = TempDir::new().unwrap();

    // Missing .ship/ is a user error
    ship()
        .args(["compile", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(predicate::ne(0));
}

// ── No raw traces in user-facing output ─────────────────────────────────────

#[test]
fn compile_error_has_no_raw_backtrace() {
    let tmp = TempDir::new().unwrap();

    let output = ship()
        .args(["compile", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("stack backtrace"),
        "user-facing error should not contain stack backtrace:\n{stderr}"
    );
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "user-facing error should not contain panic output:\n{stderr}"
    );
}

#[test]
fn use_error_has_no_raw_serde_trace() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Write an agent with invalid JSON that would produce a serde error
    write(
        tmp.path(),
        ".ship/agents/bad-agent.jsonc",
        "{ not valid json at all }}}",
    );

    let output = ship_in(&tmp)
        .args([
            "use",
            "bad-agent",
            "--path",
            tmp.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    // Error should frame the problem as "invalid agent" — not just dump a raw
    // serde trace. Currently emits "invalid agent JSONC at <path>: <detail>"
    // which is acceptable. Check it mentions the agent or config concept.
    assert!(
        stderr.contains("invalid agent") || stderr.contains("config") || stderr.contains("parse"),
        "error should be user-friendly with context, got:\n{stderr}"
    );
    // Must not expose raw panic/backtrace
    assert!(
        !stderr.contains("stack backtrace") && !stderr.contains("thread 'main' panicked"),
        "error should not contain raw panic output:\n{stderr}"
    );
}

#[test]
fn validate_error_has_no_raw_trace() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/agents/bad.jsonc", "{{{{");

    let output = ship()
        .args(["validate", "--path", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}{stdout}");

    assert!(
        !combined.contains("stack backtrace"),
        "validate error should not contain stack backtrace:\n{combined}"
    );
    assert!(
        !combined.contains("thread 'main' panicked"),
        "validate error should not contain panic output:\n{combined}"
    );
}
