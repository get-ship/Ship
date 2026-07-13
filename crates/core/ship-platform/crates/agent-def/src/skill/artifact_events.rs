//! Subscription prefixes for agent-emitted skill event namespaces.
//!
//! Agents emit skill events under their skill ID namespace and subscribe to
//! those namespaces for routed events.

use crate::skill::Skill;

/// Compute the custom skill namespace prefixes an actor should subscribe to.
///
/// Returns a deduplicated list of `{skill.id}.` prefixes — one per skill — for
/// routing agent-emitted skill events.
pub fn skill_event_subscriptions(skills: &[Skill]) -> Vec<String> {
    skill_custom_namespaces(skills)
}

/// Collect custom skill namespace prefixes (e.g. `"canvas."`) from a slice of
/// skills for subscribing to agent-emitted skill events.
pub fn skill_custom_namespaces(skills: &[Skill]) -> Vec<String> {
    let mut namespaces: Vec<String> = Vec::new();
    for skill in skills {
        if skill.id.is_empty() {
            continue;
        }
        let ns = format!("{}.", skill.id);
        if !namespaces.contains(&ns) {
            namespaces.push(ns);
        }
    }
    namespaces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{Skill, SkillSource};

    fn make_skill(id: &str, artifacts: &[&str]) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            version: None,
            authors: Vec::new(),
            stable_id: None,
            tags: Vec::new(),
            audience: None,
            content: String::new(),
            source: SkillSource::Custom,
            artifacts: artifacts.iter().map(|s| s.to_string()).collect(),
        }
    }

    // ── skill_event_subscriptions ─────────────────────────────────────────────

    #[test]
    fn no_skills_empty_subscriptions() {
        assert!(skill_event_subscriptions(&[]).is_empty());
    }

    #[test]
    fn returns_custom_namespace_without_ship_prefix() {
        let skills = vec![make_skill("canvas", &["html"])];
        let subs = skill_event_subscriptions(&skills);
        assert!(!subs.iter().any(|s| s.starts_with("ship.")));
        assert!(subs.contains(&"canvas.".to_string()));
    }

    #[test]
    fn skill_custom_namespace_included() {
        let skills = vec![make_skill("canvas", &["html"])];
        let subs = skill_event_subscriptions(&skills);
        assert!(subs.contains(&"canvas.".to_string()));
    }

    #[test]
    fn deduplicates_across_skills() {
        let skills = vec![
            make_skill("skill-a", &["adr"]),
            make_skill("skill-b", &["adr"]),
        ];
        let subs = skill_event_subscriptions(&skills);
        assert!(subs.contains(&"skill-a.".to_string()));
        assert!(subs.contains(&"skill-b.".to_string()));
        // no reserved ship.* entries
        assert!(!subs.iter().any(|s| s.starts_with("ship.")));
    }

    #[test]
    fn skill_with_no_artifacts_still_gets_custom_namespace() {
        let skills = vec![make_skill("my-tool", &[])];
        let subs = skill_event_subscriptions(&skills);
        assert!(subs.contains(&"my-tool.".to_string()));
    }

    // ── skill_custom_namespaces ───────────────────────────────────────────────

    #[test]
    fn custom_namespaces_deduped() {
        let skills = vec![make_skill("canvas", &[]), make_skill("canvas", &[])];
        let ns = skill_custom_namespaces(&skills);
        assert_eq!(ns, vec!["canvas.".to_string()]);
    }

    #[test]
    fn custom_namespaces_all_skills() {
        let skills = vec![make_skill("canvas", &[]), make_skill("pdf-viewer", &[])];
        let ns = skill_custom_namespaces(&skills);
        assert!(ns.contains(&"canvas.".to_string()));
        assert!(ns.contains(&"pdf-viewer.".to_string()));
    }

    #[test]
    fn empty_id_skipped() {
        let skills = vec![make_skill("", &[])];
        assert!(skill_custom_namespaces(&skills).is_empty());
    }
}
