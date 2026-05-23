use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    #[default]
    Custom,
    Builtin,
    AiGenerated,
    Community,
    Imported,
}

/// A skill / slash command. Stored as `agents/skills/<id>/SKILL.md` (agentskills.io spec).
/// The compiler receives pre-loaded `Skill` values — it does not read files.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    /// Canonical identifier from `stable-id:` frontmatter field.
    ///
    /// When set, this is used as the key for state file lookups instead of the
    /// directory name. Allows renaming the skill directory without orphaning state.
    /// Must be a valid skill id (`[a-z0-9][a-z0-9-]*`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_id: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub compatibility: Option<String>,
    /// Parsed from the space-delimited `allowed-tools` frontmatter key.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Arbitrary key-value metadata from the `metadata` frontmatter block.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    pub content: String,
    #[serde(default)]
    pub source: SkillSource,
    /// Resolved variable values for template substitution.
    /// Merged from: user state (~/.ship/state/skills/{id}.json) +
    /// project state (.ship/state/skills/{id}.json) + vars.json defaults.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vars: HashMap<String, serde_json::Value>,
    /// Artifact types this skill produces (e.g. `["html", "adr"]`).
    /// Used to infer platform event subscriptions at actor spawn time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    /// Capability grants this skill confers (the `grants:` frontmatter block),
    /// e.g. `["ship-orchestration"]`. **Compiler-issued only** — appears on the
    /// generated capability-skills, never hand-authored (authored skills express
    /// need via [`requires`]). The agent's grant set is the union of its resolved
    /// skills' grants — the Decision-4 loop where a skill set IS a credential
    /// scope. See `.artifacts/auth-gate-synthesis.md` + `signed-grants-design.md`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<String>,
    /// Capabilities this skill *depends on* (the `requires:` frontmatter block),
    /// e.g. `["ship-orchestration"]`. **Public/authored** — workflow + knowledge
    /// skills declare what they need; this confers nothing. An agent whose skills
    /// require a capability none of its skills grant is a compile error (unmet).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
}

/// The union (deduped) of capability grants across a set of resolved skills —
/// an agent's grant set. The gate checks an op's restricted domain against this.
pub fn grants_from_skills(skills: &[Skill]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    for skill in skills {
        for grant in &skill.grants {
            seen.insert(grant.clone());
        }
    }
    seen.into_iter().collect()
}

/// Capabilities required by the skill set but granted by none of it — the unmet
/// dependencies. Empty == every `requires:` is covered by some skill's `grants:`.
/// A non-empty result is a compile error: the agent can't run its skills.
pub fn unmet_requirements(skills: &[Skill]) -> Vec<String> {
    let granted: std::collections::BTreeSet<&str> =
        skills.iter().flat_map(|s| s.grants.iter().map(String::as_str)).collect();
    let mut unmet = std::collections::BTreeSet::new();
    for skill in skills {
        for req in &skill.requires {
            if !granted.contains(req.as_str()) {
                unmet.insert(req.clone());
            }
        }
    }
    unmet.into_iter().collect()
}

pub fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_skill_names() {
        assert!(is_valid_skill_name("my-skill"));
        assert!(is_valid_skill_name("review-pr"));
        assert!(is_valid_skill_name("deploy"));
        assert!(is_valid_skill_name("a1b2c3"));
        assert!(is_valid_skill_name("x"));
        // 64-char boundary
        assert!(is_valid_skill_name(&"a".repeat(64)));
    }

    #[test]
    fn invalid_skill_names() {
        assert!(!is_valid_skill_name(""));
        assert!(!is_valid_skill_name(&"a".repeat(65)));
        assert!(!is_valid_skill_name("-leading"));
        assert!(!is_valid_skill_name("trailing-"));
        assert!(!is_valid_skill_name("double--hyphen"));
        assert!(!is_valid_skill_name("UpperCase"));
        assert!(!is_valid_skill_name("has space"));
    }

    #[test]
    fn skill_new_fields_default() {
        let skill = Skill {
            id: "test".to_string(),
            name: "test".to_string(),
            stable_id: None,
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: HashMap::new(),
            content: String::new(),
            source: SkillSource::Custom,
            vars: HashMap::new(),
            artifacts: vec![],
            grants: vec![],
            requires: vec![],
        };
        assert!(skill.stable_id.is_none());
        assert!(skill.license.is_none());
        assert!(skill.compatibility.is_none());
        assert!(skill.allowed_tools.is_empty());
        assert!(skill.metadata.is_empty());
        assert!(skill.vars.is_empty());
        assert!(skill.artifacts.is_empty());
        assert!(skill.grants.is_empty());
    }

    #[test]
    fn grants_from_skills_unions_and_dedups() {
        let mk = |id: &str, grants: &[&str]| Skill {
            id: id.to_string(),
            name: id.to_string(),
            stable_id: None,
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: HashMap::new(),
            content: String::new(),
            source: SkillSource::Custom,
            vars: HashMap::new(),
            artifacts: vec![],
            grants: grants.iter().map(|s| s.to_string()).collect(),
            requires: vec![],
        };
        let skills = vec![
            mk("orch", &["ship-orchestration"]),
            mk("author", &["ship-authoring"]),
            mk("dup", &["ship-orchestration"]), // duplicate grant
            mk("knowledge", &[]),               // no grant
        ];
        let grants = grants_from_skills(&skills);
        assert_eq!(grants, vec!["ship-authoring", "ship-orchestration"]); // sorted, deduped
    }

    #[test]
    fn unmet_requirements_flags_uncovered_deps() {
        let mk = |id: &str, grants: &[&str], requires: &[&str]| Skill {
            id: id.to_string(),
            name: id.to_string(),
            stable_id: None,
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: HashMap::new(),
            content: String::new(),
            source: SkillSource::Custom,
            vars: HashMap::new(),
            artifacts: vec![],
            grants: grants.iter().map(|s| s.to_string()).collect(),
            requires: requires.iter().map(|s| s.to_string()).collect(),
        };
        // Workflow requires orchestration; a capability-skill grants it -> met.
        let met = vec![
            mk("workflow", &[], &["ship-orchestration"]),
            mk("ship-orchestration", &["ship-orchestration"], &[]),
        ];
        assert!(unmet_requirements(&met).is_empty());

        // Same workflow without the granting skill -> unmet.
        let unmet = vec![mk("workflow", &[], &["ship-orchestration"])];
        assert_eq!(unmet_requirements(&unmet), vec!["ship-orchestration"]);
    }
}
