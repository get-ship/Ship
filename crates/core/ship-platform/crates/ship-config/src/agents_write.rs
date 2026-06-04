//! The gated agent-def WRITE path (S5 door).
//!
//! Authoring `.ship/agents/<id>.jsonc` through the [`FsBroker`] gated door —
//! confined by the SAME [`FsCap`] the reader uses ([`crate::agents`]). The save
//! preserves the hand-authored canonical fields (`hooks`, `plugins`,
//! `rules.inline`, `harness_settings`, `apps`) on re-save.
//!
//! This is the not-yet-wired write surface the gated op (S5) drives; the read
//! cutover (S2) left it here, untouched, when it split the reader out.

use std::collections::HashSet;
use std::path::Path;

use ship_substrate::ports::{FsBroker, FsMode};

use crate::agents::{agent_rel, agents_cap};
use crate::error::ConfigError;
use crate::types::AgentProfile;

/// Strip the optional `NNNN-` numeric prefix and `.md` suffix from a rule
/// reference so callers see canonical IDs (`rust` rather than `0001-rust.md`).
pub fn normalize_rule_external_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

/// Write each profile to `<id>.jsonc`, pruning files for agents no longer in the
/// desired set. Preserves hand-authored canonical fields on re-save. Writes via
/// [`FsBroker`] — never `std::fs`.
pub fn save_agents_config(
    fs: &dyn FsBroker,
    ship_dir: &Path,
    agents: &[AgentProfile],
) -> Result<(), ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadWrite);
    fs.create_dir_all(&cap, &agents_dir)?;

    let keep: HashSet<String> = agents.iter().map(|a| a.id.clone()).collect();

    // Prune files for agents no longer in the desired set.
    let entries = match fs.list_dir(&cap, &agents_dir) {
        Ok(e) => e,
        Err(ship_substrate::ports::FsError::NotFound) => Vec::new(),
        Err(e) => return Err(e.into()),
    };
    for entry in entries {
        if entry.extension().and_then(|e| e.to_str()) != Some("jsonc") {
            continue;
        }
        let Some(id) = entry.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !keep.contains(id) {
            fs.remove(&cap, &agent_rel(&agents_dir, id))?;
        }
    }

    for agent in agents {
        let mut profile = agent.clone();
        profile.rules = profile
            .rules
            .iter()
            .map(|id| normalize_rule_external_id(id))
            .collect();
        let rel = agent_rel(&agents_dir, &agent.id);
        let existing = fs.read_string(&cap, &rel).ok();
        let content = render_agent_file(existing.as_deref(), &profile)?;
        fs.write_atomic(&cap, &rel, content.as_bytes())?;
    }
    Ok(())
}

/// Render an agent profile to JSONC. When `existing` already holds a canonical
/// nested-shape file, the platform-managed fields are merged in and the
/// remaining hand-authored content (`hooks`, `plugins`, `rules.inline`,
/// `harness_settings`, `apps`, schema reference) is preserved. Otherwise the
/// platform-managed fields are emitted as a minimal canonical nested file.
fn render_agent_file(existing: Option<&str>, flat: &AgentProfile) -> Result<String, ConfigError> {
    let parsed: Option<serde_json::Value> = existing
        .map(ship_substrate::fs_util::strip_jsonc_comments)
        .and_then(|stripped| serde_json::from_str(&stripped).ok());

    let canonical_doc = parsed.filter(|v| v.get("agent").is_some());

    if let Some(mut doc) = canonical_doc {
        merge_flat_into_canonical(&mut doc, flat);
        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| ConfigError::Internal(e.to_string()))?;
        Ok(format!("{json}\n"))
    } else {
        let mut agent_block = serde_json::json!({ "id": flat.id, "name": flat.name });
        set_meta_fields(&mut agent_block, flat);
        let canonical = serde_json::json!({
            "agent": agent_block,
            "skills": { "refs": flat.skills },
            "mcp": { "servers": flat.mcp_servers },
        });
        let json = serde_json::to_string_pretty(&canonical)
            .map_err(|e| ConfigError::Internal(e.to_string()))?;
        Ok(format!("{json}\n"))
    }
}

/// Set the optional/list meta fields on an `agent` JSON block from the flat view.
fn set_meta_fields(block: &mut serde_json::Value, flat: &AgentProfile) {
    let Some(a) = block.as_object_mut() else {
        return;
    };
    if let Some(desc) = &flat.description {
        a.insert("description".into(), serde_json::Value::String(desc.clone()));
    }
    if let Some(role) = &flat.role {
        a.insert("role".into(), serde_json::Value::String(role.clone()));
    }
    if let Some(preset) = &flat.tools_preset {
        a.insert("tools_preset".into(), serde_json::Value::String(preset.clone()));
    }
    if !flat.active_tools.is_empty() {
        a.insert("active_tools".into(), to_str_array(&flat.active_tools));
    }
    if !flat.target_agents.is_empty() {
        a.insert("harnesses".into(), to_str_array(&flat.target_agents));
    }
}

