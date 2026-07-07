//! Space-shape disk walk — the resolver's `load_space_library` entry.
//!
//! Composes with the compiler leaf's [`compiler::load_library`] for the
//! bulk of the read (`.ship/{permissions,hooks,skills,agents}/`; mcp
//! servers and rules no longer have a file form — see
//! `compiler::loader`), then layers the Phase-2 space-shape pieces on
//! top:
//!
//! - `ship.config.ts` at the space root — the canonical TS manifest and
//!   the ONLY valid manifest filename (no legacy `<.ship>/ship.ts`
//!   fallback, retired in format-migration Wave 4 phase A). The leaf's
//!   `load_library` doesn't read it — the resolver owns the read, and a
//!   separate eval step (`ship_resolver::merge_ts_declarations`) harvests
//!   its declare-forms (`defineMcpServer`, `defineRule`, etc.).
//! - Skills from the additional `space.skill_paths` — the leaf already
//!   reads `space.skill_paths` from `ship.jsonc` (the additive F1
//!   manifest read) and the default `.ship/skills/` is walked by the leaf's
//!   `load_skills`. The resolver walks the EXTRA paths and merges into
//!   `ExtensionLibrary::skills`, space winning on id conflict (so a
//!   space-defined skill shadows an extension's).
//!
//! ## FsBroker for new code (R1)
//!
//! Per the spec, "new code" reads via the gated door — `&dyn FsBroker` for
//! the TS file + the additional skill dirs. The leaf's `load_library`
//! continues to use its `std::fs` walk (the leaf-clean invariant
//! `cargo tree` proves; see `compiler/src/loader.rs:1-7`). The two read
//! surfaces are deliberately orthogonal: leaf = substrate-free, resolver
//! = substrate-broker-gated.
//!
//! ## The cap
//!
//! The cap is rooted at the space root with no fixed prefix; callers
//! compose the rel path explicitly. The same
//! cap serves both the `.ship/...` reads and the space-root `ship.config.ts`
//! read. The bootstrap posture mirrors `ship-config::agents::cap_root_and_dir`
//! (`ship-config/src/agents.rs:39-50`): root at a directory that always
//! exists (the space root) and use explicit two-segment rels.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-p2-J3-resolver.md`.
//! - Census: `census-phase2-resolver.md` Part 4 (space shape) + Part 5 J3
//!   (this crate's scope).
//! - F1 decision: `decision-f1-loader.md` — Option A, the superset is
//!   the architecture. The leaf grows `space.skill_paths`; this crate
//!   composes with the leaf.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use compiler::{Skill, SkillSource, ExtensionLibrary};
use ship_substrate::ports::{FsBroker, FsCap, FsError, FsMode, FsRoot, SpaceId};

/// The canonical `.ship` dir name, mirrored from
/// `ship_substrate::project::SHIP_DIR_NAME`. The resolver's cap is rooted
/// at the space root, so the rel path is always `<.ship>/...`.
const SHIP_DIR_NAME: &str = ".ship";

/// The extension-source manifest filename at the space root, per
/// `decision-space-scan.md` (one TS file per extension: manifest identity
/// via the W1 `ExtensionManifest` export + content entry via declare-forms).
const SHIP_CONFIG_FILENAME: &str = "ship.config.ts";

/// The cap the resolver uses for space reads. No fixed prefix — callers
/// compose the rel path explicitly.
pub(crate) struct SpaceCap {
    pub cap: FsCap,
}

