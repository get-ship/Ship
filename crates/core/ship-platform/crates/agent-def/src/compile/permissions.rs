//! Apply an agent's permission overrides on top of a base `stack_model::Permissions`.
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

use agent_schema::{AgentProfile, Permissions, ToolPermissions};
use std::collections::BTreeMap;

/// A named permission preset section. The struct is a re-export of
/// `stack_model::PermissionPreset` (moved from this module with the
/// slice 3 close-out — the L0 type now owns the canonical shape, and
/// the resolver's `merge_ts_declarations` harvests it directly into
/// `ExtensionLibrary::permissions_presets`). The pre-slice-3 struct
/// lived here because `.ship/permissions.jsonc` was a leaf-side
/// construct; the descriptor's move to L0 puts the type with the
/// descriptor. Re-exported here so the existing
/// `super::compile::PermissionPreset` path keeps working for any
/// remaining callers (the compile path now consumes
/// `stack_model::PermissionPreset` directly, the re-export is a
/// forward-compat surface).
pub use agent_schema::PermissionPreset;

/// Permission rules baked into every Ship agent. Cannot be overridden.
///
/// **Categories**:
/// - **Cross-machine credentials** (Read): leaks give off-machine compromise.
///   SSH/GPG keys, cloud-provider creds, package-registry tokens, password
///   stores, browser credential DBs. Never readable by any agent, any preset.
/// - **Local-only credentials** (Read): ambient secrets that don't grant
///   off-machine access but feed downstream compromise. Daemon token, MCP
///   bearer, daemon port info under `state_dir()`.
/// - **Ship state** (Write/Edit): corruption causes silent kernel
///   inconsistencies and data loss. Event store + projections under
///   `data_dir()`.
/// - **History files** (Read): commonly contain unredacted credentials.
/// - **Bypass operations** (Bash): obvious ways to skirt the above —
///   `cat`/`head`/`less` on protected paths, `curl | sh`, `sudo`, etc.
/// - **Ship dir deletion** (Bash): `rm` patterns protecting the ship home.
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
    presets: &BTreeMap<String, PermissionPreset>,
) -> Permissions {
    let mp = &agent.permissions;

    // Slice 3 close-out: the preset layer is now an in-memory
    // lookup against `ExtensionLibrary::permissions_presets`
    // (populated by the resolver from
    // `defineConfig.agents.permissions.presets`). The pre-slice-3
    // `ship_dir`-keyed disk read died with `.ship/permissions.jsonc`.
    let preset_from_file = mp.preset.as_deref().and_then(|name| presets.get(name));

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
        // Built-in fallback — no preset in the descriptor map (or no
        // `defineConfig.agents.permissions.presets` block authored).
        // The pre-slice-3 "no permissions.jsonc" case becomes "no
        // preset contributed" post slice 3; same consumer behavior
        // either way.
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
    use agent_schema::Permissions;

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
        // Empty presets map → built-in `ship-readonly` fallback (the
        // only preset with a built-in default; the other named
        // presets must be authored in
        // `defineConfig.agents.permissions.presets`).
        let result = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("ship-readonly"),
            &BTreeMap::new(),
        );
        assert!(result.tools.deny.contains(&"Write(*)".to_string()));
        assert!(result.tools.deny.contains(&"Edit(*)".to_string()));
        assert!(result.tools.allow.contains(&"Read".to_string()));
        assert!(!result.tools.allow.contains(&"Grep".to_string()));
    }

    #[test]
    fn preset_map_overrides_default_mode() {
        // Slice 3 close-out: the named-preset layer is sourced
        // from the in-memory `permissions_presets` map (populated
        // by the resolver from
        // `defineConfig.agents.permissions.presets`). The
        // pre-slice-3 disk read against `.ship/permissions.jsonc`
        // died with this slice; the test mirrors the same lookup
        // with the new shape.
        let mut presets: BTreeMap<String, PermissionPreset> = BTreeMap::new();
        presets.insert(
            "custom-preset".to_string(),
            PermissionPreset {
                default_mode: Some("bypassPermissions".to_string()),
                tools_deny: vec!["Bash(git push --force*)".to_string()],
                tools_ask: vec!["Bash(rm -rf*)".to_string()],
                ..Default::default()
            },
        );

        let result = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("custom-preset"),
            &presets,
        );
        assert_eq!(result.default_mode.as_deref(), Some("bypassPermissions"));
        assert!(
            result
                .tools
                .deny
                .contains(&"Bash(git push --force*)".to_string())
        );
        assert!(result.tools.ask.contains(&"Bash(rm -rf*)".to_string()));
    }

    fn profile_no_preset() -> AgentProfile {
        serde_json::from_str(r#"{ "agent": { "id": "x", "name": "X" } }"#).expect("parse")
    }

    #[test]
    fn floor_denies_cross_machine_credentials_for_base_agent() {
        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_no_preset(),
            &BTreeMap::new(),
        );
        assert!(r.tools.deny.contains(&"Read(~/.ssh/**)".to_string()));
        assert!(r.tools.deny.contains(&"Read(~/.aws/**)".to_string()));
        assert!(
            r.tools
                .deny
                .contains(&"Read(~/.docker/config.json)".to_string())
        );
        assert!(
            r.tools
                .deny
                .contains(&"Read(~/.cargo/credentials.toml)".to_string())
        );
    }

    #[test]
    fn floor_denies_local_only_credentials_for_base_agent() {
        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_no_preset(),
            &BTreeMap::new(),
        );
        assert!(r.tools.deny.contains(&"Read(~/.ship/state/**)".to_string()));
    }

    #[test]
    fn floor_denies_ship_state_writes() {
        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_no_preset(),
            &BTreeMap::new(),
        );
        assert!(r.tools.deny.contains(&"Write(~/.ship/data/**)".to_string()));
        assert!(r.tools.deny.contains(&"Edit(~/.ship/data/**)".to_string()));
    }

    #[test]
    fn floor_survives_readonly_preset() {
        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("ship-readonly"),
            &BTreeMap::new(),
        );
        assert!(r.tools.deny.contains(&"Read(~/.ssh/**)".to_string()));
        assert!(r.tools.deny.contains(&"Read(~/.ship/state/**)".to_string()));
    }

    #[test]
    fn floor_survives_per_agent_overrides() {
        let mut presets: BTreeMap<String, PermissionPreset> = BTreeMap::new();
        presets.insert(
            "permissive".to_string(),
            PermissionPreset {
                default_mode: Some("bypassPermissions".to_string()),
                tools_allow: vec!["Read".to_string(), "Bash(*)".to_string()],
                ..Default::default()
            },
        );

        let r = apply_agent_permissions(
            Permissions::default(),
            &profile_with_preset("permissive"),
            &presets,
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
        let once = apply_agent_permissions(
            Permissions::default(),
            &profile_no_preset(),
            &BTreeMap::new(),
        );
        let twice = apply_agent_permissions(once.clone(), &profile_no_preset(), &BTreeMap::new());
        for floor in FLOOR_DENIES {
            let count = twice.tools.deny.iter().filter(|d| d == floor).count();
            assert_eq!(count, 1, "floor entry {floor:?} duplicated to {count}");
        }
    }
}