/// Overlay the platform-managed fields of a flat [`AgentProfile`] onto a
/// canonical nested JSONC document. Fields the platform does not own (`hooks`,
/// `plugins`, `rules.inline`, `harness_settings`, `apps`) are left untouched.
fn merge_flat_into_canonical(doc: &mut serde_json::Value, flat: &AgentProfile) {
    let Some(root) = doc.as_object_mut() else {
        return;
    };
    let agent_obj = root
        .entry("agent".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(a) = agent_obj.as_object_mut() {
        a.insert("id".into(), serde_json::Value::String(flat.id.clone()));
        a.insert("name".into(), serde_json::Value::String(flat.name.clone()));
    }
    set_meta_fields(agent_obj, flat);

    let skills_obj = root
        .entry("skills".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(s) = skills_obj.as_object_mut() {
        s.insert("refs".into(), to_str_array(&flat.skills));
    }

    let mcp_obj = root
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(m) = mcp_obj.as_object_mut() {
        m.insert("servers".into(), to_str_array(&flat.mcp_servers));
    }
}

fn to_str_array(items: &[String]) -> serde_json::Value {
    serde_json::Value::Array(items.iter().cloned().map(serde_json::Value::String).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::read_agent_profiles;
    use ship_substrate::ports::LocalFsBroker;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// Returns `ship_dir` for direct broker writes in tests. Canonicalizes the
    /// tempdir root: on WSL/macOS `/tmp` is a symlink, and the broker's
    /// symlink-escape guard canonicalizes the deepest existing ancestor before
    /// comparing it to the cap root. Production ship dirs are already canonical.
    fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
        let base = std::fs::canonicalize(tmp.path()).unwrap();
        ship_substrate::home::ShipHome::project_ship_dir(base)
    }

    #[test]
    fn normalize_strips_nnnn_prefix_and_md_suffix() {
        assert_eq!(normalize_rule_external_id("0001-rust.md"), "rust");
        assert_eq!(normalize_rule_external_id("rust"), "rust");
        assert_eq!(normalize_rule_external_id("rust.md"), "rust");
        // Non-numeric prefix is preserved.
        assert_eq!(normalize_rule_external_id("ab-cd"), "ab-cd");
    }

    #[test]
    fn save_agents_config_prunes_dropped_agents() {
        let tmp = tempdir().unwrap();
        let dir = ship_dir(&tmp);
        let fs = LocalFsBroker::new();

        let a = AgentProfile { id: "alpha".into(), name: "Alpha".into(), ..Default::default() };
        let b = AgentProfile { id: "beta".into(), name: "Beta".into(), ..Default::default() };
        save_agents_config(&fs, &dir, &[a.clone(), b.clone()]).unwrap();
        // Read back through THE reader — the save wrote canonical-shape files.
        assert_eq!(read_agent_profiles(&fs, &dir).unwrap().len(), 2);

        // Re-save with only alpha — beta's file must be pruned.
        save_agents_config(&fs, &dir, &[a]).unwrap();
        let after = read_agent_profiles(&fs, &dir).unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].agent.id, "alpha");
    }

    #[test]
    fn save_preserves_hand_authored_canonical_fields_on_resave() {
        let tmp = tempdir().unwrap();
        let dir = ship_dir(&tmp);
        let fs = LocalFsBroker::new();
        let (cap, agents_dir) = agents_cap(&dir, FsMode::ReadWrite);
        fs.create_dir_all(&cap, &agents_dir).unwrap();

        // Hand-authored canonical file with fields the platform does NOT own.
        let authored = serde_json::json!({
            "agent": { "id": "alpha", "name": "Alpha" },
            "skills": { "refs": [] },
            "mcp": { "servers": [] },
            "hooks": { "stop": "echo bye" },
            "rules": { "inline": "be careful" },
            "harness_settings": { "claude": { "model": "opus" } },
            "apps": { "studio": true }
        });
        fs.write_atomic(&cap, &agent_rel(&agents_dir, "alpha"), authored.to_string().as_bytes())
            .unwrap();

        // Re-save the platform-managed view (changes skills only).
        let profile = AgentProfile {
            id: "alpha".into(),
            name: "Alpha".into(),
            skills: vec!["rust".into()],
            ..Default::default()
        };
        save_agents_config(&fs, &dir, &[profile]).unwrap();

        // The hand-authored fields must survive the re-save.
        let raw = fs.read_string(&cap, &agent_rel(&agents_dir, "alpha")).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(doc["hooks"]["stop"], "echo bye", "hooks preserved");
        assert_eq!(doc["rules"]["inline"], "be careful", "rules.inline preserved");
        assert_eq!(doc["harness_settings"]["claude"]["model"], "opus", "harness_settings preserved");
        assert_eq!(doc["apps"]["studio"], true, "apps preserved");
        // And the platform-managed change landed.
        assert_eq!(doc["skills"]["refs"][0], "rust", "platform field updated");
    }

    #[test]
    fn save_normalizes_rule_ids_on_disk() {
        let tmp = tempdir().unwrap();
        let dir = ship_dir(&tmp);
        let fs = LocalFsBroker::new();
        let profile = AgentProfile {
            id: "alpha".into(),
            name: "Alpha".into(),
            rules: vec!["0001-rust.md".into()],
            ..Default::default()
        };
        // The save path normalizes rule ids; the file must round-trip cleanly.
        save_agents_config(&fs, &dir, &[profile]).unwrap();
        let agents = read_agent_profiles(&fs, &dir).unwrap();
        assert_eq!(agents.len(), 1);
    }
}
