//! Apply an agent's permission overrides on top of a base `compiler::Permissions`.
//!
//! Resolution order: floor wins over everything; per-agent overrides win over
//! named-preset (from `.ship/permissions.jsonc`); preset wins over base; base
//! provides the fallback. Built-in fallbacks handle `ship-readonly` even when
//! no preset file exists.
//!
//! **The floor** ([`FLOOR_DENIES`]) is a binary-baked list of deny rules every
//! agent inherits regardless of preset, project config, or per-agent override.
//! It protects the categorical security boundaries — cross-machine credential
//! exfiltration paths, Ship state corruption, ambient-token leaks. User
//! configs can ADD restrictions but cannot REMOVE floor entries. Attempts to
//! `allow` a floor-protected pattern do not override at runtime (deny wins in
//! every harness we target) and the compiler emits a warning.

use std::path::Path;

use compiler::{AgentProfile, Permissions, ToolPermissions};
use serde::Deserialize;

use super::loader::load_permission_preset;

/// A named permission preset section from `permissions.jsonc`.
/// Matches the `"ship-standard"`, `"ship-autonomous"`, etc. keys.
#[derive(Deserialize, Default, Clone)]
pub struct PermissionPreset {
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub tools_allow: Vec<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
}

/// Permission rules baked into every Ship agent. Cannot be overridden.
///
/// **Categories**:
/// - **Cross-machine credentials** (Read): leaks give off-machine compromise.
///   SSH/GPG keys, cloud-provider creds, package-registry tokens, password
///   stores, browser credential DBs. Never readable by any agent, any preset.
/// - **Local-only credentials** (Read): ambient secrets that don't grant
///   off-machine access but feed downstream compromise. Daemon token, MCP
///   bearer, daemon port info under `~/.ship/state/`.
/// - **Ship state** (Write/Edit): corruption causes silent kernel
///   inconsistencies and data loss. Event store + projections under
///   `~/.ship/data/`.
/// - **History files** (Read): commonly contain unredacted credentials.
/// - **Bypass operations** (Bash): obvious ways to skirt the above —
///   `cat`/`head`/`less` on protected paths, `curl | sh`, `sudo`, etc.
/// - **Ship dir deletion** (Bash): `rm` patterns protecting `~/.ship/`.
pub const FLOOR_DENIES: &[&str] = &[
    // ── Cross-machine credentials — leak = off-machine compromise ──────────
    "Read(~/.ssh/**)",
    "Read(~/.gnupg/**)",
    "Read(~/.aws/**)",
    "Read(~/.azure/**)",
    "Read(~/.gcloud/**)",
    "Read(~/.config/gcloud/**)",
    "Read(~/.kube/config)",
    "Read(~/.netrc)",
    "Read(~/.git-credentials)",
    "Read(~/.config/git/credentials)",
    "Read(~/.docker/config.json)",
    "Read(~/.cargo/credentials.toml)",
    "Read(~/.npmrc)",
    "Read(~/.pypirc)",
    "Read(~/.password-store/**)",
    "Read(~/.config/Bitwarden/**)",
    "Read(~/.config/1Password/**)",
    "Read(~/Library/Keychains/**)",
    "Read(~/Library/Cookies/**)",
    "Read(~/.mozilla/firefox/*/key*.db)",
    "Read(~/.mozilla/firefox/*/logins.json)",
    "Read(~/.config/google-chrome/**/Login Data)",
    // ── Local-only credentials — ambient secrets ────────────────────────────
    "Read(~/.ship/state/**)",
    // ── Ship state — corruption is data loss / silent kernel inconsistency ─
    "Write(~/.ship/data/**)",
    "Edit(~/.ship/data/**)",
    // ── History files (often contain credentials) ───────────────────────────
    "Read(~/.bash_history)",
    "Read(~/.zsh_history)",
    "Read(~/.psql_history)",
    "Read(~/.mysql_history)",
    "Read(~/.python_history)",
    // ── Bash bypass patterns ────────────────────────────────────────────────
    "Bash(cat *~/.ship/state/*)",
    "Bash(cat *~/.ssh/*)",
    "Bash(cat *~/.aws/*)",
    "Bash(cat *~/.netrc*)",
    "Bash(head *~/.ship/state/*)",
    "Bash(less *~/.ship/state/*)",
    "Bash(tail *~/.ship/state/*)",
    "Bash(xxd *~/.ship/state/*)",
    "Bash(base64 *~/.ship/state/*)",
    "Bash(curl *|*sh)",
    "Bash(curl *|*bash)",
    "Bash(wget *|*sh)",
    "Bash(wget *|*bash)",
    "Bash(sudo:*)",
    // ── Ship dir deletion (consolidates prior _base_deny entries) ──────────
    "Bash(rm */.ship/*)",
    "Bash(rm -r */.ship/*)",
    "Bash(rm -rf */.ship/*)",
    "Bash(rm *~/.ship/*)",
    "Bash(rm -r *~/.ship/*)",
    "Bash(rm -rf *~/.ship/*)",
];

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
                    // Ship ops reach the daemon via `ship ops call` / `ship *`
                    // over the host socket — no MCP tool surface.
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

    // Floor — last writer, never removable. Even if a preset or per-agent
    // override placed a floor pattern in `allow`, the deny precedence rule
    // in every harness we target means deny wins at runtime. Compile-time
    // validation surfaces a warning so authors don't think their allow rule
    // is taking effect.
    for floor in FLOOR_DENIES {
        if tools.allow.iter().any(|a| a == floor) {
            eprintln!(
                "ship compile: warning — agent allow rule {floor:?} \
                 conflicts with Ship security floor; deny wins at runtime."
            );
        }
        if !tools.deny.iter().any(|d| d == floor) {
            tools.deny.push((*floor).to_string());
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
        let result =
            apply_agent_permissions(Permissions::default(), &profile_with_preset("ship-readonly"), None);
        assert!(result.tools.deny.contains(&"Write(*)".to_string()));
        assert!(result.tools.deny.contains(&"Edit(*)".to_string()));
        assert!(result.tools.allow.contains(&"Read".to_string()));
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
        )
        .unwrap();

        let result = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("custom-preset"),
            Some(tmp.path()),
        );
        assert_eq!(result.default_mode.as_deref(), Some("bypassPermissions"));
        assert!(result.tools.deny.contains(&"Bash(git push --force*)".to_string()));
        assert!(result.tools.ask.contains(&"Bash(rm -rf*)".to_string()));
    }

    fn profile_no_preset() -> AgentProfile {
        serde_json::from_str(r#"{ "agent": { "id": "x", "name": "X" } }"#).expect("parse")
    }

    #[test]
    fn floor_denies_cross_machine_credentials_for_base_agent() {
        let r = apply_agent_permissions(Permissions::default(), &profile_no_preset(), None);
        assert!(r.tools.deny.contains(&"Read(~/.ssh/**)".to_string()));
        assert!(r.tools.deny.contains(&"Read(~/.aws/**)".to_string()));
        assert!(r.tools.deny.contains(&"Read(~/.docker/config.json)".to_string()));
        assert!(r.tools.deny.contains(&"Read(~/.cargo/credentials.toml)".to_string()));
    }

    #[test]
    fn floor_denies_local_only_credentials_for_base_agent() {
        let r = apply_agent_permissions(Permissions::default(), &profile_no_preset(), None);
        assert!(r.tools.deny.contains(&"Read(~/.ship/state/**)".to_string()));
    }

    #[test]
    fn floor_denies_ship_state_writes() {
        let r = apply_agent_permissions(Permissions::default(), &profile_no_preset(), None);
        assert!(r.tools.deny.contains(&"Write(~/.ship/data/**)".to_string()));
        assert!(r.tools.deny.contains(&"Edit(~/.ship/data/**)".to_string()));
    }

    #[test]
    fn floor_survives_readonly_preset() {
        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("ship-readonly"),
            None,
        );
        assert!(r.tools.deny.contains(&"Read(~/.ssh/**)".to_string()));
        assert!(r.tools.deny.contains(&"Read(~/.ship/state/**)".to_string()));
    }

    #[test]
    fn floor_survives_per_agent_overrides() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("permissions.jsonc"),
            r#"{
              "permissive": {
                "default_mode": "bypassPermissions",
                "tools_allow": ["Read", "Bash(*)"],
                "tools_deny": []
              }
            }"#,
        )
        .unwrap();

        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("permissive"),
            Some(tmp.path()),
        );
        for floor in FLOOR_DENIES {
            assert!(
                r.tools.deny.iter().any(|d| d == floor),
                "floor entry {floor:?} missing from compiled deny list"
            );
        }
    }

    #[test]
    fn floor_is_idempotent_across_calls() {
        let once = apply_agent_permissions(Permissions::default(), &profile_no_preset(), None);
        let twice = apply_agent_permissions(once.clone(), &profile_no_preset(), None);
        for floor in FLOOR_DENIES {
            let count = twice.tools.deny.iter().filter(|d| *d == floor).count();
            assert_eq!(count, 1, "floor entry {floor:?} duplicated to {count}");
        }
    }
}
