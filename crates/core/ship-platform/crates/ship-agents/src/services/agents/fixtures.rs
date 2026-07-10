//! Deterministic `example()` constructors for the `agents.*` wire shapes, and
//! the `examples()` registry that maps each data op to its example JSON.
//!
//! The Rust struct is the single source: `example()` builds a real instance
//! (shape compiler-enforced — a field rename breaks this file), and serde
//! serializes it to the exact JSON the daemon emits. `cargo xtask gen-fixtures`
//! writes one committed file per op from this registry; tests feed the same
//! builders live. No faker/zod/json-schema layer — the type is the schema.
//!
//! Gated behind `cfg(test)` (runnable now) or the `fixtures` feature (so the
//! gen-fixtures binary can call into it). Supersedes the `view_state_fixtures`
//! spike.

use serde_json::Value;

use super::create::{
    builtin_harnesses, builtin_permission_presets, builtin_templates, build_create_ctx,
    AgentsCreateCtx,
};
use super::settings_view::{build_tiers, AgentSettingsCtx};
use super::state::{IndexEntry, ScopeTag};
use super::view_state::{
    AgentCatalogs, AgentDetail, AgentSummary, AgentsSummariesResponse, CatalogEntry, Permissions,
    ToolPatternEntry, SOURCE_GLOBAL, SOURCE_PROJECT,
};
use super::wire::{AgentWireCtx, WireEdge, WireNode};
use super::wire_types::{
    AgentGetResponse, AgentIndexResponse, AgentTemplatesResponse,
    PermissionPresetsResponse,
};

impl AgentSummary {
    /// A representative project-scoped agent. Deterministic — a committed
    /// fixture should diff cleanly.
    pub fn example() -> Self {
        AgentSummary {
            id: "code-reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            description: "Reviews diffs for security, correctness, and architecture drift.".to_string(),
            skills_count: 3,
            mcp_count: 1,
            rules_count: 2,
            letter: "C".to_string(),
            source: SOURCE_PROJECT.to_string(),
            tags: vec!["review".to_string(), "workflow".to_string()],
            active: false,
        }
    }
}

impl AgentsSummariesResponse {
    pub fn example() -> Self {
        AgentsSummariesResponse {
            items: vec![
                AgentSummary::example(),
                AgentSummary {
                    id: "rust-lane".to_string(),
                    name: "Rust Lane".to_string(),
                    description: "Full Rust implementation lane — compiler, runtime, MCP server.".to_string(),
                    skills_count: 2,
                    mcp_count: 1,
                    rules_count: 4,
                    letter: "R".to_string(),
                    source: SOURCE_PROJECT.to_string(),
                    tags: vec!["implementation".to_string()],
                    active: true,
                },
                AgentSummary {
                    id: "shipwright".to_string(),
                    name: "Shipwright".to_string(),
                    description: "Creates and iterates on Ship's own feature surfaces.".to_string(),
                    skills_count: 5,
                    mcp_count: 2,
                    rules_count: 1,
                    letter: "S".to_string(),
                    source: SOURCE_GLOBAL.to_string(),
                    tags: vec!["scaffold".to_string(), "workflow".to_string()],
                    active: false,
                },
            ],
        }
    }
}

impl IndexEntry {
    pub fn example() -> Self {
        IndexEntry { id: "code-reviewer".to_string(), scope: ScopeTag::Project }
    }
}

impl AgentIndexResponse {
    pub fn example() -> Self {
        AgentIndexResponse {
            items: vec![
                IndexEntry::example(),
                IndexEntry { id: "shipwright".to_string(), scope: ScopeTag::User },
            ],
        }
    }
}

impl Permissions {
    pub fn example() -> Self {
        Permissions {
            preset: "ship-standard".to_string(),
            tools_allow: vec!["Read".to_string(), "Edit".to_string(), "Bash(git *)".to_string()],
            tools_ask: vec!["Bash(rm -rf *)".to_string()],
            tools_deny: vec![],
        }
    }
}

impl AgentDetail {
    pub fn example() -> Self {
        AgentDetail {
            id: "code-reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            description: "Reviews diffs for security, correctness, and architecture drift.".to_string(),
            letter: "C".to_string(),
            icon: Some("shield-check".to_string()),
            version: Some("1.0.0".to_string()),
            tags: vec!["review".to_string(), "workflow".to_string()],
            skills: vec!["diff-review".to_string(), "a11y-audit".to_string()],
            mcp_servers: vec!["ship".to_string()],
            rules: vec!["no-placeholders".to_string()],
            inline_rules: "Prefer minimal diffs.".to_string(),
            permissions_tier: "standard".to_string(),
            permissions: Permissions::example(),
            source: SOURCE_PROJECT.to_string(),
        }
    }
}

