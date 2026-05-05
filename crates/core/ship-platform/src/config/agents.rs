use super::project::AgentProfile;
use anyhow::{Context, Result};
use ship_substrate::fs_util::strip_jsonc_comments;
use ship_substrate::project::agents_ns;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Strip the optional `NNNN-` numeric prefix and `.md` suffix from a rule
/// reference so callers see canonical IDs (`rust` rather than `0001-rust.md`).
fn normalize_rule_external_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

fn agent_file(ship_dir: &Path, id: &str) -> std::path::PathBuf {
    agents_ns(ship_dir).join(format!("{id}.jsonc"))
}

pub(super) fn get_agents_config(ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    let dir = agents_ns(ship_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut agents = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("jsonc") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let stripped = strip_jsonc_comments(&raw);
        let profile: AgentProfile = serde_json::from_str(&stripped)
            .with_context(|| format!("parse agent profile {}", path.display()))?;
        agents.push(profile);
    }
    agents.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(agents)
}

pub(super) fn save_agents_config(ship_dir: &Path, agents: &[AgentProfile]) -> Result<()> {
    let dir = agents_ns(ship_dir);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let keep: HashSet<String> = agents.iter().map(|a| a.id.clone()).collect();

    // Drop files for agents no longer in the desired set.
    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonc") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !keep.contains(id) {
                fs::remove_file(&path)
                    .with_context(|| format!("remove {}", path.display()))?;
            }
        }
    }

    for agent in agents {
        let mut profile = agent.clone();
        profile.rules = profile
            .rules
            .iter()
            .map(|id| normalize_rule_external_id(id))
            .collect();
        let json = serde_json::to_string_pretty(&profile)?;
        let path = agent_file(ship_dir, &agent.id);
        fs::write(&path, format!("{json}\n"))
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

/// One-shot migration from the legacy `runtime::*` KV namespace into
/// `<ship_dir>/agents/<id>.jsonc` files. Reads the legacy `agents` row
/// from KV, decodes the row format used before file-canonical reads,
/// writes one `.jsonc` per agent profile, and deletes the legacy KV row.
/// Idempotent: re-running on a drained install is a no-op.
pub fn drain_legacy_kv() -> Result<usize> {
    use ship_substrate::db::{block_on, ensure_kv_db, open_kv_db};
    use sqlx::Row;

    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT namespace, value_json FROM kv_state \
             WHERE namespace LIKE 'runtime::%' AND key = 'agents'",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    if rows.is_empty() {
        return Ok(0);
    }

    let mut migrated = 0usize;
    for row in &rows {
        let namespace: String = row.get(0);
        let json_str: String = row.get(1);
        // Recover the per-project ship_dir from the namespace string —
        // legacy shape was `format!("runtime::{}", ship_dir.display())`.
        let Some(ship_dir_str) = namespace.strip_prefix("runtime::") else {
            continue;
        };
        let ship_dir = Path::new(ship_dir_str);
        let Ok(legacy_rows) = serde_json::from_str::<Vec<LegacyAgentRow>>(&json_str) else {
            continue;
        };
        let profiles: Vec<AgentProfile> = legacy_rows
            .into_iter()
            .map(LegacyAgentRow::into_profile)
            .collect();
        if save_agents_config(ship_dir, &profiles).is_ok() {
            migrated += profiles.len();
        }
    }

    block_on(async {
        sqlx::query(
            "DELETE FROM kv_state WHERE namespace LIKE 'runtime::%' AND key = 'agents'",
        )
        .execute(&mut conn)
        .await
    })?;
    Ok(migrated)
}

/// Legacy KV row shape — kept here only to deserialize during the
/// one-shot drain. Do not reference from new code.
#[derive(serde::Deserialize)]
struct LegacyAgentRow {
    id: String,
    name: String,
    description: Option<String>,
    role: Option<String>,
    #[serde(default)]
    tools_preset: Option<String>,
    active_tools_json: String,
    mcp_refs_json: String,
    skill_refs_json: String,
    rule_refs_json: String,
    prompt_id: Option<String>,
    hooks_json: String,
    permissions_json: String,
    target_agents_json: String,
}

impl LegacyAgentRow {
    fn into_profile(self) -> AgentProfile {
        AgentProfile {
            id: self.id,
            name: self.name,
            description: self.description,
            role: self.role,
            tools_preset: self.tools_preset,
            active_tools: serde_json::from_str(&self.active_tools_json).unwrap_or_default(),
            mcp_servers: serde_json::from_str(&self.mcp_refs_json).unwrap_or_default(),
            skills: serde_json::from_str(&self.skill_refs_json).unwrap_or_default(),
            rules: serde_json::from_str(&self.rule_refs_json).unwrap_or_default(),
            prompt_id: self.prompt_id,
            hooks: serde_json::from_str(&self.hooks_json).unwrap_or_default(),
            permissions: serde_json::from_str(&self.permissions_json).unwrap_or_default(),
            target_agents: serde_json::from_str(&self.target_agents_json).unwrap_or_default(),
        }
    }
}
