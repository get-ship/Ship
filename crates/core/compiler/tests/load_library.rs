//! Tests for `compiler::load_library` — the LEAF `.ship/` → `ExtensionLibrary`
//! loader. The compiler crate is L0 / substrate-free, so this loader uses
//! only `std::fs` + the crate's own `jsonc` + `skill_parser` modules.
//!
//! Post format-migration Wave 4 phase A: the leaf no longer reads
//! `.ship/mcp.jsonc` or `.ship/rules/*.md` — those nouns arrive via
//! `defineMcpServer` / `defineRule` declare-forms in the space's
//! `ship.config.ts`, harvested by the resolver's TS eval and merged into
//! `ExtensionLibrary` (see
//! `crates/core/ship-resolver/src/lib_tests.rs::every_declare_form_round_trips_through_resolve_compose_descriptor`
//! for the full descriptor test). This file covers the LEAF half:
//! agents + skills from disk, mcp_servers + rules explicitly empty,
//! permissions / hooks from disk (unchanged), missing-dir contract.

use std::path::PathBuf;

use compiler::{load_library, ExtensionLibrary};

fn fixture_ship_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR points at crates/core/compiler; the fixture sits
    // beside this test file under tests/load_library_fixture/.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("load_library_fixture")
        .join(".ship")
}

#[test]
fn load_library_reads_agents_and_skills_leaves_mcp_rules_empty() {
    let ship_dir = fixture_ship_dir();
    let lib: ExtensionLibrary =
        load_library(&ship_dir).expect("load_library must succeed against the fixture .ship/");

    // 1. Agent profile loaded from .ship/agents/test-agent.jsonc.
    let agents = &lib.agent_profiles;
    assert_eq!(agents.len(), 1, "expected exactly one agent in the fixture");
    let agent = &agents[0];
    assert_eq!(agent.agent.id, "test-agent");
    assert_eq!(agent.agent.name, "Test Agent");
    assert_eq!(agent.skills.refs, vec!["example-skill".to_string()]);
    assert_eq!(agent.mcp.servers, vec!["test-server".to_string()]);
    assert_eq!(agent.rules.inline.as_deref(), Some("Test rule body."));

    // 2. Skill loaded from .ship/skills/example-skill/SKILL.md.
    assert_eq!(lib.skills.len(), 1, "expected one skill in the fixture");
    let skill = &lib.skills[0];
    assert_eq!(skill.id, "example-skill");
    assert_eq!(skill.name, "example-skill");
    assert_eq!(
        skill.description.as_deref(),
        Some("Example skill for the load_library fixture.")
    );
    assert!(skill.content.contains("Example Skill"));

    // 3. MCP server and rule vectors are EMPTY post W4 — the leaf
    //    no longer reads .ship/mcp.jsonc or .ship/rules/*.md. Both
    //    arrive via defineMcpServer / defineRule declare-forms in
    //    ship.config.ts and land through the resolver's merge step.
    //    The full descriptor path is exercised in ship-resolver.
    assert!(
        lib.mcp_servers.is_empty(),
        "mcp_servers must be empty — the leaf no longer reads .ship/mcp.jsonc"
    );
    assert!(
        lib.rules.is_empty(),
        "rules must be empty — the leaf no longer reads .ship/rules/*.md"
    );

    // 4. Hooks loaded from .ship/hooks.jsonc (fixture does not include it).
    assert!(
        lib.hooks.is_empty(),
        "fixture has no hooks.jsonc, library must default to empty"
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
