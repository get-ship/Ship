use std::path::Path;

use super::*;
use stack_model::lockfile::{LockPackage, Rails, ShipLock};
use stack_model::{Skill, SkillSource};
use tempfile::TempDir;

// `resolve_dep_agents` retired with format-migration Wave 5. The
// coverage test that pinned the equivalence with the new
// `ship_resolver::roll_up` flow lives in
// `ship-resolver/src/lib_tests.rs::roll_up_covers_every_case_resolve_dep_agents_covers`.

// ── load_cached_extensions (Phase-3 J2) ─────────────────────────────

/// Per-package bundle: each locked package gets its own `CachedExtension` with
/// the namespaced agents + skills. Two packages, distinct content hashes,
/// distinct bundles. The roll-up uses these to build per-package
/// `ExtensionDescriptor`s.
#[test]
fn load_cached_extensions_bundles_per_package() {
    let tmp = TempDir::new().unwrap();
    let cache_root = tmp.path().join("cache");

    // Package 1: one agent, no skills.
    let hex1 = "abc123def456";
    let obj1 = cache_root.join("objects").join(hex1);
    std::fs::create_dir_all(obj1.join("agents")).unwrap();
    std::fs::write(
        obj1.join("ship.jsonc"),
        r#"{ "id": "p1", "module": { "name": "github.com/x/y", "version": "1.0.0" },
             "exports": { "skills": [], "agents": ["agents/helper.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj1.join("agents/helper.jsonc"),
        r#"{ "agent": { "id": "helper", "name": "Helper" } }"#,
    )
    .unwrap();

    // Package 2: one skill (namespace dir = a single skill under `skills/foo`).
    let hex2 = "feedface0000";
    let obj2 = cache_root.join("objects").join(hex2);
    std::fs::create_dir_all(obj2.join("skills").join("foo")).unwrap();
    std::fs::write(
        obj2.join("ship.jsonc"),
        r#"{ "id": "p2", "module": { "name": "github.com/o/p", "version": "0.1.0" },
             "exports": { "skills": ["skills/foo"], "agents": [] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj2.join("skills").join("foo").join("SKILL.md"),
        "---\nname: Foo\n---\nFoo content.",
    )
    .unwrap();

    // Lockfile with both packages.
    let lock = ShipLock {
        version: 1,
        packages: vec![
            LockPackage {
                path: "github.com/x/y".to_string(),
                version: "1.0.0".to_string(),
                commit: "deadbeef".to_string(),
                hash: format!("sha256:{hex1}"),
                rails: Rails::Stack,
            },
            LockPackage {
                path: "github.com/o/p".to_string(),
                version: "0.1.0".to_string(),
                commit: "cafebabe".to_string(),
                hash: format!("sha256:{hex2}"),
                rails: Rails::Stack,
            },
        ],
    };
    let lock_path = tmp.path().join("ship.lock");
    lock.write_atomic(&lock_path).unwrap();

    let bundles = load_cached_extensions(&lock_path, Some(&cache_root)).unwrap();
    assert_eq!(bundles.len(), 2, "one bundle per locked package");

    // `write_atomic` sorts packages by path (lockfile determinism), so the
    // bundle order is alphabetical by `pkg_path`, not the original struct
    // declaration order.
    let p_op = &bundles[0];
    assert_eq!(p_op.pkg_path, "github.com/o/p");
    assert_eq!(p_op.content_hash, hex2);
    assert!(p_op.agent_profiles.is_empty());
    assert_eq!(p_op.skills.len(), 1, "one dep skill from exports.skills");
    assert_eq!(p_op.skills[0].id, "github.com/o/p/skills/foo");

    let p_xy = &bundles[1];
    assert_eq!(p_xy.pkg_path, "github.com/x/y");
    assert_eq!(p_xy.content_hash, hex1);
    assert_eq!(p_xy.agent_profiles.len(), 1);
    assert_eq!(p_xy.agent_profiles[0].agent.id, "github.com/x/y/helper");
    assert!(p_xy.skills.is_empty());
}

#[test]
fn load_cached_extensions_empty_without_lockfile() {
    let tmp = TempDir::new().unwrap();
    let bundles =
        load_cached_extensions(&tmp.path().join("absent.lock"), Some(tmp.path())).unwrap();
    assert!(bundles.is_empty(), "no lockfile => no bundles");
}

/// A package whose cache object dir is missing must be skipped silently —
/// partial install is a recoverable state, not a hard error. The remaining
/// packages still surface in the bundle vec.
#[test]
fn load_cached_extensions_skips_missing_manifest() {
    let tmp = TempDir::new().unwrap();
    let cache_root = tmp.path().join("cache");

    // Package 1: present in cache.
    let hex1 = "11111111";
    let obj1 = cache_root.join("objects").join(hex1);
    std::fs::create_dir_all(obj1.join("agents")).unwrap();
    std::fs::write(
        obj1.join("ship.jsonc"),
        r#"{ "id": "p1", "exports": { "skills": [], "agents": ["agents/a.jsonc"] } }"#,
    )
    .unwrap();
    std::fs::write(
        obj1.join("agents/a.jsonc"),
        r#"{ "agent": { "id": "a", "name": "A" } }"#,
    )
    .unwrap();

    // Package 2: NOT in cache (object dir absent).
    let lock = ShipLock {
        version: 1,
        packages: vec![
            LockPackage {
                path: "github.com/p/one".to_string(),
                version: "0.0.1".to_string(),
                commit: "a".repeat(40),
                hash: format!("sha256:{hex1}"),
                rails: Rails::Stack,
            },
            LockPackage {
                path: "github.com/p/two".to_string(),
                version: "0.0.1".to_string(),
                commit: "b".repeat(40),
                hash: "sha256:missing".to_string(),
                rails: Rails::Stack,
            },
        ],
    };
    let lock_path = tmp.path().join("ship.lock");
    lock.write_atomic(&lock_path).unwrap();

    let bundles = load_cached_extensions(&lock_path, Some(&cache_root)).unwrap();
    assert_eq!(bundles.len(), 1, "missing-cache package is skipped");
    assert_eq!(bundles[0].pkg_path, "github.com/p/one");
}

// ── transitive_capabilities (#119) ────────────────────────────────────

#[test]
fn transitive_capabilities_unions_requires_and_grants() {
    let tmp = TempDir::new().unwrap();
    let hex = "cafebabe";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/ship.jsonc"),
        r#"{ "id": "d", "module": { "name": "github.com/o/p", "version": "1.0.0" }, "exports": { "skills": ["skills/foo"], "agents": [] } }"#,
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/foo/SKILL.md"),
        "---\nname: Foo\nrequires:\n  - cap.x\ngrants:\n  - cap.y\n---\n\nContent.",
    );
    let lock = make_lock("github.com/o/p", &format!("sha256:{hex}"));
    let lock_path = tmp.path().join("ship.lock");
    lock.write_atomic(&lock_path).unwrap();

    let caps = transitive_capabilities(&lock_path, Some(tmp.path())).unwrap();
    assert_eq!(
        caps.required,
        vec!["cap.x".to_string()],
        "union of requires"
    );
    assert_eq!(caps.granted, vec!["cap.y".to_string()], "union of grants");
    assert_eq!(
        caps.unmet,
        vec!["cap.x".to_string()],
        "cap.x required but not granted within the dep tree"
    );
}

/// #106 regression: a dep skill declaring its capabilities inline
/// (`grants: [..]`, `requires: [..]`) must flow through
/// `transitive_capabilities` exactly like the block form did. The earlier
/// hand-rolled `parse_dep_skill` only recognized block lists and silently
/// DROPPED inline declarations — agents loaded from a dep package were
/// missing half their declared capability surface. The canonical parser
/// (`stack_model::skill_parser::parse_skill`) accepts both forms; this pins
/// the dep-skill path's contract so a future parser regression is caught
/// at the install surface, not at runtime when an agent discovers it
/// can't perform its declared op.
#[test]
fn transitive_capabilities_includes_inline_grants_and_requires() {
    let tmp = TempDir::new().unwrap();
    let hex = "feedface";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/ship.jsonc"),
        r#"{ "id": "d2", "module": { "name": "github.com/o/p2", "version": "1.0.0" }, "exports": { "skills": ["skills/bar"], "agents": [] } }"#,
    );
    // Inline-list form on BOTH grants and requires — the shape the old
    // hand-rolled parser dropped silently.
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/bar/SKILL.md"),
        "---\nname: Bar\ngrants: [cap.a, cap.b]\nrequires: [cap.c]\n---\n\nContent.",
    );
    let lock = make_lock("github.com/o/p2", &format!("sha256:{hex}"));
    let lock_path = tmp.path().join("ship.lock");
    lock.write_atomic(&lock_path).unwrap();

    let caps = transitive_capabilities(&lock_path, Some(tmp.path())).unwrap();
    assert_eq!(
        caps.granted,
        vec!["cap.a".to_string(), "cap.b".to_string()],
        "inline `grants: [..]` must surface — the #106 regression case"
    );
    assert_eq!(
        caps.required,
        vec!["cap.c".to_string()],
        "inline `requires: [..]` must surface"
    );
    assert_eq!(
        caps.unmet,
        vec!["cap.c".to_string()],
        "cap.c required but not granted within the dep tree"
    );
}

