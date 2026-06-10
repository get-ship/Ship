//! Skill discovery and context injection.
//!
//! Skills are the user-customization surface: markdown files with a YAML
//! frontmatter block (`--- ... ---`) carrying `name`, `description`, and an
//! optional `requires` array.  The catalog is injected into the system prompt;
//! skill bodies are loaded on demand when the agent activates one.

use std::path::{Path, PathBuf};

/// A discovered skill.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    /// Path to the `SKILL.md` file.
    pub path: PathBuf,
    /// Capability ids declared by the skill (`requires: [...]`).  Empty if absent.
    pub requires: Vec<String>,
}

/// Scan `root/*/SKILL.md` (one level deep), parse frontmatter, return all
/// valid skills.  Dirs without a readable, valid `SKILL.md` are silently
/// skipped.
pub fn discover_skills(root: &Path) -> Vec<Skill> {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let candidate = entry.path().join("SKILL.md");
        if !candidate.is_file() {
            continue;
        }
        let src = match std::fs::read_to_string(&candidate) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Some(skill) = parse_skill(&src, candidate) {
            skills.push(skill);
        }
    }
    skills
}

/// Parse a `SKILL.md` source and return a `Skill` if frontmatter is valid.
fn parse_skill(src: &str, path: PathBuf) -> Option<Skill> {
    let (fm, _body) = extract_frontmatter(src)?;
    let name = fm_scalar(fm, "name")?;
    let description = fm_scalar(fm, "description")?;
    let requires = fm_array(fm, "requires").unwrap_or_default();
    Some(Skill { name, description, path, requires })
}

/// Split `--- ... ---` frontmatter from body.  Returns `(frontmatter, body)`
/// or `None` if no valid fence is found.
fn extract_frontmatter(src: &str) -> Option<(&str, &str)> {
    let src = src.trim_start();
    let src = src.strip_prefix("---")?;
    // Accept `---\n` or `---` followed by a newline
    let src = src.strip_prefix('\n').or_else(|| src.strip_prefix("\r\n"))?;
    // Find closing `---`
    let close = src.find("\n---")?;
    let fm = &src[..close];
    let after = &src[close + 4..]; // skip `\n---`
    // skip optional newline after closing fence
    let body = after.strip_prefix('\n').or_else(|| after.strip_prefix("\r\n")).unwrap_or(after);
    Some((fm, body))
}

/// Read a scalar value for `key: value` from frontmatter text.
fn fm_scalar<'a>(fm: &'a str, key: &str) -> Option<String> {
    for line in fm.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            if let Some(rest) = rest.strip_prefix(':') {
                let val = rest.trim().trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

/// Read a simple inline array `key: [a, b, c]` from frontmatter text.
/// Returns `None` if the key is absent; `Some(vec![])` if present but empty.
fn fm_array(fm: &str, key: &str) -> Option<Vec<String>> {
    for line in fm.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            if let Some(rest) = rest.strip_prefix(':') {
                let rest = rest.trim();
                // Expect `[...]`
                let inner = rest.strip_prefix('[')?.strip_suffix(']')?;
                let items: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return Some(items);
            }
        }
    }
    None
}

/// Build the catalog string injected into the system prompt.  Sorted by name
/// for determinism.
pub fn render_catalog(skills: &[Skill]) -> String {
    let mut sorted: Vec<&Skill> = skills.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    sorted
        .iter()
        .map(|s| format!("{} — {}", s.name, s.description))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Read the `SKILL.md` at `skill.path`, strip the frontmatter, return the
/// markdown body.
pub fn load_body(skill: &Skill) -> std::io::Result<String> {
    let src = std::fs::read_to_string(&skill.path)?;
    let (_fm, body) = extract_frontmatter(&src)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no frontmatter"))?;
    Ok(body.to_string())
}

// ---------------------------------------------------------------------------
// Behavioral tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_skill(dir: &Path, subdir: &str, content: &str) {
        let d = dir.join(subdir);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("SKILL.md"), content).unwrap();
    }

    const SKILL_A: &str = "\
---
name: alpha
description: Does alpha things
requires: [ship.collections.read, ship.runtime.dispatch]
---
# Alpha

