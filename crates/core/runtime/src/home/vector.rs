//! Canonical customization-vector list.
//!
//! Per ADR `home-layout-and-scope-convention`, these ten vectors are applied
//! uniformly at every authoring scope (user `library/` and project `.ship/`).
//! Adding a new vector is a one-line enum change that guarantees uniform
//! treatment across scopes, loaders, lint rules, and docs.

use serde::{Deserialize, Serialize};

/// One of the ten canonical customization vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
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
    Mcp,
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
            Vector::Mcp => "mcp",
        }
    }

    /// Complete, ordered list of all ten canonical vectors.
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
            Vector::Mcp,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lists_ten_canonical_vectors() {
        assert_eq!(Vector::all().len(), 10);
    }

    #[test]
    fn as_str_matches_kebab_name() {
        assert_eq!(Vector::Agents.as_str(), "agents");
        assert_eq!(Vector::Collections.as_str(), "collections");
        assert_eq!(Vector::Mcp.as_str(), "mcp");
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
