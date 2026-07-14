use super::*;
use std::fs;
use tempfile::tempdir;

/// Helper: create a temp dir with the given files.
fn make_tree(files: &[(&str, &str)]) -> anyhow::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    for (path, content) in files {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, content)?;
    }
    Ok(dir)
}

// ── compute_tree_hash — manifest + domain dirs allow-list ──────────────

#[test]
fn tree_hash_is_deterministic_on_a_known_manifest_plus_skills_tree()
-> anyhow::Result<()> {
    let dir = make_tree(&[
        ("stack.config.ts", "export default {};\n"),
        ("skills/review/SKILL.md", "# Review\n"),
    ])?;
    let h1 = compute_tree_hash(dir.path())?;
    let h2 = compute_tree_hash(dir.path())?;
    assert!(h1.starts_with("sha256:"));
    assert_eq!(h1, h2, "same tree must produce same hash");
    Ok(())
}

#[test]
fn tree_hash_ignores_files_outside_the_allow_list() -> anyhow::Result<()> {
    // The pack allow-list is `stack.config.ts` + each domain dir. A
    // file at the root that is not the manifest (e.g. a stray README.md)
    // must not enter the hash — it is not stack content.
    let dir = make_tree(&[
        ("stack.config.ts", "export default {};\n"),
        ("skills/review/SKILL.md", "# Review\n"),
    ])?;
    let h_before = compute_tree_hash(dir.path())?;

    fs::write(dir.path().join("README.md"), "notes")?;
    fs::write(dir.path().join("justfile"), "default:\n")?;
    fs::create_dir_all(dir.path().join("contrib"))?;
    fs::write(dir.path().join("contrib/notes.md"), "scratch")?;
    fs::create_dir_all(dir.path().join("scripts"))?;
    fs::write(dir.path().join("scripts/build.sh"), "#!/bin/sh\n")?;

    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(
        h_before, h_after,
        "non-domain top-level entries must not affect the hash"
    );
    Ok(())
}

#[test]
fn tree_hash_ignores_build_output_under_a_repo_root() -> anyhow::Result<()> {
    // The old deny-list (`target`, `node_modules`, `dist`, etc.) is
    // gone. The allow-list guarantees the same outcome by construction:
    // a stack that lives at a repo root cannot pack build output
    // because the dirs are not domain dirs, so the hash is unchanged
    // when build output appears.
    let dir = make_tree(&[
        ("stack.config.ts", "export default {};\n"),
        ("agents/reviewer.ts", "agent\n"),
    ])?;
    let h_before = compute_tree_hash(dir.path())?;

    fs::create_dir_all(dir.path().join("target/debug"))?;
    fs::write(
        dir.path().join("target/debug/huge.rlib"),
        "BUILD ARTIFACT",
    )?;
    fs::create_dir_all(dir.path().join("node_modules/left-pad"))?;
    fs::write(
        dir.path().join("node_modules/left-pad/index.js"),
        "module.exports = 1;",
    )?;
    fs::create_dir_all(dir.path().join("dist"))?;
    fs::write(dir.path().join("dist/bundle.js"), "var x = 1;\n")?;

    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(
        h_before, h_after,
        "build output at the repo root must not affect the stack hash"
    );
    Ok(())
}

