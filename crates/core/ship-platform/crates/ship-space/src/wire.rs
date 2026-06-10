//! ConnectRPC wire layer for `SpaceService`.
//!
//! `SpaceWire<F, G>` wraps the service with two per-request factories:
//! - `make_ctx` builds the Runtime-scoped `SpaceContext` (registry + kernel).
//! - `resolve_space` resolves the per-request *target space* — `(space_id,
//!   per-space DataStore, global settings snapshot)` — for the Space-scoped reads
//!   (`Info`) and writes (`SetSetting`). The daemon owns the global snapshot
//!   source + the per-space store factory; this keeps the daemon thin while the
//!   overlay composition lives in the crate.
//!
//! It implements the generated `ship_ops_rpc::SpaceService` trait so shipd can
//! mount it via `SpaceServiceExt::register`.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::ports::{DataStore, KernelLog, SpaceContext};

use ship_ops_rpc::{
    ActiveSpace, Space, SpaceInfo, SpaceList, SpaceService as SpaceServiceTrait,
    SpaceSettingValue, WatchEvent,
};

use crate::service::SpaceService;
use crate::types::SpaceRecord;

/// The per-request target-space resolution the daemon supplies for the
/// Space-scoped RPCs: the resolved space id (None in global-only mode), the
/// per-space `DataStore` (binds the space's `space.db`), and the daemon-global
/// `settings.state` snapshot that is the overlay base.
pub struct TargetSpace {
    pub space_id: Option<String>,
    pub settings_store: Arc<dyn DataStore>,
    pub global_settings: serde_json::Value,
}

/// Wraps SpaceService with per-request context factories.
pub struct SpaceWire<F, G> {
    make_ctx: F,
    resolve_space: G,
    reactive: Arc<ReactiveRegistry>,
}

impl<F, G> SpaceWire<F, G>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
    G: Fn(&RequestContext) -> TargetSpace + Send + Sync + 'static,
{
    pub fn new(kernel: Arc<KernelLog>, make_ctx: F, resolve_space: G) -> Self {
        Self {
            make_ctx,
            resolve_space,
            reactive: Arc::new(ReactiveRegistry::new(kernel)),
        }
    }

    fn make_svc(&self, rctx: &RequestContext) -> SpaceService {
        SpaceService { ctx: (self.make_ctx)(rctx) }
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
impl<F, G> SpaceServiceTrait for SpaceWire<F, G>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
    G: Fn(&RequestContext) -> TargetSpace + Send + Sync + 'static,
{
    async fn list(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceListReqView,
    ) -> ServiceResult<SpaceList> {
        let svc = self.make_svc(&ctx);
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
        let svc = self.make_svc(&ctx);
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
        let svc = self.make_svc(&ctx);
        let target = (self.resolve_space)(&ctx);
        let composed = svc
            .info(
                target.settings_store.as_ref(),
                target.space_id.as_deref(),
                target.global_settings,
            )
            .await
            .map_err(ConnectError::from)?;
        let settings_json = serde_json::to_string(&composed.settings)
            .map_err(|e| ConnectError::internal(e.to_string()))?;
        Ok(Response::new(SpaceInfo {
            space_id: composed.space_id,
            root: composed.root,
            name: composed.name,
            settings_json,
            ..Default::default()
        }))
    }

    async fn add(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedAddSpaceReqView,
    ) -> ServiceResult<Space> {
        let svc = self.make_svc(&ctx);
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
        let svc = self.make_svc(&ctx);
        let record = svc.activate(request.id).await.map_err(ConnectError::from)?;
        Ok(Response::new(ActiveSpace {
            active: Some(record_to_proto(record)).into(),
            ..Default::default()
        }))
    }

    async fn set_setting(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSpaceSettingReqView,
    ) -> ServiceResult<SpaceSettingValue> {
        let svc = self.make_svc(&ctx);
        let target = (self.resolve_space)(&ctx);
        let space_id = target.space_id.ok_or_else(|| {
            ConnectError::failed_precondition("no space bound for this connection")
        })?;
        let out = svc
            .set_setting(
                target.settings_store.as_ref(),
                &space_id,
                request.key,
                request.value_json,
            )
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(SpaceSettingValue {
            key: out.key,
            value_json: out.value_json,
            ..Default::default()
        }))
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSpaceWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let ctx_arc = self.make_svc(&ctx).ctx.clone();
        let space_id = ctx_arc.space_id().to_string();
        let owned = request.to_owned_message();

        let (scope, prefix) = match owned.scope {
            Some(ship_ops_rpc::__buffa::oneof::space_watch_req::Scope::Registry(_)) => {
                ("registry".to_string(), "project.")
            }
            Some(ship_ops_rpc::__buffa::oneof::space_watch_req::Scope::Space(id)) => {
                (format!("space:{id}"), "space.setting.")
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

/// Snapshot the registry (list + active pointer) for the Watch root frame. The
/// per-space settings overlay snapshot is intentionally registry-shaped here —
/// the daemon's per-space store is resolved per request, not per watcher, so the
/// reactive root carries the registry view; the FE re-fetches `Info` on a frame.
async fn registry_snapshot(ctx: Arc<SpaceContext>) -> serde_json::Value {
    let spaces = crate::registry::list(ctx.kernel()).await.unwrap_or_default();
    let active = crate::projection::get_active_id(ctx.data()).await.ok().flatten();
    serde_json::json!({
        "spaces": spaces.iter().map(|s| serde_json::json!({
            "id": s.id, "path": s.path, "name": s.name,
            "created_at": s.created_at, "last_seen_at": s.last_seen_at,
        })).collect::<Vec<_>>(),
        "active_id": active,
    })
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
