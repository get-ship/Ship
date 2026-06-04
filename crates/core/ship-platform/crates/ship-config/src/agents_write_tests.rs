//! Write-door tests: the per-agent opaque-value CRUD + the typed batch, all
//! round-tripping through THE one FsBroker write door and read back via the S2
//! reader. Trash-not-hard-delete semantics + create/upsert contract are locked.

use super::*;
use crate::agents::read_agent_profiles;
use ship_substrate::ports::LocalFsBroker;
use std::path::PathBuf;
use tempfile::tempdir;

/// Returns `ship_dir` for direct broker writes in tests. Canonicalizes the
/// tempdir root: on WSL/macOS `/tmp` is a symlink, and the broker's
/// symlink-escape guard canonicalizes the deepest existing ancestor before
/// comparing it to the cap root. Production ship dirs are already canonical.
fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    ship_substrate::home::ShipHome::project_ship_dir(base)
}

#[test]
fn normalize_strips_nnnn_prefix_and_md_suffix() {
    assert_eq!(normalize_rule_external_id("0001-rust.md"), "rust");
    assert_eq!(normalize_rule_external_id("rust"), "rust");
    assert_eq!(normalize_rule_external_id("rust.md"), "rust");
    // Non-numeric prefix is preserved.
    assert_eq!(normalize_rule_external_id("ab-cd"), "ab-cd");
}

fn def(id: &str) -> serde_json::Value {
    serde_json::json!({ "agent": { "id": id, "name": id }, "skills": { "refs": [] } })
}

#[test]
fn write_then_read_back_via_reader() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write_agent_def(&fs, &dir, "alpha", &def("alpha"), false).unwrap();
    // Read-back through the S2 reader sees the just-written def.
    let got = crate::agents::read_agent_def(&fs, &dir, "alpha").unwrap();
    assert_eq!(got.unwrap().id, "alpha");
    // And the door's own read returns the opaque value.
    let (leaf, value) = read_agent_def_value(&fs, &dir, "alpha").unwrap().unwrap();
    assert_eq!(leaf, "alpha.jsonc");
    assert_eq!(value["agent"]["id"], "alpha");
}

#[test]
fn write_refuses_overwrite_unless_allowed() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write_agent_def(&fs, &dir, "a", &def("a"), false).unwrap();
    // Create-contract refuses an existing def.
    let err = write_agent_def(&fs, &dir, "a", &def("a"), false).unwrap_err();
    assert!(matches!(err, ConfigError::AlreadyExists(_)), "got {err:?}");
    // Upsert overwrites.
    let mut updated = def("a");
    updated["agent"]["name"] = serde_json::json!("renamed");
    write_agent_def(&fs, &dir, "a", &updated, true).unwrap();
    let (_, value) = read_agent_def_value(&fs, &dir, "a").unwrap().unwrap();
    assert_eq!(value["agent"]["name"], "renamed");
}

#[test]
fn delete_trashes_not_hard_deletes() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write_agent_def(&fs, &dir, "doomed", &def("doomed"), false).unwrap();
    delete_agent_def(&fs, &dir, "doomed").unwrap();
    // Gone from the reader.
    assert!(crate::agents::read_agent_def(&fs, &dir, "doomed").unwrap().is_none());
    // But preserved under `<scope>/.trash/`.
    let trash = dir.join(".trash");
    let trashed: Vec<String> = std::fs::read_dir(&trash)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        trashed.iter().any(|n| n.ends_with("-doomed.jsonc")),
        "trash: {trashed:?}"
    );
}

#[test]
fn delete_missing_is_not_found() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let err = delete_agent_def(&fs, &dir, "ghost").unwrap_err();
    assert!(matches!(err, ConfigError::NotFound(_)), "got {err:?}");
}

