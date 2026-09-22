//! Metadata stored as `ship.json` at the root of a stack archive.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::manifest::ManifestDepValue;

/// The self-describing header for a published stack archive.
///
/// `hash` is the canonical tree hash computed by `ship-store`; this type keeps
/// it opaque so `stack-model` does not grow a second hashing implementation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchiveHeader {
    /// Marketplace-native stack path, e.g. `github.com/owner/repo`.
    pub path: String,
    /// Published semantic version.
    pub version: String,
    /// Source commit represented by this archive.
    pub commit: String,
    /// Canonical `sha256:<hex>` tree hash.
    pub hash: String,
    /// Direct stack dependencies and their version constraints.
    #[serde(default)]
    pub dependencies: IndexMap<String, ManifestDepValue>,
    /// Agent ids highlighted to consumers after installation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommended_agents: Vec<String>,
    /// Skill ids highlighted to consumers after installation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommended_skills: Vec<String>,
}
