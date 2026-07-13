//! Tests that previously-removed commands don't exist, and that retained
//! commands work post-cull.
//!
//! Post-cull: removed `retained_install_still_works` (ship install was cut)
//! and refactored `retained_use_still_works` to scaffold `.ship/` manually
//! now that `ship init` is gone.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ship() -> Command { Command::cargo_bin("ship").unwrap() }

fn ship_in(tmp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ship").unwrap();
    cmd.env("SHIP_HOME", tmp.path());
    cmd
}

fn scaffold_ship_dir(tmp: &TempDir) {
    let root = tmp.path();
    let ship_dir = root.to_path_buf(); // stack root
    fs::create_dir_all(ship_dir.join("agents")).unwrap();
    fs::write(ship_dir.join("stack.config.ts"), "export default {};\n").unwrap();
    fs::write(
        ship_dir.join("agents/test-agent.jsonc"),
        r#"{ "agent": { "name": "Test", "id": "test-agent", "harnesses": ["claude"] } }"#,
    )
    .unwrap();
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
fn init_subcommand_does_not_exist() {
    // ship init was cut in this cull.
    ship()
        .args(["init"])
        .assert()
        .code(2);
}

#[test]
fn install_subcommand_does_not_exist() {
    // ship install was cut in this cull.
    ship()
        .args(["install"])
        .assert()
        .code(2);
}

#[test]
fn retained_compile_still_works() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("stack.config.ts"), "export default {};\n").unwrap();
    // Rules-format cut: the rule body lives on the agent composition
    // as `rules: { inline: "..." }`. The on-disk `rules/<id>.md`
    // convention died.
    std::fs::create_dir_all(tmp.path().join("agents")).unwrap();
    std::fs::write(
        tmp.path().join("agents/demo.ts"),
        r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
  rules: { inline: "Use explicit types." },
});
"#,
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
    scaffold_ship_dir(&tmp);

    ship_in(&tmp)
        .args(["use", "test-agent", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success();
}
