//! Tests for `stack_model::load_library` — the LEAF `.ship/` → `ExtensionLibrary`
//! loader. The compiler crate is L0 / substrate-free, so this loader uses
//! only `std::fs` + the crate's own `jsonc` module.
//!
//! Post format-migration Wave 5: the leaf no longer reads `.ship/mcp.jsonc`,
//! `.ship/rules/*.md`, `.ship/agents/*.jsonc`, or `.ship/skills/<id>/SKILL.md`.
//! All four nouns arrive via the corresponding `define*` declare-forms in
//! the space's `ship.config.ts`, harvested by the resolver's TS eval and
//! merged into `ExtensionLibrary` (see
//! `crates/core/ship-resolver/src/lib_tests.rs::every_declare_form_round_trips_through_resolve_compose_descriptor`
//! for the full descriptor test). The leaf returns all four vectors
//! empty — the TS-resolver merge is the single source of truth.
//! Permissions / hooks are unchanged: read from `.ship/permissions.jsonc` and
//! `.ship/hooks.jsonc` (presets / hook config — fenced nouns not in Wave 5's
//! scope; see census-format-migration.md §2).

use stack_model::{ExtensionLibrary, load_library};

#[test]
fn load_library_returns_empty_noun_vectors_post_wave5() {
    // No `.ship/` fixture: the leaf no longer reads any noun from disk
    // — the four compose-time noun vectors (mcp_servers, rules, skills,
    // agent_profiles) all arrive via `ship.config.ts` declare-forms.
    // A `.ship/` with just `hooks.jsonc` / `permissions.jsonc` would
    // populate those two; the other four stay empty regardless.
    let tmp = tempfile::tempdir().expect("tempdir");
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let lib: ExtensionLibrary =
        load_library(&ship_dir).expect("load_library on a fresh .ship/ must succeed");

    assert!(
        lib.agent_profiles.is_empty(),
        "agent_profiles must be empty — the leaf no longer reads .ship/agents/*.jsonc (Wave 5)"
    );
    assert!(
        lib.mcp_servers.is_empty(),
        "mcp_servers must be empty — the leaf no longer reads .ship/mcp.jsonc (Wave 4)"
    );
    assert!(
        lib.rules.is_empty(),
        "rules must be empty — the leaf no longer reads .ship/rules/*.md (Wave 4)"
    );
    assert!(
        lib.skills.is_empty(),
        "skills must be empty — the leaf no longer reads .ship/skills/<id>/SKILL.md (Wave 5)"
    );
    assert!(
        lib.hooks.is_empty(),
        "hooks must be empty — fixture has no .ship/hooks.jsonc"
    );
}

#[test]
fn load_library_on_missing_dir_yields_empty_library() {
    // A non-existent .ship/ must NOT be a hard error — the existing
    // agent-def loader contract is "missing files are silently skipped;
    // an empty library is valid". The leaf loader must match.
    let tmp = tempfile::tempdir().expect("tempdir");
    let absent = tmp.path().join("does-not-exist");
    let lib = load_library(&absent).expect("load_library on a missing dir must succeed");
    assert!(lib.agent_profiles.is_empty());
    assert!(lib.mcp_servers.is_empty());
    assert!(lib.skills.is_empty());
    assert!(lib.rules.is_empty());
    assert!(lib.hooks.is_empty());
}
