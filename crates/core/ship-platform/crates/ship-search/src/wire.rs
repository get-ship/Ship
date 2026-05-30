//! ConnectRPC wire layer for `SearchService`.
//!
//! ## Context approach
//!
//! `ship_ops_rpc::SearchService` trait methods only receive a `RequestContext`
//! (the Connect call envelope), not a `SearchContext` (the per-request scope
//! port). Rather than injecting `SearchContext` through axum request extensions
//! (which couples this crate to axum's extract machinery), we wrap
//! `SearchService` in `SearchWire<F>` where `F` is a factory closure
//! `Fn(&RequestContext) -> C`. shipd constructs `SearchWire` once with a factory
//! that captures `ApiState` and builds `ShipdSearchContext` per call. Tests can
//! supply an in-memory factory. The factory is synchronous because
//! `ShipdSearchContext` captures a runtime reference and defers all I/O to the
//! lazy `SearchContext` trait calls.

use connectrpc::{RequestContext, Response, ServiceResult};

use ship_ops_rpc::{
    EntityKind as ProtoEntityKind, OwnedSearchRequestView, SearchHit as ProtoSearchHit,
    SearchResponse as ProtoSearchResponse,
};

use crate::service::{SearchContext, SearchService, sanitize_query};
use crate::types::{Cursor, EntityKind, EntityKindFilter, SearchOpts};

/// Wraps `SearchService` with a per-request context factory. Implements the
/// generated `ship_ops_rpc::SearchService` trait so shipd can mount it via
/// `SearchServiceExt::register`.
pub struct SearchWire<F> {
    service: SearchService,
    make_ctx: F,
}

impl<F> SearchWire<F> {
    pub fn new(make_ctx: F) -> Self {
        Self { service: SearchService::new(), make_ctx }
    }
}

#[allow(refining_impl_trait)]
impl<F, C> ship_ops_rpc::SearchService for SearchWire<F>
where
    F: Fn(&RequestContext) -> C + Send + Sync + 'static,
    C: SearchContext + Send + Sync + 'static,
{
    async fn search(
        &self,
        ctx: RequestContext,
        request: OwnedSearchRequestView,
    ) -> ServiceResult<ProtoSearchResponse> {
        let search_ctx = (self.make_ctx)(&ctx);

        let q = sanitize_query(&request.q);

        let kinds: Option<Vec<EntityKindFilter>> = if request.kinds.is_empty() {
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
        let opts = SearchOpts { types: kinds, limit, cursor };

        let result = self.service.run(&search_ctx, &q, opts).await;

        let hits = result.hits.into_iter().map(platform_hit_to_proto).collect();
        let next_cursor = result.cursor.unwrap_or_default();

        Ok(Response::new(ProtoSearchResponse { hits, next_cursor, ..Default::default() }))
    }
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

fn platform_hit_to_proto(hit: crate::types::SearchHit) -> ProtoSearchHit {
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
        ..Default::default()
    }
}
