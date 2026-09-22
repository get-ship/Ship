//! Resolve skill discovery directories on disk.
//!
//! By default, skills live in `<space_root>/skills/` (the space root being
//! the directory that contains `stack.config.ts`). The constant
//! [`stack_model::loader::DEFAULT_SPACE_SKILL_PATH`] is the single source of
//! truth — this module imports it. The pre-cutover `space.skill_paths`
//! ship.jsonc field died with the dot-ship close-out: nobody set it, and
//! custom skill paths are now a declare-form problem in `stack.config.ts`.

use std::path::{Path, PathBuf};

use stack_model::loader::DEFAULT_SPACE_SKILL_PATH;

/// Resolve the on-disk skill dirs for one space. Returns the single
/// default entry — there is no extras walk (the `space.skill_paths`
/// manifest field is gone).
pub fn read_skill_paths(space_root: &Path) -> Vec<PathBuf> {
    vec![space_root.join(DEFAULT_SPACE_SKILL_PATH)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn defaults_to_space_root_skills() {
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let paths = read_skill_paths(space_root);
        assert_eq!(paths, vec![space_root.join("skills/")]);
    }

    #[test]
    fn no_extras_when_manifest_lacks_skill_paths() {
        // The manifest field is dead — a ship.jsonc with no
        // `space.skill_paths` is the same as no ship.jsonc at all.
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = space_root.join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();
        fs::write(&ship_dir.join("ship.jsonc"), r#"{"id": "test123"}"#).unwrap();
        let paths = read_skill_paths(space_root);
        assert_eq!(paths, vec![space_root.join("skills/")]);
    }

    #[test]
    fn no_extras_when_manifest_omits_skill_paths_field() {
        // The same for a manifest that simply omits the field.
        let tmp = tempdir().unwrap();
        let space_root = tmp.path();
        let ship_dir = space_root.join(".ship");
        fs::create_dir_all(&ship_dir).unwrap();
        fs::write(
            &ship_dir.join("ship.jsonc"),
            r#"{"id": "test123", "space": {}}"#,
        )
        .unwrap();
        let paths = read_skill_paths(space_root);
        assert_eq!(paths, vec![space_root.join("skills/")]);
    }
}
