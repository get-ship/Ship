//! `CollectionsService` handler — proto-as-SSOT migration of `collections.*`
//! (#89), mounted additively beside the generated ReadOps/CommandOps until the
//! GUI flip (#72).
//!
//! Collections are the data plane (sqlite-backed). Reads are C0 and carry opaque
//! JSON, so they route through the sync read registry and wrap the result in
//! `JsonResponse`. Writes are C1 and route through `services::command::call`
//! (the ONE gate); item writes decode the typed `CollectionItem` / `Deletion`,
//! schema writes stay `JsonResponse`.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use ship_ops_rpc::{CollectionItem, CollectionsService, Deletion, JsonResponse};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

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

    async fn get_item(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetCollectionItemReqView,
    ) -> ServiceResult<JsonResponse> {
        // No read-registry op exists for the single-item GET, so call the
        // platform service directly (as the old REST handler did) and mirror
        // its `{ ok, data }` envelope. NotFound maps to the Connect not_found.
        use ship_platform::services::collections::service::Error;
        let item = self
            .state()
            .collections()
            .get_item(request.collection, request.id)
            .await
            .map_err(|e| match e {
                Error::NotFound(m) => ConnectError::not_found(m),
                Error::Validation(m) => ConnectError::invalid_argument(m),
                other => ConnectError::internal(other.to_string()),
            })?;
        let value = json!({ "ok": true, "data": item });
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn create_item(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateCollectionItemRequestView,
    ) -> ServiceResult<CollectionItem> {
        let args = json!({
            "collection": request.collection,
            "data_json": request.data_json,
            "id": request.id,
        });
        self.collections_typed_write("collections.create_item", args, &ctx).await
    }

    async fn update_item(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedUpdateCollectionItemRequestView,
    ) -> ServiceResult<CollectionItem> {
        let args = json!({
            "collection": request.collection,
            "id": request.id,
            "data_json": request.data_json,
        });
        self.collections_typed_write("collections.update_item", args, &ctx).await
    }

    async fn delete_item(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteCollectionItemRequestView,
    ) -> ServiceResult<Deletion> {
        let args = json!({ "collection": request.collection, "id": request.id });
        self.collections_typed_write("collections.delete_item", args, &ctx).await
    }

    async fn create_schema(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateCollectionSchemaRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "schema_json": request.schema_json });
        self.collections_json_write("collections.create_schema", args, &ctx).await
    }

    async fn update_schema(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedUpdateCollectionSchemaRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "collection": request.collection, "schema_json": request.schema_json });
        self.collections_json_write("collections.update_schema", args, &ctx).await
    }
}

impl OpsImpl {
    /// C0 read → sync registry dispatch, wrapped as opaque JSON.
    fn collections_read(&self, op: &str, args: Value) -> ServiceResult<JsonResponse> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    /// C1 write → the one gate, decoding the typed result (`CollectionItem` /
    /// `Deletion`).
    async fn collections_typed_write<T: DeserializeOwned>(
        &self,
        op: &str,
        args: Value,
        ctx: &RequestContext,
    ) -> ServiceResult<T> {
        let value = self.collections_call(op, args, ctx).await?;
        let msg = serde_json::from_value::<T>(value)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))?;
        Ok(Response::new(msg))
    }

    /// C1 write whose result is opaque JSON (schema writes).
    async fn collections_json_write(
        &self,
        op: &str,
        args: Value,
        ctx: &RequestContext,
    ) -> ServiceResult<JsonResponse> {
        let value = self.collections_call(op, args, ctx).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    /// Shared gated dispatch for collections writes.
    async fn collections_call(
        &self,
        op: &str,
        args: Value,
        ctx: &RequestContext,
    ) -> Result<Value, ConnectError> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)
    }
}
