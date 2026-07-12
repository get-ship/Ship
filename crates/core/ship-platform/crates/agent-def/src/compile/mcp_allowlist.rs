//! MCP tool pre-approval for each harness.
//!
//! After compilation, pre-approves the ship-mcp server tools in each harness's
//! global or project-local settings so the user is not prompted on every
//! session start.

use anyhow::{Context, Result};
use harness_adaptors::ensure_parent;

// ── Generic JSON helpers ─────────────────────────────────────────────────────

/// Ensure a JSON pattern is in an array at the given JSON pointer path.
/// Creates intermediate objects if needed. Returns Ok(false) if already present.
pub fn ensure_json_array_entry(
    settings: &mut serde_json::Value,
    pointer_parts: &[&str],
    pattern: &str,
) -> Result<bool> {
    let pointer = format!("/{}", pointer_parts.join("/"));
    let already_present = settings
        .pointer(&pointer)
        .and_then(|v| v.as_array())
        .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(pattern)));
    if already_present {
        return Ok(false);
    }

    let mut current = settings.as_object_mut().context("root must be object")?;
    for &part in &pointer_parts[..pointer_parts.len() - 1] {
        let entry = current.entry(part).or_insert(serde_json::json!({}));
        current = entry
            .as_object_mut()
            .context("intermediate must be object")?;
    }
    let last = pointer_parts.last().context("empty path")?;
    let arr_val = current.entry(*last).or_insert(serde_json::json!([]));
    let arr = arr_val.as_array_mut().context("leaf must be array")?;
    arr.push(serde_json::json!(pattern));
    Ok(true)
}

/// Read-modify-write a JSON settings file. Calls `mutate` to apply changes.
/// Creates the file and parent dirs if they don't exist.
fn update_json_settings(
    path: &std::path::Path,
    mutate: impl FnOnce(&mut serde_json::Value) -> Result<bool>,
) -> Result<()> {
    ensure_parent(path)?;
    let mut settings: serde_json::Value = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if mutate(&mut settings)? {
        std::fs::write(path, serde_json::to_string_pretty(&settings)?)?;
    }
    Ok(())
}

// ── Harness-specific allow-listing ──────────────────────────────────────────

/// Claude: write `mcp__ship__*` to `~/.claude/settings.json` permissions.allow.
pub fn ensure_ship_mcp_allowed_claude() -> Result<()> {
    let path = home::home_dir()
        .context("could not determine home directory")?
        .join(".claude")
        .join("settings.json");
    update_json_settings(&path, |s| {
        ensure_json_array_entry(s, &["permissions", "allow"], "mcp__ship__*")
    })
}
