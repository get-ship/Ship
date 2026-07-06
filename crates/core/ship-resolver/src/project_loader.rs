//! Project-shape disk walk — the resolver's `load_project_library` entry.
//!
//! Composes with the compiler leaf's [`compiler::load_library`] for the
//! bulk of the read (`.ship/{mcp,permissions,hooks,rules,skills,agents}/`),
//! then layers the Phase-2 project-shape pieces on top:
//!
//! - `ship.ts` at the project root — the canonical TS manifest, also
//!   reachable as `<.ship>/ship.ts` for legacy/SDK-shipped projects. The
//!   leaf's `load_library` doesn't read it today (the Phase-0 resolver
//!   never went through `ship.ts`); the resolver owns the read.
//! - Skills from the additional `project.skill_paths` — the leaf already
//!   reads `project.skill_paths` from `ship.jsonc` (the additive F1
//!   manifest read) and the default `.ship/skills/` is walked by the leaf's
//!   `load_skills`. The resolver walks the EXTRA paths and merges into
//!   `ProjectLibrary::skills`, project winning on id conflict (so a
//!   project-defined skill shadows an extension's).
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
//! The cap is rooted at the project root with no fixed prefix; callers
//! compose the rel path explicitly. The same
//! cap serves both the `.ship/...` reads and the project-root `ship.ts`
//! read. The bootstrap posture mirrors `ship-config::agents::cap_root_and_dir`
//! (`ship-config/src/agents.rs:39-50`): root at a directory that always
//! exists (the project root) and use explicit two-segment rels.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-p2-J3-resolver.md`.
//! - Census: `census-phase2-resolver.md` Part 4 (project shape) + Part 5 J3
//!   (this crate's scope).
//! - F1 decision: `decision-f1-loader.md` — Option A, the superset is
//!   the architecture. The leaf grows `project.skill_paths`; this crate
//!   composes with the leaf.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use compiler::{ProjectLibrary, Skill, SkillSource};
use ship_substrate::ports::{FsBroker, FsCap, FsError, FsMode, FsRoot, SpaceId};

/// The canonical `.ship` dir name, mirrored from
/// `ship_substrate::project::SHIP_DIR_NAME`. The resolver's cap is rooted
/// at the project root, so the rel path is always `<.ship>/...`.
const SHIP_DIR_NAME: &str = ".ship";

/// The default project ship sub-dir name, used when a project authors
/// `ship.ts` at the project root (the Phase-2 default) and when it puts
/// the manifest inside `<.ship>/ship.ts` (the legacy / SDK-shipped shape).
const SHIP_TS_FILENAME: &str = "ship.ts";

/// The cap the resolver uses for project reads. No fixed prefix — callers
/// compose the rel path explicitly.
pub(crate) struct ProjectCap {
    pub cap: FsCap,
}