/// #106 — the dep-skill parser itself accepts inline-form frontmatter.
/// Belt-and-braces alongside the transitive test: if the parser regresses
/// but the transitive path still sees the cap (e.g. via a manual union),
/// this test catches it at the parser surface.
#[test]
fn parse_dep_skill_handles_inline_form() {
    let raw = "---\nname: bar\ngrants: [g1, g2]\nrequires: [r1]\n---\n\nBody.";
    let skill = parse_dep_skill("github.com/o/p/skills/bar", raw);
    assert_eq!(
        skill.source,
        SkillSource::Community,
        "dep skills are Community"
    );
    assert_eq!(skill.grants, vec!["g1".to_string(), "g2".to_string()]);
    assert_eq!(skill.requires, vec!["r1".to_string()]);
    assert_eq!(skill.name, "bar", "frontmatter name is preserved");
}

// ── is_dep_ref ────────────────────────────────────────────────────────

#[test]
fn is_dep_ref_github_prefix() {
    assert!(is_dep_ref("github.com/owner/pkg/skills/name"));
    assert!(is_dep_ref("github.com/a/b/skills/c"));
}

#[test]
fn is_dep_ref_any_git_host() {
    assert!(is_dep_ref("gitlab.com/owner/pkg/skills/name"));
    assert!(is_dep_ref("codeberg.org/owner/pkg/skills/name"));
    assert!(is_dep_ref("git.example.com/owner/pkg/skill-name"));
}

