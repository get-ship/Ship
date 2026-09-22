//! Space-shape disk walk — the resolver's `load_space_library` entry.
//!
//! ## Wave 4 phase B: multi-extension space scan
//!
//! The space root is a DIRECTORY (operator ruling, `decision-space-scan.md`).
//! Enumeration scans the space root for `stack.config.ts` at candidate
//! roots: the space root itself (reserved identity `"root"`) and each
//! direct child directory that carries `stack.config.ts` (identity =
//! child dir name). No TS eval during enumeration, no git awareness, no
//! recursion past depth 1, no symlink following. Missing or empty space
//! dir → empty enumeration, not an error.
//!
//! Each discovered root is loaded as its own `ExtensionLibrary` (the
//! per-root loader is [`load_root_library`]). The space-root descriptor
//! is the SOLE contributor of `permissions` + `hooks` +
//! `permissions_presets` (the PIN — an extension must not grant itself;
//! the fence survives multi-extension). Every other noun (agents, mcp,
//! rules, skills, pipelines, triggers, collections, views, hooks) flows
//! through `compose_space` and resolves deterministically: root first,
//! then children lexicographic. The `permissions_presets` map is
//! included in the PIN's "space-root wins" rule — the root's preset
//! definitions survive composition; every child-root's preset map is
//! dropped (same as the existing permissions/hooks zeroing).
//!
//! **Slice 3 close-out (A3+A4)**: hooks arrive as on-disk files at
//! `<hooks_dir>/<event-stem>.<ext>` and are walked by
//! [`walk_hooks_dir`]; named permission presets arrive via
//! `defineConfig({ agents: { permissions: { presets: { ... } } } })`
//! and are harvested by `merge_ts_declarations` (resolver-level, in
//! `lib.rs`) into `ExtensionLibrary::permissions_presets`. The leaf's
//! `load_library` is a no-op for these nouns (both files died with
//! this slice). The on-disk hook walk + the per-module preset map are
//! the resolver's source of truth.
//!
//! ## FsBroker for new code (R1)
//!
//! Per the spec, "new code" reads via the gated door — `&dyn FsBroker` for
//! the TS file + the additional skill dirs. The leaf's `load_library`
//! continues to use its `std::fs` walk (the leaf-clean invariant
//! `cargo tree` proves; see `compiler/src/loader.rs:1-7`). The two read
//! surfaces are deliberately orthogonal: leaf = substrate-free, resolver
//! = substrate-broker-gated. The enumeration scan itself is a one-pass
//! `std::fs::read_dir` — it needs metadata (file vs directory vs
//! symlink) that the `FsBroker` trait does not expose; the scan does not
//! read content, only the immediate entries' metadata, and feeds the
//! results to the FsBroker-gated per-root loaders.
//!
//! ## The cap
//!
//! The cap is rooted at the space root with no fixed prefix; callers
//! compose the rel path explicitly. The same
//! cap serves both the `.ship/...` reads and the space-root `stack.config.ts`
//! read. The bootstrap posture mirrors `ship-settings::agents::cap_root_and_dir`
//! (`ship-settings/src/agents.rs:39-50`): root at a directory that always
//! exists (the space root) and use explicit two-segment rels. Per-root
//! caps mirror this for each discovered child root.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-fm-w4b-space-scan.md`.
//! - Decision: `decision-space-scan.md` (space = DIRECTORY, filename
//!   scan, no eval, no git) + `decision-morning-answers.md` §4 (the
//!   polyrepo is ONE space; repos own EXTENSIONS).

use std::path::{Path, PathBuf};

use agent_schema::skill::SkillSource;
use agent_schema::{HookConfig, HookTrigger, Skill};
use anyhow::{Context, Result};
use ship_substrate::ports::{FsBroker, FsCap, FsError, FsMode, FsRoot, SpaceId};
use stack_model::ExtensionLibrary;

/// The canonical `.ship` dir name, mirrored from
/// `ship_substrate::project::SHIP_DIR_NAME`. The resolver's cap is rooted
/// at the space root, so the rel path is always `<.ship>/...`.
const SHIP_DIR_NAME: &str = ".ship";

