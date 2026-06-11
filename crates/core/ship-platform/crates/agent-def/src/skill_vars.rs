//! Skill variable state — read/write for CLI and MCP tools.
//!
//! Storage: all scopes use `platform.db` KV. No files.
//!
//! KV namespaces:
//! - global  `skill_vars:{skill_id}`                      machine-wide preferences
//! - local   `skill_vars.local:{ctx}:{skill_id}`          per-context, not shared
//! - project `skill_vars.project:{ctx}:{skill_id}`        per-context, intended to be shared
//!
//! Where `ctx` is a stable hex token derived from the ship_dir path.
//!
//! Merge order (last wins): defaults → global → local → project.
//!
//! vars.json lives at `skills/{id}/assets/vars.json`.

use anyhow::Result;
use serde_json::Value;
use crate::skill_vars_store::KvNamespace;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

// ── Context key ───────────────────────────────────────────────────────────────

/// Stable 16-char hex token derived from the canonical ship_dir path.
/// Scopes local/project state to a specific context without exposing the path.
fn context_key(ship_dir: &Path) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ship_dir.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// ── KV namespaces ─────────────────────────────────────────────────────────────

fn kv_ns_global(skill_id: &str) -> KvNamespace {
    KvNamespace::SkillVarsGlobal(skill_id.to_string())
}

fn kv_ns_local(ctx: &str, skill_id: &str) -> KvNamespace {
    KvNamespace::SkillVarsLocal(format!("{ctx}:{skill_id}"))
}

fn kv_ns_project(ctx: &str, skill_id: &str) -> KvNamespace {
    KvNamespace::SkillVarsProject(format!("{ctx}:{skill_id}"))
}

// ── vars.json schema (minimal, read-only) ─────────────────────────────────────

#[derive(PartialEq)]
enum StorageHint {
    Global,
    Local,
    Project,
}

struct VarMeta {
    storage_hint: StorageHint,
    default: Option<Value>,
}

/// Parse `assets/vars.json` for a skill. Searches all configured skill_paths.
/// Returns `None` if the file is absent in all paths.
fn parse_vars_schema(ship_dir: &Path, skill_id: &str) -> Option<HashMap<String, VarMeta>> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let content = crate::skill_paths::read_skill_paths(ship_dir, project_root)
        .into_iter()
        .map(|dir| dir.join(skill_id).join("assets").join("vars.json"))
        .find_map(|path| std::fs::read_to_string(&path).ok())?;
    let raw: serde_json::Map<String, Value> = serde_json::from_str(&content).ok()?;

    let mut result = HashMap::new();
    for (key, val) in &raw {
        if key == "$schema" {
            continue;
        }
        let obj = val.as_object()?;
        // `storage-hint` is a config-storage scope, NOT a space. An absent or
        // unrecognized hint defaults to the machine-wide `Global` scope. Matched
        // on the raw `Option` (no `unwrap_or` default) so this never reads as a
        // space-`"global"` fallback (S2 home-space kill).
        let storage_hint = match obj.get("storage-hint").and_then(|v| v.as_str()) {
            Some("local") => StorageHint::Local,
            Some("project") => StorageHint::Project,
            _ => StorageHint::Global,
        };
        result.insert(
            key.clone(),
            VarMeta {
                storage_hint,
                default: obj.get("default").cloned(),
            },
        );
    }
    Some(result)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Read merged variable state for a skill.
///
/// Returns `None` if the skill has no `assets/vars.json`.
/// Merge order: defaults → KV-state-for-storage-hint. Defaults seed the
/// map; storage-hint-routed KV reads then override.
pub fn get_skill_vars(ship_dir: &Path, skill_id: &str) -> Result<Option<HashMap<String, Value>>> {
    let Some(schema) = parse_vars_schema(ship_dir, skill_id) else {
        return Ok(None);
    };

    let ctx = context_key(ship_dir);
    let mut merged: HashMap<String, Value> = HashMap::new();

    for (name, meta) in &schema {
        // 1. Default value (lowest precedence).
        if let Some(v) = &meta.default {
            merged.insert(name.clone(), v.clone());
        }
        // 2. KV value for this var's storage hint (overrides default).
        let ns = match meta.storage_hint {
            StorageHint::Global => kv_ns_global(skill_id),
            StorageHint::Local => kv_ns_local(&ctx, skill_id),
            StorageHint::Project => kv_ns_project(&ctx, skill_id),
        };
        if let Ok(Some(val)) = crate::skill_vars_store::get(ns, name) {
            merged.insert(name.clone(), val);
        }
    }

    Ok(Some(merged))
}

