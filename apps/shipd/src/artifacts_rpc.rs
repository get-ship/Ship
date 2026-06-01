//! `ArtifactsService` handler — proto-as-SSOT migration of `artifacts.*` (#89),
//! the typed-return variant of the recipe: writes return `ArtifactRecord` /
//! `Deletion`, not `ScopedMutation`. Mounted additively beside the generated
//! ReadOps/CommandOps until the GUI flip (#72).
//!
//! The C0 read routes through the sync read registry (`registry::call`, which
//! opens the data DB directly — the list query is sync). The C1 writes route
//! through `services::command::call`, the ONE gate, which also enforces the
//! per-artifact scope ACL (`can_write_artifacts` / `can_delete_artifacts`).
//! Byte-moving store/get are NOT here — blobs stay on serving routes.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde::de::DeserializeOwned;
use serde_json::json;

use ship_ops_rpc::{ArtifactRecord, ArtifactsListResponse, ArtifactsService, Deletion, WatchEvent};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl ArtifactsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListArtifactsRequestView,
    ) -> ServiceResult<ArtifactsListResponse> {
        let args = json!({
            "scope": request.scope,
            "mime": request.mime,
            "collection_id": request.collection_id,
            "item_id": request.item_id,
            "owner_actor": request.owner_actor,
            "content_hash": request.content_hash,
            "sort": request.sort,
            "limit": request.limit,
            "offset": request.offset,
        });
        let value = crate::services::registry::call("artifacts.list", &args, self.state())
            .map_err(|e| ConnectError::internal(format!("artifacts.list: {e}")))?;
        let msg = serde_json::from_value::<ArtifactsListResponse>(value)
            .map_err(|e| ConnectError::internal(format!("artifacts.list decode: {e}")))?;
        Ok(Response::new(msg))
    }

    #[allow(refining_impl_trait)]
    async fn watch(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedArtifactsWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        Err(ConnectError::unimplemented(
            "artifacts Watch: use the ship-artifacts service mount (registered after this one)",
        ))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchArtifactRequestView,
    ) -> ServiceResult<ArtifactRecord> {
        let args = json!({
            "id": request.id,
            "name": request.name,
            "owner_actor": request.owner_actor,
            "expires_at": request.expires_at,
        });
        self.gated_artifact_call("artifacts.patch", args, &ctx).await
    }

    async fn promote(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPromoteArtifactRequestView,
    ) -> ServiceResult<ArtifactRecord> {
        let args = json!({
            "id": request.id,
            "target_scope": request.target_scope,
            "run_id": request.run_id,
            "user_id": request.user_id,
        });
        self.gated_artifact_call("artifacts.promote", args, &ctx).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteArtifactRequestView,
    ) -> ServiceResult<Deletion> {
        let args = json!({ "id": request.id });
        self.gated_artifact_call("artifacts.delete", args, &ctx).await
    }
}

impl OpsImpl {
    /// Run an `artifacts.*` write through the one gate and decode its JSON
    /// projection onto `T` (`ArtifactRecord` for patch/promote, `Deletion` for
    /// delete). Generic over the return type — the typed-return analog of
    /// `rules_rpc`'s `gated_mutation`.
    async fn gated_artifact_call<T: DeserializeOwned>(
        &self,
        op: &str,
        args: serde_json::Value,
        ctx: &RequestContext,
    ) -> ServiceResult<T> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value = command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        let msg = serde_json::from_value::<T>(value)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))?;
        Ok(Response::new(msg))
    }
}
