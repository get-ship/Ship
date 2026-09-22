//! Parse SKILL.md frontmatter per the agentskills.io spec.
//!
//! The parser is the CANONICAL one in `stack_model::skill_parser` (single
//! source for both the compile path here and the dep-skill cache in
//! `ship-store` — see #106 for the inline-form regression the duplicate
//! parsers caused). Re-exported here so existing
//! `agent_def::compile::skill_parser::parse_skill` paths keep working
//! without churn. The platform's own
//! `agents::skill::storage::parse_skill_spec` is a separate, stricter
//! parser used by the skill management service (install/list).
//!
//! Post Wave 5 + dot-ship close-out: there is no on-disk skill walk in
//! this crate. Skills arrive via `defineSkill` declare-forms in
//! `stack.config.ts` and land in `ExtensionLibrary::skills` through
//! `space_resolver::merge_ts_declarations` (one layer up). The
//! `space.skill_paths` ship.jsonc field died with the cutover; the
//! option moved to `defineConfig({ skills: { paths } })` in each
//! extension's `stack.config.ts` (per-extension-project manifest; the
//! default-export form is purely for ecosystem conformance — the
//! emit is what the harvest reads), resolved per-extension against
//! the dir containing that `stack.config.ts`. The resolver rolls
//! the per-extension values into `ExtensionLibrary::skill_paths`
//! post-eval and walks them for on-disk skills. The only on-disk
//! skill path used by this crate is the unified default
//! `<extension-dir>/skills/` (for skill-VAR resolution in
//! `agent_def::skill_vars` and the skills view in
//! `ship_agents::view_state`) — which sits at
//! `DEFAULT_SPACE_SKILL_PATH` and does not see custom-path
//! overrides (the SkillService call path is out of scope for this
//! slice — see the spec's SPEC CHANGE rule 4 for the deferral note).

use agent_schema::Skill;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// SMELL: the `include_dir!` macro below pointed at the in-tree
/// `skills/sdk/` path that was deleted in this cleanup. The right
/// answer is to depend on a `ship-sdk-skills` crate ship-contracts
/// publishes and use its `include_dir!` macro. Until that ship-side
/// dependency is wired, the include is disabled and the consumers
/// below return empty. Re-enable when ship-contracts publishes.
// static SDK_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../../../../skills/sdk");

pub fn embedded_capability_skills() -> Vec<Skill> {
    // Disabled: the include_dir! above is commented out. Re-enable when
    // ship-contracts publishes a ship-sdk-skills crate. Until then,
    // no embedded capability skills; the compile pool is disk-only.
    Vec::new()
}

/// The non-`SKILL.md` files inside an embedded sdk-skill directory, as
/// `(path-relative-to-skill-dir, content)` — e.g.
/// `("references/docs/git.create.mdoc", "...")`. Empty for ids that aren't
/// embedded sdk-skills. The compiler writes `SKILL.md` itself (it's templated);
/// these are the static pages its TOC links to, delivered beside it so the
/// links resolve.
pub fn embedded_skill_reference_files(_id: &str) -> Vec<(String, String)> {
    // Disabled: see embedded_capability_skills above. Re-enable when
    // ship-contracts publishes ship-sdk-skills.
    Vec::new()
}

#[allow(dead_code)]
fn collect_embedded_files_unused() {}

// `load_compile_skills` retired with Wave 5. The compile-time skill
// pool used to be disk + embedded; post format-migration Wave 5 the
// project-side skills arrive via `defineSkill` declare-forms in the
// space's `stack.config.ts` and land in
// `ExtensionLibrary::skills` through
// `space_resolver::merge_ts_declarations`. The embedded SDK
// capability-skills path stays (the future ship-sdk-skills dep wires
// it back in — see the `embedded_capability_skills` doc comment above
// for the deferred `include_dir!` / `ship-sdk-skills` migration).

// ── Skill parser ─────────────────────────────────────────────────────────────
//
// Re-export the CANONICAL `stack_model::skill_parser::parse_skill` so callers
// in this crate and its tests keep using `agent_def::compile::skill_parser::parse_skill`
// without churn. The agent-def specific helpers below stay local — they own
// the embedded SDK skills + the disk loader for compile.
pub use stack_model::skill_parser::parse_skill;