#[test]
fn is_dep_ref_local_refs_are_false() {
    assert!(!is_dep_ref("my-skill"));
    assert!(!is_dep_ref("review-pr"));
    assert!(!is_dep_ref(""));
    assert!(!is_dep_ref("skills/foo"));
    // No dot in first segment — not a host
    assert!(!is_dep_ref("owner/pkg/skill"));
}

// ── parse_dep_ref ─────────────────────────────────────────────────────

#[test]
fn parse_dep_ref_splits_correctly() {
    let (pkg, within) = parse_dep_ref("github.com/owner/pkg/skills/name").unwrap();
    assert_eq!(pkg, "github.com/owner/pkg");
    assert_eq!(within, "skills/name");
}

#[test]
fn parse_dep_ref_nested_within_path() {
    let (pkg, within) = parse_dep_ref("github.com/owner/repo/skills/foo/bar").unwrap();
    assert_eq!(pkg, "github.com/owner/repo");
    assert_eq!(within, "skills/foo/bar");
}

#[test]
fn parse_dep_ref_too_short_returns_none() {
    // Only package path, no within-path
    assert!(parse_dep_ref("github.com/owner/pkg").is_none());
    // No owner slash — only "github.com/skills/name" has 3 segments but
    // within_path is after the third segment, so with only two it's None.
    assert!(parse_dep_ref("github.com/only-two-segments").is_none());
}

