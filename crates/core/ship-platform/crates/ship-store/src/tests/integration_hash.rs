/// Integration tests — hash sub-domain.
///
/// Exercises `compute_tree_hash` and `compute_file_hash` at the
/// cross-boundary level: real filesystem trees, allow-list behaviour,
/// and determinism across separate equal sub-trees.
use std::fs;
use tempfile::tempdir;

use crate::hash::{compute_file_hash, compute_tree_hash};

// ── compute_tree_hash — cross-boundary properties ─────────────────────

#[test]
fn registry_hash_tree_deterministic_across_two_separate_equal_trees()
-> anyhow::Result<()> {
    let dir_a = tempdir()?;
    let dir_b = tempdir()?;

    for dir in [dir_a.path(), dir_b.path()] {
        fs::create_dir_all(dir.join("skills/alpha"))?;
        fs::write(dir.join("skills/alpha/SKILL.md"), "# Alpha\nBody.")?;
        fs::write(dir.join("skills/alpha/rules.md"), "rules here")?;
        fs::write(dir.join("stack.config.ts"), "export default {};\n")?;
    }

    let h_a = compute_tree_hash(dir_a.path())?;
    let h_b = compute_tree_hash(dir_b.path())?;

    assert!(h_a.starts_with("sha256:"), "hash has sha256: prefix");
    assert_eq!(
        h_a, h_b,
        "equal trees must produce equal hashes regardless of tmpdir path"
    );
    Ok(())
}

#[test]
fn registry_hash_tree_changes_when_nested_file_modified()
-> anyhow::Result<()> {
    let dir = tempdir()?;
    fs::create_dir_all(dir.path().join("skills/alpha"))?;
    fs::write(dir.path().join("skills/alpha/SKILL.md"), "v1")?;
    fs::write(dir.path().join("stack.config.ts"), "export default {};\n")?;

    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("skills/alpha/SKILL.md"), "v2")?;
    let h_after = compute_tree_hash(dir.path())?;

    assert_ne!(
        h_before, h_after,
        "nested content change must change tree hash"
    );
    Ok(())
}

#[test]
fn registry_hash_tree_ignores_non_domain_files_and_noisy_top_level_entries()
-> anyhow::Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("stack.config.ts"), "export default {};\n")?;
    fs::write(dir.path().join("payload.txt"), "content")?;
    let h_base = compute_tree_hash(dir.path())?;

    // The allow-list governs everything: anything at the root that
    // is not the manifest, and any dir that is not a domain dir, is
    // not stack content. Adding a "noisy" top-level entry must not
    // change the hash.
    fs::create_dir_all(dir.path().join(".git/objects"))?;
    fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main")?;
    fs::write(dir.path().join(".git/objects/blob"), "git object")?;
    fs::write(dir.path().join(".DS_Store"), "apple")?;
    fs::write(dir.path().join("Thumbs.db"), "windows")?;
    fs::write(dir.path().join(".swap.swp"), "vim")?;
    fs::write(
        dir.path().join("ship.lock"),
        r#"{"version":1,"packages":[]}"#,
    )?;
    fs::create_dir_all(dir.path().join("contrib"))?;
    fs::write(dir.path().join("contrib/notes.md"), "scratch")?;
    fs::create_dir_all(dir.path().join("scripts"))?;
    fs::write(dir.path().join("scripts/build.sh"), "#!/bin/sh\n")?;
    fs::create_dir_all(dir.path().join("ops"))?;
    fs::write(dir.path().join("ops/runbook.md"), "ops notes")?;

    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_base, h_after, "noise files must not affect tree hash");
    Ok(())
}

// ── compute_file_hash — format + error ─────────────────────────────────

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
