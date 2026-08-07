//! `shipctl doctor` surfaces fleet org membership under isolated XDG data home.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn shipctl_isolated(tmp: &TempDir, data_home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("shipctl").unwrap();
    cmd.env("SHIP_HOME", tmp.path());
    cmd.env("HOME", tmp.path());
    cmd.env("XDG_DATA_HOME", data_home);
    cmd
}

#[test]
fn doctor_lists_fleet_membership_roots() {
    let tmp = TempDir::new().unwrap();
    let data_home = tmp.path().join("xdg-data");
    let org_dir = data_home.join("bridge").join("orgs").join("ship");
    fs::create_dir_all(&org_dir).unwrap();

    let root_a = tmp.path().join("meta");
    let root_b = tmp.path().join("helm");
    fs::create_dir_all(&root_a).unwrap();
    fs::create_dir_all(&root_b).unwrap();

    let members = serde_json::json!({
        "members": [
            root_a.display().to_string(),
            root_b.display().to_string(),
        ],
    });
    fs::write(
        org_dir.join("members.json"),
        serde_json::to_string_pretty(&members).unwrap() + "\n",
    )
    .unwrap();

    shipctl_isolated(&tmp, &data_home)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("fleet membership"))
        .stdout(predicate::str::contains(data_home.display().to_string()))
        .stdout(predicate::str::contains(
            data_home.join("bridge").join("orgs").display().to_string(),
        ))
        .stdout(predicate::str::contains("org"))
        .stdout(predicate::str::contains("ship"))
        .stdout(predicate::str::contains(root_a.display().to_string()))
        .stdout(predicate::str::contains(root_b.display().to_string()));
}

#[test]
fn doctor_membership_none_when_no_orgs() {
    let tmp = TempDir::new().unwrap();
    let data_home = tmp.path().join("xdg-data");
    fs::create_dir_all(&data_home).unwrap();

    shipctl_isolated(&tmp, &data_home)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("fleet membership"))
        .stdout(predicate::str::contains("no orgs registered"));
}

#[test]
fn doctor_membership_notes_malformed_members_json() {
    let tmp = TempDir::new().unwrap();
    let data_home = tmp.path().join("xdg-data");
    let org_dir = data_home.join("bridge").join("orgs").join("broken");
    fs::create_dir_all(&org_dir).unwrap();
    fs::write(org_dir.join("members.json"), "{not valid json").unwrap();

    shipctl_isolated(&tmp, &data_home)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("fleet membership"))
        .stdout(predicate::str::contains("broken"))
        .stdout(predicate::str::contains("MALFORMED"));
}
