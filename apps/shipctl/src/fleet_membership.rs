//! Fleet org membership on disk (read-only diagnostic view).
//!
//! Layout matches fleet `agent-schema` org paths under XDG data home.
//! shipctl is polyrepo-separated from fleet — read the layout directly.

use std::path::{Path, PathBuf};

/// On-disk layout under XDG data home.
pub(crate) const BRIDGE_DATA_DIRNAME: &str = "bridge";
pub(crate) const ORGS_DIRNAME: &str = "orgs";
pub(crate) const MEMBERS_FILE: &str = "members.json";

/// One org's membership file state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OrgMembers {
    /// Parsed project roots (may be empty).
    Listed(Vec<PathBuf>),
    /// `members.json` unreadable or not the expected shape.
    Malformed(String),
}

/// Resolve XDG data home the same way fleet org membership does:
/// `$XDG_DATA_HOME` if set and non-empty, else `$HOME/.local/share`.
pub(crate) fn resolve_data_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg);
    }
    home::home_dir()
        .map(|h| h.join(".local/share"))
        .unwrap_or_else(|| PathBuf::from(".local/share"))
}

/// Scan `$data_home/bridge/orgs/*/members.json` for registered project roots.
///
/// Missing orgs root → empty vec (not an error). Missing `members.json` for an
/// org dir → empty member list. Corrupt JSON → [`OrgMembers::Malformed`].
pub(crate) fn list_fleet_org_membership(data_home: &Path) -> Vec<(String, OrgMembers)> {
    let orgs_root = data_home.join(BRIDGE_DATA_DIRNAME).join(ORGS_DIRNAME);
    let Ok(entries) = std::fs::read_dir(&orgs_root) else {
        return Vec::new();
    };

    let mut orgs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(org_id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        orgs.push((org_id, read_members_file(&path.join(MEMBERS_FILE))));
    }
    orgs.sort_by(|a, b| a.0.cmp(&b.0));
    orgs
}

fn read_members_file(path: &Path) -> OrgMembers {
    if !path.is_file() {
        return OrgMembers::Listed(Vec::new());
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return OrgMembers::Malformed(e.to_string()),
    };
    if text.trim().is_empty() {
        return OrgMembers::Listed(Vec::new());
    }

    #[derive(serde::Deserialize)]
    struct MembersFile {
        #[serde(default)]
        members: Vec<String>,
    }

    match serde_json::from_str::<MembersFile>(&text) {
        Ok(file) => {
            let mut members: Vec<PathBuf> = file.members.into_iter().map(PathBuf::from).collect();
            members.sort();
            members.dedup();
            OrgMembers::Listed(members)
        }
        Err(e) => OrgMembers::Malformed(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn list_membership_empty_when_no_orgs_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let listed = list_fleet_org_membership(tmp.path());
        assert!(listed.is_empty());
    }

    #[test]
    fn list_membership_reads_roots_and_malformed() {
        let tmp = tempfile::tempdir().unwrap();
        let data_home = tmp.path().join("share");
        let ship_org = data_home
            .join(BRIDGE_DATA_DIRNAME)
            .join(ORGS_DIRNAME)
            .join("ship");
        let other_org = data_home
            .join(BRIDGE_DATA_DIRNAME)
            .join(ORGS_DIRNAME)
            .join("other");
        fs::create_dir_all(&ship_org).unwrap();
        fs::create_dir_all(&other_org).unwrap();

        let root_a = tmp.path().join("proj-a");
        let root_b = tmp.path().join("proj-b");
        fs::create_dir_all(&root_a).unwrap();
        fs::create_dir_all(&root_b).unwrap();

        let members = serde_json::json!({
            "members": [root_b.display().to_string(), root_a.display().to_string()],
        });
        fs::write(
            ship_org.join(MEMBERS_FILE),
            serde_json::to_string_pretty(&members).unwrap(),
        )
        .unwrap();
        fs::write(other_org.join(MEMBERS_FILE), "not-json{{{").unwrap();

        let listed = list_fleet_org_membership(&data_home);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].0, "other");
        match &listed[0].1 {
            OrgMembers::Malformed(msg) => assert!(!msg.is_empty()),
            other => panic!("expected malformed, got {other:?}"),
        }
        assert_eq!(listed[1].0, "ship");
        match &listed[1].1 {
            OrgMembers::Listed(members) => {
                assert_eq!(members, &vec![root_a.clone(), root_b.clone()]);
            }
            other => panic!("expected listed, got {other:?}"),
        }
    }
}
