//! Parse SKILL.md frontmatter per the agentskills.io spec.
//!
//! One canonical parser — used by the compile-pipeline (`agent-def`), the dep
//! skill cache (`ship-store`), and any future consumer. Earlier each call
//! site carried its own hand-rolled parser; the dep-skill path (ship-store)
//! only recognized the BLOCK form (`grants:\n  - …`) and silently DROPPED
//! inline declared capabilities (`grants: [..]`) — agents loaded from a dep
//! package were missing half their declared capability surface (#106).
//!
//! Frontmatter keys handled: `name`, `stable-id`, `description`, `license`,
//! `compatibility`, `allowed-tools` (space-delimited), `artifacts`,
//! `grants`, `requires`, `metadata` block. Legacy top-level `version`/`author`
//! fold into `metadata`.
//!
//! All list-valued keys (`grants`, `requires`, `artifacts`) accept BOTH
//! inline (`grants: [a, b]` / `grants: a b`) and block (`grants:\n  - a`)
//! forms. Block continuations are tracked via `in_*` section flags so the
//! first indented `- …` line gets attached to the previous key.

use agent_schema::skill::{Skill, SkillSource, is_valid_skill_name};
use std::collections::HashMap;

/// Which list-key section we're inside, if any. Tracks block-form
/// continuations for `grants` / `requires` / `artifacts` so the caller's
/// `in_metadata`-style line loop can append indented `- …` items.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ListSection {
    None,
    Grants,
    Requires,
    Artifacts,
}

