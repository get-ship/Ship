/// Integration tests — hash sub-domain.
///
/// Exercises `compute_tree_hash`, `compute_file_hash`, and `compute_export_hashes`
/// at the cross-boundary level: real filesystem trees, exclusion behaviour, and
/// export-hash determinism across multiple sub-trees.
use std::fs;
use tempfile::tempdir;

use crate::hash::{compute_export_hashes, compute_file_hash, compute_tree_hash};

// ── compute_tree_hash — cross-boundary properties ─────────────────────────────

#[test]
fn registry_hash_tree_deterministic_across_two_separate_equal_trees() -> anyhow::Result<()> {
    let dir_a = tempdir()?;
    let dir_b = tempdir()?;

    for dir in [dir_a.path(), dir_b.path()] {
        fs::create_dir_all(dir.join("skills/alpha"))?;
        fs::write(dir.join("skills/alpha/SKILL.md"), "# Alpha\nBody.")?;
        fs::write(dir.join("skills/alpha/rules.md"), "rules here")?;
        fs::write(dir.join("manifest.json"), r#"{"name":"test"}"#)?;
    }

    let h_a = compute_tree_hash(dir_a.path())?;
    let h_b = compute_tree_hash(dir_b.path())?;

    assert!(h_a.starts_with("sha256:"), "hash has sha256: prefix");
    assert_eq!(h_a, h_b, "equal trees must produce equal hashes regardless of tmpdir path");
    Ok(())
}

#[test]
fn registry_hash_tree_changes_when_nested_file_modified() -> anyhow::Result<()> {
    let dir = tempdir()?;
    fs::create_dir_all(dir.path().join("skills/alpha"))?;
    fs::write(dir.path().join("skills/alpha/SKILL.md"), "v1")?;

    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("skills/alpha/SKILL.md"), "v2")?;
    let h_after = compute_tree_hash(dir.path())?;

    assert_ne!(h_before, h_after, "nested content change must change tree hash");
    Ok(())
}

#[test]
fn registry_hash_tree_excludes_all_noise_files() -> anyhow::Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("payload.txt"), "content")?;
    let h_base = compute_tree_hash(dir.path())?;

    // Add all exclusion targets
    fs::create_dir_all(dir.path().join(".git/objects"))?;
    fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main")?;
    fs::write(dir.path().join(".git/objects/blob"), "git object")?;
    fs::write(dir.path().join(".DS_Store"), "apple")?;
    fs::write(dir.path().join("Thumbs.db"), "windows")?;
    fs::write(dir.path().join(".swap.swp"), "vim")?;
    fs::write(dir.path().join("ship.lock"), r#"{"version":1,"packages":[]}"#)?;
    fs::create_dir_all(dir.path().join("state/skills"))?;
    fs::write(dir.path().join("state/skills/my.json"), "{}")?;

    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_base, h_after, "noise files must not affect tree hash");
    Ok(())
}

// ── compute_file_hash — format + error ────────────────────────────────────────

#[test]
fn registry_hash_file_known_value_for_hello() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("f.txt");
    fs::write(&path, "hello")?;
    let hash = compute_file_hash(&path)?;
    // SHA-256("hello") is a well-known constant.
    assert_eq!(
        hash,
        "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    Ok(())
}

#[test]
fn registry_hash_file_missing_returns_error() {
    let dir = tempdir().unwrap();
    let err = compute_file_hash(&dir.path().join("no-such-file.txt"));
    assert!(err.is_err(), "missing file must return Err");
}

// ── compute_export_hashes — cross-export determinism ─────────────────────────

#[test]
fn registry_hash_export_hashes_combined_changes_when_one_export_changes() -> anyhow::Result<()> {
    let ship_dir = tempdir()?;
    let skills = ship_dir.path().join("skills");

    fs::create_dir_all(skills.join("alpha"))?;
    fs::create_dir_all(skills.join("beta"))?;
    fs::write(skills.join("alpha/SKILL.md"), "alpha v1")?;
    fs::write(skills.join("beta/SKILL.md"), "beta stable")?;

    let exports = vec!["skills/alpha".to_string(), "skills/beta".to_string()];
    let r1 = compute_export_hashes(ship_dir.path(), &exports, &[])?;

    // Mutate only alpha.
    fs::write(skills.join("alpha/SKILL.md"), "alpha v2")?;
    let r2 = compute_export_hashes(ship_dir.path(), &exports, &[])?;

    assert_ne!(
        r1.per_export["skills/alpha"], r2.per_export["skills/alpha"],
        "alpha hash must change after mutation"
    );
    assert_eq!(
        r1.per_export["skills/beta"], r2.per_export["skills/beta"],
        "beta hash must NOT change when only alpha changed"
    );
    assert_ne!(r1.combined, r2.combined, "combined must change when any export changes");
    Ok(())
}

#[test]
fn registry_hash_export_hashes_missing_skill_dir_errors() {
    let dir = tempdir().unwrap();
    let err = compute_export_hashes(dir.path(), &["skills/nonexistent".into()], &[]);
    assert!(
        err.is_err(),
        "missing export directory must return Err"
    );
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("not found"),
        "error should mention 'not found': {msg}"
    );
}

#[test]
fn registry_hash_export_hashes_missing_agent_file_errors() {
    let dir = tempdir().unwrap();
    let err = compute_export_hashes(dir.path(), &[], &["agents/missing.md".into()]);
    assert!(
        err.is_err(),
        "missing agent export file must return Err"
    );
}
