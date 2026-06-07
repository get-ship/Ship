//! Structural JSON diff → Vec<Patch>.
//!
//! Entry point: `diff(old, new, base_path)`.
//!
//! Arrays of objects with stable `id` fields are diffed by id (append/insert/
//! remove/merge). Everything else falls back to `set` at the path.

use serde_json::Value;
use crate::patch::Patch;

/// Compute the minimal patch list to transform `old` into `new` rooted at `base_path`.
///
/// Returns an empty vec when the values are identical.
/// The first emission from a resolver is always a synthetic root `set`, not a diff.
pub fn diff(old: &Value, new: &Value, base_path: &str) -> Vec<Patch> {
    if old == new {
        return vec![];
    }
    match (old, new) {
        (Value::Object(o), Value::Object(n)) => diff_objects(o, n, base_path),
        (Value::Array(o), Value::Array(n)) => diff_arrays(o, n, base_path),
        _ => vec![Patch::set(base_path, new.clone())],
    }
}

fn child_path(base: &str, key: &str) -> String {
    if base.is_empty() { key.to_owned() } else { format!("{base}.{key}") }
}

fn index_path(base: &str, idx: usize) -> String {
    if base.is_empty() {
        format!("[{idx}]")
    } else {
        format!("{base}[{idx}]")
    }
}

fn diff_objects(
    old: &serde_json::Map<String, Value>,
    new: &serde_json::Map<String, Value>,
    base: &str,
) -> Vec<Patch> {
    let mut patches = Vec::new();

    for (key, nv) in new {
        let path = child_path(base, key);
        match old.get(key) {
            None => patches.push(Patch::set(path, nv.clone())),
            Some(ov) if ov != nv => patches.extend(diff(ov, nv, &path)),
            _ => {}
        }
    }

    for key in old.keys() {
        if !new.contains_key(key) {
            patches.push(Patch::remove(child_path(base, key)));
        }
    }

    patches
}

fn has_id(v: &Value) -> bool {
    matches!(v, Value::Object(m) if m.get("id").and_then(Value::as_str).is_some())
}

fn id_of(v: &Value) -> &str {
    v.get("id").and_then(Value::as_str).unwrap_or("")
}

fn diff_arrays(old: &[Value], new: &[Value], base: &str) -> Vec<Patch> {
    let stable_ids = old.iter().all(has_id) && new.iter().all(has_id);
    if !stable_ids {
        return vec![Patch::set(base, Value::Array(new.to_vec()))];
    }

    let old_ids: Vec<&str> = old.iter().map(id_of).collect();
    let new_ids: Vec<&str> = new.iter().map(id_of).collect();

    let old_id_set: std::collections::HashSet<&str> = old_ids.iter().copied().collect();
    let new_id_set: std::collections::HashSet<&str> = new_ids.iter().copied().collect();

    let mut patches = Vec::new();

    // Removes — descending index order so earlier indices aren't shifted.
    let mut removed: Vec<usize> = old
        .iter()
        .enumerate()
        .filter(|(_, v)| !new_id_set.contains(id_of(v)))
        .map(|(i, _)| i)
        .collect();
    removed.sort_unstable_by(|a, b| b.cmp(a));
    for idx in removed {
        patches.push(Patch::remove(index_path(base, idx)));
    }

    // Additions — append if last in new, insert otherwise.
    for (ni, nv) in new.iter().enumerate() {
        let id = id_of(nv);
        if old_id_set.contains(id) {
            continue;
        }
        if ni == new.len() - 1 && !old_ids.contains(&id) {
            patches.push(Patch::append(base, nv.clone()));
        } else {
            patches.push(Patch::insert(base, nv.clone(), ni as i64));
        }
    }

    // Per-item field changes — merge at new index.
    let old_by_id: std::collections::HashMap<&str, &Value> =
        old.iter().map(|v| (id_of(v), v)).collect();
    for (ni, nv) in new.iter().enumerate() {
        let id = id_of(nv);
        if let Some(ov) = old_by_id.get(id) {
            if *ov != nv {
                let changed = shallow_diff_fields(ov, nv);
                if !changed.is_empty() {
                    patches.push(Patch::merge(
                        index_path(base, ni),
                        Value::Object(changed),
                    ));
                }
            }
        }
    }

    // Reorders — emit `move` for items whose position shifted (simple linear scan).
    // Only run when no adds/removes to avoid conflating index shifts.
    let only_moves = old.len() == new.len()
        && old_ids.iter().all(|id| new_id_set.contains(id));
    if only_moves {
        emit_moves(old, new, base, &mut patches);
    }

    patches
}

