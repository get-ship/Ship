use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A job spec — the structured input to a pipeline run.
///
/// Stored as markdown with YAML frontmatter in `.artifacts/`.
/// The frontmatter fields are defined here. The body is free-form markdown.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobSpec {
    /// Human-readable title.
    pub title: Option<String>,
    /// Spec lifecycle status.
    #[serde(default)]
    pub status: SpecStatus,
    /// Pipeline to execute (references `.ship/pipelines/<name>.jsonc`).
    pub pipeline: Option<String>,
    /// Verifiable acceptance criteria.
    #[serde(default)]
    pub criteria: Vec<String>,
    /// File paths this spec touches.
    #[serde(default)]
    pub scope: Vec<String>,
    /// Pipeline variables.
    #[serde(default)]
    pub vars: HashMap<String, String>,
    /// Priority (0 = highest).
    pub priority: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpecStatus {
    #[default]
    Draft,
    Ready,
    Launched,
    Completed,
    Failed,
}

/// Parse YAML frontmatter from a markdown string into a JobSpec.
/// Returns None if no frontmatter is present.
pub fn parse_spec_frontmatter(content: &str) -> Option<JobSpec> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_open = &trimmed[3..];
    let close = after_open.find("\n---")?;
    let yaml = &after_open[..close].trim();
    serde_yaml::from_str(yaml).ok()
}

/// Extract the markdown body (everything after frontmatter).
pub fn spec_body(content: &str) -> &str {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content;
    }
    let after_open = &trimmed[3..];
    match after_open.find("\n---") {
        Some(pos) => {
            let rest = &after_open[pos + 4..];
            rest.trim_start_matches('\n')
        }
        None => content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spec_from_frontmatter() {
        let content = r#"---
title: Auth Middleware
pipeline: tdd
status: draft
criteria:
  - All /api routes return 401
  - Token validated
scope:
  - apps/shipd/src/auth.rs
vars:
  impl_agent: rust-lane
---

## Goal
Add auth.
"#;
        let spec = parse_spec_frontmatter(content).unwrap();
        assert_eq!(spec.title.as_deref(), Some("Auth Middleware"));
        assert_eq!(spec.pipeline.as_deref(), Some("tdd"));
        assert_eq!(spec.status, SpecStatus::Draft);
        assert_eq!(spec.criteria.len(), 2);
        assert_eq!(spec.scope.len(), 1);
        assert_eq!(spec.vars.get("impl_agent").unwrap(), "rust-lane");
    }

    #[test]
    fn parse_no_frontmatter_returns_none() {
        assert!(parse_spec_frontmatter("just markdown").is_none());
    }

    #[test]
    fn spec_body_extracts_after_frontmatter() {
        let content = "---\ntitle: Test\n---\n\n## Goal\nDo stuff.";
        assert_eq!(spec_body(content), "## Goal\nDo stuff.");
    }

    #[test]
    fn spec_body_no_frontmatter_returns_all() {
        assert_eq!(spec_body("just markdown"), "just markdown");
    }
}
