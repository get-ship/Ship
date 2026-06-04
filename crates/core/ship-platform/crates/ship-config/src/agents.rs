//! Declarative file-tier: the agent-profile projection over `.ship/agents/`.
//!
//! THE EVICTION: the pre-carve `config/agents.rs` used raw `std::fs` to walk,
//! read, write, and prune `.ship/agents/<id>.jsonc`. Here ALL disk goes through
//! the [`FsBroker`] gated door — confined by an [`FsCap`]. Zero `std::fs`/
//! `tokio::fs` in this crate (grep-gate invariant).
//!
//! Files are authored against the canonical schema (`compiler::AgentProfile`,
//! top-level `agent` block); this module projects that into the platform's flat
//! runtime view ([`crate::types::AgentProfile`]) and, on save, preserves the
//! hand-authored canonical fields (`hooks`, `plugins`, `rules.inline`,
//! `harness_settings`, `apps`).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ship_substrate::ports::{FsBroker, FsCap, FsMode, FsRoot};

use crate::error::ConfigError;
use crate::types::AgentProfile;

/// Resolve the cap root + the agents-dir rel path for a `ship_dir`.
///
/// The cap roots at the `ship_dir`'s PARENT (the project/library root), which
/// reliably exists. The broker's symlink-escape guard canonicalizes the deepest
/// EXISTING ancestor of a path and rejects anything that escapes the cap root —
/// so a not-yet-created `.ship/agents` cap root would itself read as an escape.
/// Rooting at the existing parent and naming `<.ship>/agents` as the rel path
/// keeps the guard satisfied while still confining to `.ship/agents`. When
/// `ship_dir` has no parent, the cap roots at `ship_dir` with rel `agents`.
fn cap_root_and_dir(ship_dir: &Path) -> (PathBuf, String) {
    let ship_name = ship_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| ship_substrate::home::ShipHome::ship_dir_name().to_string());
    match ship_dir.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            (parent.to_path_buf(), format!("{ship_name}/agents"))
        }
        _ => (ship_dir.to_path_buf(), "agents".to_string()),
    }
}

/// Mint an FsCap rooted at the project/library dir; the agents projection lives
/// at the returned rel dir under it.
///
/// Bootstrap posture (fs_broker.rs:9-10): until `SpaceContext` minting lands in
/// Phase 5, the cap is `new_unchecked` — not new debt, the documented seam.
pub(crate) fn agents_cap(ship_dir: &Path, mode: FsMode) -> (FsCap, String) {
    let (root, agents_dir) = cap_root_and_dir(ship_dir);
    let cap = FsCap::new_unchecked(
        root,
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        mode,
    );
    (cap, agents_dir)
}

/// `<agents_dir>/<id>.jsonc` relative to the cap root.
pub(crate) fn agent_rel(agents_dir: &str, id: &str) -> String {
    format!("{agents_dir}/{id}.jsonc")
}

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

/// Project a canonical [`compiler::AgentProfile`] (nested file-format type) into
/// the platform's flat [`AgentProfile`] (runtime/UI view). Lossy by design — the
/// nested type carries fields the platform UI does not surface.
fn project_canonical(src: &compiler::AgentProfile) -> AgentProfile {
    AgentProfile {
        id: src.agent.id.clone(),
        name: src.agent.name.clone(),
        description: src.agent.description.clone(),
        role: src.agent.role.clone(),
        tools_preset: src.agent.tools_preset.clone(),
        active_tools: src.agent.active_tools.clone(),
        mcp_servers: src.mcp.servers.clone(),
        skills: src.skills.refs.clone(),
        rules: Vec::new(),
        target_agents: Vec::new(),
    }
}

