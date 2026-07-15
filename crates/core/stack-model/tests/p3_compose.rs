//! Golden tests for the Phase-3 composition law (`stack_model::compose`).
//!
//! The law under test (census-phase3-agents.md Parts 3 + 5; tier split per
//! the format-migration Wave 2 census Part 3.4): id-keyed shadowing per
//! noun, cache < space < per-agent overlay, losers retained as
//! ShadowRecords, permissions/hooks pass
//! through from the space tier verbatim and are NEVER composed across
//! tiers.

use stack_model::compose::{ComposedLibrary, NounKind, compose_space};
use stack_model::resolve::{ExtSource, ExtensionDescriptor, ExtensionLibrary, RollupInputs};
use stack_model::types::{
    AgentProfile, HookConfig, McpServerConfig, Permissions, ProfileMeta, Rule, Skill,
};
use serde_json::json;

fn agent(id: &str, name: &str) -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta {
            id: id.to_string(),
            name: name.to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn skill(id: &str, name: &str) -> Skill {
    serde_json::from_value(json!({"id": id, "name": name, "content": "content"})).unwrap()
}

fn rule(file_name: &str, content: &str) -> Rule {
    serde_json::from_value(json!({"file_name": file_name, "content": content})).unwrap()
}

fn mcp(id: &str, name: &str) -> McpServerConfig {
    serde_json::from_value(json!({"id": id, "name": name, "command": "cmd"})).unwrap()
}

fn hook(id: &str, command: &str) -> HookConfig {
    serde_json::from_value(json!({"id": id, "trigger": "PreToolUse", "command": command})).unwrap()
}

fn ext(source: ExtSource, hash: &str, descriptor: ExtensionLibrary) -> ExtensionDescriptor {
    ExtensionDescriptor {
        source,
        space_root: std::path::PathBuf::from("/x"),
        descriptor,
        content_hash: hash.to_string(),
    }
}

fn inputs(extensions: Vec<ExtensionDescriptor>) -> RollupInputs {
    RollupInputs {
        extensions,
        precedence: RollupInputs::default_precedence(),
    }
}

fn compose(extensions: Vec<ExtensionDescriptor>) -> ComposedLibrary {
    compose_space(&inputs(extensions)).expect("compose must succeed")
}

// ── Shadowing ────────────────────────────────────────────────────────────────

#[test]
fn space_shadows_cache_on_id_collision() {
    let user = ExtensionLibrary {
        agents: vec![
            agent("writer", "User Writer"),
            agent("planner", "User Planner"),
        ],
        ..Default::default()
    };
    let space = ExtensionLibrary {
        agents: vec![agent("writer", "Space Writer")],
        ..Default::default()
    };
    let composed = compose(vec![
        ext(ExtSource::Cache, "hash-user", user),
        ext(ExtSource::Space, "hash-space", space),
    ]);

    // The colliding id resolves to the space tier; the un-shadowed
    // cache agent survives.
    let names: Vec<(&str, &str)> = composed
        .library
        .agents
        .iter()
        .map(|a| (a.agent.id.as_str(), a.agent.name.as_str()))
        .collect();
    assert_eq!(
        names,
        vec![("writer", "Space Writer"), ("planner", "User Planner")]
    );

    // The loser is retained as provenance, not dropped.
    assert_eq!(composed.shadows.len(), 1);
    let shadow = &composed.shadows[0];
    assert_eq!(shadow.noun, NounKind::Agent);
    assert_eq!(shadow.id, "writer");
    assert_eq!(shadow.losing_source, ExtSource::Cache);
    assert_eq!(shadow.losing_tier_hash, "hash-user");
    assert_eq!(shadow.winning_source, ExtSource::Space);
}

#[test]
fn agent_overlay_wins_over_space_and_cache() {
    let user = ExtensionLibrary {
        skills: vec![skill("lint", "user lint")],
        ..Default::default()
    };
    let space = ExtensionLibrary {
        skills: vec![skill("lint", "space lint")],
        ..Default::default()
    };
    let overlay = ExtensionLibrary {
        skills: vec![skill("lint", "overlay lint")],
        ..Default::default()
    };
    let composed = compose(vec![
        // Declared order deliberately scrambled — rank decides, not position.
        ext(ExtSource::Agent("writer".into()), "hash-overlay", overlay),
        ext(ExtSource::Cache, "hash-user", user),
        ext(ExtSource::Space, "hash-space", space),
    ]);

    assert_eq!(composed.library.skills.len(), 1);
    assert_eq!(composed.library.skills[0].name, "overlay lint");
    // Two collisions resolved: cache→space, space→overlay.
    assert_eq!(composed.shadows.len(), 2);
    assert_eq!(
        composed.shadows[1].winning_source,
        ExtSource::Agent("writer".into())
    );
}

#[test]
fn rules_shadow_by_file_name_and_mcp_by_id() {
    let user = ExtensionLibrary {
        rules: vec![rule("style.md", "user style")],
        mcp_servers: vec![mcp("server-a", "user server-a")],
        ..Default::default()
    };
    let space = ExtensionLibrary {
        rules: vec![rule("style.md", "space style")],
        mcp_servers: vec![mcp("server-a", "space server-a")],
        ..Default::default()
    };
    let composed = compose(vec![
        ext(ExtSource::Cache, "h-user", user),
        ext(ExtSource::Space, "h-space", space),
    ]);

    assert_eq!(composed.library.rules.len(), 1);
    assert_eq!(composed.library.rules[0].content, "space style");
    assert_eq!(composed.library.mcp_servers.len(), 1);
    assert_eq!(composed.library.mcp_servers[0].name, "space server-a");
    let nouns: Vec<NounKind> = composed.shadows.iter().map(|s| s.noun).collect();
    assert!(nouns.contains(&NounKind::Rule) && nouns.contains(&NounKind::McpServer));
}

// ── Permissions / hooks fence ────────────────────────────────────────────────

#[test]
fn permissions_and_hooks_pass_through_space_tier_verbatim() {
    let mut user_permissions = Permissions::default();
    user_permissions.tools.allow = vec!["everything".to_string()];
    let user = ExtensionLibrary {
        permissions: user_permissions,
        hooks: vec![hook("h-user", "user-hook")],
        ..Default::default()
    };

    let mut space_permissions = Permissions::default();
    space_permissions.tools.allow = vec!["read".to_string()];
    let space = ExtensionLibrary {
        permissions: space_permissions,
        hooks: vec![hook("h-space", "space-hook")],
        ..Default::default()
    };

    let composed = compose(vec![
        ext(ExtSource::Cache, "h-user", user),
        ext(ExtSource::Space, "h-space", space.clone()),
    ]);

    // Byte-identical to the space tier — no cross-tier merge, no records.
    assert_eq!(
        serde_json::to_string(&composed.library.permissions).unwrap(),
        serde_json::to_string(&space.permissions).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&composed.library.hooks).unwrap(),
        serde_json::to_string(&space.hooks).unwrap(),
    );
    assert!(composed.shadows.is_empty());
}

