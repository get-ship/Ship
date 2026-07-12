//! Output writer — emits [`CompileOutput`] to the filesystem.

use anyhow::{Result, anyhow};
use std::path::Path;

use crate::compile::{CompileOutput, get_harness};
use stack_model::jsonc::{merge_json, merge_json_file};

/// Write a compiled harness output to the filesystem at `root`.
///
/// For `.mcp.json` (Claude): reads existing file, preserves
/// user-authored entries that don't carry `_ship.managed: true`, then writes
/// Ship-managed entries with the marker. All other files are written directly.
pub fn write_output(root: &Path, harness_id: &str, output: &CompileOutput) -> Result<()> {
    let desc = get_harness(harness_id).ok_or_else(|| anyhow!("unknown harness: {}", harness_id))?;

    // Context file (CLAUDE.md, AGENTS.md)
    if let (Some(content), Some(file_name)) =
        (&output.context_content, desc.context_file.file_name())
    {
        std::fs::write(root.join(file_name), content)?;
    }

    // MCP config file
    if let Some(ref mcp_path) = output.mcp_config_path {
        if harness_id == "codex" {
            // Codex: TOML patch written separately — skip JSON write here
        } else {
            // All other harnesses (Claude): merge preserving user entries
            let path = root.join(mcp_path);
            ensure_parent(&path)?;
            write_mcp_preserving_user_entries(&path, desc.mcp_key.as_str(), &output.mcp_servers)?;
        }
    }

    // Skill files
    for (rel_path, content) in &output.skill_files {
        let path = root.join(rel_path);
        ensure_parent(&path)?;
        std::fs::write(&path, content)?;
    }

    // Per-file rule outputs (no current harness emits these)
    for (rel_path, content) in &output.rule_files {
        let path = root.join(rel_path);
        ensure_parent(&path)?;
        std::fs::write(&path, content)?;
    }

    // Claude settings patch → .claude/settings.json
    if let Some(patch) = &output.claude_settings_patch {
        let path = root.join(".claude/settings.json");
        ensure_parent(&path)?;
        merge_json_file(&path, patch)?;
    }

    // Harness agent files (.claude/agents/*.md etc.)
    for (rel_path, content) in &output.agent_files {
        let path = root.join(rel_path);
        ensure_parent(&path)?;
        std::fs::write(&path, content)?;
    }

    // Codex TOML patch → .codex/config.toml
    if let Some(toml_str) = &output.codex_config_patch {
        let path = root.join(".codex/config.toml");
        ensure_parent(&path)?;
        std::fs::write(&path, toml_str)?;
    }

    // Codex hooks → .codex/hooks.json
    if let Some(hooks) = &output.codex_hooks_json {
        let path = root.join(".codex/hooks.json");
        ensure_parent(&path)?;
        std::fs::write(&path, serde_json::to_string_pretty(hooks)?)?;
    }

    // Codex execpolicy → .codex/rules/ship.rules (or delete if None)
    let rules_path = root.join(".codex/rules/ship.rules");
    match &output.codex_policy_content {
        Some(content) => {
            ensure_parent(&rules_path)?;
            std::fs::write(&rules_path, content)?;
        }
        None => {
            let _ = std::fs::remove_file(&rules_path);
        }
    }

    // OpenCode config → opencode.json
    if let Some(patch) = &output.opencode_config_patch {
        std::fs::write(
            root.join("opencode.json"),
            serde_json::to_string_pretty(patch)?,
        )?;
    }

    Ok(())
}

// ── MCP merge ─────────────────────────────────────────────────────────────────

/// Write MCP config, preserving user-authored entries and marking Ship entries.
///
/// Entries in the existing file that carry `{"_ship": {"managed": true}}` are
/// treated as Ship-owned and replaced on every write. Entries without the
/// marker are user-authored and are left untouched.
fn write_mcp_preserving_user_entries(
    path: &Path,
    mcp_key: &str,
    ship_servers: &serde_json::Value,
) -> Result<()> {
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mut merged = serde_json::Map::new();

    // Preserve user entries (those without _ship.managed: true)
    if let Some(existing) = root.get(mcp_key).and_then(|v| v.as_object()) {
        for (id, entry) in existing {
            let is_managed = entry
                .get("_ship")
                .and_then(|s| s.get("managed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !is_managed {
                merged.insert(id.clone(), entry.clone());
            }
        }
    }

    // Write Ship-managed entries with the marker
    if let Some(servers) = ship_servers.as_object() {
        for (id, entry) in servers {
            let mut tagged = entry.clone();
            if let Some(obj) = tagged.as_object_mut() {
                obj.insert("_ship".to_string(), serde_json::json!({ "managed": true }));
            }
            merged.insert(id.clone(), tagged);
        }
    }

    root["_ship"] = serde_json::json!({
        "managed": true,
        "note": "Generated by Ship. Run `ship compile` to regenerate."
    });
    root[mcp_key] = serde_json::Value::Object(merged);

    std::fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}