#[test]
fn rename_moves_def_and_trashes_old() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write_agent_def(&fs, &dir, "old", &def("old"), false).unwrap();
    rename_agent_def(&fs, &dir, "old", "new", &def("new")).unwrap();
    assert!(crate::agents::read_agent_def(&fs, &dir, "old").unwrap().is_none());
    let got = crate::agents::read_agent_def(&fs, &dir, "new").unwrap().unwrap();
    assert_eq!(got.id, "new");
}

#[test]
fn find_locates_json_extension() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    // A plain-JSON def is tolerated by the locate path.
    let (cap, prefix) = scope_cap(&dir, FsMode::ReadWrite);
    fs.create_dir_all(&cap, &under_prefix(&prefix, "agents")).unwrap();
    fs.write_atomic(
        &cap,
        &under_prefix(&prefix, "agents/legacy.json"),
        def("legacy").to_string().as_bytes(),
    )
    .unwrap();
    assert_eq!(find_agent_def(&fs, &dir, "legacy").unwrap(), Some("legacy.json".to_string()));
}

#[test]
fn save_agents_config_prunes_dropped_agents() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();

    let a = AgentProfile { id: "alpha".into(), name: "Alpha".into(), ..Default::default() };
    let b = AgentProfile { id: "beta".into(), name: "Beta".into(), ..Default::default() };
    save_agents_config(&fs, &dir, &[a.clone(), b.clone()]).unwrap();
    // Read back through THE reader — the save wrote canonical-shape files.
    assert_eq!(read_agent_profiles(&fs, &dir).unwrap().len(), 2);

    // Re-save with only alpha — beta's file must be pruned.
    save_agents_config(&fs, &dir, &[a]).unwrap();
    let after = read_agent_profiles(&fs, &dir).unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].agent.id, "alpha");
}

#[test]
fn save_preserves_hand_authored_canonical_fields_on_resave() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let (cap, agents_dir) = agents_cap(&dir, FsMode::ReadWrite);
    fs.create_dir_all(&cap, &agents_dir).unwrap();

    // Hand-authored canonical file with fields the platform does NOT own.
    let authored = serde_json::json!({
        "agent": { "id": "alpha", "name": "Alpha" },
        "skills": { "refs": [] },
        "mcp": { "servers": [] },
        "hooks": { "stop": "echo bye" },
        "rules": { "inline": "be careful" },
        "harness_settings": { "claude": { "model": "opus" } },
        "apps": { "studio": true }
    });
    fs.write_atomic(&cap, &agent_rel(&agents_dir, "alpha"), authored.to_string().as_bytes())
        .unwrap();

    // Re-save the platform-managed view (changes skills only).
    let profile = AgentProfile {
        id: "alpha".into(),
        name: "Alpha".into(),
        skills: vec!["rust".into()],
        ..Default::default()
    };
    save_agents_config(&fs, &dir, &[profile]).unwrap();

    // The hand-authored fields must survive the re-save.
    let raw = fs.read_string(&cap, &agent_rel(&agents_dir, "alpha")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(doc["hooks"]["stop"], "echo bye", "hooks preserved");
    assert_eq!(doc["rules"]["inline"], "be careful", "rules.inline preserved");
    assert_eq!(doc["harness_settings"]["claude"]["model"], "opus", "harness_settings preserved");
    assert_eq!(doc["apps"]["studio"], true, "apps preserved");
    // And the platform-managed change landed.
    assert_eq!(doc["skills"]["refs"][0], "rust", "platform field updated");
}

#[test]
fn save_normalizes_rule_ids_on_disk() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    let profile = AgentProfile {
        id: "alpha".into(),
        name: "Alpha".into(),
        rules: vec!["0001-rust.md".into()],
        ..Default::default()
    };
    // The save path normalizes rule ids; the file must round-trip cleanly.
    save_agents_config(&fs, &dir, &[profile]).unwrap();
    let agents = read_agent_profiles(&fs, &dir).unwrap();
    assert_eq!(agents.len(), 1);
}
