//! `agents.templates()` / `agents.permission_presets()` / `agents.harnesses()`
//! — catalogs for the create-agent wizard.
//!
//! Today these are builtin lists. When they become user-configurable
//! they'll move onto disk; the shape stays the same.

use serde::Serialize;

/// A starter template for the create-wizard. Pre-wires skills + MCP +
/// a permission preset so a user can produce a working agent fast.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AgentTemplate {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Avatar glyph initial for the preview tile.
    pub letter: &'static str,
    /// Conic-gradient hue key (matches `.hue-<name>` in the view CSS).
    pub hue: &'static str,
    pub skill_ids: Vec<&'static str>,
    pub permission_preset: &'static str,
}

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PermissionPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

pub fn builtin_templates() -> Vec<AgentTemplate> {
    vec![
        AgentTemplate { id: "blank",        label: "Blank",         description: "Start from nothing. Just an identity.",         letter: "?", hue: "slate",  skill_ids: vec![],                                      permission_preset: "standard" },
        AgentTemplate { id: "lane",         label: "Language Lane", description: "Narrow agent scoped to one stack. Sandbox preset.", letter: "L", hue: "red",    skill_ids: vec!["rust-compile", "tdd", "git-hygiene"],  permission_preset: "sandbox"  },
        AgentTemplate { id: "reviewer",     label: "Reviewer",      description: "Read-only critic for PRs and diffs.",           letter: "R", hue: "amber",  skill_ids: vec!["diff-review", "a11y-audit"],          permission_preset: "read-only" },
        AgentTemplate { id: "orchestrator", label: "Orchestrator",  description: "Routes work to specialist agents.",             letter: "O", hue: "violet", skill_ids: vec!["cmdr", "spawn-agent", "ship-agents"], permission_preset: "standard" },
        AgentTemplate { id: "writer",       label: "Test Writer",   description: "TDD loop — writes failing tests first.",        letter: "T", hue: "teal",   skill_ids: vec!["tdd", "rust-compile", "git-hygiene"], permission_preset: "standard" },
        AgentTemplate { id: "investigator", label: "Investigator",  description: "Reads, searches, summarises. Never writes.",    letter: "I", hue: "blue",   skill_ids: vec!["docs-first", "a11y-audit"],          permission_preset: "read-only" },
    ]
}

pub fn builtin_permission_presets() -> Vec<PermissionPreset> {
    vec![
        PermissionPreset { id: "standard",   label: "Standard",   description: "Human confirms risky actions before they run." },
        PermissionPreset { id: "autonomous", label: "Autonomous", description: "Agent acts end-to-end; safety rails still apply." },
        PermissionPreset { id: "read-only",  label: "Read-only",  description: "Agent inspects and reports. No writes." },
        PermissionPreset { id: "sandbox",    label: "Sandbox",    description: "Allow anything inside a worktree; block outside." },
    ]
}

pub fn builtin_harnesses() -> Vec<String> {
    vec!["claude", "codex", "cursor", "opencode"]
        .into_iter()
        .map(String::from)
        .collect()
}

/// Full create-wizard payload. Registered as `agents.create_ctx` so the
/// file-routed `/agents/create` template has one call rather than three.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AgentsCreateCtx {
    pub templates: Vec<AgentTemplate>,
    pub permission_presets: Vec<PermissionPreset>,
    pub available_harnesses: Vec<String>,
}

pub fn build_create_ctx() -> AgentsCreateCtx {
    AgentsCreateCtx {
        templates: builtin_templates(),
        permission_presets: builtin_permission_presets(),
        available_harnesses: builtin_harnesses(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_have_stable_ids() {
        let t = builtin_templates();
        assert!(t.iter().any(|x| x.id == "blank"));
        assert!(t.iter().any(|x| x.id == "orchestrator"));
    }

    #[test]
    fn presets_cover_four_tiers() {
        let p = builtin_permission_presets();
        assert_eq!(p.len(), 4);
    }

    #[test]
    fn harnesses_include_claude() {
        assert!(builtin_harnesses().contains(&"claude".to_string()));
    }
}
