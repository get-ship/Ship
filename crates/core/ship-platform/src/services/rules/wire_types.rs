//! Named wire shapes for `rules.*` ops (formerly anonymous `json!`).
//! Sibling to `state.rs` per the typed-op single-source convention.

use serde::Serialize;

use super::state::{IndexEntry, ScopeTag};

/// Wire shape of `ship.rules.list()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RulesIndexResponse {
    pub items: Vec<IndexEntry>,
}

/// Wire shape of `ship.rules.get()` — raw rule body at a resolved scope.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RuleGetResponse {
    pub id: String,
    pub scope: ScopeTag,
    pub body: String,
}
