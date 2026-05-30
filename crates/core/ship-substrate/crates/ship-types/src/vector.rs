//! Canonical customization-vector list.
//!
//! Per ADR `home-layout-and-scope-convention`, these directory-vectors are
//! applied uniformly at every authoring scope (user `library/` and project
//! `.ship/`). Adding a new directory-vector is a one-line enum change that
//! guarantees uniform treatment across scopes, loaders, lint rules, and docs.
//!
//! `mcp` is intentionally NOT in this enum — at every scope it's a single
//! file (`<scope>/mcp.jsonc`), not a directory. The directory-symmetry the
//! migrator originally wanted (`mcp/config.jsonc`) was overengineering.

use serde::{Deserialize, Serialize};

/// One of the canonical directory-vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "kebab-case")]
pub enum Vector {
    Agents,
    Skills,
    Collections,
    Views,
    Pipelines,
    Triggers,
    Rules,
    Permissions,
    Themes,
}

impl Vector {
    /// Kebab-case directory name for this vector.
    pub fn as_str(&self) -> &'static str {
        match self {
            Vector::Agents => "agents",
            Vector::Skills => "skills",
            Vector::Collections => "collections",
            Vector::Views => "views",
            Vector::Pipelines => "pipelines",
            Vector::Triggers => "triggers",
            Vector::Rules => "rules",
            Vector::Permissions => "permissions",
            Vector::Themes => "themes",
        }
    }

    /// Complete, ordered list of all canonical directory-vectors.
    pub fn all() -> &'static [Vector] {
        &[
            Vector::Agents,
            Vector::Skills,
            Vector::Collections,
            Vector::Views,
            Vector::Pipelines,
            Vector::Triggers,
            Vector::Rules,
            Vector::Permissions,
            Vector::Themes,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lists_canonical_directory_vectors() {
        assert_eq!(Vector::all().len(), 9);
    }

    #[test]
    fn as_str_matches_kebab_name() {
        assert_eq!(Vector::Agents.as_str(), "agents");
        assert_eq!(Vector::Collections.as_str(), "collections");
        assert_eq!(Vector::Triggers.as_str(), "triggers");
    }

    #[test]
    fn all_entries_are_unique() {
        let all = Vector::all();
        let mut seen = std::collections::HashSet::new();
        for v in all {
            assert!(seen.insert(v.as_str()), "duplicate vector: {}", v.as_str());
        }
    }
}
