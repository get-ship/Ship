//! Deterministic `example()` fixtures for `mcp.*` wire shapes.
//! Gated behind `cfg(test)` / the `fixtures` feature; drives gen-fixtures.

use serde_json::Value;

use super::state::{IndexEntry, ScopeTag};
use super::wire_types::{McpGetResponse, McpIndexResponse};

impl McpIndexResponse {
    pub fn example() -> Self {
        McpIndexResponse {
            items: vec![
                IndexEntry { id: "ship".to_string(), scope: ScopeTag::Project },
                IndexEntry { id: "github".to_string(), scope: ScopeTag::User },
            ],
        }
    }
}

impl McpGetResponse {
    pub fn example() -> Self {
        McpGetResponse {
            id: "ship".to_string(),
            scope: ScopeTag::Project,
            entry: serde_json::json!({
                "name": "ship",
                "type": "stdio",
                "command": "ship",
                "args": ["mcp", "serve"],
            }),
        }
    }
}

/// Op name → example JSON for every `mcp.*` data op.
pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("mcp.list", v(McpIndexResponse::example())),
        ("mcp.get", v(McpGetResponse::example())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mcp_examples_are_structured() {
        for (op, value) in examples() {
            assert!(value.is_object(), "{op} example is an object");
        }
    }
}