/// Set a single variable value for a skill.
///
/// Routes to the appropriate KV namespace based on the var's `storage-hint`.
pub fn set_skill_var(ship_dir: &Path, skill_id: &str, key: &str, value: Value) -> Result<()> {
    let schema = parse_vars_schema(ship_dir, skill_id)
        .ok_or_else(|| anyhow::anyhow!("skill '{}' has no assets/vars.json", skill_id))?;

    let meta = schema
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("unknown variable '{}' for skill '{}'", key, skill_id))?;

    let ctx = context_key(ship_dir);
    match meta.storage_hint {
        StorageHint::Global => {
            crate::skill_vars_store::set(kv_ns_global(skill_id), key, &value)?;
        }
        StorageHint::Local => {
            crate::skill_vars_store::set(kv_ns_local(&ctx, skill_id), key, &value)?;
        }
        StorageHint::Project => {
            crate::skill_vars_store::set(kv_ns_project(&ctx, skill_id), key, &value)?;
        }
    }
    Ok(())
}

/// Delete a single variable for a skill across whichever scope stores it.
/// Routes by the var's `storage-hint`. Returns `true` if a value was present
/// and removed, `false` if the key had no stored override (defaults remain).
pub fn delete_skill_var(ship_dir: &Path, skill_id: &str, key: &str) -> Result<bool> {
    let schema = parse_vars_schema(ship_dir, skill_id)
        .ok_or_else(|| anyhow::anyhow!("skill '{}' has no assets/vars.json", skill_id))?;
    let meta = schema
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("unknown variable '{}' for skill '{}'", key, skill_id))?;
    let ctx = context_key(ship_dir);
    let ns = match meta.storage_hint {
        StorageHint::Global => kv_ns_global(skill_id),
        StorageHint::Local => kv_ns_local(&ctx, skill_id),
        StorageHint::Project => kv_ns_project(&ctx, skill_id),
    };
    let had = crate::skill_vars_store::get(ns.clone(), key)?.is_some();
    if had {
        crate::skill_vars_store::delete(ns, key)?;
    }
    Ok(had)
}

/// Clear all stored state for a skill across all three scopes.
pub fn reset_skill_vars(ship_dir: &Path, skill_id: &str) -> Result<bool> {
    let ctx = context_key(ship_dir);
    let mut removed = false;

    for ns in [
        kv_ns_global(skill_id),
        kv_ns_local(&ctx, skill_id),
        kv_ns_project(&ctx, skill_id),
    ] {
        for k in crate::skill_vars_store::list_keys(ns.clone())? {
            crate::skill_vars_store::delete(ns.clone(), &k)?;
            removed = true;
        }
    }

    Ok(removed)
}

