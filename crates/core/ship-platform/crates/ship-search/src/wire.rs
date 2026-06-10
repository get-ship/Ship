//! ConnectRPC wire layer for `SearchService`.
//!
//! ## Context approach
//!
//! `ship_ops_rpc::SearchService` trait methods only receive a `RequestContext`
//! (the Connect call envelope), not a `SearchContext` (the per-request scope
//! port). Rather than injecting `SearchContext` through axum request extensions
//! (which couples this crate to axum's extract machinery), we wrap
//! `SearchService` in `SearchWire<F>` where `F` is a factory closure
//! `Fn(&RequestContext) -> C` that builds one per-request context. Tests can
//! supply an in-memory factory. The factory is synchronous because the
//! context defers all I/O to the lazy `SearchContext` trait calls.
//!
//! `SearchWire<F>` is the SINGLE-CONTEXT (this-space) wire — it runs exactly
//! one context and ignores `scope`. The GLOBAL fan-out needs the space
//! registry + per-space contexts, which live in shipd, so shipd owns a wire
//! that branches on scope and calls [`crate::global::run_global`]. Request
//! parsing ([`parse_request`]) and hit→proto mapping ([`hit_to_proto`]) are
//! shared by both wires so the boundary logic lives once.

use ship_ops_rpc::{
    EntityKind as ProtoEntityKind, OwnedSearchRequestView, SearchHit as ProtoSearchHit,
    SearchScope as ProtoSearchScope,
};

use crate::service::sanitize_query;
use crate::types::{Cursor, EntityKind, EntityKindFilter, SearchOpts};

/// Resolved search reach. `Unspecified` is normalized to `ThisSpace` here so
/// downstream context selection only ever branches on the two real cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchScope {
    ThisSpace,
    Global,
}

impl SearchScope {
    /// Map the proto scope to the resolved reach. `UNSPECIFIED` defaults to
    /// `ThisSpace` (the safe, historical behavior).
    pub fn from_proto(scope: ProtoSearchScope) -> Self {
        match scope {
            ProtoSearchScope::SEARCH_SCOPE_GLOBAL => Self::Global,
            ProtoSearchScope::SEARCH_SCOPE_THIS_SPACE
            | ProtoSearchScope::SEARCH_SCOPE_UNSPECIFIED => Self::ThisSpace,
        }
    }
}

/// A `SearchRequest` decoded into the platform's domain shapes. Shared by
/// shipd's global-fan-out wire so request parsing
/// (sanitization, kind filtering, cursor decode, scope resolution) lives once.
pub struct ParsedSearchRequest {
    pub query: String,
    pub opts: SearchOpts,
    pub scope: SearchScope,
}

/// Decode an `OwnedSearchRequestView` into [`ParsedSearchRequest`].
pub fn parse_request(request: &OwnedSearchRequestView) -> ParsedSearchRequest {
    let query = sanitize_query(&request.q);

    let types: Option<Vec<EntityKindFilter>> = if request.kinds.is_empty() {
        None
    } else {
        Some(
            request
                .kinds
                .iter()
                .filter_map(|k| k.as_known().and_then(proto_kind_to_filter))
                .collect(),
        )
    };

    let cursor = if request.cursor.is_empty() {
        None
    } else {
        Cursor::decode(&request.cursor)
    };

    let limit = if request.limit == 0 { 20 } else { request.limit as usize };
    let scope = request
        .scope
        .as_known()
        .map(SearchScope::from_proto)
        .unwrap_or(SearchScope::ThisSpace);

    ParsedSearchRequest { query, opts: SearchOpts { types, limit, cursor }, scope }
}

fn proto_kind_to_filter(k: ProtoEntityKind) -> Option<EntityKindFilter> {
    match k {
        ProtoEntityKind::ENTITY_KIND_VIEW => Some(EntityKindFilter::View),
        ProtoEntityKind::ENTITY_KIND_COLLECTION => Some(EntityKindFilter::Collection),
        ProtoEntityKind::ENTITY_KIND_COLLECTION_ITEM => Some(EntityKindFilter::CollectionItem),
        ProtoEntityKind::ENTITY_KIND_AGENT => Some(EntityKindFilter::Agent),
        ProtoEntityKind::ENTITY_KIND_SKILL => Some(EntityKindFilter::Skill),
        ProtoEntityKind::ENTITY_KIND_SESSION => Some(EntityKindFilter::Session),
        ProtoEntityKind::ENTITY_KIND_ARTIFACT => Some(EntityKindFilter::Artifact),
        ProtoEntityKind::ENTITY_KIND_UNSPECIFIED => None,
    }
}

/// Map a platform `SearchHit` to its proto form. Public so shipd's global
/// fan-out wire maps merged hits identically (single mapping, one boundary).
/// `space_id` rides through — empty for this-space hits, set by the global
/// merge — so the client can compose the `ship://space/<id><path>` deeplink.
pub fn hit_to_proto(hit: crate::types::SearchHit) -> ProtoSearchHit {
    let (kind, collection_id) = match &hit.kind {
        EntityKind::View => (ProtoEntityKind::ENTITY_KIND_VIEW, String::new()),
        EntityKind::Collection => (ProtoEntityKind::ENTITY_KIND_COLLECTION, String::new()),
        EntityKind::CollectionItem { collection_id } => {
            (ProtoEntityKind::ENTITY_KIND_COLLECTION_ITEM, collection_id.clone())
        }
        EntityKind::Agent => (ProtoEntityKind::ENTITY_KIND_AGENT, String::new()),
        EntityKind::Skill => (ProtoEntityKind::ENTITY_KIND_SKILL, String::new()),
        EntityKind::Session => (ProtoEntityKind::ENTITY_KIND_SESSION, String::new()),
        EntityKind::Artifact => (ProtoEntityKind::ENTITY_KIND_ARTIFACT, String::new()),
    };
    ProtoSearchHit {
        kind: buffa::EnumValue::from(kind),
        id: hit.id,
        label: hit.label,
        path: hit.path,
        icon: hit.icon.unwrap_or_default(),
        snippet: hit.snippet.unwrap_or_default(),
        score: hit.score,
        collection_id,
        space_id: hit.space_id,
        ..Default::default()
    }
}
