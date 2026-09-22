//! Named wire shapes for the `agents.*` ops that previously returned an
//! anonymous `json!({...})`. Each struct serializes byte-identically to the
//! object the registry used to hand-build, but now carries a name so the
//! `OpDescriptor.returns` is a real type and `data.ts` can drop `unknown`.
//!
//! Sibling to `view_state.rs` (already over the file cap) per the typed-op
//! single-source convention: new wire types live here, not in view_state.

use serde::Serialize;

use super::create::{AgentTemplate, PermissionPreset};
use super::state::{IndexEntry, ScopeTag};

/// Wire shape of `ship.agents.list()` — minimal index rows across scopes.
#[derive(Serialize)]
pub struct AgentIndexResponse {
    pub items: Vec<IndexEntry>,
}

/// Wire shape of `ship.agents.get()` — the raw agent doc at a resolved scope.
#[derive(Serialize)]
pub struct AgentGetResponse {
    pub id: String,
    pub scope: ScopeTag,
    pub doc: serde_json::Value,
}

/// Wire shape of `ship.agents.templates()` — create-wizard starter templates.
#[derive(Serialize)]
pub struct AgentTemplatesResponse {
    pub items: Vec<AgentTemplate>,
}

/// Wire shape of `ship.agents.permission_presets()`.
#[derive(Serialize)]
pub struct PermissionPresetsResponse {
    pub items: Vec<PermissionPreset>,
}
