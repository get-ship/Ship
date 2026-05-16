//! Search wire types — shared between adapters, aggregator, and the
//! HTTP handler. `EntityKind` is the discriminator on every hit; the
//! frontend renders kind-appropriate icons + section headers off it.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What kind of entity a hit refers to. `CollectionItem` carries the
/// parent collection's id so the client can build a path back to the
/// item without a second lookup.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntityKind {
    View,
    Collection,
    CollectionItem { collection_id: String },
    Agent,
    Skill,
    Session,
    Artifact,
}

/// Lighter-weight filter token — what the wire `types=` query param
/// resolves to. Decouples filtering from `EntityKind` (which carries
/// per-hit payload like `collection_id`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EntityKindFilter {
    View,
    Collection,
    CollectionItem,
    Agent,
    Skill,
    Session,
    Artifact,
}

impl EntityKindFilter {
    /// Token form, used as the adapter registry key + the wire `types=`
    /// query value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Collection => "collection",
            Self::CollectionItem => "collection_item",
            Self::Agent => "agent",
            Self::Skill => "skill",
            Self::Session => "session",
            Self::Artifact => "artifact",
        }
    }

    /// Parse from the wire token. Returns `None` for unknown tokens
    /// rather than erroring — the handler treats unknown filters as
    /// "no filter" so future kinds don't break existing clients.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "view" => Some(Self::View),
            "collection" => Some(Self::Collection),
            "collection_item" => Some(Self::CollectionItem),
            "agent" => Some(Self::Agent),
            "skill" => Some(Self::Skill),
            "session" => Some(Self::Session),
            "artifact" => Some(Self::Artifact),
            _ => None,
        }
    }
}

/// A single search hit. Score is computed by `ranking::score_hit`; the
/// frontend doesn't need to use it but the aggregator orders by it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHit {
    #[serde(flatten)]
    pub kind: EntityKind,
    pub id: String,
    pub label: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    pub score: f64,
}

/// Query options. `types` is the optional adapter allowlist;
/// `cursor` carries per-adapter pagination offsets.
#[derive(Clone, Debug, Default)]
pub struct SearchOpts {
    pub types: Option<Vec<EntityKindFilter>>,
    pub limit: usize,
    pub cursor: Option<Cursor>,
}

impl SearchOpts {
    pub fn limit_or_default(&self) -> usize {
        if self.limit == 0 { 20 } else { self.limit.min(200) }
    }
}

/// Per-adapter offset map. Keys are adapter names (`view`, `agent`, …);
/// values are the offset the adapter should resume from. Encoded as
/// base64-of-JSON on the wire.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cursor {
    #[serde(flatten)]
    pub per_adapter: HashMap<String, usize>,
}

impl Cursor {
    pub fn get(&self, adapter: &str) -> usize {
        self.per_adapter.get(adapter).copied().unwrap_or(0)
    }

    pub fn set(&mut self, adapter: &str, offset: usize) {
        self.per_adapter.insert(adapter.to_string(), offset);
    }

    pub fn is_empty(&self) -> bool {
        self.per_adapter.is_empty()
    }

    /// Encode to the opaque base64-of-JSON wire form.
    pub fn encode(&self) -> String {
        use base64::Engine;
        let json = serde_json::to_string(&self.per_adapter).unwrap_or_default();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
    }

    /// Decode from the wire form. Returns `None` for malformed input.
    pub fn decode(s: &str) -> Option<Self> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .ok()?;
        let map: HashMap<String, usize> = serde_json::from_slice(&bytes).ok()?;
        Some(Self { per_adapter: map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_roundtrip() {
        let mut c = Cursor::default();
        c.set("view", 10);
        c.set("agent", 5);
        let encoded = c.encode();
        let decoded = Cursor::decode(&encoded).unwrap();
        assert_eq!(decoded.get("view"), 10);
        assert_eq!(decoded.get("agent"), 5);
    }

    #[test]
    fn cursor_decode_malformed_returns_none() {
        assert!(Cursor::decode("!!!not-base64!!!").is_none());
    }

    #[test]
    fn kind_filter_parse_unknown_returns_none() {
        assert!(EntityKindFilter::parse("nonsense").is_none());
    }
}
