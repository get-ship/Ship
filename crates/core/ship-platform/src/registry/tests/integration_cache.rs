/// Integration tests — cache sub-domain.
///
/// Tests the full `PackageCache` pipeline: `with_root` → `store` → `get` →
/// `verify`, plus eviction-on-corruption and multi-version isolation.
use std::fs;
use tempfile::tempdir;

use crate::registry::cache::PackageCache;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Build a content directory with the given files.
fn make_content(files: &[(&str, &str)]) -> anyhow::Result<tempfile::TempDir> {
    let dir = tempdir()?;
    for (name, data) in files {
        fs::write(dir.path().join(name), data)?;
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

    let content = make_content(&[("SKILL.md", "# Skill"), ("guide.md", "guide")])?;
    let stored = cache.store("github.com/owner/pkg", "v1.0.0", &commit('a'), content.path())?;

    assert!(stored.hash.starts_with("sha256:"), "hash has sha256: prefix");
    assert!(stored.dir.is_dir(), "object dir exists");

    let hit = cache.get("github.com/owner/pkg", "v1.0.0").expect("cache hit");
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

    let content = make_content(&[("a.txt", "data")])?;
    let stored = cache.store("github.com/owner/pkg", "v1.0.0", &commit('b'), content.path())?;

    cache.verify(&stored)?;
    Ok(())
}

#[test]
fn registry_cache_verify_detects_corruption_and_evicts_index() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache = PackageCache::with_root(root.path().to_path_buf());

    let content = make_content(&[("a.txt", "original")])?;
    let stored = cache.store("github.com/owner/pkg", "v1.0.0", &commit('c'), content.path())?;

    // Corrupt the stored content.
    fs::write(stored.dir.join("a.txt"), "tampered")?;

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

    let c1 = make_content(&[("main.md", "v1 content")])?;
    let c2 = make_content(&[("main.md", "v2 content")])?;

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

    let content = make_content(&[("f.txt", "identical")])?;
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
    assert_eq!(cache.root(), root.path(), "root() must return the configured path");
    Ok(())
}
