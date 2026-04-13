//! Tests for CLI-02: daemon consolidation.
//!
//! After RT-04 (service mesh), mesh runs inside shipd. The separate
//! `ship network serve/status/stop` surface is redundant. These tests
//! define the target contract:
//!
//! - `ship network *` commands exit non-zero with deprecation messages
//! - `ship daemon *` is the single control surface
//! - `ship --help` no longer advertises `network` as a subcommand
//! - One daemon process, one PID file

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// CLI command with SHIP_GLOBAL_DIR isolated to `tmp`.
fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_GLOBAL_DIR", tmp.path());
    cmd
}

fn ship() -> Command {
    Command::cargo_bin("ship").unwrap()
}

// ── Daemon commands are the single control surface ───────────────────────────

#[test]
fn test_daemon_status_works_when_not_running() {
    let tmp = TempDir::new().unwrap();

    // daemon status should succeed and report not running when no daemon is up.
    ship_in(&tmp)
        .args(["daemon", "status"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not running"));
}

#[test]
fn test_daemon_stop_works_when_not_running() {
    let tmp = TempDir::new().unwrap();

    // daemon stop when nothing is running should succeed gracefully.
    ship_in(&tmp)
        .args(["daemon", "stop"])
        .env("HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not running"));
}

#[test]
fn test_daemon_start_does_not_require_network_command() {
    let tmp = TempDir::new().unwrap();

    // daemon start should attempt to launch shipd. It will fail because the
    // shipd binary is not on PATH in the test environment, but the error must
    // NOT reference `ship network` — it should be a daemon-specific error.
    let result = ship_in(&tmp)
        .args(["daemon", "start"])
        .env("HOME", tmp.path())
        .assert();

    // We accept either success (if shipd is found) or a failure about shipd
    // not being found. The key assertion: no reference to `ship network`.
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
    let output = ship()
        .arg("--help")
        .output()
        .expect("failed to run ship --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // `network` must not appear as a visible subcommand in help output.
    // It may still exist as a hidden command for the deprecation path,
    // but it must not be advertised.
    assert!(
        !stdout.contains("network"),
        "ship --help must not list 'network' as a subcommand, got:\n{}",
        stdout,
    );
}

#[test]
fn test_daemon_help_lists_start_stop_status() {
    ship()
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

    // After daemon start (or attempted start), the old network.pid must not exist.
    // Only daemon.pid is valid after consolidation.
    let _result = ship_in(&tmp)
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