/// The extension-source manifest filename at the space root, per
/// `decision-space-scan.md` (one TS file per extension: manifest identity
/// via the W1 `ExtensionManifest` export + content entry via declare-forms).
const SHIP_CONFIG_FILENAME: &str = "stack.config.ts";

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
/// One discovered extension root inside a space.
///
/// Read ONE extension root and return its `ExtensionLibrary` (raw
/// disk shape; no TS merge).
///
/// The bulk of the read (the `skill_paths` field) is delegated to the
/// leaf's [`stack_model::load_library`] at `<root>/.ship/`. The
/// Phase-2 pieces layered on top:
///
/// 1. Read `stack.config.ts` at the root and append a
///    `ModuleDescriptor` to `imported_modules`. No dual filename
///    support — the legacy `<.ship>/ship.ts` shape is retired
///    (format-migration Wave 4 phase A).
///
/// **Slice 3 close-out (A3+A4)**: the leaf no longer reads
/// `.ship/hooks.jsonc` or `.ship/permissions.jsonc` (both files died
/// with the dot-ship close-out). `permissions` + `hooks` are
/// populated one layer up in `merge_ts_declarations`:
/// - `hooks` arrives via the on-disk `<hooks_dir>/<event-stem>.<ext>`
///   walk (the dir pointer comes from `defineConfig({ hooks_dir })`;
///   default `"hooks/"`).
/// - `permissions_presets` arrives via the
///   `defineConfig({ agents: { permissions: { presets: { ... } } } })`
///   harvest.
/// The leaf's `load_library` is now a no-op for these nouns
/// (`stack_model::load_library` returns default `permissions`, empty
/// `hooks`, empty `permissions_presets`); this fn returns that
/// default. The PIN is still applied one layer up in
/// [`load_space_library`] / the `roll_up` orchestrator (only the
/// space-root descriptor's permissions/hooks survive the
/// composition) — the zero-out for child roots remains correct
/// (a child that authors hooks/permissions through the resolver
/// has its values dropped, the same as a child that authored
/// `.ship/hooks.jsonc` / `.ship/permissions.jsonc` in the pre-
/// slice-3 world).
///
/// Skills (per Wave 5) arrive exclusively via `defineSkill`
/// declare-forms in the space's `stack.config.ts` and land in
/// `ExtensionLibrary::skills` through
/// `space_resolver::merge_ts_declarations` one layer up. The leaf no
/// longer reads `.ship/skills/<id>/SKILL.md` from disk; the
/// `space.skill_paths` ship.jsonc field is gone, so this fn does not
/// walk an extras dir either. `library.skill_paths` carries the
/// default single entry as metadata (the canonical constant
/// [`stack_model::loader::DEFAULT_SPACE_SKILL_PATH`]) but no consumer
/// in this crate iterates over it post-cutover.
/// Slice 2c close-out: the per-noun TS source dirs (agents/collections/
/// pipelines/triggers) live beside `stack.config.ts`. Each carries one
/// declare-form per noun. The manifest is a POINTER manifest
/// (`ship-stack-format.md` §"Manifest nature"): `stack.config.ts` points
/// at the source dirs; the dirs contain source; the manifest carries no
/// per-call content. The dir names default to `agents/`, `collections/`,
/// `pipelines/`, `triggers/` (the `defineConfig` pointer values; the
/// per-module override path is read in `merge_ts_declarations`).
pub(crate) async fn load_root_library(fs: &dyn FsBroker, root: &Path) -> Result<ExtensionLibrary> {
    // 1. Delegate to the leaf for the bulk of the read. The leaf is
    //    sync `std::fs`; it's the canonical phase-0 surface and ships
    //    on its own contract (`stack_model::load_library`).
    let ship_dir = root.join(SHIP_DIR_NAME);
    let mut library = stack_model::load_library(&ship_dir).with_context(|| {
        format!(
            "stack_model::load_library failed for {}",
            ship_dir.display()
        )
    })?;

    // 2. Read the TS manifest. W4 phase A retired the legacy
    //    `<.ship>/ship.ts` shape — `stack.config.ts` at the root is
    //    the only valid manifest. Phase 3 extends the closed-world
    //    loader to admit project-relative imports; until then, every
    //    root contributes one module.
    let cap = SpaceCap::read_only(root);
    if let Some(module) = read_ship_config_ts(fs, &cap.cap, root)
        .with_context(|| format!("read stack.config.ts under {}", root.display()))?
    {
        let content_hash = extension_library_module_hash(&module);
        let mut module = module;
        module.content_hash = content_hash;
        library.imported_modules.push(module);
    }

    // 3. Enumerate the per-noun source dirs and append each `.ts` file
    //    as a `ModuleDescriptor` into `imported_modules`. The fan-out
    //    (`ts_resolver::resolve_ts_files`) already iterates that vec
    //    and merges `Declarations` per-module — slice 2c just feeds
    //    existing machinery (per the job spec: "enumeration feeding
    //    existing machinery, not new eval code"). The four dirs are
    //    enumerated in declared order; within each dir entries sort by
    //    `file_name` so the harvest order is deterministic.
    for dir in PER_NOUN_SOURCE_DIRS {
        append_ts_dir_to_imported_modules(fs, &cap.cap, root, dir, &mut library.imported_modules)?;
    }

    Ok(library)
}

