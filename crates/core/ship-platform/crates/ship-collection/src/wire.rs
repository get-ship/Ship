//! ConnectRPC wire layer for `CollectionsService`.
//!
//! `CollectionWire<F>` wraps the service with a per-request context factory.
//! It implements the generated `ship_proto::ship::ops::v1::CollectionsService` trait so
//! shipd can mount it via `CollectionsServiceExt::register`.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::{KernelLog, SpaceContext};

use ship_proto::ship::ops::v1::{CollectionDetail, CollectionList, CollectionSummary, CollectionsService as CollectionServiceTrait, Deletion, Item, ItemPage, WatchEvent};

use crate::error::CollectionError;
use crate::patch::{patch_frame_to_proto, snapshot_for_scope};
use crate::projection::SchemaRegistry;
use crate::service::CollectionService;
use crate::types::ItemRecord;

/// Wraps CollectionService with a per-request context factory.
pub struct CollectionWire<F> {
    make_ctx: F,
    schemas: Arc<SchemaRegistry>,
    reactive: Arc<ReactiveRegistry>,
}

impl<F> CollectionWire<F>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
{
    pub fn new(kernel: Arc<KernelLog>, make_ctx: F) -> Self {
        Self::with_schemas(kernel, Arc::new(SchemaRegistry::new()), make_ctx)
    }

    /// Construct with a caller-owned `SchemaRegistry` so the daemon can hydrate
    /// the file-overlay schemas once and share ONE registry instance across the
    /// wire and the internal (trigger / view-authoring) service callers — a
    /// schema created over the wire must be visible to a trigger insert.
    pub fn with_schemas(kernel: Arc<KernelLog>, schemas: Arc<SchemaRegistry>, make_ctx: F) -> Self {
        Self {
            make_ctx,
            schemas,
            reactive: Arc::new(ReactiveRegistry::new(kernel)),
        }
    }

    fn make_svc(&self, rctx: &RequestContext) -> CollectionService {
        CollectionService {
            ctx: (self.make_ctx)(rctx),
            schemas: self.schemas.clone(),
        }
    }
}

fn item_to_proto(r: ItemRecord) -> Item {
    let data_json = serde_json::to_vec(&r.data).unwrap_or_default();
    Item {
        id: r.id,
        collection: r.collection,
        data_json,
        created_at: r.created_at,
        updated_at: r.updated_at,
        ..Default::default()
    }
}

