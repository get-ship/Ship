//! Deterministic `example()` fixtures for `rules.*` wire shapes.
//! Gated behind `cfg(test)` / the `fixtures` feature; drives gen-fixtures.

use serde_json::Value;

use super::state::{IndexEntry, ScopeTag};
use super::wire_types::{RuleGetResponse, RulesIndexResponse};

impl RulesIndexResponse {
    pub fn example() -> Self {
        RulesIndexResponse {
            items: vec![
                IndexEntry { id: "no-placeholders".to_string(), scope: ScopeTag::Project },
                IndexEntry { id: "commit-style".to_string(), scope: ScopeTag::User },
            ],
        }
    }
}

impl RuleGetResponse {
    pub fn example() -> Self {
        RuleGetResponse {
            id: "no-placeholders".to_string(),
            scope: ScopeTag::Project,
            body: "# No placeholders\n\nNo TODOs, no fake data. If it's not finished, remove it.\n"
                .to_string(),
        }
    }
}

/// Op name → example JSON for every `rules.*` data op.
pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("rules.list", v(RulesIndexResponse::example())),
        ("rules.get", v(RuleGetResponse::example())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rules_examples_are_structured() {
        for (op, value) in examples() {
            assert!(value.is_object(), "{op} example is an object");
        }
    }
}
