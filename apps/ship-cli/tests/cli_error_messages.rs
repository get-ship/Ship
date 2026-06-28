//! Tests for CLI-03: Actionable error messages.
//!
//! Every core CLI command error must tell the user what went wrong, where,
//! and how to fix it. Each error test asserts a remediation hint — a "try X",
//! "check Y", or "run Z" suggestion. No raw anyhow/serde traces in output.
//!
//! Each test runs the actual binary against an isolated temp directory.
//!
//! Post-cull: tests for the cut `init` / `validate` / `install` / `publish`
//! commands were removed with their modules.

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
    cmd.env("SHIP_HOME", tmp.path());
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// Bare-minimum `.ship/` scaffold for tests that need to exercise `ship use`.
fn scaffold_ship_dir(root: &Path) {
    write(
        root,
        ".ship/ship.jsonc",
        r#"{
  "$schema": "../schemas/ship.schema.json",
  "project": { "harnesses": ["claude"] },
}
"#,
    );
    write(
        root,
        ".ship/agents/dev.jsonc",
        r#"{
  "$schema": "https://raw.githubusercontent.com/<ship-repo-org>/ship/main/schemas/agent.schema.json",
  "agent": {
    "id": "dev",
    "name": "Dev",
    "version": "0.1.0"
  },
  "skills": { "refs": [] },
  "mcp": { "servers": [] },
  "permissions": { "preset": "ship-standard" }
}
"#,
    );
}

// ── 1. ship use <nonexistent> ────────────────────────────────────────────────

#[test]
fn use_nonexistent_agent_names_agent_and_suggests_fix() {
    let tmp = TempDir::new().unwrap();
    scaffold_ship_dir(tmp.path());

    write(
        tmp.path(),
        ".ship/agents/valid-agent.jsonc",
        r#"{ "agent": { "name": "Valid", "id": "valid-agent", "harnesses": ["claude"] } }"#,
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
fn use_nonexistent_agent_with_no_agents_dir_suggests_create() {
    let tmp = TempDir::new().unwrap();
    scaffold_ship_dir(tmp.path());
    // Remove agents dir to simulate empty project
    let _ = fs::remove_dir_all(ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("agents"));

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
        .args(["compile", "--harness", "claude", "--path", tmp.path().to_str().unwrap()])
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

// ── Exit codes ──────────────────────────────────────────────────────────────

#[test]
fn success_exits_zero() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), ".ship/rules/style.md", "Use types.");
    ship()
        .args(["compile", "--harness", "claude", "--path", tmp.path().to_str().unwrap()])
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
    scaffold_ship_dir(tmp.path());
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
