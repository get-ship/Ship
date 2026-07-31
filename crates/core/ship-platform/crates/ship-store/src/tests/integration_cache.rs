/// Integration tests — cache sub-domain.
///
/// Tests the full `PackageCache` pipeline: `with_root` → `store` → `get` →
/// `verify`, plus eviction-on-corruption and multi-version isolation.
use std::fs;
use tempfile::tempdir;

use crate::cache::PackageCache;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Build a content directory with the given files. The pack allow-list
/// requires a manifest at the root; a fixture without one would hash to
/// the empty-tree constant regardless of its other content, defeating
/// every assertion in this file. The fixture's other files are placed
fn make_content(files: &[(&str, &str)]) -> anyhow::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    fs::write(dir.path().join("stack.config.ts"), "export default {};\n")?;
    for (name, data) in files {
        let full = dir.path().join(name);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, data)?;
    }
    Ok(dir)
}

fn commit(tag: char) -> String {
    tag.to_string().repeat(40)
}

// ── store → get round-trip ────────────────────────────────────────────────────

#[test]
fn registry_cache_store_and_get_round_trip() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    // Place the content under a domain dir — the new allow-list hash
    // only reads the manifest + domain-dir entries, so a fixture that
    // top-level-files its content would hash to the same value as an
    // empty fixture.
    let content = make_content(&[
        ("skills/foo/SKILL.md", "# Skill"),
        ("skills/foo/guide.md", "guide"),
    ])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        &commit('a'),
        content.path(),
    )?;

    assert!(
        stored.hash.starts_with("sha256:"),
        "hash has sha256: prefix"
    );
    assert!(stored.dir.is_dir(), "object dir exists");

    let hit = cache
        .get("github.com/owner/pkg", "v1.0.0")
        .expect("cache hit");
    assert_eq!(hit.hash, stored.hash, "get returns same hash as store");
    assert_eq!(hit.commit, commit('a'), "commit preserved");
    assert_eq!(hit.path, "github.com/owner/pkg");
    Ok(())
}

// ── cache miss ────────────────────────────────────────────────────────────────

#[test]
fn registry_cache_get_miss_returns_none() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    let result = cache.get("github.com/no/such", "v1.0.0");
    assert!(result.is_none(), "absent entry must return None");
    Ok(())
}

// ── verify — clean and corrupt ────────────────────────────────────────────────

#[test]
fn registry_cache_verify_passes_on_clean_entry() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    let content = make_content(&[("skills/a/SKILL.md", "data")])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        &commit('b'),
        content.path(),
    )?;

    cache.verify(&stored)?;
    Ok(())
}

#[test]
fn registry_cache_verify_detects_corruption_and_evicts_index() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    let content = make_content(&[("skills/a/SKILL.md", "original")])?;
    let stored = cache.store(
        "github.com/owner/pkg",
        "v1.0.0",
        &commit('c'),
        content.path(),
    )?;

    // Corrupt the stored content (a domain-dir file; the manifest stays
    // intact so the hash shift comes from the file we corrupted, not
    // from moving outside the allow-list).
    fs::write(stored.dir.join("skills/a/SKILL.md"), "tampered")?;

    let err = cache.verify(&stored);
    assert!(err.is_err(), "corrupted entry must fail verify");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("corruption") || msg.contains("mismatch"),
        "error message should indicate corruption: {msg}"
    );

    // Index entry must be evicted so subsequent get() returns None.
    let post = cache.get("github.com/owner/pkg", "v1.0.0");
    assert!(post.is_none(), "corrupted entry must be evicted from index");
    Ok(())
}

// ── multi-version isolation ────────────────────────────────────────────────────

#[test]
fn registry_cache_multiple_versions_independent() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    // Distinct content under a domain dir — only this gives the test
    // a non-trivial hash to compare.
    let c1 = make_content(&[("skills/foo/SKILL.md", "v1 content")])?;
    let c2 = make_content(&[("skills/foo/SKILL.md", "v2 content")])?;

    let s1 = cache.store("github.com/owner/pkg", "v1.0.0", &commit('d'), c1.path())?;
    let s2 = cache.store("github.com/owner/pkg", "v2.0.0", &commit('e'), c2.path())?;

    assert_ne!(s1.hash, s2.hash, "different content → different hashes");

    let g1 = cache.get("github.com/owner/pkg", "v1.0.0").expect("v1 hit");
    let g2 = cache.get("github.com/owner/pkg", "v2.0.0").expect("v2 hit");

    assert_eq!(g1.hash, s1.hash);
    assert_eq!(g2.hash, s2.hash);
    Ok(())
}

// ── store idempotency ─────────────────────────────────────────────────────────

#[test]
fn registry_cache_store_idempotent_returns_same_hash() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    let content = make_content(&[("skills/f/SKILL.md", "identical")])?;
    let sha = commit('f');

    let s1 = cache.store("github.com/owner/pkg", "v1.0.0", &sha, content.path())?;
    let s2 = cache.store("github.com/owner/pkg", "v1.0.0", &sha, content.path())?;

    assert_eq!(s1.hash, s2.hash, "idempotent store must return same hash");
    Ok(())
}

// ── root accessor ─────────────────────────────────────────────────────────────

#[test]
fn registry_cache_root_returns_configured_path() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());
    assert_eq!(
        cache.root(),
        root.path(),
        "root() must return the configured path"
    );
    Ok(())
}
