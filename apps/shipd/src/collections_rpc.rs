//! `CollectionsService` handler — proto-as-SSOT migration of `collections.*`
//! (#89), mounted additively beside the generated ReadOps/CommandOps until the
//! GUI flip (#72).
//!
//! Writes (C1) gate inline via `OpsImpl::gate` (proto policy + `gate::decide`)
//! and call the platform `ship-collection` crate directly — no more
//! `services::command::call` string dispatch. The typed `CollectionItem` /
//! `Deletion` responses are built from the crate's return, never round-tripped
//! through `serde_json::from_value` on the apparatus path.
//!
//! Reads (C0) still route through the sync read registry: the four read ops
//! (`collections.state/list/detail/items`) compose view-shapes here in shipd
//! (built-in + file-overlay schema merge, per-collection status summary, etc.)
//! that the platform crate doesn't project. The read-side registry stays as
//! the assembly layer for that composition; deleting the slice would force
//! the composition into the handler and lose the daemon-local projection
//! boundary. See job-spec: "flag it, don't force a bad move".

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{CollectionItem, CollectionsService, Deletion, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl CollectionsService for OpsImpl {
    async fn state(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionsStateRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.collections_read("collections.state", json!({ "active": request.active }))
    }

    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListCollectionsRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.collections_read("collections.list", Value::Null)
    }

    #[allow(refining_impl_trait)]
    async fn list_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListCollectionsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("collections.list", json!({})).await
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionDetailRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.collections_read("collections.detail", json!({ "id": request.id }))
    }

    async fn items(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionItemsRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.collections_read("collections.items", json!({ "collection": request.collection }))
    }

    #[allow(refining_impl_trait)]
    async fn state_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionsStateRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("collections.state", json!({ "active": request.active })).await
    }

    #[allow(refining_impl_trait)]
    async fn detail_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionDetailRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("collections.detail", json!({ "id": request.id })).await
    }

    #[allow(refining_impl_trait)]
    async fn items_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionItemsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("collections.items", json!({ "collection": request.collection })).await
    }

    async fn get_item(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionItemReqView,
    ) -> ServiceResult<JsonResponse> {
        // No read-registry op exists for the single-item GET, so call the
        // platform service directly and mirror its `{ ok, data }` envelope.
        // The service error maps to Connect codes.
        let svc = self
            .state()
            .collections()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        let item = svc
            .get_item(request.collection, request.id)
            .await
            .map_err(ConnectError::from)?;
        let value = json!({ "ok": true, "data": item });
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn create_item(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateCollectionItemRequestView,
    ) -> ServiceResult<CollectionItem> {
        self.gate(&ctx, "collections.create_item")?;
        let svc = self
            .state()
            .collections()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        // The service mints the item id; the caller's `id` is ignored (kept on
        // the wire for protocol compatibility, matches the prior `services::command::collections`).
        let data = parse_data(request.data_json, "data_json")?;
        let item = svc
            .create_item(request.collection, &data)
            .await
            .map_err(collections_err)?;
        Ok(Response::new(item_record_to_proto(&item)))
    }

    async fn update_item(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedUpdateCollectionItemRequestView,
    ) -> ServiceResult<CollectionItem> {
        self.gate(&ctx, "collections.update_item")?;
        let svc = self
            .state()
            .collections()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        let data = parse_data(request.data_json, "data_json")?;
        let item = svc
            .update_item(request.collection, request.id, &data)
            .await
            .map_err(collections_err)?;
        Ok(Response::new(item_record_to_proto(&item)))
    }

    async fn delete_item(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteCollectionItemRequestView,
    ) -> ServiceResult<Deletion> {
        self.gate(&ctx, "collections.delete_item")?;
        let svc = self
            .state()
            .collections()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        svc.delete_item(request.collection, request.id)
            .await
            .map_err(collections_err)?;
        Ok(Response::new(Deletion { id: request.id.to_string(), deleted: true, ..Default::default() }))
    }

    async fn create_schema(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateCollectionSchemaRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.gate(&ctx, "collections.create_schema")?;
        let svc = self
            .state()
            .collections()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        // Parse to recover the id; the service re-parses + persists.
        let doc = ship_collection::schema::parse_schema_doc(request.schema_json.as_bytes())
            .map_err(collections_err)?;
        let id = doc.id.clone();
        svc.create_schema(&id, request.schema_json.as_bytes())
            .await
            .map_err(collections_err)?;
        let value = json!({ "ok": true, "collection": id });
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn update_schema(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedUpdateCollectionSchemaRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.gate(&ctx, "collections.update_schema")?;
        let svc = self
            .state()
            .collections()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        svc.update_schema(request.collection, request.schema_json.as_bytes())
            .await
            .map_err(collections_err)?;
        let value = json!({ "ok": true, "collection": request.collection });
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

impl OpsImpl {
    /// C0 read → sync registry dispatch, wrapped as opaque JSON. The read-side
    /// registry stays — see module doc on the composition it owns.
    fn collections_read(&self, op: &str, args: Value) -> ServiceResult<JsonResponse> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

/// Validate a JSON-string arg and pass the bytes through.
fn parse_data(raw: &str, key: &str) -> Result<Vec<u8>, ConnectError> {
    serde_json::from_str::<Value>(raw)
        .map_err(|e| ConnectError::invalid_argument(format!("`{key}` is not valid JSON: {e}")))?;
    Ok(raw.as_bytes().to_vec())
}

/// `CollectionError` → `ConnectError` (not-found vs bad-args vs internal).
fn collections_err(e: ship_collection::error::CollectionError) -> ConnectError {
    use ship_collection::error::CollectionError;
    match e {
        CollectionError::NotFound(m) => ConnectError::not_found(m),
        CollectionError::ReservedId(m)
        | CollectionError::Validation(m)
        | CollectionError::UnsupportedQuery(m) => ConnectError::invalid_argument(m),
        other => ConnectError::internal(other.to_string()),
    }
}

/// Project a `ItemRecord` (the crate's typed row) into the wire `CollectionItem`.
/// `data_json` is the `serde_json::Value::Object` re-serialized as a string.
fn item_record_to_proto(item: &ship_collection::types::ItemRecord) -> CollectionItem {
    let data_json = serde_json::Value::Object(item.data.clone()).to_string();
    CollectionItem {
        id: item.id.clone(),
        collection: item.collection.clone(),
        data_json,
        created_by: None,
        created_at: item.created_at.clone(),
        updated_at: item.updated_at.clone(),
        ..Default::default()
    }
}
