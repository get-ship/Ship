use super::*;
use std::fs;
use tempfile::tempdir;

/// Build a content dir that satisfies the pack allow-list: a manifest at
/// the root plus the requested on-disk files. Tests that want to assert
/// hash determinism / corruption detection on the cached object shape
/// need the same shape a real pack would produce — the new
/// `compute_tree_hash` reads the allow-list, not a deny-list, so a
/// manifest-less fixture would hash to the empty-tree constant regardless
/// of what else is in the dir. Tests that want to assert hash shifts
fn make_content_dir(content: &[(&str, &str)]) -> anyhow::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    fs::write(dir.path().join("stack.config.ts"), "export default {};\n")?;
    for (name, data) in content {
        let full = dir.path().join(name);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, data)?;
    }
    Ok(dir)
}

#[test]
fn test_url_encode() {
    assert_eq!(
        url_encode("github.com/owner/repo"),
        "github.com%2Fowner%2Frepo"
    );
    assert_eq!(url_encode("simple"), "simple");
    assert_eq!(url_encode("feat/branch"), "feat%2Fbranch");
}

#[test]
fn test_store_and_get_round_trip() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("skills/a/SKILL.md", "# Hello")])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        "a".repeat(40).as_str(),
        content.path(),
    )?;

    assert!(stored.hash.starts_with("sha256:"));
    assert!(stored.dir.is_dir());

    let retrieved = cache
        .get("github.com/owner/pkg", "v1.0.0")
        .expect("cache hit");
    assert_eq!(retrieved.hash, stored.hash);
    assert_eq!(retrieved.commit, "a".repeat(40));
    Ok(())
}

#[test]
fn test_cache_miss_returns_none() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let result = cache.get("github.com/missing/pkg", "v1.0.0");
    assert!(result.is_none());
    Ok(())
}

#[test]
fn test_verify_passes_on_clean_cache() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("skills/a/SKILL.md", "data")])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        &"b".repeat(40),
        content.path(),
    )?;

    cache.verify(&stored)?;
    Ok(())
}

#[test]
fn test_verify_detects_corruption() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("skills/a/SKILL.md", "original")])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        &"c".repeat(40),
        content.path(),
    )?;

    // Corrupt a domain-dir file (the manifest stays intact so the
    // hash shift comes from the file we corrupted, not from moving
    // outside the allow-list).
    fs::write(stored.dir.join("skills/a/SKILL.md"), "tampered")?;

    let result = cache.verify(&stored);
    assert!(result.is_err());

    // Index entry should be deleted; next get() returns None.
    let hit = cache.get("github.com/owner/pkg", "v1.0.0");
    assert!(
        hit.is_none(),
        "corrupted entry should be evicted from index"
    );
    Ok(())
}

#[test]
fn test_store_idempotent() -> anyhow::Result<()> {
    let cache_root = tempdir()?;
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let content = make_content_dir(&[("skills/f/SKILL.md", "data")])?;
    let commit = "d".repeat(40);

    let s1 = cache.store("github.com/owner/pkg", "v1.0.0", &commit, content.path())?;
    let s2 = cache.store("github.com/owner/pkg", "v1.0.0", &commit, content.path())?;

    assert_eq!(s1.hash, s2.hash);
    Ok(())
}
