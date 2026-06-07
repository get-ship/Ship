//! Deterministic `example()` fixtures for `permissions.*` wire shapes.
//! Gated behind `cfg(test)` / the `fixtures` feature; drives gen-fixtures.

use serde_json::Value;

use super::state::{IndexEntry, ScopeTag};
use super::wire_types::{
    PermissionGetResponse, PermissionsIndexResponse, ToolPattern, ToolPatternsResponse,
};

impl PermissionsIndexResponse {
    pub fn example() -> Self {
        PermissionsIndexResponse {
            items: vec![
                IndexEntry { id: "ship-standard".to_string(), scope: ScopeTag::Project },
                IndexEntry { id: "ship-readonly".to_string(), scope: ScopeTag::User },
            ],
        }
    }
}

impl PermissionGetResponse {
    pub fn example() -> Self {
        PermissionGetResponse {
            id: "ship-standard".to_string(),
            scope: ScopeTag::Project,
            entry: serde_json::json!({
                "description": "Human confirms risky actions before they run.",
                "tools": { "allow": ["Read", "Edit"], "ask": ["Bash(rm -rf *)"], "deny": [] },
            }),
        }
    }
}

impl ToolPatternsResponse {
    pub fn example() -> Self {
        ToolPatternsResponse {
            patterns: vec![
                ToolPattern {
                    pattern: "Read".to_string(),
                    label: "Read".to_string(),
                    source: "harness".to_string(),
                },
                ToolPattern {
                    pattern: "Bash(git *)".to_string(),
                    label: "Bash git · all git subcommands".to_string(),
                    source: "bash".to_string(),
                },
                ToolPattern {
                    pattern: "mcp__ship__*".to_string(),
                    label: "mcp · ship · all tools".to_string(),
                    source: "mcp-ship".to_string(),
                },
            ],
        }
    }
}

/// Op name → example JSON for every `permissions.*` data op.
pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("permissions.list", v(PermissionsIndexResponse::example())),
        ("permissions.get", v(PermissionGetResponse::example())),
        ("permissions.tool_patterns", v(ToolPatternsResponse::example())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn permissions_examples_are_structured() {
        for (op, value) in examples() {
            assert!(value.is_object(), "{op} example is an object");
        }
    }
}
