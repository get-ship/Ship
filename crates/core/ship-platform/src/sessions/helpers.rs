use anyhow::Result;
use chrono::{DateTime, Utc};

// ---- Parsing helpers -------------------------------------------------------

pub(crate) fn parse_datetime(value: &str) -> DateTime<Utc> {
    std::str::FromStr::from_str(value).unwrap_or_else(|_| Utc::now())
}

pub(crate) fn parse_datetime_opt(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|entry| std::str::FromStr::from_str(&entry).ok())
}

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn normalize_agent_ref(agent: &str) -> Option<String> {
    let trimmed = agent.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn normalize_harness_ref(harness: &str) -> Option<String> {
    let trimmed = harness.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// ---- Branch key ------------------------------------------------------------

pub(crate) fn ensure_branch_key(branch: &str) -> Result<&str> {
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("Workspace branch/key cannot be empty"));
    }
    Ok(trimmed)
}

