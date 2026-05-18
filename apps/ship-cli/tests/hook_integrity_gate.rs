//! Integration tests for the PreToolUse integrity gate exposed via
//! `ship hook before-tool` — defense Layer 2 per `SECURITY.md`.
//!
//! These tests exercise the real `ship` binary the way Claude Code (and
//! every other Ship-compiled harness) does at runtime: pipe a hook payload
//! into stdin, parse stdout, assert the permission decision. They are the
//! contract test for the on-disk hook protocol.

use assert_cmd::Command;
use serde_json::{Value, json};
use tempfile::TempDir;

fn run_hook(stdin: &str) -> (i32, String) {
    let tmp = TempDir::new().unwrap();
    let assert = Command::cargo_bin("ship")
        .unwrap()
        // Isolate from any real ship state — the gate must work even when
        // no daemon is running and no project is initialised.
        .env("SHIP_HOME", tmp.path())
        .args(["hook", "before-tool"])
        .write_stdin(stdin)
        .assert();
    let out = assert.get_output();
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    (code, stdout)
}

fn deny_decision(stdout: &str) -> Option<Value> {
    let v: Value = serde_json::from_str(stdout.trim()).ok()?;
    let inner = v.get("hookSpecificOutput")?;
    if inner.get("permissionDecision")?.as_str()? == "deny" {
        Some(inner.clone())
    } else {
        None
    }
}

#[test]
fn write_to_ship_machine_denies_with_reason() {
    let payload = json!({
        "tool_name": "Write",
        "tool_input": { "file_path": "~/.ship/machine/daemon/token" },
    })
    .to_string();
    let (code, stdout) = run_hook(&payload);
    assert_eq!(code, 0, "hook must exit 0 to surface JSON; stdout={stdout}");
    let inner = deny_decision(&stdout)
        .unwrap_or_else(|| panic!("expected deny decision, got: {stdout}"));
    let reason = inner["permissionDecisionReason"].as_str().unwrap();
    assert!(
        reason.contains(".ship/machine"),
        "deny reason should name the protected segment: {reason}"
    );
}

#[test]
fn bash_tar_ship_data_denies() {
    let payload = json!({
        "tool_name": "Bash",
        "tool_input": { "command": "tar -czf /tmp/x.tgz ~/.ship/data/" },
    })
    .to_string();
    let (code, stdout) = run_hook(&payload);
    assert_eq!(code, 0);
    assert!(
        deny_decision(&stdout).is_some(),
        "expected deny for tar of ~/.ship/data/; stdout={stdout}"
    );
}

#[test]
fn ordinary_write_passes_through() {
    let payload = json!({
        "tool_name": "Write",
        "tool_input": { "file_path": "/tmp/ship-gate-test.txt" },
    })
    .to_string();
    let (code, stdout) = run_hook(&payload);
    assert_eq!(code, 0);
    // Allow path: gate stays silent on this branch; downstream daemon
    // logging may produce stdout but it must not be a deny decision.
    assert!(
        deny_decision(&stdout).is_none(),
        "ordinary write must not be denied; stdout={stdout}"
    );
}

#[test]
fn malformed_input_fails_closed() {
    let (code, stdout) = run_hook("{not valid json");
    assert_eq!(code, 0);
    let inner = deny_decision(&stdout)
        .unwrap_or_else(|| panic!("malformed input must deny; stdout={stdout}"));
    let reason = inner["permissionDecisionReason"].as_str().unwrap();
    assert!(
        reason.contains("malformed"),
        "reason should explain fail-closed: {reason}"
    );
}
