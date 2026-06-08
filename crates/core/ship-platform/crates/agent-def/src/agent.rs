//! `Agent` — the unified agent definition (Floki carve D2b).
//!
//! The ONE keystone both runtimes read: Floki (embedded code-mode) and ship-code
//! (third-party-harness drivers). It collapses the old split between the nested
//! file-format profile ([`compiler::AgentProfile`]) and the flat runtime config
//! ([`compiler::AgentConfig`]) into a single settled shape.
//!
//! ```text
//! Agent {
//!   harness:      { id, <flattened harness-specific settings> }  // "floki" | "claude" | …
//!   identity:     { id, name, version, description, tags }
//!   disposition:  { rules, model?, temperature?, max_turns? }
//!   capabilities: [ { domain, blast } ]   // ship-op authority; gate-enforced
//!   skills:       [ ref ]                  // markdown skill-library refs
//!   mcp:          [ ref ]                  // mcp server refs
//! }
//! ```
//!
//! What this shape deliberately DROPS vs the legacy profile (settled design,
//! not loss): `role`, `tools_preset`, `active_tools` (→ `capabilities`); the
//! top-level `permissions` block, `plugins`, `hooks`, `apps` (foreign-tool /
//! harness policy belongs in `harness.settings`, interpreted per-driver — NOT in
//! the shared def); and any `runtime` discriminant (the `harness.id` IS the
//! discriminant). Skills' `requires:` is demoted to a non-load-bearing hint —
//! the gate enforces capability, not the skill.
//!
//! `BlastRadius` / `Capability` are re-exported from the `compiler` crate, which
//! owns the single canonical definition (so the file-format profile, the flat
//! config, and this keystone all share one type). See
//! `compiler::types::capability`.

use serde::{Deserialize, Serialize};

pub use compiler::{BlastRadius, Capability};

/// 0 · Harness — the runtime discriminant + its open pass-through settings.
///
/// `id` selects the harness (`"floki"`, `"claude"`, `"codex"`, …). `settings`
/// is an open `serde_json` object each driver interprets however it likes
/// (model, temperature, sandbox, vendor flags); Ship does not validate it here.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Harness {
    /// Harness id — selects the driver. Also the runtime discriminant.
    pub id: String,
    /// Open, harness-specific settings merged verbatim by the driver.
    #[serde(flatten, default)]
    pub settings: serde_json::Map<String, serde_json::Value>,
}

/// 1 · Identity — pure distribution metadata (registry discovery).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub(crate) fn default_version() -> String {
    "0.1.0".to_string()
}

/// 2 · Disposition — how the agent behaves. `rules` is the inline system-prompt
/// body (the user's word — kept, not "instructions"). `model` is top-level
/// because the Floki-native runtime has no harness to nest it under; for a
/// third-party harness it mirrors `harness.settings.model`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Disposition {
    #[serde(default)]
    pub rules: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<usize>,
}

/// The unified agent definition. Distributable, gated, read by both runtimes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub harness: Harness,
    pub identity: Identity,
    #[serde(default)]
    pub disposition: Disposition,
    /// Ship-op authority grants. C0/C1 are an implicit baseline; C3 is never
    /// agent-settable. Validate with [`Agent::validate_capabilities`].
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// Skill-library refs (markdown). Trigger-loaded; not gated here.
    #[serde(default)]
    pub skills: Vec<String>,
    /// MCP server refs.
    #[serde(default)]
    pub mcp: Vec<String>,
}