impl SpaceCap {
    /// Mint a RO cap over the space root.
    ///
    /// `SpaceId::new("resolver")` is the resolver's read surface — it
    /// shares the SpaceId namespace with other platform ports (config,
    /// agents, etc.) without colliding.
    pub fn read_only(space_root: &Path) -> Self {
        Self {
            cap: FsCap::new_unchecked(
                space_root,
                FsRoot::Space(SpaceId::new("resolver")),
                FsMode::ReadOnly,
            ),
        }
    }
}
/// Read the space root + the FsBroker, and return a fully-populated
/// `ExtensionLibrary`.
///
/// The bulk of the read (`mcp.jsonc`, `permissions.jsonc`, `hooks.jsonc`,
/// `rules/`, `.ship/skills/`, `agents/`) is delegated to the leaf's
/// [`compiler::load_library`]; this fn layers the Phase-2 pieces on top:
///
/// 1. Walk the additional `space.skill_paths` (everything past the
///    default `.ship/skills/`) and merge skills into `library.skills`.
///    The leaf's `load_library` already populated the default path's
///    skills; this fn appends the others, space winning on id.
/// 2. Read `ship.config.ts` at the space root and return its
///    `ModuleDescriptor` in `imported_modules` (Phase-3 multi-file shape
///    is the consumer's; the descriptor carries one module in Phase 2
///    since the loader still admits only `"ship"`). No dual filename
///    support — the legacy `<.ship>/ship.ts` shape is retired along
///    with the format-migration Wave 4 JSONC/MD cutover.
pub async fn load_space_library(
    fs: &dyn FsBroker,
    space_root: &Path,
) -> Result<ExtensionLibrary> {

    // 1. Delegate to the leaf for the bulk of the read. The leaf is sync
    //    `std::fs`; it's the canonical phase-0 surface and ships on its
    //    own contract (`compiler::load_library`).
    let ship_dir = space_root.join(SHIP_DIR_NAME);
    let mut library = compiler::load_library(&ship_dir)
        .with_context(|| format!("compiler::load_library failed for {}", ship_dir.display()))?;

    // 2. Add the additional skill paths on top of the leaf's default walk.
    //    The leaf's `load_library` already populated `library.skill_paths`
    //    (the additive F1 manifest read) and `library.skills` (the
    //    default-path skills). We walk the extras here, via FsBroker.
    let cap = SpaceCap::read_only(space_root);
    if !library.skill_paths.is_empty() {
        let mut seen_ids: std::collections::HashSet<String> =
            library.skills.iter().map(|s| s.id.clone()).collect();
        for path in library.skill_paths.iter() {
            // The default `.ship/skills/` is already walked by the leaf —
            // skip the dup. The leaf's `read_space_skill_paths` is the
            // single source of truth for the default entry.
            if path == ".ship/skills/" {
                continue;
            }
            for skill in walk_skills_dir(fs, &cap.cap, path)
                .with_context(|| format!("walk skills dir: {path}"))?
            {
                if seen_ids.insert(skill.id.clone()) {
                    library.skills.push(skill);
                }
                // Conflict: space-defined skill (already in `library.skills`)
                // wins; the extension's skill is silently dropped. This is
                // agent-def's "space id wins on conflict" rule, per
                // `compile/skill_parser.rs:50-64`.
            }
        }
        // Stable order — the descriptor cache's content hash depends on
        // a stable iteration order.
        library.skills.sort_by(|a, b| a.id.cmp(&b.id));
    }

    // 3. Read the TS manifest (one module in Phase 2; Phase 3 extends
    //    the closed-world loader to admit space-relative imports and
    //    the fan-out grows). W4 phase A retires the legacy
    //    `<.ship>/ship.ts` shape — `ship.config.ts` at the space root
    //    is the only valid manifest.
    if let Some(module) = read_ship_config_ts(fs, &cap.cap, space_root)
        .with_context(|| format!("read ship.config.ts under {}", space_root.display()))?
    {
        let content_hash = extension_library_module_hash(&module);
        let mut module = module;
        module.content_hash = content_hash;
        library.imported_modules.push(module);
    }

    Ok(library)
}