/// Discover a member root's own nested child extensions: immediate
/// child directories that carry their own `stack.config.ts` (the W4
/// phase B multi-extension shape — a stack root may embed further
/// extension dirs beside its per-noun source dirs). Returned in
/// lexicographic order by dir name for deterministic composition.
///
/// This is NOT the sibling-member scan `roll_up_members` deliberately
/// forgoes (membership across stacks is explicit, never scanned) — it
/// is the per-member nested-extension shape: one registered stack root
/// may itself embed further extension dirs directly beneath it.
pub(crate) fn discover_child_extension_roots(
    fs: &dyn FsBroker,
    root: &Path,
) -> Result<Vec<PathBuf>> {
    let cap = SpaceCap::read_only(root);
    let entries = match fs.list_dir(&cap.cap, "") {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut names: Vec<String> = entries
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .filter(|name| {
            fs.exists(&cap.cap, &format!("{name}/{SHIP_CONFIG_FILENAME}"))
                .unwrap_or(false)
        })
        .collect();
    names.sort();
    Ok(names.into_iter().map(|name| root.join(name)).collect())
}

/// Default per-noun source dirs. The values are the slice-2c spec
/// defaults: each lives beside the `stack.config.ts` at the space
/// root. Per-extension-project overrides are read from `defineConfig`
/// (slice 2c), but the space loader reads defaults — the override is a
/// future plumbing pass; the ship repo's own manifest uses defaults.
const PER_NOUN_SOURCE_DIRS: &[&str] = &["agents", "collections", "pipelines", "triggers"];

/// Walk one skill dir and return its `Skill` records (id derived from the
/// dir name, source = `Custom`). Missing dir → empty. Malformed
/// `SKILL.md` → skip the entry (lenient contract).
///
/// `space_rel` is the space-relative path (a value from
/// `defineConfig({ skills: { paths } })` in each extension's
/// `stack.config.ts` — the per-extension-project authoring surface, NOT
/// the dead `space.skill_paths` ship.jsonc read). The cap is rooted
/// at the space root, so the rel path under the cap is the
/// space-relative path verbatim. `extra-skills/` lands at
/// `<root>/extra-skills/`; the default `skills/` (a `defineConfig`
/// value matching [`stack_model::loader::DEFAULT_SPACE_SKILL_PATH`])
/// is filtered out upstream so this fn walks the extras only.
pub(crate) fn walk_skills_dir(
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
        let mut skill = stack_model::skill_parser::parse_skill(stem, &raw_str);
        skill.source = SkillSource::Custom;
        // No explicit `stable-id:` frontmatter → the on-disk dir name is
        // the canonical identity (deterministic, same convention as
        // agent-def's declare-form skills defaulting stable_id to id).
        if skill.stable_id.is_none() {
            skill.stable_id = Some(skill.id.clone());
        }
        skills.push(skill);
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(skills)
}
/// Map a `<event-stem>.<ext>` filename (e.g. `stop.sh`,
/// `pre-tool-use.bash`, `notification.py`) to the canonical
/// `HookTrigger` enum. Returns `None` for unknown stems — the
/// walk silently drops files whose stem doesn't match a known
/// event, the same lenient contract `load_hooks` had on the
/// old `.ship/hooks.jsonc` read.
///
/// The supported stems mirror the [`stack_model::HookTrigger`] enum
/// members exactly (slice 3 close-out, A3): `pre-tool-use` /
/// `post-tool-use` / `notification` / `stop` / `subagent-stop` /
/// `pre-compact`. Snake-case spellings (`pre_tool_use`, etc.)
/// are NOT accepted here — the on-disk convention is
/// kebab-case, the same way the on-disk agent TS files are
/// `agents/<id>.ts` and the on-disk skill dirs are
/// `skills/<id>/SKILL.md`. The space_root's `stack.config.ts`
/// is the canonical authoring surface; this walker is the
/// resolver-side projection.
fn hook_trigger_from_stem(stem: &str) -> Option<HookTrigger> {
    Some(match stem {
        "pre-tool-use" => HookTrigger::PreToolUse,
        "post-tool-use" => HookTrigger::PostToolUse,
        "notification" => HookTrigger::Notification,
        "stop" => HookTrigger::Stop,
        "subagent-stop" => HookTrigger::SubagentStop,
        "pre-compact" => HookTrigger::PreCompact,
        _ => return None,
    })
}

/// Walk one hooks dir and return its `HookConfig` records (one
/// per file, with `id = <event-stem>` and `command =
/// "bash <hooks_dir>/<event-stem>.<ext>"`). Missing dir →
/// empty. Unknown event stems → skip the entry silently (the
/// lenient contract the old `load_hooks` had).
///
/// `space_rel` is the space-relative hooks dir (a value from
/// `defineConfig({ hooks_dir })` in each extension's
/// `stack.config.ts`; the per-extension-project authoring surface,
/// default `"hooks/"`). The cap is rooted at the space root, so
/// the rel path under the cap is the space-relative path verbatim.
///
/// The walk enumerates the dir via `FsBroker::list_dir` (one
/// `list_dir` call → one batch of rel-paths) and reads each
/// file with `fs.read` (so the on-disk content is reachable
/// from the gated door; `R1` new code uses `FsBroker`, not
/// raw `std::fs`). The read succeeds for any file (the broker
/// does not gate on `is_executable` — execution is the
/// harness's call at hook-firing time). Files whose stem does
/// not match a known event name are silently dropped.
pub(crate) fn walk_hooks_dir(
    fs: &dyn FsBroker,
    cap: &FsCap,
    space_rel: &str,
) -> Result<Vec<HookConfig>> {
    let rel = space_rel.trim_start_matches('/').trim_end_matches('/');

    let entries = match fs.list_dir(cap, &rel) {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut hooks = Vec::new();
    for entry in entries {
        let path = entry.clone();
        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Filename must be `<event-stem>.<ext>`. Drop files
        // with no extension or empty stem (the broker returns
        // every entry — sub-dirs would be ignored here too, but
        // hook files are flat at the top level). `ext` is
        // captured to enforce the `<stem>.<ext>` shape but
        // isn't read after the split — the command below
        // re-uses the original `filename` so the on-disk
        // extension is preserved verbatim.
        let (stem, _ext) = match filename.split_once('.') {
            Some((s, e)) if !s.is_empty() && !e.is_empty() => (s, e),
            _ => continue,
        };
        let Some(trigger) = hook_trigger_from_stem(stem) else {
            continue;
        };
        // Touch the file via the broker so a missing file
        // (broker returns NotFound) is silently skipped, the
        // same way an unknown stem is.
        let file_rel = format!("{rel}/{filename}");
        match fs.read(cap, &file_rel) {
            Ok(_) => {}
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        }
        hooks.push(HookConfig {
            id: stem.to_string(),
            trigger,
            // The job spec (A3) is explicit: "command = the
            // file invocation, relative to extension root".
            // The extension root = the dir holding
            // `stack.config.ts`; the hooks dir is a dir name
            // beside it. So the command is the bash invocation
            // of the file at `<hooks_dir>/<stem>.<ext>` —
            // `bash hooks/stop.sh` for the canonical case.
            command: format!("bash {rel}/{filename}"),
            matcher: None,
            cursor_event: None,
        });
    }
    hooks.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hooks)
}

/// Read the space's `stack.config.ts` manifest. Returns `Ok(None)` when
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
) -> Result<Option<stack_model::ModuleDescriptor>> {
    match fs.read(cap, SHIP_CONFIG_FILENAME) {
        Ok(bytes) => {
            let source = String::from_utf8_lossy(&bytes).into_owned();
            Ok(Some(stack_model::ModuleDescriptor {
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
fn extension_library_module_hash(module: &stack_model::ModuleDescriptor) -> String {
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

/// Enumerate one per-noun source dir (`<root>/<dir>/*.ts`) and append
/// each `.ts` file as a `ModuleDescriptor` into `imported_modules`.
/// Missing dir is the common case for projects that don't author a
/// given noun in TS source form — return `Ok(())` and the dir is
/// simply absent (the fan-out sees one fewer module; harvest lands
/// nothing for that noun). A non-`.ts` entry (a stray `.md` or
/// sub-dir) is skipped via the read-probe pattern (mirrors
/// `walk_skills_dir`'s `SKILL.md` existence check).
fn append_ts_dir_to_imported_modules(
    fs: &dyn FsBroker,
    cap: &FsCap,
    _root: &Path,
    dir: &str,
    imported_modules: &mut Vec<stack_model::ModuleDescriptor>,
) -> Result<()> {
    let entries = match fs.list_dir(cap, dir) {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let mut stems: Vec<String> = entries
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .filter(|stem| stem.ends_with(".ts"))
        .collect();
    stems.sort();
    for stem in stems {
        let rel = format!("{dir}/{stem}");
        let bytes = match fs.read(cap, &rel) {
            Ok(b) => b,
            // Sub-dir or unreadable entry — skip (the read-probe
            // mirrors `walk_skills_dir` and stays out of the broker's
            // stat path; a true read error other than NotFound surfaces
            // here so a misconfigured perms/perm gate isn't swallowed).
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        };
        let source = String::from_utf8_lossy(&bytes).into_owned();
        let mut module = stack_model::ModuleDescriptor {
            path: PathBuf::from(&rel),
            source,
            content_hash: String::new(), // filled in below
            exports: Vec::new(),
        };
        module.content_hash = extension_library_module_hash(&module);
        imported_modules.push(module);
    }
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::ports::LocalFsBroker;
    use tempfile::TempDir;

    /// Build a space root with `.ship/{agents,skills}` and a
    /// `stack.config.ts` at the space root. Returns the temp dir so
    /// the caller can keep it alive.
    ///
    /// Post Wave 5 + dot-ship close-out: the leaf does not read
    /// skill files from disk, and the `space.skill_paths` ship.jsonc
    /// field is gone, so the fixture carries no skills dir on disk
    /// either.
    fn write_space() -> (TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // .ship/ structure (the leaf's surface)
        let ship = root.join(".ship");
        std::fs::create_dir_all(ship.join("skills")).unwrap();
        std::fs::create_dir_all(ship.join("agents")).unwrap();

        // stack.config.ts at the space root. `load_space_library` only
        // stores the raw module source in `imported_modules` here — TS
        // eval + declare-form harvest (defineMcpServer/etc.)
        // happens one layer up, in `resolve_space_with_broker`/`roll_up`
        // (`lib.rs::merge_ts_declarations`), so mcp/rule content has no
        // effect on THIS function's output and is omitted.
        std::fs::write(
            root.join("stack.config.ts"),
            r#"
import { defineConfig } from "ship";
export default defineConfig({});
"#,
        )
        .unwrap();
        // ARC A5: agent nouns live beside the manifest as JSONC.
        std::fs::create_dir_all(root.join("agents")).unwrap();
        std::fs::write(
            root.join("agents/demo.jsonc"),
            r#"{ "agent": { "id": "demo", "name": "Demo" } }"#,
        )
        .unwrap();

        (tmp, root)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_space_library_reads_leaf_and_ship_config_ts() {
        let (_tmp, root) = write_space();
        let fs = LocalFsBroker::new();
        let lib = load_root_library(&fs, &root).await.unwrap();

        // load_root_library only stores stack.config.ts + dir modules —
        // JSONC agents land in merge_jsonc_agent_nouns one layer up.
        assert!(lib.mcp_servers.is_empty());
        assert!(lib.skills.is_empty(), "no on-disk skills walk in leaf");
        assert!(lib.rules.is_empty());
        assert_eq!(lib.skill_paths, vec!["skills/".to_string()],);
        assert_eq!(lib.imported_modules.len(), 1);
        let module = &lib.imported_modules[0];
        assert_eq!(module.path, PathBuf::from("stack.config.ts"));
        assert!(module.source.contains("defineConfig"));
        assert_eq!(module.content_hash.len(), 64, "SHA-256 hex");
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

    // ── Wave 4 phase B: multi-extension discovery + per-root load ──────────

    /// The PIN: a child-dir extension's permissions + hooks are
    /// dropped during the merge, so the merged library carries only
    /// the space root's contributions. The fixture gives both
    /// roots a `.ship/hooks.jsonc` (the leaf parses hooks verbatim
    /// — permissions are a presets document the leaf leaves to
    /// caller-side preset resolution). The assertion: the merged
    /// library's hooks come from the ROOT, never the child, even
    /// when the child tries to grant more.
    #[tokio::test(flavor = "current_thread")]
    async fn load_space_library_drops_child_hooks() {
        let (_tmp, root) = write_space();
        // Slice 3 close-out (A3): hooks arrive as on-disk files at
        // `<hooks_dir>/<event>.<ext>` (the leaf no longer reads
        // `.ship/hooks.jsonc` — that file died with this slice).
        // The leaf's `load_library` is a no-op for hooks now; the
        // resolver's `merge_ts_declarations` walks the
        // per-extension-project `hooks/` dir and populates
        // `ExtensionLibrary::hooks` (see the
        // `roll_up_hooks_pin_keeps_root_only` test in
        // `crates/core/space-resolver/src/lib.rs` for the
        // roll-up-side exercise of the same intent).
        //
        // At the leaf level, `load_space_library` returns empty
        // `hooks` for any input (the leaf's disk walk is gone).
        // The PIN (zero-out on child descriptors) still applies
        // for any future path that reintroduces leaf-side hook
        // discovery; today both root and child contribute empty
        // `library.hooks` so the PIN's effect is indistinguishable
        // from "all empty".
        let child = root.join("child-with-grants");
        std::fs::create_dir_all(child.join("hooks")).unwrap();
        std::fs::write(
            child.join("hooks").join("pre-tool-use.sh"),
            "#!/bin/bash\necho child-cmd",
        )
        .unwrap();
        std::fs::write(
            child.join("stack.config.ts"),
            r#"import { defineConfig } from "ship";
export default defineConfig({});
"#,
        )
        .unwrap();
        std::fs::create_dir_all(child.join("agents")).unwrap();
        std::fs::write(
            child.join("agents/child-agent.jsonc"),
            r#"{ "agent": { "id": "child-agent", "name": "Child" } }"#,
        )
        .unwrap();
        std::fs::create_dir_all(root.join("hooks")).unwrap();
        std::fs::write(
            root.join("hooks").join("pre-tool-use.sh"),
            "#!/bin/bash\necho root-cmd",
        )
        .unwrap();

        let fs = LocalFsBroker::new();
        let lib = load_root_library(&fs, &root).await.unwrap();

        // Post slice-3: the leaf returns empty hooks (the
        // per-extension on-disk walk is the resolver's job, not
        // the leaf's). The PIN's effect is the same either way —
        // both root and child contribute empty `hooks`, and the
        // merged library carries empty `hooks`. The stronger
        // test (root wins, child dropped) lives in the roll-up
        // integration test, which exercises the on-disk walk +
        // the descriptor's PIN.
        assert!(
            lib.hooks.is_empty(),
            "post-slice-3 leaf returns empty hooks; the walk lives in merge_ts_declarations. lib.hooks={:?}",
            lib.hooks
        );
        // Permissions: leaf treats `permissions.jsonc` as a
        // presence-only check (it returns `Permissions::default()`
        // either way — the file is a presets document, not a
        // direct permissions blob). The PIN is therefore encoded
        // at the descriptor build site, not the leaf. Test the
        // stronger side via the `roll_up` integration test
        // (`roll_up_pin_drops_child_permissions_and_hooks`).
    }
}
