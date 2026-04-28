//! `SchemaSource` trait — pluggable backend resolution for `data`
//! declarations and component-tag schemas.
//!
//! Different implementations:
//! - **Ship daemon** (`crates/sail-ship-source` — Layer 3, not OSS)
//! - **OpenAPI** (`@sail/openapi-source` — npm)
//! - **GraphQL** (`@sail/graphql-source` — npm)
//! - **Static fixture** (`@sail/static-source` — npm, for prototyping)
//!
//! See `.artifacts/specs/sail/05-server-data-bridge.md` for the design.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A pluggable data/component schema resolver.
pub trait SchemaSource: Send + Sync {
    /// Resolve `data <name> = <expr>` against this source.
    ///
    /// Returns `Ok(None)` if this source doesn't recognize the
    /// expression's namespace (the compiler tries the next source).
    fn resolve_data(&self, expr: &DataExpr) -> Result<Option<DataResolution>>;

    /// Look up a component tag's schema (for prop/event/slot validation).
    fn resolve_component(&self, tag: &str) -> Option<ComponentSchema>;

    /// Look up a design token by name (`color.surface`, `space.3`).
    fn resolve_token(&self, name: &str) -> Option<TokenValue>;

    /// SSR-side fetch — invoke the resolved data query, return JSON.
    fn ssr_fetch(&self, resolution: &DataResolution, ctx: &SsrContext) -> Result<serde_json::Value>;

    /// Build the SSE subscription URL for a `data live` declaration.
    fn live_channel(&self, resolution: &DataResolution) -> Option<String>;
}

/// Parsed `data` expression. Sources interpret the namespace + method.
#[derive(Debug, Clone)]
pub struct DataExpr {
    /// Full expression text — `ship.agents.list({ workspace: 'x' })`
    pub raw: String,
    /// First identifier — `ship`, `api`, `graph`, etc.
    pub namespace: String,
    /// Dotted path after namespace — `agents.list` for `ship.agents.list(...)`
    pub path: Vec<String>,
    /// Args — verbatim JS source
    pub args: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataResolution {
    pub return_type: TypeRef,
    pub initial_query: String,
    pub patch_subscription: Option<PatchSubscription>,
    pub live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSubscription {
    pub channel: String,
    pub args: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRef {
    /// Type as it would appear in TypeScript source: `Agent[]`,
    /// `{ id: string, name: string }`, `Promise<User>`, etc.
    pub ts_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    pub tag: String,
    pub props: HashMap<String, PropSchema>,
    pub events: HashMap<String, EventSchema>,
    pub slots: Vec<SlotSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropSchema {
    pub ty: TypeRef,
    pub default: Option<serde_json::Value>,
    pub required: bool,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchema {
    pub payload: TypeRef,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSchema {
    pub name: String,
    pub props: HashMap<String, TypeRef>,
}

#[derive(Debug, Clone)]
pub struct TokenValue {
    /// CSS variable name to emit, e.g., `--ship-color-surface`.
    pub css_var: String,
    /// Optional default value if the variable isn't set.
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SsrContext {
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub query: HashMap<String, String>,
}