#[test]
fn parse_dep_ref_non_github_returns_none() {
    assert!(parse_dep_ref("my-skill").is_none());
}

#[test]
fn parse_dep_ref_gitlab() {
    let (pkg, within) = parse_dep_ref("gitlab.com/owner/pkg/skills/name").unwrap();
    assert_eq!(pkg, "gitlab.com/owner/pkg");
    assert_eq!(within, "skills/name");
}

#[test]
fn parse_dep_ref_codeberg() {
    let (pkg, within) = parse_dep_ref("codeberg.org/owner/pkg/skills/name").unwrap();
    assert_eq!(pkg, "codeberg.org/owner/pkg");
    assert_eq!(within, "skills/name");
}

#[test]
fn parse_dep_ref_self_hosted() {
    let (pkg, within) = parse_dep_ref("git.example.com/owner/pkg/skill-name").unwrap();
    assert_eq!(pkg, "git.example.com/owner/pkg");
    assert_eq!(within, "skill-name");
}

#[test]
fn parse_dep_ref_no_dot_in_host_returns_none() {
    // First segment has no dot — not a valid host
    assert!(parse_dep_ref("owner/pkg/skill").is_none());
}

// ── hash_from_lock ────────────────────────────────────────────────────

fn make_lock(path: &str, hash: &str) -> ShipLock {
    ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: path.to_string(),
            version: "main".to_string(),
            commit: "a".repeat(40),
            hash: hash.to_string(),
            rails: Rails::Stack,
        }],
    }
}

#[test]
fn hash_from_lock_found() {
    let lock = make_lock("github.com/owner/pkg", "sha256:abc123");
    let hex = hash_from_lock(&lock, "github.com/owner/pkg").unwrap();
    assert_eq!(hex, "abc123");
}

#[test]
fn hash_from_lock_missing_dep_errors() {
    let lock = make_lock("github.com/owner/pkg", "sha256:abc");
    let err = hash_from_lock(&lock, "github.com/other/repo").unwrap_err();
    assert!(err.to_string().contains("not in cache"), "got: {err}");
    assert!(err.to_string().contains("ship install"), "got: {err}");
}

#[test]
fn hash_from_lock_malformed_hash_errors() {
    let lock = make_lock("github.com/owner/pkg", "noshaeprefix");
    let err = hash_from_lock(&lock, "github.com/owner/pkg").unwrap_err();
    assert!(err.to_string().contains("malformed hash"), "got: {err}");
}

// ── cache_skill_path ──────────────────────────────────────────────────

#[test]
fn cache_skill_path_builds_correct_path() {
    let root = Path::new("/home/user/.ship/cache");
    let path = cache_skill_path(root, "abcdef", "skills/my-skill");
    assert_eq!(
        path,
        Path::new("/home/user/.ship/cache/objects/abcdef/skills/my-skill")
    );
}

// ── resolve_dep_skill (integration with temp filesystem) ─────────────

fn write_file(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn resolve_dep_skill_reads_skill_md() {
    let tmp = TempDir::new().unwrap();
    let hex = "deadbeef";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/my-skill/SKILL.md"),
        "---\nname: My Skill\ndescription: Does stuff\n---\n\nInstructions.",
    );

    let lock = make_lock("github.com/owner/pkg", &format!("sha256:{hex}"));
    let skills =
        resolve_dep_skill("github.com/owner/pkg/skills/my-skill", &lock, tmp.path()).unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].id, "github.com/owner/pkg/skills/my-skill");
    assert_eq!(skills[0].name, "My Skill");
    assert_eq!(skills[0].description.as_deref(), Some("Does stuff"));
    assert_eq!(skills[0].content, "Instructions.");
}

