//! `agents.settings(id)` — panel-layout detail payload.
//!
//! The settings panel is the pre-wire detail view: agent doc + canonical
//! permission tiers (as a radio-card triad) + catalogs for the picker
//! dialog, filtered to exclude what's already attached.

use serde::Serialize;
use std::path::Path;

use super::view_state::{AgentCatalogs, AgentDetail, list_agent_catalogs, load_agent};

#[derive(Serialize, Clone)]
pub struct TierRow {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub selected: bool,
}

pub fn build_tiers(selected: &str) -> Vec<TierRow> {
    [
        (
            "standard",
            "Standard",
            "Human confirms risky actions before they run.",
        ),
        (
            "autonomous",
            "Autonomous",
            "Agent acts end-to-end; safety rails still apply.",
        ),
        (
            "read-only",
            "Read Only",
            "Agent inspects and reports. No writes.",
        ),
    ]
    .iter()
    .map(|(id, label, desc)| TierRow {
        id,
        label,
        description: desc,
        selected: *id == selected,
    })
    .collect()
}

#[derive(Serialize)]
pub struct AgentSettingsCtx {
    pub agent: AgentDetail,
    pub tiers: Vec<TierRow>,
    pub catalogs: AgentCatalogs,
}

/// Build the settings-panel payload. Returns `None` when the agent
/// doesn't exist in either scope.
pub fn build(id: &str, ship_dir: &Path) -> Option<AgentSettingsCtx> {
    let agent = load_agent(ship_dir, id)?;
    let tiers = build_tiers(&agent.permissions_tier);
    let mut catalogs = list_agent_catalogs(ship_dir);
    catalogs.skills.retain(|e| !agent.skills.contains(&e.id));
    catalogs.mcp.retain(|e| !agent.mcp_servers.contains(&e.id));
    catalogs.rules.retain(|e| !agent.rules.contains(&e.id));
    Some(AgentSettingsCtx {
        agent,
        tiers,
        catalogs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tiers_marks_selected() {
        let rows = build_tiers("autonomous");
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().find(|t| t.id == "autonomous").unwrap().selected);
        assert!(!rows.iter().find(|t| t.id == "standard").unwrap().selected);
    }

    #[test]
    fn build_tiers_no_match_selects_none() {
        let rows = build_tiers("bogus");
        assert!(rows.iter().all(|t| !t.selected));
    }
}