fn schema_to_detail(schema: &crate::types::SchemaDoc, item_count: usize) -> CollectionDetail {
    let schema_json = serde_json::to_vec(schema).unwrap_or_default();
    CollectionDetail {
        id: schema.id.clone(),
        schema_json,
        item_count: item_count as u32,
        ..Default::default()
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl<F> CollectionServiceTrait for CollectionWire<F>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
{
    async fn list(
        &self,
        ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedEmptyView,
    ) -> ServiceResult<CollectionList> {
        let svc = self.make_svc(&ctx);
        let schemas = svc.list();
        let mut collections = Vec::with_capacity(schemas.len());
        for s in schemas {
            // Match `get`/`update` — count via the projection-backed
            // `DataStore::count` rather than hardcoding 0. Sequential awaits
            // are fine: the summary list is short (built-in + user schemas)
            // and `count_items_for_wire` falls back to 0 on error, so a
            // transient read never breaks the list response.
            let count = count_items_for_wire(&svc, &s.id).await;
            collections.push(CollectionSummary {
                id: s.id,
                title: s.title.unwrap_or_default(),
                item_count: count as u32,
                system: s.system,
                ..Default::default()
            });
        }
        Ok(Response::new(CollectionList {
            collections,
            ..Default::default()
        }))
    }

    async fn get(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetReqView,
    ) -> ServiceResult<CollectionDetail> {
        let svc = self.make_svc(&ctx);
        let id = request.id;
        let (schema, count) = svc
            .get(id)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        Ok(Response::new(schema_to_detail(&schema, count)))
    }

    async fn get_item(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetItemReqView,
    ) -> ServiceResult<Item> {
        let svc = self.make_svc(&ctx);
        let item = svc
            .get_item(request.collection, request.id)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        Ok(Response::new(item_to_proto(item)))
    }

    async fn query_items(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedQueryReqView,
    ) -> ServiceResult<ItemPage> {
        let svc = self.make_svc(&ctx);
        let cursor = if request.cursor.is_empty() {
            None
        } else {
            Some(request.cursor.to_string())
        };
        let limit = request.limit;

        let owned = request.to_owned_message();
        let (items, next_cursor) = match owned.query {
            Some(ship_proto::ship::ops::v1::__buffa::oneof::query_req::Query::NamedQuery(name)) => {
                return Err(CollectionError::NamedQueryNotFound(name).into());
            }
            Some(ship_proto::ship::ops::v1::__buffa::oneof::query_req::Query::InlineAst(ast_bytes)) => svc
                .query_items_ast(&ast_bytes, cursor, limit)
                .await
                .map_err(|e: CollectionError| ConnectError::from(e))?,
            None => {
                return Err(ConnectError::invalid_argument(
                    "query must be set (named_query or inline_ast)",
                ));
            }
        };

        let proto_items = items.into_iter().map(item_to_proto).collect();
        Ok(Response::new(ItemPage {
            items: proto_items,
            next_cursor: next_cursor.unwrap_or_default(),
            ..Default::default()
        }))
    }

    async fn create_item(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedCreateItemReqView,
    ) -> ServiceResult<Item> {
        let svc = self.make_svc(&ctx);
        let item = svc
            .create_item(request.collection, request.data_json)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        Ok(Response::new(item_to_proto(item)))
    }

    async fn update_item(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedUpdateItemReqView,
    ) -> ServiceResult<Item> {
        let svc = self.make_svc(&ctx);
        let item = svc
            .update_item(request.collection, request.id, request.data_json)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        Ok(Response::new(item_to_proto(item)))
    }

    async fn delete_item(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedDeleteItemReqView,
    ) -> ServiceResult<Deletion> {
        let svc = self.make_svc(&ctx);
        svc.delete_item(request.collection, request.id)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        Ok(Response::new(Deletion {
            id: request.id.to_string(),
            ..Default::default()
        }))
    }

    async fn create_schema(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedCreateSchemaReqView,
    ) -> ServiceResult<CollectionDetail> {
        let svc = self.make_svc(&ctx);
        let doc = svc
            .create_schema(request.id, request.schema_json)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        Ok(Response::new(schema_to_detail(&doc, 0)))
    }

    async fn update_schema(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedUpdateSchemaReqView,
    ) -> ServiceResult<CollectionDetail> {
        let svc = self.make_svc(&ctx);
        let id = request.id.to_string();
        let doc = svc
            .update_schema(&id, request.schema_json)
            .await
            .map_err(|e: CollectionError| ConnectError::from(e))?;
        let count = count_items_for_wire(&svc, &id).await;
        Ok(Response::new(schema_to_detail(&doc, count)))
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let space_id = self.make_svc(&ctx).ctx.space_id().to_string();
        let owned = request.to_owned_message();

        let scope = match owned.scope {
            Some(ship_proto::ship::ops::v1::__buffa::oneof::watch_req::Scope::Collections(_)) => {
                "collections".to_string()
            }
            Some(ship_proto::ship::ops::v1::__buffa::oneof::watch_req::Scope::Collection(id)) => {
                format!("collection:{id}")
            }
            Some(ship_proto::ship::ops::v1::__buffa::oneof::watch_req::Scope::Item(item_ref)) => {
                format!("item:{}:{}", item_ref.collection, item_ref.id)
            }
            None => {
                return Err(ConnectError::invalid_argument("watch scope is required"));
            }
        };

        let query_hash: u64 = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            owned.query_json.hash(&mut h);
            h.finish()
        };

        let key = WatchKey {
            space_id,
            scope: scope.clone(),
            query_hash,
        };

        let ctx_arc = self.make_svc(&ctx).ctx.clone();
        let schemas = self.schemas.clone();
        let scope_clone = scope.clone();

        let spec = ResolverSpec {
            prefilter: Box::new(move |env| env.event_type.starts_with("collection.")),
            snapshot: Box::new(move || {
                let ctx = ctx_arc.clone();
                let schemas = schemas.clone();
                let scope = scope_clone.clone();
                Box::pin(async move { snapshot_for_scope(ctx, schemas, &scope).await })
            }),
        };

        let filter = EventFilter::default();
        let mut rx = self.reactive.watch(key, filter, spec);

        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(frame) => yield Ok(patch_frame_to_proto(frame)),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }
}

async fn count_items_for_wire(svc: &CollectionService, id: &str) -> usize {
    crate::projection::count_items(svc.ctx.data(), id)
        .await
        .unwrap_or(0)
}