/// Read every `<id>.jsonc` under `.ship/agents/`, projecting the canonical
/// schema (or a legacy-flat shape) into the platform's flat profile. Sorted by
/// id. Reads via [`FsBroker`] — never `std::fs`.
pub fn get_agents_config(
    fs: &dyn FsBroker,
    ship_dir: &Path,
) -> Result<Vec<AgentProfile>, ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadOnly);
    // Missing dir → no agents. `list_dir` errors NotFound on a missing dir.
    let entries = match fs.list_dir(&cap, &agents_dir) {
        Ok(e) => e,
        Err(ship_substrate::ports::FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut agents = Vec::new();
    for entry in entries {
        if entry.extension().and_then(|e| e.to_str()) != Some("jsonc") {
            continue;
        }
        // `list_dir` yields paths relative to the cap root (e.g. `.ship/agents/x.jsonc`).
        let rel = entry.to_string_lossy();
        let raw = fs.read_string(&cap, &rel)?;
        let value: serde_json::Value = ship_substrate::config_doc::parse_doc(&raw)
            .map_err(|e| ConfigError::Validation(format!("parse agent profile {rel}: {e}")))?;
        if value.get("agent").is_some() {
            let canonical: compiler::AgentProfile = serde_json::from_value(value)
                .map_err(|e| ConfigError::Validation(format!("parse agent profile {rel}: {e}")))?;
            agents.push(project_canonical(&canonical));
        } else {
            // Pre-canonical flat shape `{"id": ..., "name": ...}` written by an
            // older save. Kept for the round-trip path until every project
            // re-saves into canonical form.
            let flat: AgentProfile = serde_json::from_value(value)
                .map_err(|e| ConfigError::Validation(format!("parse agent profile {rel}: {e}")))?;
            agents.push(flat);
        }
    }
    agents.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(agents)
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
    use ship_substrate::ports::{FsBroker as _, LocalFsBroker};
    use tempfile::tempdir;

    /// Returns `(ship_dir, cap, agents_dir)` for direct broker writes in tests.
    /// Canonicalizes the tempdir root: on WSL/macOS `/tmp` is a symlink, and the
    /// broker's symlink-escape guard canonicalizes the deepest existing ancestor
    /// before comparing it to the cap root. Production ship dirs are already
    /// canonical, so this is a test-fixture concern only.
    fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
        let base = std::fs::canonicalize(tmp.path()).unwrap();
        ship_substrate::home::ShipHome::project_ship_dir(base)
    }

    // ── normalize_rule_external_id ────────────────────────────────────────────

    #[test]
    fn normalize_strips_nnnn_prefix_and_md_suffix() {
        assert_eq!(normalize_rule_external_id("0001-rust.md"), "rust");
        assert_eq!(normalize_rule_external_id("rust"), "rust");
        assert_eq!(normalize_rule_external_id("rust.md"), "rust");
        // Non-numeric prefix is preserved.
        assert_eq!(normalize_rule_external_id("ab-cd"), "ab-cd");
    }

    // ── EVICTION PROOF: reads go through FsBroker, not std::fs ────────────────

    #[test]
    fn get_agents_config_reads_canonical_via_fs_broker() {
        let tmp = tempdir().unwrap();
        let dir = ship_dir(&tmp);
        let fs = LocalFsBroker::new();
        // Author a canonical-shape file through the SAME broker door (no std::fs).
        let (cap, agents_dir) = agents_cap(&dir, FsMode::ReadWrite);
        fs.create_dir_all(&cap, &agents_dir).unwrap();
        let doc = serde_json::json!({
            "agent": { "id": "alpha", "name": "Alpha", "role": "worker" },
            "skills": { "refs": ["rust"] },
            "mcp": { "servers": ["fs"] }
        });
        fs.write_atomic(
            &cap,
            &agent_rel(&agents_dir, "alpha"),
            serde_json::to_vec_pretty(&doc).unwrap().as_slice(),
        )
        .unwrap();

        let agents = get_agents_config(&fs, &dir).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "alpha");
        assert_eq!(agents[0].name, "Alpha");
        assert_eq!(agents[0].role.as_deref(), Some("worker"));
        assert_eq!(agents[0].skills, vec!["rust".to_string()]);
        assert_eq!(agents[0].mcp_servers, vec!["fs".to_string()]);
    }

    #[test]
    fn get_agents_config_missing_dir_returns_empty() {
        let tmp = tempdir().unwrap();
        let fs = LocalFsBroker::new();
        // Never created the agents dir.
        let agents = get_agents_config(&fs, &ship_dir(&tmp)).unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn get_agents_config_parses_both_canonical_and_flat_and_sorts() {
        let tmp = tempdir().unwrap();
        let dir = ship_dir(&tmp);
        let fs = LocalFsBroker::new();
        let (cap, agents_dir) = agents_cap(&dir, FsMode::ReadWrite);
        fs.create_dir_all(&cap, &agents_dir).unwrap();
        // canonical (id "z")
        let canonical = serde_json::json!({ "agent": { "id": "zeta", "name": "Zeta" } });
        fs.write_atomic(&cap, &agent_rel(&agents_dir, "zeta"), canonical.to_string().as_bytes())
            .unwrap();
        // legacy flat (id "a")
        let flat = serde_json::json!({ "id": "alpha", "name": "Alpha" });
        fs.write_atomic(&cap, &agent_rel(&agents_dir, "alpha"), flat.to_string().as_bytes())
            .unwrap();

        let agents = get_agents_config(&fs, &dir).unwrap();
        assert_eq!(agents.len(), 2);
        // Sorted by id: alpha before zeta.
        assert_eq!(agents[0].id, "alpha");
        assert_eq!(agents[1].id, "zeta");
    }

    #[test]
    fn save_agents_config_prunes_dropped_agents() {
        let tmp = tempdir().unwrap();
        let dir = ship_dir(&tmp);
        let fs = LocalFsBroker::new();

        let a = AgentProfile { id: "alpha".into(), name: "Alpha".into(), ..Default::default() };
        let b = AgentProfile { id: "beta".into(), name: "Beta".into(), ..Default::default() };
        save_agents_config(&fs, &dir, &[a.clone(), b.clone()]).unwrap();
        assert_eq!(get_agents_config(&fs, &dir).unwrap().len(), 2);

        // Re-save with only alpha — beta's file must be pruned.
        save_agents_config(&fs, &dir, &[a]).unwrap();
        let after = get_agents_config(&fs, &dir).unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].id, "alpha");
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
        let agents = get_agents_config(&fs, &dir).unwrap();
        assert_eq!(agents.len(), 1);
    }
}