// ── Degenerate and identity cases ────────────────────────────────────────────

#[test]
fn space_tier_alone_composes_to_itself_unchanged() {
    let space = ExtensionLibrary {
        agents: vec![agent("writer", "Writer")],
        skills: vec![skill("lint", "lint")],
        rules: vec![rule("style.md", "style")],
        ..Default::default()
    };
    let composed = compose(vec![ext(ExtSource::Space, "h", space.clone())]);
    assert_eq!(
        serde_json::to_string(&composed.library).unwrap(),
        serde_json::to_string(&space).unwrap(),
    );
    assert!(composed.shadows.is_empty());
}

#[test]
fn no_extensions_compose_to_the_empty_library() {
    let composed = compose(vec![]);
    assert_eq!(
        serde_json::to_string(&composed.library).unwrap(),
        serde_json::to_string(&ExtensionLibrary::default()).unwrap(),
    );
    assert!(composed.shadows.is_empty());
}

#[test]
fn missing_precedence_row_is_an_error_not_a_default() {
    let bad = RollupInputs {
        extensions: vec![ext(ExtSource::Cache, "h", ExtensionLibrary::default())],
        precedence: vec![], // no rows at all
    };
    assert!(compose_space(&bad).is_err());
}

// ── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn identical_inputs_compose_to_identical_bytes() {
    let user = ExtensionLibrary {
        agents: vec![agent("a", "ua"), agent("b", "ub"), agent("c", "uc")],
        skills: vec![skill("s1", "u1"), skill("s2", "u2")],
        ..Default::default()
    };
    let space = ExtensionLibrary {
        agents: vec![agent("b", "sb")],
        skills: vec![skill("s2", "s2-space"), skill("s3", "s3-space")],
        ..Default::default()
    };
    let make = || {
        compose(vec![
            ext(ExtSource::Cache, "h-user", user.clone()),
            ext(ExtSource::Space, "h-space", space.clone()),
        ])
    };
    let first = make();
    let second = make();
    assert_eq!(
        serde_json::to_string(&first.library).unwrap(),
        serde_json::to_string(&second.library).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&first.shadows).unwrap(),
        serde_json::to_string(&second.shadows).unwrap(),
    );
}