#[test]
fn tree_hash_excludes_ship_json_transport_header() -> anyhow::Result<()> {
    // The archive contains a synthesized ship.json; the hash must
    // exclude it so the header can carry the hash it represents.
    let dir = make_tree(&[("stack.config.ts", "export default {};\n")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("ship.json"), r#"{"hash":"sha256:pending"}"#)?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(
        h_before, h_after,
        "ship.json (transport header) must not be in the content hash"
    );
    Ok(())
}

#[test]
fn tree_hash_excludes_ship_lock() -> anyhow::Result<()> {
    // ship.lock is the dependency lockfile, kept beside the manifest.
    // It is not on the allow-list (it is not the manifest, not a domain
    // dir), so it does not affect the content hash.
    let dir = make_tree(&[("stack.config.ts", "export default {};\n")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("ship.lock"), "lock content")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_eq!(h_before, h_after, "ship.lock must be excluded");
    Ok(())
}

#[test]
fn tree_hash_includes_every_domain_dir() -> anyhow::Result<()> {
    // The set of domain dirs is a contract. Adding a file to any
    // domain dir must change the hash. Drop the manifest so a
    // missing manifest can't mask a domain-dir change.
    let base = make_tree(&[("stack.config.ts", "export default {};\n")])?;
    let h0 = compute_tree_hash(base.path())?;
    for (path, _) in [
        ("agents/reviewer.ts", "agent\n"),
        ("collections/notes.jsonc", "{}\n"),
        ("pipelines/feature.jsonc", "{}\n"),
        ("triggers/on-push.jsonc", "{}\n"),
        ("views/dashboard/.sui", "<div/>\n"),
        ("skills/review/SKILL.md", "# Review\n"),
        ("hooks/stop.sh", "#!/bin/sh\n"),
    ] {
        let dir = make_tree(&[("stack.config.ts", "export default {};\n")])?;
        let parent = dir.path().join(path).parent().unwrap().to_path_buf();
        fs::create_dir_all(parent).unwrap();
        fs::write(dir.path().join(path), path).unwrap();
        let h = compute_tree_hash(dir.path())?;
        assert_ne!(h0, h, "domain-dir entry {path} must change the hash");
    }
    Ok(())
}

#[test]
fn tree_hash_is_order_independent() -> anyhow::Result<()> {
    // Construction order on disk must not change the hash — the
    // sort by relative path inside the function is the source of
    // determinism.
    let dir1 = make_tree(&[
        ("stack.config.ts", "export default {};\n"),
        ("z/skills/last/SKILL.md", "z"),
        ("a/agents/first.ts", "a"),
    ])?;
    let dir2 = make_tree(&[
        ("stack.config.ts", "export default {};\n"),
        ("a/agents/first.ts", "a"),
        ("z/skills/last/SKILL.md", "z"),
    ])?;
    let h1 = compute_tree_hash(dir1.path())?;
    let h2 = compute_tree_hash(dir2.path())?;
    assert_eq!(h1, h2, "hash must not depend on file creation order");
    Ok(())
}

#[test]
fn tree_hash_changes_when_manifest_content_changes() -> anyhow::Result<()> {
    // The manifest is on the allow-list — editing it must change
    // the hash. (The on-disk content is what gets hashed; the
    // synthesized archive header is a separate file.)
    let dir = make_tree(&[("stack.config.ts", "v1\n")])?;
    let h_before = compute_tree_hash(dir.path())?;
    fs::write(dir.path().join("stack.config.ts"), "v2\n")?;
    let h_after = compute_tree_hash(dir.path())?;
    assert_ne!(
        h_before, h_after,
        "content change in manifest must change hash"
    );
    Ok(())
}

#[test]
fn tree_hash_empty_tree() -> anyhow::Result<()> {
    // A stack root with neither a manifest nor any domain dirs is
    // still hashable — the hash is well-defined and stable.
    let dir = tempdir()?;
    let hash = compute_tree_hash(dir.path())?;
    assert!(hash.starts_with("sha256:"));
    Ok(())
}

// ── compute_file_hash ─────────────────────────────────────────────────

#[test]
fn file_hash_known_content() -> anyhow::Result<()> {
    let dir = make_tree(&[("hello.txt", "hello")])?;
    let hash = compute_file_hash(&dir.path().join("hello.txt"))?;
    // SHA-256("hello") is a well-known constant.
    assert_eq!(
        hash,
        "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    Ok(())
}

#[test]
fn file_hash_missing_file_fails() {
    let dir = tempdir().unwrap();
    let result = compute_file_hash(&dir.path().join("nonexistent.txt"));
    assert!(result.is_err());
}