/// List all skills across configured skill_paths that have `assets/vars.json`, with their merged state.
pub fn list_skill_vars(ship_dir: &Path) -> Result<Vec<(String, HashMap<String, Value>)>> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    for skills_path in crate::skill_paths::read_skill_paths(ship_dir, project_root) {
        if !skills_path.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&skills_path)?.flatten() {
            let path = entry.path();
            if !path.is_dir() || !path.join("assets").join("vars.json").exists() {
                continue;
            }
            let skill_id = entry.file_name().to_string_lossy().to_string();
            if !seen.insert(skill_id.clone()) {
                continue; // first path wins
            }
            if let Ok(Some(state)) = get_skill_vars(ship_dir, &skill_id) {
                result.push((skill_id, state));
            }
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::init_project;
    use serde_json::json;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path()).unwrap();
        // Global skill vars persist to kv.db; ensure that per-scope db. (Project
        // / local vars are files under ship_dir, no db needed.)
        ship_substrate::db::ensure_kv_db_at(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    fn write_vars_json(ship_dir: &Path, skill_id: &str, content: &str) {
        let assets = ship_dir.join("skills").join(skill_id).join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        std::fs::write(assets.join("vars.json"), content).unwrap();
    }

    const VARS_JSON: &str = r#"{
        "commit_style": {
            "type": "enum",
            "default": "conventional",
            "storage-hint": "global",
            "values": ["conventional", "gitmoji"]
        },
        "verbose_mode": {
            "type": "bool",
            "default": false,
            "storage-hint": "project"
        },
        "local_override": {
            "type": "string",
            "storage-hint": "local"
        }
    }"#;

    #[test]
    fn get_skill_vars_uses_defaults() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["commit_style"], json!("conventional"));
        assert_eq!(vars["verbose_mode"], json!(false));
    }

    #[test]
    fn get_skill_vars_returns_none_without_vars_json() {
        let (_tmp, ship_dir) = setup();
        std::fs::create_dir_all(ship_dir.join("skills/no-vars")).unwrap();
        assert!(get_skill_vars(&ship_dir, "no-vars").unwrap().is_none());
    }

    #[test]
    fn set_skill_var_global_goes_to_kv() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "commit_style", json!("gitmoji")).unwrap();
        let val = crate::skill_vars_store::get(KvNamespace::SkillVarsGlobal("commit".to_string()), "commit_style")
            .unwrap()
            .unwrap();
        assert_eq!(val, json!("gitmoji"));
    }

    #[test]
    fn set_skill_var_project_goes_to_kv() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "verbose_mode", json!(true)).unwrap();
        let ctx = context_key(&ship_dir);
        let val = crate::skill_vars_store::get(kv_ns_project(&ctx, "commit"), "verbose_mode")
            .unwrap()
            .unwrap();
        assert_eq!(val, json!(true));
        // No state.json file should exist
        assert!(!ship_dir.join("state.json").exists());
    }

    #[test]
    fn set_skill_var_local_goes_to_kv() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "local_override", json!("mine")).unwrap();
        let ctx = context_key(&ship_dir);
        let val = crate::skill_vars_store::get(kv_ns_local(&ctx, "commit"), "local_override")
            .unwrap()
            .unwrap();
        assert_eq!(val, json!("mine"));
    }

    #[test]
    fn set_skill_var_unknown_key_errors() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        assert!(set_skill_var(&ship_dir, "commit", "nonexistent", json!(null)).is_err());
    }

    #[test]
    fn global_state_overlays_default() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "commit_style", json!("gitmoji")).unwrap();
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["commit_style"], json!("gitmoji"));
        assert_eq!(vars["verbose_mode"], json!(false));
    }

    #[test]
    fn project_state_overlays_default() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "verbose_mode", json!(true)).unwrap();
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["verbose_mode"], json!(true));
    }

    #[test]
    fn reset_clears_all_scopes() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        set_skill_var(&ship_dir, "commit", "commit_style", json!("gitmoji")).unwrap();
        set_skill_var(&ship_dir, "commit", "verbose_mode", json!(true)).unwrap();
        assert!(reset_skill_vars(&ship_dir, "commit").unwrap());
        let vars = get_skill_vars(&ship_dir, "commit").unwrap().unwrap();
        assert_eq!(vars["commit_style"], json!("conventional")); // back to default
        assert_eq!(vars["verbose_mode"], json!(false)); // back to default
    }

    #[test]
    fn list_skill_vars_returns_skills_with_vars_json() {
        let (_tmp, ship_dir) = setup();
        write_vars_json(&ship_dir, "commit", VARS_JSON);
        std::fs::create_dir_all(ship_dir.join("skills/no-vars")).unwrap();
        let list = list_skill_vars(&ship_dir).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "commit");
    }
}
