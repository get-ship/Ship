//! Apply an agent's permission overrides on top of a base `compiler::Permissions`.
//!
//! Resolution order: per-agent overrides win over named-preset (from
//! `.ship/permissions.jsonc`), preset wins over base, base provides the
//! fallback. Built-in fallbacks handle `ship-readonly` even when no
//! preset file exists.

use std::path::Path;

use compiler::{AgentProfile, Permissions, ToolPermissions};

use crate::loader::load_permission_preset;

pub fn apply_agent_permissions(
    base: Permissions,
    agent: &AgentProfile,
    ship_dir: Option<&Path>,
) -> Permissions {
    let mp = &agent.permissions;

    let preset_from_file = mp
        .preset
        .as_deref()
        .and_then(|name| ship_dir.and_then(|dir| load_permission_preset(dir, name)));

    let mut tools = if let Some(ref preset) = preset_from_file {
        ToolPermissions {
            allow: if preset.tools_allow.is_empty() {
                base.tools.allow.clone()
            } else {
                preset.tools_allow.clone()
            },
            deny: {
                let mut d = base.tools.deny.clone();
                for p in &preset.tools_deny {
                    if !d.contains(p) {
                        d.push(p.clone());
                    }
                }
                d
            },
            ask: {
                let mut a = base.tools.ask.clone();
                for p in &preset.tools_ask {
                    if !a.contains(p) {
                        a.push(p.clone());
                    }
                }
                a
            },
        }
    } else {
        // Built-in fallback — no permissions.jsonc or section not found.
        match mp.preset.as_deref() {
            Some("ship-readonly") => ToolPermissions {
                allow: vec![
                    "Read".into(),
                    "Glob".into(),
                    "LS".into(),
                    "mcp__ship__*".into(),
                    "Bash(ship *)".into(),
                ],
                deny: vec!["Write(*)".into(), "Edit(*)".into(), "Bash(rm*)".into()],
                ask: vec![],
            },
            _ => base.tools.clone(),
        }
    };

    // Agent-level additions always apply on top of preset.
    for p in &mp.tools_deny {
        if !tools.deny.contains(p) {
            tools.deny.push(p.clone());
        }
    }
    for p in &mp.tools_ask {
        if !tools.ask.contains(p) {
            tools.ask.push(p.clone());
        }
    }

    // default_mode: agent field wins, then preset file value, then base.
    let default_mode = mp
        .default_mode
        .clone()
        .or_else(|| {
            preset_from_file
                .as_ref()
                .and_then(|p| p.default_mode.clone())
        })
        .or(base.default_mode);

    Permissions {
        tools,
        default_mode,
        ..base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compiler::Permissions;

    fn profile_with_preset(preset: &str) -> AgentProfile {
        let src = format!(
            r#"{{
              "agent": {{ "id": "x", "name": "X" }},
              "permissions": {{ "preset": "{preset}" }}
            }}"#
        );
        serde_json::from_str(&src).expect("parse")
    }

    #[test]
    fn readonly_restricts_writes() {
        let result = apply_agent_permissions(Permissions::default(), &profile_with_preset("ship-readonly"), None);
        assert!(result.tools.deny.contains(&"Write(*)".to_string()));
        assert!(result.tools.deny.contains(&"Edit(*)".to_string()));
        assert!(result.tools.allow.contains(&"Read".to_string()));
        // ship-readonly has a narrow allow list — Grep is not included.
        assert!(!result.tools.allow.contains(&"Grep".to_string()));
    }

    #[test]
    fn permissions_jsonc_preset_overrides_default_mode() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("permissions.jsonc"),
            r#"{
              "custom-preset": {
                "default_mode": "bypassPermissions",
                "tools_deny": ["Bash(git push --force*)"],
                "tools_ask": ["Bash(rm -rf*)"]
              }
            }"#,
        ).unwrap();

        let result = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("custom-preset"),
            Some(tmp.path()),
        );
        assert_eq!(result.default_mode.as_deref(), Some("bypassPermissions"));
        assert!(result.tools.deny.contains(&"Bash(git push --force*)".to_string()));
        assert!(result.tools.ask.contains(&"Bash(rm -rf*)".to_string()));
    }
}