/// Walk one skill dir and return its `Skill` records (id derived from the
/// dir name, source = `Custom`). Missing dir → empty. Malformed
/// `SKILL.md` → skip the entry (lenient contract).
///
/// `space_rel` is the space-relative path (a value from
/// `space.skill_paths`); the cap is rooted at the space root, so
/// the rel path under the cap is the space-relative path verbatim. This
/// matches agent-def's [`read_skill_paths`](
/// ship_platform/crates/agent-def/src/skill_paths.rs) which joins the
/// manifest value onto the space root — `extra-skills/` lands at
/// `<root>/extra-skills/`, the default `.ship/skills/` lands at
/// `<root>/.ship/skills/`. The first case is what this fn walks; the
/// second is what the leaf's `load_skills` already covered.
fn walk_skills_dir(
    fs: &dyn FsBroker,
    cap: &FsCap,
    space_rel: &str,
) -> Result<Vec<Skill>> {
    let rel = space_rel.trim_start_matches('/').trim_end_matches('/');

    let entries = match fs.list_dir(cap, &rel) {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut skills = Vec::new();
    for entry in entries {
        let path = entry.clone();
        // Each skill is a directory with a SKILL.md inside.
        let Some(stem) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let skill_dir_rel = format!("{rel}/{stem}");
        let skill_md_rel = format!("{skill_dir_rel}/SKILL.md");
        // We can't `is_dir` through the broker without a stat; treat
        // the missing-SKILL.md case as "not a skill entry" and skip.
        let raw = match fs.read(cap, &skill_md_rel) {
            Ok(bytes) => bytes,
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        };
        let raw_str = String::from_utf8_lossy(&raw);
        let mut skill = compiler::skill_parser::parse_skill(stem, &raw_str);
        skill.source = SkillSource::Custom;
        skills.push(skill);
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(skills)
}

/// Read the space's `ship.config.ts` manifest. Returns `Ok(None)` when
/// no manifest exists — every pre-W4 author (`.jsonc`-only or
/// `.ts`+`<.ship>/ship.ts` legacy) is now retired; the TS form is the
/// only read (census-format-migration.md §4 row 3).
///
/// The cap is rooted at the space root with no fixed prefix, so the
/// rel path is verbatim. The content hash is filled in by the caller.
fn read_ship_config_ts(
    fs: &dyn FsBroker,
    cap: &FsCap,
    _space_root: &Path,
) -> Result<Option<compiler::ModuleDescriptor>> {
    match fs.read(cap, SHIP_CONFIG_FILENAME) {
        Ok(bytes) => {
            let source = String::from_utf8_lossy(&bytes).into_owned();
            Ok(Some(compiler::ModuleDescriptor {
                path: PathBuf::from(SHIP_CONFIG_FILENAME),
                source,
                content_hash: String::new(), // filled in by caller
                exports: Vec::new(),
            }))
        }
        Err(FsError::NotFound) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// SHA-256 hex of a `ModuleDescriptor`'s raw source. Standalone so the
/// resolver can store it on the descriptor (the cache key) without
/// re-reading the file.
fn extension_library_module_hash(module: &compiler::ModuleDescriptor) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(module.source.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    use std::fmt::Write;
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::ports::LocalFsBroker;
    use tempfile::TempDir;

    /// Build a space root with `.ship/{agents,skills}` + an extra
    /// `space.skill_paths` dir + a `ship.config.ts` at the space root.
    /// Returns the temp dir so the caller can keep it alive.
    fn write_space() -> (TempDir, PathBuf) {

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // .ship/ structure (the leaf's surface)
        let ship = root.join(".ship");
        std::fs::create_dir_all(ship.join("skills")).unwrap();
        std::fs::create_dir_all(ship.join("agents")).unwrap();

        // Default skill in .ship/skills/
        let default_skill = ship.join("skills").join("default-skill");
        std::fs::create_dir_all(&default_skill).unwrap();
        std::fs::write(
            default_skill.join("SKILL.md"),
            "---\nname: default-skill\ndescription: A default skill\n---\nDefault body.\n",
        )
        .unwrap();

        // Extra skill dir at space root (a `space.skill_paths` value)
        let extra_skills = root.join("extra-skills");
        std::fs::create_dir_all(extra_skills.join("extra-skill")).unwrap();
        std::fs::write(
            extra_skills.join("extra-skill").join("SKILL.md"),
            "---\nname: extra-skill\ndescription: An extra skill\n---\nExtra body.\n",
        )
        .unwrap();

        // ship.config.ts at the space root. `load_space_library` only
        // stores the raw module source in `imported_modules` here — TS
        // eval + declare-form harvest (defineMcpServer/defineRule/etc.)
        // happens one layer up, in `resolve_space_with_broker`/`roll_up`
        // (`lib.rs::merge_ts_declarations`), so mcp/rule content has no
        // effect on THIS function's output and is omitted.
        std::fs::write(
            root.join("ship.config.ts"),
            r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
});
"#,
        )
        .unwrap();

        // Add the extra path to the manifest so the resolver walks it.
        std::fs::write(
            ship.join("ship.jsonc"),
            r#"{"id": "demo", "space": {"skill_paths": ["extra-skills/"]}}"#,
        )
        .unwrap();

        (tmp, root)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_space_library_composes_with_leaf_and_adds_extras() {
        let (_tmp, root) = write_space();
        let fs = LocalFsBroker::new();
        let lib = load_space_library(&fs, &root).await.unwrap();

        // Post format-migration Wave 5: the leaf no longer reads
        // `.ship/agents/*.jsonc` or `.ship/skills/<id>/SKILL.md` —
        // those nouns land in `agent_profiles` / `skills` through
        // `ship_resolver::merge_ts_declarations` (one layer up).
        // `load_space_library` returns them empty here; the test
        // exercises the disk-shape side: the default-path skills
        // walk + the extra-path walk + the `ship.config.ts` source
        // read + the `skill_paths` field population.
        assert!(lib.mcp_servers.is_empty());
        // Post Wave 5: the leaf no longer reads `.ship/skills/<id>/SKILL.md`
        // (the default path). The leaf still walks `space.skill_paths`
        // EXTRAS (the FsBroker-gated path) — so `library.skills` has the
        // extra-skill from `extra-skills/`, not the default-skill (the
        // fixture writes `extra-skills/extra-skill/SKILL.md` but no
        // `.ship/skills/default-skill/SKILL.md`). The post-merge roll-up
        // then layers the TS-declared `defineSkill` on top via
        // `ship_resolver::merge_ts_declarations` (one layer up).
        let ids: Vec<&str> = lib.skills.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["extra-skill"]);
        assert!(lib.rules.is_empty());
        // skill_paths carries both the default and the extra — the leaf
        // populates the field, the resolver consults it (extra-skills/
        // is still walked by `walk_skills_dir` for the additional
        // `space.skill_paths` entries; the post-merge roll-up pulls
        // those into `library.skills`).
        assert_eq!(
            lib.skill_paths,
            vec![".ship/skills/".to_string(), "extra-skills/".to_string()]
        );
        // ship.config.ts: one module, content hash non-empty.
        assert_eq!(lib.imported_modules.len(), 1);
        let module = &lib.imported_modules[0];
        assert_eq!(module.path, PathBuf::from("ship.config.ts"));
        assert!(module.source.contains("defineAgent"));
        assert_eq!(module.content_hash.len(), 64, "SHA-256 hex");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_space_library_omits_ship_ts_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // .ship/ exists (the leaf needs it), but no ship.ts at the root.
        std::fs::create_dir_all(root.join(".ship")).unwrap();
        let fs = LocalFsBroker::new();
        let lib = load_space_library(&fs, &root).await.unwrap();
        assert!(lib.imported_modules.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_space_library_empty_when_no_ship_dir() {
        // The leaf's `load_library` on a missing `.ship/` returns an
        // empty library (its own lenient contract). The resolver inherits
        // that — no panic, no error.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let fs = LocalFsBroker::new();
        let lib = load_space_library(&fs, &root).await.unwrap();
        assert!(lib.mcp_servers.is_empty());
        assert!(lib.skills.is_empty());
        assert!(lib.rules.is_empty());
        assert!(lib.imported_modules.is_empty());
    }

    // The "space winning id takes precedence over extension" merge
    // test retired with Wave 5: the leaf no longer reads
    // `.ship/skills/<id>/SKILL.md` (or `.ship/agents/<id>.jsonc`), so
    // there's no leaf-side merge to test. The same precedence is
    // exercised at the roll-up / compose level by
    // `tests::roll_up_source_shadows_cached_agent_on_id_collision`
    // and the agent/skill roll-up coverage tests — they pin the
    // precedence law from the composed-descriptor side.

    #[test]
    fn space_cap_uses_resolver_spaceid() {
        let tmp = tempfile::tempdir().unwrap();
        let cap = SpaceCap::read_only(tmp.path());
        assert!(matches!(cap.cap.mode(), FsMode::ReadOnly));
        assert_eq!(cap.cap.root(), tmp.path());
    }

}