/// Parse a SKILL.md file per the agentskills.io spec.
///
/// `id` is the canonical id the caller passes in (directory name or dep ref).
/// The frontmatter `name:` (if present) overrides the caller's id, with a
/// warning when they disagree. `stable-id:`, when present AND a valid skill
/// name, is kept as the canonical key for state lookups.
pub fn parse_skill(id: &str, raw: &str) -> Skill {
    let mut name = id.to_string();
    let mut stable_id: Option<String> = None;
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools = vec![];
    let mut artifacts = vec![];
    let mut grants = vec![];
    let mut requires = vec![];
    let mut metadata: HashMap<String, String> = HashMap::new();
    let mut content_start = 0usize;

    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let mut in_metadata = false;
        let mut in_list: ListSection = ListSection::None;
        for line in fm.lines() {
            let indented = line.starts_with("  ") || line.starts_with('\t');

            // (1) Block-form continuation for `grants`/`requires`/`artifacts`.
            // A bare `- …` line under one of those keys appends to the list.
            if in_list != ListSection::None && indented {
                let item = line
                    .trim()
                    .trim_start_matches('-')
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !item.is_empty() {
                    match in_list {
                        ListSection::Grants => grants.push(item.to_string()),
                        ListSection::Requires => requires.push(item.to_string()),
                        ListSection::Artifacts => artifacts.push(item.to_string()),
                        ListSection::None => unreachable!(),
                    }
                }
                continue;
            } else if in_list != ListSection::None && !indented && !line.trim().is_empty() {
                // Left the list section — only re-opens on the matching key.
                in_list = ListSection::None;
            }

            // (2) Block-form metadata block.
            if in_metadata {
                if indented {
                    let trimmed = line.trim();
                    if let Some((k, v)) = trimmed.split_once(':') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    continue;
                } else {
                    in_metadata = false;
                }
            }

            // (3) Top-level keys.
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("stable-id:") {
                let sid = v.trim().to_string();
                if is_valid_skill_name(&sid) {
                    stable_id = Some(sid);
                } else {
                    eprintln!(
                        "warning: skill '{}': invalid stable-id '{}' ignored",
                        id, sid
                    );
                }
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("license:") {
                license = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("compatibility:") {
                compatibility = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("allowed-tools:") {
                allowed_tools = v.split_whitespace().map(str::to_string).collect();
            } else if let Some(v) = line.strip_prefix("artifacts:") {
                // Inline form: `[a, b, c]` or `a b c` on the same line. The
                // block form is handled by the `in_list == Artifacts` arm.
                let inline = parse_inline_list(v);
                if inline.is_empty() && line.trim_end() == "artifacts:" {
                    // Bare `artifacts:` — open the block section for
                    // subsequent indented `- …` lines.
                    in_list = ListSection::Artifacts;
                } else {
                    artifacts.extend(inline);
                }
            } else if let Some(v) = line.strip_prefix("grants:") {
                let inline = parse_inline_list(v);
                if inline.is_empty() && line.trim_end() == "grants:" {
                    in_list = ListSection::Grants;
                } else {
                    grants.extend(inline);
                }
            } else if let Some(v) = line.strip_prefix("requires:") {
                let inline = parse_inline_list(v);
                if inline.is_empty() && line.trim_end() == "requires:" {
                    in_list = ListSection::Requires;
                } else {
                    requires.extend(inline);
                }
            } else if line.trim_end() == "metadata:" {
                in_metadata = true;
            } else if let Some(v) = line.strip_prefix("version:") {
                metadata.insert("version".to_string(), v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("author:") {
                metadata.insert("author".to_string(), v.trim().to_string());
            }
        }
        content_start = 4 + end + 5; // "---\n" + fm + "\n---\n"
    }

    let normalized_name = name.to_lowercase().replace(' ', "-");
    if normalized_name != id && name != id {
        eprintln!(
            "warning: skill '{}': frontmatter name '{}' does not match directory name",
            id, name
        );
    }

    // Slice 2b close-out: trim only LEADING whitespace. The
    // declare-form `agent_config_eval::SkillDeclare::into_skill`
    // preserves the body string verbatim (incl. trailing whitespace),
    // so byte-fidelity between the two Skill-creation paths requires
    // the on-disk path to preserve trailing content too. Trailing
    // whitespace in a SKILL.md body is non-meaningful; this change is
    // transparent to consumers. `RawSkillParser` postconditions
    // (`tests/skill_parser.rs`) still pass — every test in that file
    // verifies the content with no trailing whitespace either way.
    let content = raw[content_start..].trim_start().to_string();
    Skill {
        id: id.to_string(),
        name,
        stable_id,
        description,
        license,
        compatibility,
        allowed_tools,
        artifacts,
        grants,
        requires,
        metadata,
        content,
        source: SkillSource::default(),
        vars: Default::default(),
        version: None,
        var_schema: Default::default(),
        signature: None,
    }
}

/// Parse the value side of `grants:` / `requires:` / `artifacts:` when the
/// caller wrote the value on the same line as the key. Returns an empty vec
/// when the value is `:` only (caller treats that as opening a block section).
fn parse_inline_list(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_grants() {
        let raw = "---\nname: ship-orchestration\ngrants: ship-orchestration\n---\n\nContent.";
        let skill = parse_skill("ship-orchestration", raw);
        assert_eq!(skill.grants, vec!["ship-orchestration".to_string()]);
    }

    #[test]
    fn parses_inline_grants_list() {
        // #106 regression: the dep-skill path's hand-rolled parser only
        // handled the block form and silently dropped this case. The
        // canonical parser in compiler/skill_parser must round-trip both.
        let raw = "---\nname: ship-orchestration\ngrants: [ship-orchestration, ship-other]\n---\n\nContent.";
        let skill = parse_skill("ship-orchestration", raw);
        assert_eq!(
            skill.grants,
            vec!["ship-orchestration".to_string(), "ship-other".to_string()]
        );
    }

    #[test]
    fn parses_block_form_grants() {
        let raw = "---\nname: ship-orchestration\ngrants:\n  - ship-orchestration\n  - ship-other\n---\n\nContent.";
        let skill = parse_skill("ship-orchestration", raw);
        assert_eq!(
            skill.grants,
            vec!["ship-orchestration".to_string(), "ship-other".to_string()]
        );
    }

    #[test]
    fn parses_block_form_requires() {
        // Mirrors the dep-skill test fixture the canonical parser must match.
        let raw = "---\nname: foo\nrequires:\n  - cap.x\ngrants:\n  - cap.y\n---\n\nContent.";
        let skill = parse_skill("foo", raw);
        assert_eq!(skill.requires, vec!["cap.x".to_string()]);
        assert_eq!(skill.grants, vec!["cap.y".to_string()]);
    }

    #[test]
    fn parses_inline_requires_and_artifacts() {
        let raw = "---\nname: x\nrequires: [r1, r2]\nartifacts: [html, adr]\n---\n\nContent.";
        let skill = parse_skill("x", raw);
        assert_eq!(skill.requires, vec!["r1".to_string(), "r2".to_string()]);
        assert_eq!(skill.artifacts, vec!["html".to_string(), "adr".to_string()]);
    }

    #[test]
    fn empty_grants_key_yields_empty_vec() {
        let raw = "---\nname: my-skill\ndescription: x\n---\n\nContent.";
        let skill = parse_skill("my-skill", raw);
        assert!(skill.grants.is_empty());
    }

    #[test]
    fn parses_license_and_compatibility() {
        let raw = "---\nname: my-skill\ndescription: Does stuff\nlicense: MIT\ncompatibility: claude >= 3\n---\n\nInstructions.";
        let skill = parse_skill("my-skill", raw);
        assert_eq!(skill.license.as_deref(), Some("MIT"));
        assert_eq!(skill.compatibility.as_deref(), Some("claude >= 3"));
    }

    #[test]
    fn parses_allowed_tools_space_delimited() {
        let raw =
            "---\nname: my-skill\ndescription: x\nallowed-tools: Read Edit Grep\n---\n\nContent.";
        let skill = parse_skill("my-skill", raw);
        assert_eq!(skill.allowed_tools, vec!["Read", "Edit", "Grep"]);
    }

    #[test]
    fn parses_metadata_block() {
        let raw = "---\nname: my-skill\ndescription: x\nmetadata:\n  team: infra\n  priority: high\n---\n\nContent.";
        let skill = parse_skill("my-skill", raw);
        assert_eq!(
            skill.metadata.get("team").map(String::as_str),
            Some("infra")
        );
        assert_eq!(
            skill.metadata.get("priority").map(String::as_str),
            Some("high")
        );
    }

    #[test]
    fn folds_legacy_version_author_into_metadata() {
        let raw =
            "---\nname: my-skill\ndescription: x\nversion: 1.2.3\nauthor: alice\n---\n\nContent.";
        let skill = parse_skill("my-skill", raw);
        assert_eq!(
            skill.metadata.get("version").map(String::as_str),
            Some("1.2.3")
        );
        assert_eq!(
            skill.metadata.get("author").map(String::as_str),
            Some("alice")
        );
    }
}
