//! Resolve skill discovery directories from the manifest.
//!
//! By default, skills live in `.ship/skills/`. The `space.skill_paths` field
//! in `ship.jsonc` allows additional directories (relative to the space root).

use std::path::{Path, PathBuf};

/// Default skill dir, joined onto `ship_dir` — the source directory on disk
/// (see `ShipHome::project_ship_dir`). So this is `"skills/"`, NOT
/// `".ship/skills/"`: it resolves to `<space>/.ship/skills/`. Do not re-add the
/// `.ship/` prefix — that double-counts to `<space>/.ship/.ship/skills/` (the
/// bug from b2f75a7b2 "skills/ → .ship/skills/").
const DEFAULT_SKILL_DIR: &str = "skills/";

/// Read `space.skill_paths` from the manifest and return resolved absolute
/// directory paths.
///
/// - `.ship/skills/` is always included as the first entry.
/// - Custom paths from `space.skill_paths` resolve relative to `space_root`.
pub fn read_skill_paths(ship_dir: &Path, space_root: &Path) -> Vec<PathBuf> {
    let mut result = vec![ship_dir.join(DEFAULT_SKILL_DIR)];

    let raw_paths = read_raw_skill_paths(ship_dir);
    for p in &raw_paths {
        if Path::new(p).is_absolute() {
            continue;
        }
        let resolved = space_root.join(p);
        if resolved != result[0] {
            result.push(resolved);
        }
    }

    result
}

/// Read the raw string values from the manifest without resolving them.
/// Returns an empty vec when the field is absent or unreadable.
fn read_raw_skill_paths(ship_dir: &Path) -> Vec<String> {
    let primary = ship_dir.join(ship_config::project::PRIMARY_CONFIG_FILE);
    let content = match std::fs::read_to_string(&primary) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let parsed: serde_json::Value = match serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(&content)) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let paths = parsed
        .get("space")
        .and_then(|p| p.get("skill_paths"))
        .and_then(|v| v.as_array());

    match paths {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect(),
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_manifest(ship_dir: &Path, content: &str) {
        fs::create_dir_all(ship_dir).unwrap();
        fs::write(ship_dir.join(ship_config::project::PRIMARY_CONFIG_FILE), content).unwrap();
    }

    #[test]
    fn defaults_to_skills_when_absent() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(space_root);
        write_manifest(&ship_dir, r#"{"id": "test123"}"#);

        let paths = read_skill_paths(&ship_dir, space_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn defaults_when_no_manifest() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(space_root);
        fs::create_dir_all(&ship_dir).unwrap();

        let paths = read_skill_paths(&ship_dir, space_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn reads_custom_skill_paths() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(space_root);
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "space": {"skill_paths": ["docs/", "internal/skills/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir, space_root);
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], ship_dir.join("skills/"));
        assert_eq!(paths[1], space_root.join("docs/"));
        assert_eq!(paths[2], space_root.join("internal/skills/"));
    }

    #[test]
    fn rejects_absolute_paths() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(space_root);
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "space": {"skill_paths": ["/etc/evil/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir, space_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn empty_array_falls_back_to_default() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(space_root);
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "space": {"skill_paths": []}}"#,
        );

        let paths = read_skill_paths(&ship_dir, space_root);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], ship_dir.join("skills/"));
    }

    #[test]
    fn space_root_relative_custom_path() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(space_root);
        write_manifest(
            &ship_dir,
            r#"{"id": "test123", "space": {"skill_paths": ["custom-skills/"]}}"#,
        );

        let paths = read_skill_paths(&ship_dir, space_root);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], ship_dir.join("skills/"));
        assert_eq!(paths[1], space_root.join("custom-skills/"));
    }
}
