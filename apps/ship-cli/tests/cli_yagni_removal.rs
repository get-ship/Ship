//! Tests that removed commands don't appear in help, and retained commands work.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn ship() -> Command { Command::cargo_bin("ship").unwrap() }

fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_GLOBAL_DIR", tmp.path());
    cmd
}

#[test]
fn help_does_not_list_agents_subcommand() {
    ship()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("agents").not());
}

#[test]
fn help_does_not_list_skills_subcommand() {
    ship()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("skills").not());
}

#[test]
fn help_does_not_list_vars_subcommand() {
    ship()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vars").not());
}

#[test]
fn retained_compile_still_works() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".ship/rules")).unwrap();
    std::fs::write(
        tmp.path().join(".ship/rules/style.md"),
        "Use explicit types.",
    )
    .unwrap();

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
}

#[test]
fn retained_use_still_works() {
    let tmp = TempDir::new().unwrap();

    ship_in(&tmp)
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .success();

    std::fs::create_dir_all(tmp.path().join(".ship/agents")).unwrap();
    std::fs::write(
        tmp.path().join(".ship/agents/test-agent.jsonc"),
        r#"{ "agent": { "name": "Test", "id": "test-agent", "providers": ["claude"] } }"#,
    )
    .unwrap();

    ship_in(&tmp)
        .args(["use", "test-agent", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn retained_install_still_works() {
    let tmp = TempDir::new().unwrap();

    std::fs::create_dir_all(tmp.path().join(".ship")).unwrap();
    std::fs::write(
        tmp.path().join(".ship/ship.toml"),
        "[module]\nname = \"github.com/test/repo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    ship()
        .args(["install"])
        .current_dir(tmp.path())
        .assert()
        .success();
}