Body of alpha skill.
";

    const SKILL_B: &str = "\
---
name: beta
description: Does beta things
---
# Beta

Body of beta skill.
";

    const NOT_A_SKILL: &str = "No frontmatter here at all.";

    // -----------------------------------------------------------------------
    // discover_skills
    // -----------------------------------------------------------------------

    #[test]
    fn discovers_valid_skills_and_skips_invalid() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "skill-a", SKILL_A);
        write_skill(tmp.path(), "skill-b", SKILL_B);
        write_skill(tmp.path(), "not-a-skill", NOT_A_SKILL);

        let skills = discover_skills(tmp.path());
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();

        assert!(names.contains(&"alpha"), "alpha should be discovered");
        assert!(names.contains(&"beta"), "beta should be discovered");
        // The broken dir must be skipped — verify it is not present
        assert!(!names.contains(&"not-a-skill"), "invalid SKILL.md should be skipped");
    }

    #[test]
    fn skips_dir_without_skill_md() {
        let tmp = TempDir::new().unwrap();
        // Dir exists but no SKILL.md
        fs::create_dir_all(tmp.path().join("empty-dir")).unwrap();
        write_skill(tmp.path(), "real-skill", SKILL_A);

        let skills = discover_skills(tmp.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "alpha");
    }

    #[test]
    fn parses_name_and_description() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "s", SKILL_B);

        let skills = discover_skills(tmp.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "beta");
        assert_eq!(skills[0].description, "Does beta things");
    }

    // -----------------------------------------------------------------------
    // requires parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parses_requires_array() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "s", SKILL_A);

        let skills = discover_skills(tmp.path());
        assert_eq!(skills.len(), 1);
        let req = &skills[0].requires;
        assert!(req.contains(&"ship.collections.read".to_string()));
        assert!(req.contains(&"ship.runtime.dispatch".to_string()));
    }

    #[test]
    fn absent_requires_yields_empty_vec() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "s", SKILL_B);

        let skills = discover_skills(tmp.path());
        assert_eq!(skills.len(), 1);
        assert!(skills[0].requires.is_empty(), "no requires → empty vec");
    }

    // -----------------------------------------------------------------------
    // render_catalog
    // -----------------------------------------------------------------------

    #[test]
    fn catalog_is_sorted_and_contains_name_and_description() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "sa", SKILL_A);
        write_skill(tmp.path(), "sb", SKILL_B);

        let skills = discover_skills(tmp.path());
        let catalog = render_catalog(&skills);

        // Both names must appear
        assert!(catalog.contains("alpha"), "catalog must contain alpha");
        assert!(catalog.contains("beta"), "catalog must contain beta");
        // Both descriptions must appear
        assert!(catalog.contains("Does alpha things"));
        assert!(catalog.contains("Does beta things"));
        // Sorted: alpha before beta
        let alpha_pos = catalog.find("alpha").unwrap();
        let beta_pos = catalog.find("beta").unwrap();
        assert!(alpha_pos < beta_pos, "catalog must be sorted by name");
    }

    // -----------------------------------------------------------------------
    // load_body
    // -----------------------------------------------------------------------

    #[test]
    fn load_body_strips_frontmatter() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "s", SKILL_A);

        let skills = discover_skills(tmp.path());
        assert_eq!(skills.len(), 1);

        let body = load_body(&skills[0]).unwrap();
        // Body present
        assert!(body.contains("Body of alpha skill."), "body should be present");
        // Frontmatter fence must not appear in the body
        assert!(!body.contains("---"), "frontmatter fence must be stripped");
        assert!(!body.contains("name:"), "frontmatter key must be stripped");
    }

    #[test]
    fn load_body_returns_error_for_missing_file() {
        let skill = Skill {
            name: "ghost".into(),
            description: "gone".into(),
            path: PathBuf::from("/nonexistent/path/SKILL.md"),
            requires: vec![],
        };
        assert!(load_body(&skill).is_err());
    }
}
