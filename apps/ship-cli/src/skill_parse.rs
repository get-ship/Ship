//! Parse SKILL.md frontmatter per the agentskills.io spec.

use compiler::{Skill, SkillSource};
use std::collections::HashMap;

/// Parse a SKILL.md file per the agentskills.io spec.
///
/// Frontmatter keys: `name`, `stable-id`, `description`, `license`, `compatibility`,
/// `allowed-tools` (space-delimited), `artifacts`, `metadata` block. Legacy top-level
/// `version`/`author` fold into `metadata`.
pub fn parse_skill(id: &str, raw: &str) -> Skill {
    let mut name = id.to_string();
    let mut stable_id: Option<String> = None;
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools = vec![];
    let mut artifacts = vec![];
    let mut metadata: HashMap<String, String> = HashMap::new();
    let mut content_start = 0usize;

    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let mut in_metadata = false;
        for line in fm.lines() {
            if in_metadata {
                if line.starts_with("  ") || line.starts_with('\t') {
                    let trimmed = line.trim();
                    if let Some((k, v)) = trimmed.split_once(':') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    continue;
                } else {
                    in_metadata = false;
                }
            }
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("stable-id:") {
                let sid = v.trim().to_string();
                if crate::vars::state::validate_skill_id(&sid).is_ok() {
                    stable_id = Some(sid);
                } else {
                    eprintln!("warning: skill '{}': invalid stable-id '{}' ignored", id, sid);
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
                let trimmed = v.trim().trim_start_matches('[').trim_end_matches(']');
                artifacts = trimmed
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
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

    let content = raw[content_start..].trim().to_string();
    Skill {
        id: id.to_string(),
        name,
        stable_id,
        description,
        license,
        compatibility,
        allowed_tools,
        artifacts,
        metadata,
        content,
        source: SkillSource::default(),
        vars: Default::default(),
    }
}
