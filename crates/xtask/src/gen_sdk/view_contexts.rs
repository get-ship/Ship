//! Surface every `*Ctx` struct in `apps/shipd/src/views_*.rs` and
//! `apps/shipd/src/specs_view.rs`. For now we emit a structural sketch of
//! each struct (name, source location, field name/type pairs) rather than
//! a full JSON Schema — bringing `schemars` into `runtime`/`shipd` is a
//! sizeable dep change that belongs in its own PR. Downstream tools (view-
//! meta endpoint, AST lint) can already use name + fields for most checks,
//! and the `skipped` flag makes the gap explicit so a later pass can light
//! up real schemas without moving the file format.

use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{json, Map, Value};

const SOURCES: &[&str] = &[
    "apps/shipd/src/specs_view.rs",
    "apps/shipd/src/views_settings.rs",
    "apps/shipd/src/views_chat.rs",
    "apps/shipd/src/views_git.rs",
    "apps/shipd/src/views_events.rs",
    "apps/shipd/src/views_skills.rs",
    "apps/shipd/src/services/agents/wire.rs",
    "apps/shipd/src/services/agents/create.rs",
    "apps/shipd/src/services/agents/settings_view.rs",
];

pub fn extract(repo_root: &Path) -> Result<Value> {
    let struct_re =
        Regex::new(r"(?m)^(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*Ctx)\b(?:<[^>]*>)?\s*\{([^}]*)\}")
            .unwrap();
    let field_re = Regex::new(
        r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,\n][^,]*),?\s*$",
    )
    .unwrap();

    let mut out = Map::new();
    for src in SOURCES {
        let path = repo_root.join(src);
        if !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        for caps in struct_re.captures_iter(&text) {
            let name = caps.get(1).unwrap().as_str();
            let body = caps.get(2).unwrap().as_str();
            let line = text[..caps.get(0).unwrap().start()]
                .bytes()
                .filter(|&b| b == b'\n')
                .count()
                + 1;
            let fields: Vec<Value> = field_re
                .captures_iter(body)
                .filter_map(|fc| {
                    let fname = fc.get(1)?.as_str().to_string();
                    let ftype = fc.get(2)?.as_str().trim().trim_end_matches(',').to_string();
                    // Skip lines that are attributes like `#[serde(...)]` or doc comments.
                    if fname.starts_with('#') || fname.starts_with("///") {
                        return None;
                    }
                    Some(json!({ "name": fname, "type": ftype }))
                })
                .collect();

            let view_key = view_name_for(name);
            out.insert(
                view_key,
                json!({
                    "source": format!("{src}:{line}::{name}"),
                    "struct": name,
                    "fields": fields,
                    "schema": Value::Null,
                    "skipped": "full JsonSchema deferred — `schemars` not yet wired into runtime/shipd; struct fields surfaced for structural consumers",
                }),
            );
        }
    }
    Ok(Value::Object(out))
}

/// Map `PipelinesCtx` → `pipelines`, `RunDetailCtx` → `run-detail`, etc.
fn view_name_for(struct_name: &str) -> String {
    let stem = struct_name.trim_end_matches("Ctx");
    // Snake-case the stem.
    let mut s = String::with_capacity(stem.len() + 4);
    for (i, ch) in stem.chars().enumerate() {
        if ch.is_ascii_uppercase() && i != 0 {
            s.push('-');
        }
        s.push(ch.to_ascii_lowercase());
    }
    s
}
