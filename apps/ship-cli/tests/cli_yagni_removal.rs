//! Tests that removed commands don't exist, and retained commands work.

use assert_cmd::Command;
use tempfile::TempDir;

fn ship() -> Command { Command::cargo_bin("ship").unwrap() }

fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_HOME", tmp.path());
    cmd
}

/// Clap exits with code 2 for unrecognised subcommands.
#[test]
fn agents_subcommand_does_not_exist() {
    ship()
        .args(["agents"])
        .assert()
        .code(2);
}

#[test]
fn skills_subcommand_does_not_exist() {
    ship()
        .args(["skills"])
        .assert()
        .code(2);
}

#[test]
fn vars_subcommand_does_not_exist() {
    ship()
        .args(["vars"])
        .assert()
        .code(2);
}

#[test]
fn retained_compile_still_works() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("rules")).unwrap();
    std::fs::write(
        ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("rules/style.md"),
        "Use explicit types.",
    )
    .unwrap();

    ship()
        .args([
            "compile",
            "--harness",
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

    std::fs::create_dir_all(ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("agents")).unwrap();
    std::fs::write(
        ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("agents/test-agent.jsonc"),
        r#"{ "agent": { "name": "Test", "id": "test-agent", "harnesses": ["claude"] } }"#,
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

    std::fs::create_dir_all(ship_substrate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    std::fs::write(
        ship_substrate::ShipHome::project_ship_dir(tmp.path()).join("ship.jsonc"),
        "{ \"name\": \"github.com/test/repo\" }\n",
    )
    .unwrap();

    ship()
        .args(["install"])
        .current_dir(tmp.path())
        .assert()
        .success();
}
