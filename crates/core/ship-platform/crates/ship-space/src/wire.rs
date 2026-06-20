//! ConnectRPC wire layer for `SpaceService`.
//!
//! `SpaceWire<F>` wraps the service with one per-request factory:
//! - `make_ctx` builds the Runtime-scoped `SpaceContext` (registry + kernel).
//!
//! It implements the generated `ship_ops_rpc::SpaceService` trait so shipd can
//! mount it via `SpaceServiceExt::register`.
//!
//! Per-space settings live on the unified `settings_overlay` (S4) and are
//! served by `SettingsService`. The `SpaceService.Watch` stream covers the
//! registry (list + active pointer) AND, on its `space` variant, per-space
//! `settings.changed` events. The per-space reactive prefix was repointed
//! from the legacy `space.setting.` to `settings.changed` in S4.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::ports::{KernelLog, SpaceContext};

use ship_ops_rpc::{
    ActiveSpace, Space, SpaceInfo, SpaceList, SpaceService as SpaceServiceTrait, WatchEvent,
};

use crate::service::SpaceService;
use crate::types::SpaceRecord;

/// Wraps SpaceService with the per-request Runtime-scoped context factory.
pub struct SpaceWire<F> {
    make_ctx: F,
    reactive: Arc<ReactiveRegistry>,
}

impl<F> SpaceWire<F>
where
    F: Fn(&RequestContext) -> Result<Arc<SpaceContext>, ConnectError> + Send + Sync + 'static,
{
    pub fn new(kernel: Arc<KernelLog>, make_ctx: F) -> Self {
        Self {
            make_ctx,
            reactive: Arc::new(ReactiveRegistry::new(kernel)),
        }
    }

    fn make_svc(&self, rctx: &RequestContext) -> Result<SpaceService, ConnectError> {
        Ok(SpaceService { ctx: (self.make_ctx)(rctx)? })
    }
}

fn record_to_proto(r: SpaceRecord) -> Space {
    Space {
        id: r.id,
        path: r.path,
        name: r.name,
        created_at: r.created_at,
        last_seen_at: r.last_seen_at,
        ..Default::default()
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl<F> SpaceServiceTrait for SpaceWire<F>
where
    F: Fn(&RequestContext) -> Result<Arc<SpaceContext>, ConnectError> + Send + Sync + 'static,
{
    async fn list(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceListReqView,
    ) -> ServiceResult<SpaceList> {
        let svc = self.make_svc(&ctx)?;
        let spaces = svc.list().await.map_err(ConnectError::from)?;
        let active_id = svc.active_id().await.map_err(ConnectError::from)?;
        Ok(Response::new(SpaceList {
            spaces: spaces.into_iter().map(record_to_proto).collect(),
            active_id,
            ..Default::default()
        }))
    }

    async fn active(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceActiveReqView,
    ) -> ServiceResult<ActiveSpace> {
        let svc = self.make_svc(&ctx)?;
        let active = svc.active().await.map_err(ConnectError::from)?;
        Ok(Response::new(ActiveSpace {
            active: active.map(record_to_proto).into(),
            ..Default::default()
        }))
    }

    async fn info(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceInfoReqView,
    ) -> ServiceResult<SpaceInfo> {
        let svc = self.make_svc(&ctx)?;
        let bound = svc.ctx.space_id().as_str();
        let composed = svc
            .info(if bound.is_empty() { None } else { Some(bound) })
            .await
        .map_err(ConnectError::from)?;

        Ok(Response::new(SpaceInfo {
            space_id: composed.space_id,
            root: composed.root,
            name: composed.name,
            ..Default::default()
        }))
    }

    async fn add(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedAddSpaceReqView,
    ) -> ServiceResult<Space> {
        let svc = self.make_svc(&ctx)?;
        let path = std::path::Path::new(request.path);
        let name = if request.name.is_empty() { None } else { Some(request.name) };
        let record = svc.add(path, name).await.map_err(ConnectError::from)?;
        Ok(Response::new(record_to_proto(record)))
    }

    async fn activate(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedActivateSpaceReqView,
    ) -> ServiceResult<ActiveSpace> {
        let svc = self.make_svc(&ctx)?;
        let record = svc.activate(request.id).await.map_err(ConnectError::from)?;
        Ok(Response::new(ActiveSpace {
            active: Some(record_to_proto(record)).into(),
            ..Default::default()
        }))
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSpaceWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let ctx_arc = self.make_svc(&ctx)?.ctx.clone();
        let space_id = ctx_arc.space_id().to_string();
        let owned = request.to_owned_message();

        let (scope, prefix) = match owned.scope {
            Some(ship_ops_rpc::__buffa::oneof::space_watch_req::Scope::Registry(_)) => {
                ("registry".to_string(), "project.")
            }
            Some(ship_ops_rpc::__buffa::oneof::space_watch_req::Scope::Space(id)) => {
                // Per-space stream — `settings.changed` (Space scope) for that
                // space. The legacy `space.setting.` prefix was repointed in S4
                // to the unified `settings.changed` event.
                (format!("space:{id}"), "settings.")
            }
            None => {
                return Err(ConnectError::invalid_argument("watch scope is required"));
            }
        };

        let key = WatchKey { space_id, scope: scope.clone(), query_hash: 0 };
        let resolver_ctx = ctx_arc.clone();
        let spec = ResolverSpec {
            prefilter: Box::new(move |env| env.event_type.starts_with(prefix)),
            snapshot: Box::new(move || {
                let ctx = resolver_ctx.clone();
                Box::pin(async move { registry_snapshot(ctx).await })
            }),
        };

        let mut rx = self.reactive.watch(key, EventFilter::default(), spec);
        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(frame) => yield Ok(crate::wire::patch_frame_to_proto(frame)),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}

/// Snapshot the registry (list + active pointer) for the Watch root frame.
/// The per-space settings overlay snapshot was removed in S4 — settings changes
/// stream on `SettingsService.Watch`. The FE re-fetches `Info` on a frame.
async fn registry_snapshot(ctx: Arc<SpaceContext>) -> serde_json::Value {
    use serde_json::json;
    let svc = SpaceService { ctx };
    let spaces = svc.list().await.unwrap_or_default();
    let active_id = svc.active_id().await.unwrap_or_default();
    let spaces_json: Vec<_> = spaces
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "path": r.path,
                "name": r.name,
                "created_at": r.created_at,
                "last_seen_at": r.last_seen_at,
            })
        })
        .collect();
    json!({ "spaces": spaces_json, "active_id": active_id })
}

