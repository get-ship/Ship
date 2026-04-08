//! Tests for CLI-01 YAGNI removal — removed commands show helpful messages,
//! `--help` no longer lists them, retained commands still work.

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
fn agents_list_shows_removal_message() {
    ship()
        .args(["agents", "list"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn agents_create_shows_removal_message() {
    ship()
        .args(["agents", "create", "test-agent"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn agents_edit_shows_removal_message() {
    ship()
        .args(["agents", "edit", "test-agent"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn agents_delete_shows_removal_message() {
    ship()
        .args(["agents", "delete", "test-agent"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn agents_clone_shows_removal_message() {
    ship()
        .args(["agents", "clone", "src-agent", "dst-agent"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn agents_log_shows_removal_message() {
    ship()
        .args(["agents", "log", "some message"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(predicate::str::contains(".ship/agents/")),
        );
}

#[test]
fn skills_list_shows_removal_message() {
    let tmp = TempDir::new().unwrap();
    ship()
        .args(["skills", "list"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("ship.jsonc")),
                ),
        );
}

#[test]
fn skills_remove_shows_removal_message() {
    ship()
        .args(["skills", "remove", "some-skill"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("ship.jsonc")),
                ),
        );
}

#[test]
fn skills_create_shows_removal_message() {
    ship()
        .args(["skills", "create", "new-skill"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("ship.jsonc")),
                ),
        );
}

#[test]
fn vars_set_shows_removal_message() {
    ship()
        .args(["vars", "set", "skill-id", "key", "value"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("skill variables")),
                ),
        );
}

#[test]
fn vars_get_shows_removal_message() {
    ship()
        .args(["vars", "get", "skill-id"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("skill variables")),
                ),
        );
}

#[test]
fn vars_append_shows_removal_message() {
    ship()
        .args(["vars", "append", "skill-id", "key", r#""elem""#])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("skill variables")),
                ),
        );
}

#[test]
fn vars_reset_shows_removal_message() {
    ship()
        .args(["vars", "reset", "skill-id"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("removed")
                .or(predicate::str::contains("Removed"))
                .and(
                    predicate::str::contains(".ship/skills/")
                        .or(predicate::str::contains("skill variables")),
                ),
        );
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