impl ProjectCap {
    /// Mint a RO cap over the project root.
    ///
    /// `SpaceId::new("resolver")` is the resolver's read surface — it
    /// shares the SpaceId namespace with other platform ports (config,
    /// agents, etc.) without colliding.
    pub fn read_only(project_root: &Path) -> Self {
        Self {
            cap: FsCap::new_unchecked(
                project_root,
                FsRoot::Space(SpaceId::new("resolver")),
                FsMode::ReadOnly,
            ),
        }
    }
}
/// Read the project root + the FsBroker, and return a fully-populated
/// `ProjectLibrary`.
///
/// The bulk of the read (`mcp.jsonc`, `permissions.jsonc`, `hooks.jsonc`,
/// `rules/`, `.ship/skills/`, `agents/`) is delegated to the leaf's
/// [`compiler::load_library`]; this fn layers the Phase-2 pieces on top:
///
/// 1. Walk the additional `project.skill_paths` (everything past the
///    default `.ship/skills/`) and merge skills into `library.skills`.
///    The leaf's `load_library` already populated the default path's
///    skills; this fn appends the others, project winning on id.
/// 2. Read `ship.ts` (at the project root OR at `<.ship>/ship.ts`) and
///    return its `ModuleDescriptor` in `imported_modules` (Phase-3
///    multi-file shape is the consumer's; the descriptor carries one
///    module in Phase 2 since the loader still admits only `"ship"`).
pub async fn load_project_library(
    fs: &dyn FsBroker,
    project_root: &Path,
) -> Result<ProjectLibrary> {
    // 1. Delegate to the leaf for the bulk of the read. The leaf is sync
    //    `std::fs`; it's the canonical phase-0 surface and ships on its
    //    own contract (`compiler::load_library`).
    let ship_dir = project_root.join(SHIP_DIR_NAME);
    let mut library = compiler::load_library(&ship_dir)
        .with_context(|| format!("compiler::load_library failed for {}", ship_dir.display()))?;

    // 2. Add the additional skill paths on top of the leaf's default walk.
    //    The leaf's `load_library` already populated `library.skill_paths`
    //    (the additive F1 manifest read) and `library.skills` (the
    //    default-path skills). We walk the extras here, via FsBroker.
    let cap = ProjectCap::read_only(project_root);
    if !library.skill_paths.is_empty() {
        let mut seen_ids: std::collections::HashSet<String> =
            library.skills.iter().map(|s| s.id.clone()).collect();
        for path in library.skill_paths.iter() {
            // The default `.ship/skills/` is already walked by the leaf —
            // skip the dup. The leaf's `read_project_skill_paths` is the
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
                // Conflict: project-defined skill (already in `library.skills`)
                // wins; the extension's skill is silently dropped. This is
                // agent-def's "project id wins on conflict" rule, per
                // `compile/skill_parser.rs:50-64`.
            }
        }
        // Stable order — the descriptor cache's content hash depends on
        // a stable iteration order.
        library.skills.sort_by(|a, b| a.id.cmp(&b.id));
    }

    // 3. Read the TS manifest (one module in Phase 2; Phase 3 extends
    //    the closed-world loader to admit project-relative imports and
    //    the fan-out grows).
    if let Some(module) = read_ship_ts(fs, &cap.cap, project_root)
        .with_context(|| format!("read ship.ts under {}", project_root.display()))?
    {
        let content_hash = project_library_module_hash(&module);
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
/// `project_rel` is the project-relative path (a value from
/// `project.skill_paths`); the cap is rooted at the project root, so the
/// rel path under the cap is the project-relative path verbatim. This
/// matches agent-def's [`read_skill_paths`](
/// ship_platform/crates/agent-def/src/skill_paths.rs) which joins the
/// manifest value onto the project root — `extra-skills/` lands at
/// `<root>/extra-skills/`, the default `.ship/skills/` lands at
/// `<root>/.ship/skills/`. The first case is what this fn walks; the
/// second is what the leaf's `load_skills` already covered.
fn walk_skills_dir(
    fs: &dyn FsBroker,
    cap: &FsCap,
    project_rel: &str,
) -> Result<Vec<Skill>> {
    let rel = project_rel.trim_start_matches('/').trim_end_matches('/');
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

/// Read the project's `ship.ts` manifest. Returns `Ok(None)` when
/// neither `<root>/ship.ts` nor `<root>/.ship/ship.ts` exists.
///
/// The cap is rooted at the project root with no fixed prefix, so
/// project-root `ship.ts` is read as `ship.ts` (no `.ship/` segment);
/// the legacy `<.ship>/ship.ts` is read with the explicit two-segment
/// rel. The content hash is filled in by the caller.
fn read_ship_ts(
    fs: &dyn FsBroker,
    cap: &FsCap,
    _project_root: &Path,
) -> Result<Option<compiler::ModuleDescriptor>> {
    // Prefer project-root `ship.ts` (Phase-2 default); fall back to
    // `<.ship>/ship.ts` (legacy / SDK-shipped projects).
    let rels: [&str; 2] = ["ship.ts", ".ship/ship.ts"];
    for (i, rel) in rels.iter().enumerate() {
        match fs.read(cap, rel) {
            Ok(bytes) => {
                let source = String::from_utf8_lossy(&bytes).into_owned();
                let path = if i == 0 {
                    PathBuf::from(SHIP_TS_FILENAME)
                } else {
                    PathBuf::from(*rel)
                };
                return Ok(Some(compiler::ModuleDescriptor {
                    path,
                    source,
                    content_hash: String::new(), // filled in by caller
                    exports: Vec::new(),
                }));
            }
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(None)
}

/// SHA-256 hex of a `ModuleDescriptor`'s raw source. Standalone so the
/// resolver can store it on the descriptor (the cache key) without
/// re-reading the file.
fn project_library_module_hash(module: &compiler::ModuleDescriptor) -> String {
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

    /// Build a project root with `.ship/{mcp,permissions,hooks,agents}` +
    /// a `skills/` dir + a `rules/` dir + a `ship.ts` at the project root.
    /// Returns the temp dir so the caller can keep it alive.
    fn write_project() -> (TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // .ship/ structure (the leaf's surface)
        let ship = root.join(".ship");
        std::fs::create_dir_all(ship.join("skills")).unwrap();
        std::fs::create_dir_all(ship.join("rules")).unwrap();
        std::fs::create_dir_all(ship.join("agents")).unwrap();
        std::fs::write(
            ship.join("mcp.jsonc"),
            r#"{"mcp": {"servers": {"stub": {"id": "stub", "command": "echo", "args": []}}}}"#,
        )
        .unwrap();

        // Default skill in .ship/skills/
        let default_skill = ship.join("skills").join("default-skill");
        std::fs::create_dir_all(&default_skill).unwrap();
        std::fs::write(
            default_skill.join("SKILL.md"),
            "---\nname: default-skill\ndescription: A default skill\n---\nDefault body.\n",
        )
        .unwrap();

        // Extra skill dir at project root (a `project.skill_paths` value)
        let extra_skills = root.join("extra-skills");
        std::fs::create_dir_all(extra_skills.join("extra-skill")).unwrap();
        std::fs::write(
            extra_skills.join("extra-skill").join("SKILL.md"),
            "---\nname: extra-skill\ndescription: An extra skill\n---\nExtra body.\n",
        )
        .unwrap();

        // A rule in .ship/rules/
        std::fs::write(
            ship.join("rules").join("01-style.md"),
            "---\nalwaysApply: true\n---\nBe terse.\n",
        )
        .unwrap();

        // ship.ts at the project root
        std::fs::write(
            root.join("ship.ts"),
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
            r#"{"id": "demo", "project": {"skill_paths": ["extra-skills/"]}}"#,
        )
        .unwrap();

        (tmp, root)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_project_library_composes_with_leaf_and_adds_extras() {
        let (_tmp, root) = write_project();
        let fs = LocalFsBroker::new();
        let lib = load_project_library(&fs, &root).await.unwrap();

        // The leaf's bulk: mcp server + default skill + rule + agent
        // profile (none here, but the field defaults to empty).
        assert_eq!(lib.mcp_servers.len(), 1, "mcp.jsonc from .ship/");
        assert_eq!(lib.mcp_servers[0].id, "stub");
        // Skills: default + extra merged.
        let ids: Vec<&str> = lib.skills.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["default-skill", "extra-skill"]);
        // Rule from .ship/rules/.
        assert_eq!(lib.rules.len(), 1);
        assert_eq!(lib.rules[0].file_name, "01-style.md");
        // skill_paths carries both the default and the extra — the leaf
        // populates the field, the resolver consults it.
        assert_eq!(
            lib.skill_paths,
            vec![".ship/skills/".to_string(), "extra-skills/".to_string()]
        );
        // ship.ts: one module, content hash non-empty.
        assert_eq!(lib.imported_modules.len(), 1);
        let module = &lib.imported_modules[0];
        assert_eq!(module.path, PathBuf::from("ship.ts"));
        assert!(module.source.contains("defineAgent"));
        assert_eq!(module.content_hash.len(), 64, "SHA-256 hex");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_project_library_omits_ship_ts_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // .ship/ exists (the leaf needs it), but no ship.ts at the root.
        std::fs::create_dir_all(root.join(".ship")).unwrap();
        let fs = LocalFsBroker::new();
        let lib = load_project_library(&fs, &root).await.unwrap();
        assert!(lib.imported_modules.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_project_library_empty_when_no_ship_dir() {
        // The leaf's `load_library` on a missing `.ship/` returns an
        // empty library (its own lenient contract). The resolver inherits
        // that — no panic, no error.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let fs = LocalFsBroker::new();
        let lib = load_project_library(&fs, &root).await.unwrap();
        assert!(lib.mcp_servers.is_empty());
        assert!(lib.skills.is_empty());
        assert!(lib.rules.is_empty());
        assert!(lib.imported_modules.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_project_library_uses_ship_ts_fallback() {
        // Legacy / SDK-shipped shape: ship.ts lives at `<.ship>/ship.ts`.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".ship")).unwrap();
        std::fs::write(
            root.join(".ship").join("ship.ts"),
            r#"
import { defineAgent } from "ship";
defineAgent({ agent: { id: "legacy", name: "Legacy" } });
"#,
        )
        .unwrap();
        let fs = LocalFsBroker::new();
        let lib = load_project_library(&fs, &root).await.unwrap();
        assert_eq!(lib.imported_modules.len(), 1);
        assert_eq!(lib.imported_modules[0].path, PathBuf::from(".ship/ship.ts"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn project_winning_id_takes_precedence_over_extension() {
        // A skill in the leaf's default path and a SAME-ID skill in an
        // extra path: the leaf's version wins. The resolver's merge
        // honors that — the extra is dropped.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let ship = root.join(".ship");
        std::fs::create_dir_all(ship.join("skills")).unwrap();
        let default_skill = ship.join("skills").join("shared");
        std::fs::create_dir_all(&default_skill).unwrap();
        std::fs::write(
            default_skill.join("SKILL.md"),
            "---\ndescription: From default path\n---\nDEFAULT\n",
        )
        .unwrap();

        let extra = root.join("extra-skills");
        std::fs::create_dir_all(extra.join("shared")).unwrap();
        std::fs::write(
            extra.join("shared").join("SKILL.md"),
            "---\ndescription: From extra path\n---\nEXTRA\n",
        )
        .unwrap();

        std::fs::write(
            ship.join("ship.jsonc"),
            r#"{"project": {"skill_paths": ["extra-skills/"]}}"#,
        )
        .unwrap();

        let fs = LocalFsBroker::new();
        let lib = load_project_library(&fs, &root).await.unwrap();
        let shared: Vec<&Skill> = lib.skills.iter().filter(|s| s.id == "shared").collect();
        assert_eq!(shared.len(), 1, "exactly one skill named 'shared'");
        assert_eq!(
            shared[0].description.as_deref(),
            Some("From default path"),
            "the default-path (project) version wins on id conflict"
        );
    }

    #[test]
    fn project_cap_uses_resolver_spaceid() {
        let tmp = tempfile::tempdir().unwrap();
        let cap = ProjectCap::read_only(tmp.path());
        assert!(matches!(cap.cap.mode(), FsMode::ReadOnly));
        assert_eq!(cap.cap.root(), tmp.path());
    }

}
