//! JSONC value helpers for agent config projection.
//!
//! These helpers operate on already-parsed `serde_json::Value`s — the
//! read (composed roll-up or index snapshot) is the caller's job
//! (`view_state::compose_space_library`, `ship_config::agents_index`). The
//! comment-stripping itself delegates to
//! `ship_substrate::fs_util::strip_jsonc_comments` (one canonical parser).
//!
//! Moved here from `ship_files::jsonc` per D12 — the helpers are shaped
//! around the agent config schema, so their canonical home is the
//! agents service. `ship_files::jsonc::strip_jsonc` stays behind as the
//! generic JSONC entry point; nothing else in the agents service needs
//! to be aware of the substrate path.

/// First character of `s`, upper-cased (or `"?"` for an empty string).
/// Powers the agent-row "letter avatar" badge.
pub fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// Extract a flat `Vec<String>` from a JSON value: string entries of an
/// array, or empty for anything else.
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