impl CatalogEntry {
    pub fn example() -> Self {
        CatalogEntry {
            id: "diff-review".to_string(),
            name: "Diff Review".to_string(),
            description: Some("Structured review of a unified diff.".to_string()),
        }
    }
}

impl ToolPatternEntry {
    pub fn example() -> Self {
        ToolPatternEntry {
            pattern: "Bash(git *)".to_string(),
            label: "Bash git · all git subcommands".to_string(),
            source: "bash".to_string(),
        }
    }
}

impl AgentCatalogs {
    pub fn example() -> Self {
        AgentCatalogs {
            skills: vec![CatalogEntry::example()],
            mcp: vec![CatalogEntry { id: "ship".to_string(), name: "Ship".to_string(), description: None }],
            rules: vec![CatalogEntry { id: "no-placeholders".to_string(), name: "no-placeholders".to_string(), description: None }],
            presets: vec![CatalogEntry { id: "ship-standard".to_string(), name: "ship-standard".to_string(), description: Some("Human confirms risky actions.".to_string()) }],
            tool_patterns: vec![ToolPatternEntry::example()],
        }
    }
}

impl AgentGetResponse {
    pub fn example() -> Self {
        AgentGetResponse {
            id: "code-reviewer".to_string(),
            scope: ScopeTag::Project,
            doc: serde_json::json!({
                "agent": { "name": "Code Reviewer", "description": "Reviews diffs." },
                "skills": { "refs": ["diff-review"] },
            }),
        }
    }
}

impl AgentWireCtx {
    pub fn example() -> Self {
        AgentWireCtx {
            agent: AgentDetail::example(),
            nodes: vec![
                WireNode { kind: "agent", id: "code-reviewer".to_string(), title: "Code Reviewer".to_string(), sub: "agent".to_string(), x: 400.0, y: 240.0, w: 160.0 },
                WireNode { kind: "skill", id: "diff-review".to_string(), title: "Diff Review".to_string(), sub: "skill".to_string(), x: 120.0, y: 120.0, w: 140.0 },
            ],
            edges: vec![WireEdge { kind: "skill", id: "diff-review".to_string(), path_d: "M400,240 C260,180 200,140 120,120".to_string(), dashed: false }],
            catalogs: AgentCatalogs::example(),
        }
    }
}

impl AgentSettingsCtx {
    pub fn example() -> Self {
        AgentSettingsCtx {
            agent: AgentDetail::example(),
            tiers: build_tiers("standard"),
            catalogs: AgentCatalogs::example(),
        }
    }
}

impl AgentTemplatesResponse {
    pub fn example() -> Self {
        AgentTemplatesResponse { items: builtin_templates() }
    }
}

impl PermissionPresetsResponse {
    pub fn example() -> Self {
        PermissionPresetsResponse { items: builtin_permission_presets() }
    }
}


impl AgentsCreateCtx {
    pub fn example() -> Self {
        build_create_ctx()
    }
}

/// Op name → example JSON for every `agents.*` data op. Drives
/// `cargo xtask gen-fixtures`; one committed file per entry.
pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("agents.list", v(AgentIndexResponse::example())),
        ("agents.summaries", v(AgentsSummariesResponse::example())),
        ("agents.detail", v(AgentDetail::example())),
        ("agents.catalogs", v(AgentCatalogs::example())),
        ("agents.get", v(AgentGetResponse::example())),
        ("agents.wire", v(AgentWireCtx::example())),
        ("agents.settings", v(AgentSettingsCtx::example())),
        ("agents.templates", v(AgentTemplatesResponse::example())),
        ("agents.permission_presets", v(PermissionPresetsResponse::example())),
        // agents.harnesses moved to ship.code.v1.CodeService.Harnesses
        // (mount split 2026-07-10) — see code_wire::harnesses for the
        // live wire handler and the fixture in
        // ship.code.v1.ListHarnessesResponse.
        ("agents.create_ctx", v(AgentsCreateCtx::example())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every agents data op produces a non-null object example with the
    /// expected top-level shape — the wire contract the designer consumes.
    #[test]
    fn agents_examples_cover_every_op() {
        let examples = examples();
        assert_eq!(examples.len(), 10, "one example per agents data op");
        for (op, value) in &examples {
            assert!(value.is_object() || value.is_array(), "{op} example is structured");
        }
    }
    #[test]
    fn list_shaped_examples_carry_items() {
        for op in ["agents.list", "agents.summaries", "agents.templates"] {
            let (_, value) = examples().into_iter().find(|(n, _)| *n == op).unwrap();
            assert!(value["items"].is_array(), "{op} has items[]");
        }
    }
}
