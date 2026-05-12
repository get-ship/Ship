//! JSONC helpers for agent config parsing.
//!
//! The agent-specific helpers (`first_letter`, `parse_file`, `resolve_array_field`,
//! `extract_str_array`) live here because they're shaped around the agent
//! config schema. The comment-stripping itself delegates to
//! `ship_substrate::fs_util::strip_jsonc_comments` — one canonical
//! parser, four prior copies collapsed to one trunk implementation.

/// Strip `// …` / `/* … */` comments and trailing commas from a JSONC
/// input. Thin wrapper over `ship_substrate::fs_util::strip_jsonc_comments`
/// so callers in agents/skills/rules/permissions/mcp continue to import
/// from this module without leaking the substrate path everywhere.
pub fn strip_jsonc(input: &str) -> String {
    ship_substrate::fs_util::strip_jsonc_comments(input)
}

pub(crate) fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

pub(crate) fn parse_file(
    path: &std::path::Path,
) -> Option<(String, serde_json::Value)> {
    let id = path.file_stem().and_then(|s| s.to_str())?.to_string();
    let raw = std::fs::read_to_string(path).ok()?;
    let json_str = strip_jsonc(&raw);
    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    Some((id, parsed))
}

pub(crate) fn extract_str_array(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| match item {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Object(map) => map
                        .get("id")
                        .or_else(|| map.get("name"))
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a collection-ish field from an agent config, tolerating the two
/// shapes Ship actually emits:
///   1. Flat: `"skills": ["a", "b"]` or `[{"id": "a"}, {"name": "b"}]`
///   2. Nested: `"skills": { "refs": [...] }` / `"mcp": { "servers": [...] }`
///
/// The `nested_keys` parameter lists candidate sub-keys to inspect when the
/// top-level value is an object; the first one that contains an array wins.
pub(crate) fn resolve_array_field(
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