impl Agent {
    /// Parse a unified agent from JSON.
    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Reject any capability an agent author may not declare (C3/critical is
    /// operator-only). Returns the first offending grant's error.
    pub fn validate_capabilities(&self) -> Result<(), String> {
        for cap in &self.capabilities {
            cap.validate()?;
        }
        Ok(())
    }

    /// Project the legacy nested file-format [`compiler::AgentProfile`] onto the
    /// unified shape for a single target harness.
    ///
    /// Additive + lossless w.r.t. the keystone's own fields: identity, rules,
    /// capabilities, skills and mcp map straight across; the selected harness's
    /// `harness_settings` block becomes `harness.settings` verbatim, and its
    /// `model` / `temperature` / `max_turns` are lifted into `disposition` as the
    /// normalized read (kept in `harness.settings` too — no data dropped).
    ///
    /// Fields the unified shape intentionally does not carry (role / tools_preset
    /// / active_tools / permissions / plugins / hooks / apps) are left on the
    /// profile; this is a projection, not a destructive migration. The profile
    /// itself is untouched.
    pub fn from_profile(profile: &compiler::AgentProfile, harness_id: &str) -> Self {
        let settings = profile
            .harness_settings
            .get(harness_id)
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();

        let model = settings
            .get("model")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let temperature = settings
            .get("temperature")
            .and_then(serde_json::Value::as_f64)
            .map(|f| f as f32);
        let max_turns = settings
            .get("max_turns")
            .and_then(serde_json::Value::as_u64)
            .map(|n| n as usize);

        Agent {
            harness: Harness {
                id: harness_id.to_string(),
                settings,
            },
            identity: Identity {
                id: profile.agent.id.clone(),
                name: profile.agent.name.clone(),
                version: profile
                    .agent
                    .version
                    .clone()
                    .unwrap_or_else(default_version),
                description: profile.agent.description.clone().unwrap_or_default(),
                tags: profile.agent.tags.clone(),
            },
            disposition: Disposition {
                rules: profile.rules.inline.clone().unwrap_or_default(),
                model,
                temperature,
                max_turns,
            },
            capabilities: profile.capabilities.clone(),
            skills: profile.skills.refs.clone(),
            mcp: profile.mcp.servers.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_full_agent() {
        let src = r#"{
            "harness": { "id": "claude", "model": "opus", "temperature": 0.2 },
            "identity": {
                "id": "code-reviewer",
                "name": "Code Reviewer",
                "version": "1.2.0",
                "description": "Reviews PRs.",
                "tags": ["review", "testing"]
            },
            "disposition": { "rules": "Be precise.", "model": "opus", "max_turns": 20 },
            "capabilities": [
                { "domain": "collections", "blast": "C2" },
                { "domain": "runtime" }
            ],
            "skills": ["tdd", "web-qa"],
            "mcp": ["github", "linear"]
        }"#;
        let agent = Agent::from_json(src).expect("parse");

        assert_eq!(agent.harness.id, "claude");
        // Flattened harness settings captured the pass-through keys.
        assert_eq!(
            agent.harness.settings.get("model").and_then(|v| v.as_str()),
            Some("opus")
        );
        assert_eq!(agent.identity.id, "code-reviewer");
        assert_eq!(agent.identity.version, "1.2.0");
        assert_eq!(agent.disposition.rules, "Be precise.");
        assert_eq!(agent.disposition.max_turns, Some(20));
        assert_eq!(agent.capabilities.len(), 2);
        assert_eq!(agent.capabilities[0].blast, BlastRadius::C2Elevated);
        // Defaulted blast = C0.
        assert_eq!(agent.capabilities[1].blast, BlastRadius::C0Read);
        assert_eq!(agent.skills, vec!["tdd", "web-qa"]);
        assert_eq!(agent.mcp, vec!["github", "linear"]);

        // Re-serialize then re-parse — stable round trip.
        let json = serde_json::to_string(&agent).expect("serialize");
        let again = Agent::from_json(&json).expect("reparse");
        assert_eq!(again.identity.id, agent.identity.id);
        assert_eq!(again.capabilities.len(), 2);
    }

    #[test]
    fn minimal_agent_defaults() {
        let src = r#"{
            "harness": { "id": "floki" },
            "identity": { "id": "bare", "name": "Bare" }
        }"#;
        let agent = Agent::from_json(src).expect("parse");
        assert_eq!(agent.harness.id, "floki");
        assert_eq!(agent.identity.version, "0.1.0");
        assert!(agent.disposition.rules.is_empty());
        assert!(agent.capabilities.is_empty());
        assert!(agent.skills.is_empty());
        assert!(agent.mcp.is_empty());
    }

    #[test]
    fn validate_capabilities_rejects_c3() {
        let agent = Agent {
            harness: Harness {
                id: "claude".into(),
                settings: Default::default(),
            },
            identity: Identity {
                id: "x".into(),
                name: "X".into(),
                version: default_version(),
                description: String::new(),
                tags: vec![],
            },
            disposition: Disposition::default(),
            capabilities: vec![Capability::new("account", BlastRadius::C3Critical)],
            skills: vec![],
            mcp: vec![],
        };
        assert!(agent.validate_capabilities().is_err());
    }

    #[test]
    fn from_profile_projects_superset() {
        let src = r#"{
            "agent": {
                "id": "rev",
                "name": "Reviewer",
                "version": "2.0.0",
                "description": "desc",
                "tags": ["a"],
                "role": "worker"
            },
            "capabilities": [ { "domain": "collections", "blast": "C2" } ],
            "skills": { "refs": ["s1", "s2"] },
            "mcp": { "servers": ["github"] },
            "rules": { "inline": "be careful" },
            "permissions": { "tools_deny": ["WebFetch"] },
            "harness_settings": {
                "claude": { "model": "opus", "temperature": 0.3, "max_turns": 12, "theme": "dark" }
            }
        }"#;
        let profile: compiler::AgentProfile = serde_json::from_str(src).expect("parse profile");
        let agent = Agent::from_profile(&profile, "claude");

        assert_eq!(agent.harness.id, "claude");
        assert_eq!(agent.identity.id, "rev");
        assert_eq!(agent.identity.version, "2.0.0");
        assert_eq!(agent.identity.tags, vec!["a"]);
        assert_eq!(agent.disposition.rules, "be careful");
        assert_eq!(agent.disposition.model.as_deref(), Some("opus"));
        assert_eq!(agent.disposition.temperature, Some(0.3));
        assert_eq!(agent.disposition.max_turns, Some(12));
        assert_eq!(agent.capabilities.len(), 1);
        assert_eq!(agent.capabilities[0].domain, "collections");
        assert_eq!(agent.capabilities[0].blast, BlastRadius::C2Elevated);
        assert_eq!(agent.skills, vec!["s1", "s2"]);
        assert_eq!(agent.mcp, vec!["github"]);
        // harness.settings kept verbatim (theme survives; model lifted but kept).
        assert_eq!(
            agent.harness.settings.get("theme").and_then(|v| v.as_str()),
            Some("dark")
        );
    }

    #[test]
    fn from_profile_missing_harness_block_is_clean() {
        let src = r#"{ "agent": { "id": "x", "name": "X" } }"#;
        let profile: compiler::AgentProfile = serde_json::from_str(src).expect("parse");
        let agent = Agent::from_profile(&profile, "floki");
        assert_eq!(agent.harness.id, "floki");
        assert!(agent.harness.settings.is_empty());
        assert_eq!(agent.disposition.model, None);
        assert_eq!(agent.identity.version, "0.1.0");
    }
}