fn patch_op_to_proto(op: &ship_reactive::PatchOp) -> ship_ops_rpc::PatchOp {
    use ship_ops_rpc::PatchOp as P;
    use ship_reactive::PatchOp::*;
    match op {
        Set => P::PATCH_OP_SET,
        Merge => P::PATCH_OP_MERGE,
        Append => P::PATCH_OP_APPEND,
        Prepend => P::PATCH_OP_PREPEND,
        Insert => P::PATCH_OP_INSERT,
        Remove => P::PATCH_OP_REMOVE,
        Move => P::PATCH_OP_MOVE,
        Splice => P::PATCH_OP_SPLICE,
        Inc => P::PATCH_OP_INC,
    }
}

/// Map a reactive `PatchFrame` to the proto `WatchEvent`. Mirrors
/// ship-collection's `patch.rs::patch_frame_to_proto` (same reactive types).
pub(crate) fn patch_frame_to_proto(frame: ship_reactive::PatchFrame) -> WatchEvent {
    use ship_ops_rpc::{Patch as ProtoPatch, PatchFrame as ProtoFrame};
    let patches: Vec<ProtoPatch> = frame
        .patches
        .into_iter()
        .map(|p| ProtoPatch {
            op: buffa::EnumValue::from(patch_op_to_proto(&p.op)),
            path: p.path,
            value: p.value.map(|v| serde_json::to_vec(&v).unwrap_or_default()),
            index: p.index.unwrap_or(0) as i32,
            from: p.from.unwrap_or(0) as i32,
            to: p.to.unwrap_or(0) as i32,
            count: p.count.unwrap_or(0) as i32,
            by: p.by.unwrap_or(0.0),
            v: 1,
            ..Default::default()
        })
        .collect();
    WatchEvent {
        body: Some(ship_ops_rpc::__buffa::oneof::watch_event::Body::Patches(Box::new(
            ProtoFrame { patches, ..Default::default() },
        ))),
        ..Default::default()
    }
}

