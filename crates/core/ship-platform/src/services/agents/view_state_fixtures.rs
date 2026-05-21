//! Fixture generators for the `agents` view-state shapes — SPIKE.
//!
//! Proves the proposed fixture pipeline on one type: the Rust struct is the
//! single source. `example()` constructs a real instance (shape compiler-
//! enforced — a field rename breaks this file), and serde serializes it to the
//! exact JSON the daemon emits over the wire. That JSON is the artifact the
//! sandboxed designer consumes; tests feed the same builder live.
//!
//! Gated behind `cfg(test)` (runnable now) or the `fixtures` feature (so a
//! future `cargo xtask gen-fixtures` can call `example()` from a binary and
//! write the committed snapshot). No faker/zod/json-schema layer — the type is
//! the schema, serde is the serializer.

use super::view_state::{AgentSummary, AgentsSummariesResponse, SOURCE_GLOBAL, SOURCE_PROJECT};

impl AgentSummary {
    /// A representative project-scoped agent. Realistic, deterministic values
    /// (no randomness — a committed fixture should diff cleanly). Override any
    /// field with struct-update syntax: `AgentSummary { active: true, ..AgentSummary::example() }`.
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
    /// The wire shape of `ship.agents.summaries()` with a small realistic
    /// catalog spanning both scopes — enough for the designer to lay out a
    /// populated rail without inventing fields.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Emit the `agents.summaries` fixture and verify it serializes to the
    /// wire JSON with the expected shape. Run with:
    ///   cargo test -p ship-platform agents_summaries_fixture -- --nocapture
    #[test]
    fn agents_summaries_fixture() {
        let fixture = AgentsSummariesResponse::example();
        let json = serde_json::to_string_pretty(&fixture).expect("serialize");

        // Shape sanity: parse back as untyped JSON and assert the wire contract.
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let items = value["items"].as_array().expect("items is an array");
        assert_eq!(items.len(), 3, "three sample agents");
        for item in items {
            for key in ["id", "name", "description", "skills_count", "letter", "source", "tags", "active"] {
                assert!(item.get(key).is_some(), "field `{key}` present in wire shape");
            }
        }

        // Emit the artifact the designer would consume (best-effort write +
        // always print so `--nocapture` shows it).
        let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.artifacts/fixtures-poc");
        let _ = std::fs::create_dir_all(out_dir);
        let _ = std::fs::write(format!("{out_dir}/agents.summaries.json"), &json);

        println!("\n--- fixtures/agents.summaries.json ---\n{json}\n");
    }
}
