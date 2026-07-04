//! `Agent` — the unified agent definition (Floki carve D2b).
//!
//! The ONE keystone both runtimes read: Floki (embedded code-mode) and ship-code
//! (third-party-harness drivers). It is the harness-facing projection of the
//! single agent-definition type ([`compiler::AgentProfile`]).
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

    /// Produce the `agent.schema.json`-conformant nested shape for writing to
    /// `.ship/agents/<id>.jsonc`. The inverse of [`Agent::from_profile`] — this is
    /// the single serializer all write paths call.
    ///
    /// Fields the unified `Agent` intentionally does not carry (`hooks`,
    /// `plugins`, `permissions`, `apps`) are absent from the output. Callers
    /// that must preserve those fields should merge this output into an existing
    /// canonical doc rather than overwriting it verbatim.
    ///
    /// `harness_settings.<id>` is reconstructed from `harness.settings`. The
    /// `disposition.model / temperature / max_turns` values are folded in when
    /// the settings map does not already contain them — this ensures round-trip
    /// correctness for Agents built programmatically without going through
    /// `from_profile`.
    pub fn to_nested_json(&self) -> serde_json::Value {
        use serde_json::json;

        // ── agent identity block ─────────────────────────────────────────────
        let mut agent_block = serde_json::Map::new();
        agent_block.insert("id".into(), json!(self.identity.id));
        agent_block.insert("name".into(), json!(self.identity.name));
        if self.identity.version != default_version() {
            agent_block.insert("version".into(), json!(self.identity.version));
        }
        if !self.identity.description.is_empty() {
            agent_block.insert("description".into(), json!(self.identity.description));
        }
        if !self.identity.tags.is_empty() {
            agent_block.insert("tags".into(), json!(self.identity.tags));
        }

        let mut root = serde_json::Map::new();
        root.insert("agent".into(), serde_json::Value::Object(agent_block));

        // ── capabilities ─────────────────────────────────────────────────────
        if !self.capabilities.is_empty() {
            root.insert(
                "capabilities".into(),
                serde_json::to_value(&self.capabilities).unwrap_or(json!([])),
            );
        }

        // ── skills → skills.refs ─────────────────────────────────────────────
        if !self.skills.is_empty() {
            root.insert("skills".into(), json!({ "refs": self.skills }));
        }

        // ── mcp → mcp.servers ────────────────────────────────────────────────
        if !self.mcp.is_empty() {
            root.insert("mcp".into(), json!({ "servers": self.mcp }));
        }

        // ── disposition.rules → rules.inline ─────────────────────────────────
        if !self.disposition.rules.is_empty() {
            root.insert("rules".into(), json!({ "inline": self.disposition.rules }));
        }

        // ── harness_settings.<id> ─────────────────────────────────────────────
        // Reconstruct from harness.settings, folding in disposition scalar fields
        // that aren't already present (round-trip safety for programmatic agents).
        if !self.harness.id.is_empty() {
            let mut settings = self.harness.settings.clone();
            if let Some(ref model) = self.disposition.model {
                settings
                    .entry("model".to_string())
                    .or_insert_with(|| json!(model));
            }
            if let Some(temp) = self.disposition.temperature {
                settings
                    .entry("temperature".to_string())
                    .or_insert_with(|| json!(temp));
            }
            if let Some(max_turns) = self.disposition.max_turns {
                settings
                    .entry("max_turns".to_string())
                    .or_insert_with(|| json!(max_turns));
            }
            if !settings.is_empty() {
                let mut hs = serde_json::Map::new();
                hs.insert(self.harness.id.clone(), serde_json::Value::Object(settings));
                root.insert("harness_settings".into(), serde_json::Value::Object(hs));
            }
        }

        serde_json::Value::Object(root)
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

    // ── to_nested_json round-trip ─────────────────────────────────────────

    #[test]
    fn to_nested_json_round_trips_via_profile() {
        // Build an Agent programmatically, emit canonical nested JSON, parse as
        // compiler::AgentProfile, project back via from_profile — all fields
        // carried by Agent must survive unchanged.
        let mut hs = serde_json::Map::new();
        hs.insert("model".into(), serde_json::json!("opus"));
        hs.insert("temperature".into(), serde_json::json!(0.5_f64));
        hs.insert("theme".into(), serde_json::json!("dark")); // pass-through extra

        let agent = Agent {
            harness: Harness { id: "claude".into(), settings: hs },
            identity: Identity {
                id: "rev".into(),
                name: "Reviewer".into(),
                version: "2.0.0".into(),
                description: "Reviews code.".into(),
                tags: vec!["review".into()],
            },
            disposition: Disposition {
                rules: "be careful".into(),
                model: Some("opus".into()),
                temperature: Some(0.5),
                max_turns: Some(15),
            },
            capabilities: vec![Capability::new("collections", BlastRadius::C2Elevated)],
            skills: vec!["s1".into(), "s2".into()],
            mcp: vec!["github".into()],
        };

        let json_val = agent.to_nested_json();
        let json_str = serde_json::to_string(&json_val).expect("serialize");

        // The canonical JSON must parse as compiler::AgentProfile (the file format).
        let profile: compiler::AgentProfile =
            serde_json::from_str(&json_str).expect("must parse as AgentProfile");

        // Project back to Agent using the harness we emitted.
        let again = Agent::from_profile(&profile, "claude");

        assert_eq!(again.identity.id, agent.identity.id, "id");
        assert_eq!(again.identity.name, agent.identity.name, "name");
        assert_eq!(again.identity.version, agent.identity.version, "version");
        assert_eq!(again.identity.description, agent.identity.description, "description");
        assert_eq!(again.identity.tags, agent.identity.tags, "tags");
        assert_eq!(again.disposition.rules, agent.disposition.rules, "rules");
        assert_eq!(again.disposition.model, agent.disposition.model, "model");
        assert_eq!(again.disposition.max_turns, agent.disposition.max_turns, "max_turns");
        assert_eq!(again.skills, agent.skills, "skills");
        assert_eq!(again.mcp, agent.mcp, "mcp");
        assert_eq!(again.capabilities.len(), agent.capabilities.len(), "cap count");
        assert_eq!(again.capabilities[0].domain, agent.capabilities[0].domain, "cap domain");
        assert_eq!(again.capabilities[0].blast, agent.capabilities[0].blast, "cap blast");
        // Pass-through harness key must survive (not erased by to_nested_json).
        assert_eq!(
            again.harness.settings.get("theme").and_then(|v| v.as_str()),
            Some("dark"),
            "extra harness setting survives"
        );
    }

    #[test]
    fn to_nested_json_minimal_agent() {
        // A bare Agent with defaults should emit valid canonical JSON with only the
        // required `agent:` block.
        let agent = Agent {
            harness: Harness { id: "floki".into(), settings: Default::default() },
            identity: Identity {
                id: "bare".into(),
                name: "Bare".into(),
                version: default_version(),
                description: String::new(),
                tags: vec![],
            },
            disposition: Disposition::default(),
            capabilities: vec![],
            skills: vec![],
            mcp: vec![],
        };
        let val = agent.to_nested_json();
        assert_eq!(val["agent"]["id"], "bare");
        assert_eq!(val["agent"]["name"], "Bare");
        // Default version is not emitted (clean output).
        assert!(val["agent"].get("version").is_none(), "default version omitted");
        // Empty lists / empty rules produce no keys at root.
        assert!(val.get("skills").is_none(), "empty skills omitted");
        assert!(val.get("mcp").is_none(), "empty mcp omitted");
        assert!(val.get("capabilities").is_none(), "empty caps omitted");
        assert!(val.get("rules").is_none(), "empty rules omitted");
        // Empty harness settings → no harness_settings block.
        assert!(val.get("harness_settings").is_none(), "empty harness_settings omitted");
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
