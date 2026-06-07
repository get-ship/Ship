//! JSONC value helpers for agent config projection.
//!
//! The agent-specific helpers (`first_letter`, `resolve_array_field`,
//! `extract_str_array`) live here because they're shaped around the agent
//! config schema. They operate on already-parsed `serde_json::Value`s — the
//! file READ + JSONC parse is THE reader's job (`ship_config::agents`), so no
//! raw `std::fs` lives here. The comment-stripping itself delegates to
//! `ship_substrate::fs_util::strip_jsonc_comments` — one canonical parser.

/// Strip `// …` / `/* … */` comments and trailing commas from a JSONC
/// input. Thin wrapper over `ship_substrate::fs_util::strip_jsonc_comments`
/// so callers in agents/skills/rules/permissions/mcp continue to import
/// from this module without leaking the substrate path everywhere.
pub fn strip_jsonc(input: &str) -> String {
    ship_substrate::fs_util::strip_jsonc_comments(input)
}

pub fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}


pub fn extract_str_array(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a collection field from an agent config. Reads either the
/// top-level array (when `nested_keys` is empty) or a nested sub-key
/// (e.g. `skills.refs`, `mcp.servers`).
///
/// The `nested_keys` parameter lists candidate sub-keys to inspect when the
/// top-level value is an object; the first one that contains an array wins.
pub fn resolve_array_field(
    obj: &serde_json::Value,
    top_keys: &[&str],
    nested_keys: &[&str],
) -> Vec<String> {
    for key in top_keys {
        let Some(v) = obj.get(*key) else { continue };
        if v.is_array() {
            return extract_str_array(v);
        }
        if let Some(inner) = v.as_object() {
            for nk in nested_keys {
                if let Some(arr) = inner.get(*nk)
                    && arr.is_array()
                {
                    return extract_str_array(arr);
                }
            }
        }
    }
    Vec::new()
}
