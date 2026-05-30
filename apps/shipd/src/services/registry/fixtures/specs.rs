//! `specs.*` fixtures (list / get).

use std::collections::HashMap;

use serde_json::Value;

use crate::services::registry::wire::{SpecGetResponse, SpecsListResponse};
use crate::services::specs::shape::Spec;

/// Fixed instant so committed fixtures diff cleanly (2026-01-01T00:00:00Z).
fn fixed_dt() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_767_225_600, 0).expect("valid timestamp")
}

fn spec() -> Spec {
    let mut vars = HashMap::new();
    vars.insert("area".to_string(), "daemon".to_string());
    Spec {
        id: "typed-ops".to_string(),
        title: "Typed-op single source".to_string(),
        body_path: ".ship/specs/typed-ops.md".to_string(),
        status: "completed".to_string(),
        pipeline: Some("ship-feature".to_string()),
        agent: Some("rust-lane".to_string()),
        vars,
        criteria: vec!["Every op declared once".to_string()],
        scope: vec!["apps/shipd".to_string()],
        priority: Some(1),
        created_at: fixed_dt(),
        updated_at: fixed_dt(),
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    let get = SpecGetResponse {
        spec: spec(),
        body: "# Typed-op single source\n\nKill the shadow registry.".to_string(),
    };
    vec![
        ("specs.list", v(SpecsListResponse { items: vec![spec()] })),
        ("specs.get", v(get)),
    ]
}
