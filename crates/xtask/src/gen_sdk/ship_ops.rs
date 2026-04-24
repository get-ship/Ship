//! Re-emit `services::schema::OPS` as SDK JSON so downstream consumers
//! (lint rules, TS codegen, MCP registration) read from a single source
//! of truth. Matches the schema served live at `GET /api/ship/schema`.

use anyhow::Result;
use serde_json::{json, Map, Value};

pub fn extract() -> Result<Value> {
    let mut entries = Map::new();
    for op in shipd::services::schema::OPS {
        let args: Vec<Value> = op
            .args
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "kind": a.kind,
                    "required": a.required,
                })
            })
            .collect();
        entries.insert(
            op.name.to_string(),
            json!({
                "name": op.name,
                "description": op.description,
                "args": args,
                "returns": op.returns,
                "permission": op.permission,
                "sensitive": op.sensitive,
            }),
        );
    }
    Ok(Value::Object(entries))
}
