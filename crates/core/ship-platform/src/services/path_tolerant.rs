//! Path-tolerant RFC 6902 support shared across services.
//!
//! Extracted from AgentsService's original inline helper so McpService and
//! PermissionService can reuse it without forking. The behaviour mirrors
//! `agents::state::ensure_paths_for_adds`: walks every `add` op's path and
//! materialises missing ancestor objects (and the terminal array for
//! `/-` append ops) so `add` doesn't fail RFC-6902-strict on a missing
//! intermediate key.
//!
//! Non-`add` operations follow RFC 6902 exactly — this helper is a no-op
//! for `remove`, `replace`, `move`, `copy`, and `test`.

pub fn ensure_paths_for_adds(doc: &mut serde_json::Value, patch: &json_patch::Patch) {
    use json_patch::PatchOperation as Op;
    for op in patch.0.iter() {
        let Op::Add(add) = op else { continue };
        let path = add.path.to_string();
        if path.is_empty() { continue }
        let container_path = if path.ends_with("/-") {
            &path[..path.len() - 2]
        } else {
            path.as_str()
        };
        let segments: Vec<&str> = container_path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        if segments.is_empty() { continue }
        let last_idx = segments.len().saturating_sub(1);
        let ends_with_append = path.ends_with("/-");
        let mut cursor: &mut serde_json::Value = doc;
        for (i, seg) in segments.iter().enumerate() {
            let is_last = i == last_idx;
            if !cursor.is_object() { break }
            let obj = cursor.as_object_mut().unwrap();
            if !obj.contains_key(*seg) {
                let placeholder = if is_last && ends_with_append {
                    serde_json::Value::Array(Vec::new())
                } else {
                    serde_json::Value::Object(serde_json::Map::new())
                };
                obj.insert((*seg).to_string(), placeholder);
            }
            cursor = obj.get_mut(*seg).unwrap();
        }
        let _ = cursor;
    }
}
