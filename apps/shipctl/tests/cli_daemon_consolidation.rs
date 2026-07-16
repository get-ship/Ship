//! Tests for shipctl: daemon consolidation.
//!
//! After RT-04 (service mesh), mesh runs inside shipd. The separate
//! `ship network serve/status/stop` surface is redundant. These tests
//! define the target contract:
//!
//! - `shipctl daemon *` is the single control surface
//! - `shipctl --help` does not advertise `network` as a subcommand
//! - One daemon process, one PID file

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// CLI command with SHIP_HOME isolated to `tmp`.
fn shipctl_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("shipctl").unwrap();
    cmd.env("SHIP_HOME", tmp.path());
    cmd
}

fn shipctl() -> Command {
    Command::cargo_bin("shipctl").unwrap()
}

// ── Daemon commands are the single control surface ───────────────────────────

#[test]
fn test_daemon_status_works_when_not_running() {
    let tmp = TempDir::new().unwrap();

    shipctl_in(&tmp)
        .args(["daemon", "status"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not running"));
}

#[test]
fn test_daemon_stop_works_when_not_running() {
    let tmp = TempDir::new().unwrap();

    shipctl_in(&tmp)
        .args(["daemon", "stop"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not running"));
}

#[test]
fn test_daemon_start_does_not_require_network_command() {
    let tmp = TempDir::new().unwrap();

    let result = shipctl_in(&tmp)
        .args(["daemon", "start"])
        .env("HOME", tmp.path())
        .assert();

    let output = result.get_output().clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stderr.contains("ship network") && !stdout.contains("ship network"),
        "daemon start must not reference 'ship network', got:\nstdout: {}\nstderr: {}",
        stdout,
        stderr,
    );
}

// ── Help output reflects consolidation ───────────────────────────────────────

#[test]
fn test_help_does_not_list_network_subcommand() {
    let output = shipctl()
        .arg("--help")
        .output()
        .expect("failed to run shipctl --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("network"),
        "shipctl --help must not list 'network' as a subcommand, got:\n{}",
        stdout,
    );
}

#[test]
fn test_daemon_help_lists_start_stop_status() {
    shipctl()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("status"));
}

// ── Single process — no separate network PID ─────────────────────────────────

#[test]
fn test_daemon_uses_single_pid_file() {
    let tmp = TempDir::new().unwrap();

    let _result = shipctl_in(&tmp)
        .args(["daemon", "start"])
        .env("HOME", tmp.path())
        .output()
        .expect("failed to run daemon start");

    let ship_dir = tmp.path().join(".ship");
    assert!(
        !ship_dir.join("network.pid").exists(),
        "network.pid must not exist — daemon uses daemon.pid now",
    );
}