// `load_skills` (the agent-def disk-authoring skill walker) retired
// with Wave 5. The skill bodies now arrive via `defineSkill`
// declare-forms in the space's `stack.config.ts` and land in
// `ExtensionLibrary::skills` through
// `space_resolver::merge_ts_declarations`. The `SkillService` CRUD
// surface in `ship-agents` still writes `SKILL.md` files (the
// authoring path the Studio/db-scope gate owns; Phase-6's question) —
// the read side that compiled those files into the agent-def pipeline
// is gone. The `SkillService::list_*_skills` callers (read-side
// listing UI) read their own `ship_settings::skill` storage, not this
// fn.

// ── Vars loading ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn load_skill_vars(
    ship_dir: &Path,
    state_key: &str,
    vars_path: &Path,
) -> Result<HashMap<String, serde_json::Value>> {
    let state = crate::skill_vars::get_skill_vars(ship_dir, state_key)
        .unwrap_or_default()
        .unwrap_or_default();

    // Emit enum validation warnings for out-of-range values.
    if let Ok(content) = std::fs::read_to_string(vars_path) {
        if let Ok(defs) =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content)
        {
            for (name, def) in &defs {
                if name == "$schema" {
                    continue;
                }
                if def.get("type").and_then(|v| v.as_str()) == Some("enum") {
                    if let (Some(values), Some(current)) = (
                        def.get("values").and_then(|v| v.as_array()),
                        state.get(name),
                    ) {
                        if let Some(s) = current.as_str() {
                            let allowed: Vec<&str> =
                                values.iter().filter_map(|v| v.as_str()).collect();
                            if !allowed.contains(&s) {
                                eprintln!(
                                    "warning: skill '{}': var '{}' has value '{}' \
                                     which is not in allowed values: {}",
                                    state_key,
                                    name,
                                    s,
                                    allowed.join(", ")
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(state)
}

#[cfg(test)]
mod sdk_skill_tests {
    use super::*;
    use agent_schema::skill::SkillSource;

    #[test]
    #[ignore = "embedded SDK skills disabled until ship-contracts publishes a ship-sdk-skills crate; see embedded_capability_skills() and the commented include_dir!"]
    fn embedded_capability_skills_present_and_parsed() {
        let skills = embedded_capability_skills();
        assert!(
            !skills.is_empty(),
            "sdk capability-skills must be embedded into the binary"
        );
        let pipelines = skills
            .iter()
            .find(|s| s.id == "ship-pipelines")
            .expect("ship-pipelines capability-skill embedded");
        assert!(
            pipelines.content.contains("ship ops call"),
            "capability-skill teaches the ops CLI"
        );
        assert_eq!(pipelines.source, SkillSource::Builtin);
    }

    #[test]
    #[ignore = "embedded SDK skills disabled until ship-contracts publishes a ship-sdk-skills crate; see embedded_skill_reference_files()"]
    fn embedded_skill_reference_pages_are_available() {
        let refs = embedded_skill_reference_files("ship-pipelines");
        assert!(
            !refs.is_empty(),
            "ship-pipelines ships per-op reference pages"
        );
        assert!(
            refs.iter().all(|(p, _)| p != "SKILL.md"),
            "SKILL.md is delivered by the compiler, not as a reference file"
        );
        assert!(
            refs.iter()
                .any(|(p, _)| p.starts_with("references/docs/") && p.ends_with(".mdoc")),
            "reference pages live under references/docs/*.mdoc"
        );
        // Unknown ids (authored skills) have no embedded references.
        assert!(embedded_skill_reference_files("not-an-sdk-skill").is_empty());
    }

    // `compile_pool_includes_embedded_for_a_clean_project` retired
    // with Wave 5: the `load_compile_skills` compile-pool fn is gone
    // (project skills now arrive via `defineSkill` declare-forms in
    // `stack.config.ts`; the embedded capability-skills path is the
    // future `ship-sdk-skills` dep — see the `embedded_capability_skills`
    // doc comment above for the deferred `include_dir!` /
    // `ship-sdk-skills` migration).
}
