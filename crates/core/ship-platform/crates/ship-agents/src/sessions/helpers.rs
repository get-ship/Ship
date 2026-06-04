use anyhow::Result;
use chrono::{DateTime, Utc};

// ---- Parsing helpers -------------------------------------------------------

pub fn parse_datetime(value: &str) -> DateTime<Utc> {
    std::str::FromStr::from_str(value).unwrap_or_else(|_| Utc::now())
}

pub fn parse_datetime_opt(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|entry| std::str::FromStr::from_str(&entry).ok())
}

pub fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub fn normalize_agent_ref(agent: &str) -> Option<String> {
    let trimmed = agent.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn normalize_harness_ref(harness: &str) -> Option<String> {
    let trimmed = harness.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// ---- Branch key ------------------------------------------------------------

pub fn ensure_branch_key(branch: &str) -> Result<&str> {
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("Workspace branch/key cannot be empty"));
    }
    Ok(trimmed)
}

// ---- Space scope -----------------------------------------------------------

/// The SPACE id for a project given its `.ship` dir. This is the path-hash
/// identity of the project root (`ship_dir.parent()`), matching the `projects`
/// projection key (`ship_platform::projects::register`) and the
/// `sessions.space_id` column. The sole domain scope — never a branch slug.
/// See spec `.artifacts/specs/space/116-event-partition.md`.
pub fn space_id_for_ship_dir(ship_dir: &std::path::Path) -> String {
    let root = ship_dir.parent().unwrap_or(ship_dir);
    ship_substrate::home::path_hash(root)
}