#[test]
fn resolve_dep_skill_missing_skill_lists_available() {
    let tmp = TempDir::new().unwrap();
    let hex = "deadbeef";
    // Create a different skill so the package exists
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/other-skill/SKILL.md"),
        "content",
    );

    let lock = make_lock("github.com/owner/pkg", &format!("sha256:{hex}"));
    let err =
        resolve_dep_skill("github.com/owner/pkg/skills/missing", &lock, tmp.path()).unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("not found"), "got: {msg}");
    assert!(msg.contains("other-skill"), "got: {msg}");
}

#[test]
fn resolve_dep_skill_expands_namespace_to_leaf_skills() {
    // Real cache layout: better-auth/ is a namespace containing sub-skills,
    // NOT a skill itself (no better-auth/SKILL.md).
    let tmp = TempDir::new().unwrap();
    let hex = "83fb025b";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/best-practices/SKILL.md"),
        "---\nname: Best Practices\n---\n\nBP content.",
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/create-auth/SKILL.md"),
        "---\nname: Create Auth\n---\n\nCA content.",
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/emailAndPassword/SKILL.md"),
        "---\nname: Email and Password\n---\n\nEP content.",
    );
    // Also create a sibling namespace to verify we only expand the target
    write_file(
        tmp.path(),
        &format!("objects/{hex}/security/hardening/SKILL.md"),
        "---\nname: Hardening\n---\n\nH content.",
    );

    let lock = make_lock("github.com/better-auth/skills", &format!("sha256:{hex}"));
    let skills = resolve_dep_skill(
        "github.com/better-auth/skills/better-auth",
        &lock,
        tmp.path(),
    )
    .unwrap();

    assert_eq!(skills.len(), 3, "should expand to 3 leaf skills");
    // Sorted by sub-name
    assert_eq!(
        skills[0].id,
        "github.com/better-auth/skills/better-auth/best-practices"
    );
    assert_eq!(skills[0].name, "Best Practices");
    assert_eq!(
        skills[1].id,
        "github.com/better-auth/skills/better-auth/create-auth"
    );
    assert_eq!(
        skills[2].id,
        "github.com/better-auth/skills/better-auth/emailAndPassword"
    );
}

#[test]
fn resolve_dep_skill_missing_namespace_lists_available() {
    // When within_path doesn't match any directory at all, error lists siblings.
    let tmp = TempDir::new().unwrap();
    let hex = "deadbeef";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/better-auth/create-auth/SKILL.md"),
        "content",
    );
    write_file(
        tmp.path(),
        &format!("objects/{hex}/security/hardening/SKILL.md"),
        "content",
    );

    let lock = make_lock("github.com/better-auth/skills", &format!("sha256:{hex}"));
    let err = resolve_dep_skill(
        "github.com/better-auth/skills/missing-ns",
        &lock,
        tmp.path(),
    )
    .unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("not found"), "got: {msg}");
    assert!(
        msg.contains("better-auth"),
        "should list sibling dirs, got: {msg}"
    );
    assert!(
        msg.contains("security"),
        "should list sibling dirs, got: {msg}"
    );
}

#[test]
fn resolve_dep_skill_cache_miss_errors_with_install_hint() {
    let tmp = TempDir::new().unwrap();
    let lock = ShipLock {
        version: 1,
        packages: vec![],
    };

    let err = resolve_dep_skill("github.com/owner/pkg/skills/name", &lock, tmp.path()).unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("not in cache"), "got: {msg}");
    assert!(msg.contains("ship install"), "got: {msg}");
}

// ── resolve_dep_skills (batch) ────────────────────────────────────────