fn shallow_diff_fields(
    old: &Value,
    new: &Value,
) -> serde_json::Map<String, Value> {
    let mut changed = serde_json::Map::new();
    if let (Value::Object(om), Value::Object(nm)) = (old, new) {
        for (k, nv) in nm {
            if om.get(k) != Some(nv) {
                changed.insert(k.clone(), nv.clone());
            }
        }
    }
    changed
}

fn emit_moves(old: &[Value], new: &[Value], base: &str, patches: &mut Vec<Patch>) {
    // Simple O(n²) scan: for each item that moved, emit a `move` patch.
    // Applied patches shift subsequent indices; we apply a simulation to track.
    let old_ids: Vec<&str> = old.iter().map(id_of).collect();
    let new_ids: Vec<&str> = new.iter().map(id_of).collect();

    if old_ids == new_ids {
        return;
    }

    // Find the first pair that differs and emit a move.
    // This handles single-element moves; complex multi-moves fall back gracefully
    // (worst case: over-emit moves that are still semantically correct).
    let mut current: Vec<&str> = old_ids.clone();
    let mut i = 0;
    while i < new_ids.len() {
        if current.get(i) != new_ids.get(i) {
            if let Some(from_pos) = current.iter().position(|&id| id == new_ids[i]) {
                patches.push(Patch::move_item(base, from_pos as i64, i as i64));
                let item = current.remove(from_pos);
                current.insert(i, item);
            }
        }
        i += 1;
    }
}

// ── Tests for the diff module (unit) ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::patch::PatchOp;

    #[test]
    fn diff_identical_snapshots_emits_nothing() {
        let v = json!({"a": 1, "b": [1, 2, 3]});
        assert!(diff(&v, &v, "").is_empty());
    }

    #[test]
    fn diff_added_object_key_emits_set() {
        let old = json!({});
        let new = json!({"key": "value"});
        let patches = diff(&old, &new, "");
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].op, PatchOp::Set);
        assert_eq!(patches[0].path, "key");
        assert_eq!(patches[0].value, Some(json!("value")));
    }

    #[test]
    fn diff_removed_object_key_emits_remove() {
        let old = json!({"key": "value"});
        let new = json!({});
        let patches = diff(&old, &new, "");
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].op, PatchOp::Remove);
        assert_eq!(patches[0].path, "key");
    }

    #[test]
    fn diff_changed_scalar_emits_set() {
        let old = json!({"v": 1});
        let new = json!({"v": 2});
        let patches = diff(&old, &new, "");
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].op, PatchOp::Set);
        assert_eq!(patches[0].path, "v");
        assert_eq!(patches[0].value, Some(json!(2)));
    }

    #[test]
    fn diff_appended_list_item_emits_append() {
        let old = json!([{"id": "a"}]);
        let new = json!([{"id": "a"}, {"id": "b"}]);
        let patches = diff(&old, &new, "items");
        let appends: Vec<_> = patches.iter().filter(|p| p.op == PatchOp::Append).collect();
        assert_eq!(appends.len(), 1);
        assert_eq!(appends[0].path, "items");
        assert_eq!(appends[0].value, Some(json!({"id": "b"})));
    }

    #[test]
    fn diff_removed_list_item_emits_remove() {
        let old = json!([{"id": "a"}, {"id": "b"}]);
        let new = json!([{"id": "a"}]);
        let patches = diff(&old, &new, "items");
        let removes: Vec<_> = patches.iter().filter(|p| p.op == PatchOp::Remove).collect();
        assert_eq!(removes.len(), 1);
        assert_eq!(removes[0].path, "items[1]");
    }

    #[test]
    fn diff_item_field_change_emits_merge_at_index() {
        let old = json!([{"id": "a", "v": 1}]);
        let new = json!([{"id": "a", "v": 2}]);
        let patches = diff(&old, &new, "items");
        let merges: Vec<_> = patches.iter().filter(|p| p.op == PatchOp::Merge).collect();
        assert_eq!(merges.len(), 1);
        assert_eq!(merges[0].path, "items[0]");
        assert_eq!(merges[0].value, Some(json!({"v": 2})));
    }
}
