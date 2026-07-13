/// Integration tests — dep_skills sub-domain.
///
/// Tests `is_dep_ref`, `parse_dep_ref`, `hash_from_lock`, `cache_skill_path`,
/// and `resolve_dep_skill` end-to-end using a real temporary cache layout.
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use stack_model::lockfile::{LockPackage, Rails, ShipLock};

use crate::dep_skills::{
    cache_skill_path, hash_from_lock, is_dep_ref, parse_dep_ref, resolve_dep_skill,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn lock_with_one(path: &str, hex: &str) -> ShipLock {
    ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: path.into(),
            version: "1.0.0".into(),
            commit: "a".repeat(40),
            hash: format!("sha256:{hex}"),
            rails: Rails::Stack,
            export_hashes: Default::default(),
        }],
    }
}

/// Write `content` to `root/rel_path`, creating parent dirs.
fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

// ── is_dep_ref ────────────────────────────────────────────────────────────────

#[test]
fn registry_dep_skills_is_dep_ref_recognises_github_ref() {
    assert!(is_dep_ref("github.com/owner/pkg/skills/my-skill"));
    assert!(is_dep_ref("git.example.com/owner/pkg/skills/ns"));
}

#[test]
fn registry_dep_skills_is_dep_ref_rejects_local_ref() {
    assert!(!is_dep_ref("my-local-skill"));
    assert!(!is_dep_ref("skills/my-skill"));
    assert!(!is_dep_ref("no-dots/owner/pkg/skill"));
}

#[test]
fn registry_dep_skills_is_dep_ref_rejects_too_few_slashes() {
    // host.tld/owner/pkg — only two slashes after host, no within-package path
    assert!(!is_dep_ref("github.com/owner/pkg"));
}

// ── parse_dep_ref ─────────────────────────────────────────────────────────────

#[test]
fn registry_dep_skills_parse_dep_ref_splits_correctly() {
    let ref_str = "github.com/owner/pkg/skills/my-skill";
    let (pkg, within) = parse_dep_ref(ref_str).expect("valid dep ref");
    assert_eq!(pkg, "github.com/owner/pkg");
    assert_eq!(within, "skills/my-skill");
}

#[test]
fn registry_dep_skills_parse_dep_ref_rejects_no_within_path() {
    // Three segments only — no within-package portion.
    let result = parse_dep_ref("github.com/owner/pkg");
    assert!(result.is_none(), "no within-path → None");
}

#[test]
fn registry_dep_skills_parse_dep_ref_rejects_no_hostname() {
    let result = parse_dep_ref("nodots/owner/pkg/skill");
    assert!(result.is_none(), "no dot in first segment → None");
}

// ── hash_from_lock ────────────────────────────────────────────────────────────

#[test]
fn registry_dep_skills_hash_from_lock_returns_hex_for_known_package() {
    let hex = "deadbeefdeadbeef".repeat(2);
    let lock = lock_with_one("github.com/owner/pkg", &hex);
    let result = hash_from_lock(&lock, "github.com/owner/pkg").expect("package in lock");
    assert_eq!(result, hex, "returned hex must match lockfile entry");
}

#[test]
fn registry_dep_skills_hash_from_lock_errors_for_absent_package() {
    let lock = ShipLock::default();
    let err = hash_from_lock(&lock, "github.com/nobody/absent").expect_err("absent package");
    let msg = err.to_string();
    assert!(
        msg.contains("ship install") || msg.contains("not in cache"),
        "error should guide user to ship install: {msg}"
    );
}

// ── cache_skill_path ──────────────────────────────────────────────────────────

#[test]
fn registry_dep_skills_cache_skill_path_builds_correct_path() {
    let cache_root = Path::new("/tmp/cache");
    let hex = "abc123";
    let within = "skills/my-skill";
    let path = cache_skill_path(cache_root, hex, within);
    assert_eq!(path, Path::new("/tmp/cache/objects/abc123/skills/my-skill"));
}

// ── resolve_dep_skill — full end-to-end ───────────────────────────────────────

#[test]
fn registry_dep_skills_resolve_dep_skill_returns_single_skill() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache_root = root.path().join("cache");
    let hex = "cafebabe";

    // Populate the cache object dir with a skill.
    write_file(
        &cache_root,
        &format!("objects/{hex}/skills/auth/SKILL.md"),
        "# Auth Skill\nHandle authentication.",
    );

    let lock = lock_with_one("github.com/owner/pkg", hex);

    let skills = resolve_dep_skill("github.com/owner/pkg/skills/auth", &lock, &cache_root)?;
    assert_eq!(skills.len(), 1, "single skill expected");
    assert!(
        skills[0].id.contains("auth"),
        "skill id should contain the ref path: {}",
        skills[0].id
    );
    Ok(())
}

#[test]
fn registry_dep_skills_resolve_dep_skill_expands_namespace_dir() -> anyhow::Result<()> {
    let root = tempdir()?;
    let cache_root = root.path().join("cache");
    let hex = "deadc0de";

    // Namespace dir: two leaf skills.
    write_file(
        &cache_root,
        &format!("objects/{hex}/skills/alpha/SKILL.md"),
        "# Alpha",
    );
    write_file(
        &cache_root,
        &format!("objects/{hex}/skills/beta/SKILL.md"),
        "# Beta",
    );

    let lock = lock_with_one("github.com/owner/pkg", hex);

    let skills = resolve_dep_skill("github.com/owner/pkg/skills", &lock, &cache_root)?;
    assert_eq!(skills.len(), 2, "namespace expansion should yield 2 skills");
    Ok(())
}

#[test]
fn registry_dep_skills_resolve_dep_skill_errors_when_package_not_in_lock() {
    let root = tempdir().unwrap();
    let cache_root = root.path().join("cache");
    let lock = ShipLock::default();

    let err = resolve_dep_skill("github.com/owner/pkg/skills/auth", &lock, &cache_root)
        .expect_err("absent package must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("ship install") || msg.contains("not in cache"),
        "error should guide to ship install: {msg}"
    );
}

#[test]
fn registry_dep_skills_resolve_dep_skill_errors_when_path_not_in_cache() {
    let root = tempdir().unwrap();
    let cache_root = root.path().join("cache");
    let hex = "11223344";
    let lock = lock_with_one("github.com/owner/pkg", hex);

    // Cache dir exists but the within-package path does not.
    fs::create_dir_all(cache_root.join("objects").join(hex)).unwrap();

    let err = resolve_dep_skill("github.com/owner/pkg/skills/missing", &lock, &cache_root)
        .expect_err("missing path must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("available"),
        "error should list available skills or say not found: {msg}"
    );
}