#[test]
fn resolve_dep_skills_skips_local_refs() {
    let tmp = TempDir::new().unwrap();
    // Lock file must exist but won't be read since there are no dep refs
    let lock_path = tmp.path().join("ship.lock");
    std::fs::write(&lock_path, r#"{ "version": 1 }"#).unwrap();

    let refs = vec!["local-skill".to_string(), "another-local".to_string()];
    let result = resolve_dep_skills(&refs, &[], &lock_path, Some(tmp.path())).unwrap();
    assert!(result.is_empty(), "local refs must produce no dep skills");
}

#[test]
fn resolve_dep_skills_deduplicates_against_local() {
    let tmp = TempDir::new().unwrap();
    let hex = "cafebabe";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/my-skill/SKILL.md"),
        "content",
    );

    let lock_path = tmp.path().join("ship.lock");
    std::fs::write(
        &lock_path,
        format!(
            r#"{{
              "version": 1,
              "packages": [
                {{
                  "path": "github.com/owner/pkg",
                  "version": "main",
                  "commit": "{}",
                  "hash": "sha256:{hex}"
                }}
              ]
            }}"#,
            "a".repeat(40)
        ),
    )
    .unwrap();

    // Simulate a local skill that already has the dep ref as its id
    let existing = Skill {
        id: "github.com/owner/pkg/skills/my-skill".to_string(),
        name: "Local".to_string(),
        stable_id: None,
        description: None,
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: "local content".to_string(),
        source: SkillSource::Custom,
        vars: Default::default(),
        artifacts: vec![],
        grants: vec![],
        requires: vec![],
    };

    let refs = vec!["github.com/owner/pkg/skills/my-skill".to_string()];
    let result = resolve_dep_skills(&refs, &[existing], &lock_path, Some(tmp.path())).unwrap();
    assert!(
        result.is_empty(),
        "already-present skill must not be duplicated"
    );
}

#[test]
fn resolve_dep_skills_merges_dep_with_local() {
    let tmp = TempDir::new().unwrap();
    let hex = "cafebabe";
    write_file(
        tmp.path(),
        &format!("objects/{hex}/skills/cool-skill/SKILL.md"),
        "---\nname: Cool\n---\n\nCool content.",
    );

    let lock_path = tmp.path().join("ship.lock");
    std::fs::write(
        &lock_path,
        format!(
            r#"{{
              "version": 1,
              "packages": [
                {{
                  "path": "github.com/owner/pkg",
                  "version": "main",
                  "commit": "{}",
                  "hash": "sha256:{hex}"
                }}
              ]
            }}"#,
            "a".repeat(40)
        ),
    )
    .unwrap();

    let refs = vec![
        "local-skill".to_string(),
        "github.com/owner/pkg/skills/cool-skill".to_string(),
    ];
    let result = resolve_dep_skills(&refs, &[], &lock_path, Some(tmp.path())).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "github.com/owner/pkg/skills/cool-skill");
    assert_eq!(result[0].content, "Cool content.");
}

// ── parse_dep_skill spec fields ───────────────────────────────────────

#[test]
fn parse_dep_skill_parses_license_and_compatibility() {
    let raw = "---\nname: Cool\ndescription: x\nlicense: MIT\ncompatibility: claude >= 3\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(skill.license.as_deref(), Some("MIT"));
    assert_eq!(skill.compatibility.as_deref(), Some("claude >= 3"));
}

#[test]
fn parse_dep_skill_parses_allowed_tools() {
    let raw = "---\nname: Cool\ndescription: x\nallowed-tools: Read Edit\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(skill.allowed_tools, vec!["Read", "Edit"]);
}

#[test]
fn parse_dep_skill_parses_metadata_block() {
    let raw = "---\nname: Cool\ndescription: x\nmetadata:\n  team: platform\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(
        skill.metadata.get("team").map(String::as_str),
        Some("platform")
    );
}

#[test]
fn parse_dep_skill_folds_legacy_version_author_into_metadata() {
    let raw = "---\nname: Cool\ndescription: x\nversion: 2.0\nauthor: bob\n---\n\nContent.";
    let skill = parse_dep_skill("github.com/owner/pkg/skills/cool", raw);
    assert_eq!(
        skill.metadata.get("version").map(String::as_str),
        Some("2.0")
    );
    assert_eq!(
        skill.metadata.get("author").map(String::as_str),
        Some("bob")
    );
}
