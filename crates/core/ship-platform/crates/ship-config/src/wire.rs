//! ConnectRPC wire layer for the unified `SettingsService`.
//!
//! `SettingsWire<F>` wraps the scope-discriminated service with a per-request
//! context factory (mirroring `ship-collection`'s `CollectionWire`). The daemon
//! supplies the closure that resolves, per request: the shared kernel handle,
//! the per-account read-model store, the active-space (id + store), and the
//! caller's operator verdict. shipd does no product logic — it only marshals the
//! inputs the crate cannot own (the active-space resolution + the operator
//! verdict from `AuthContext`).
//!
//! It implements the generated `ship_ops_rpc::SettingsService` trait so shipd
//! mounts it via `SettingsServiceExt::register`. Watch is the reactive surface
//! over the shared `ReactiveRegistry`, scoped to the requested settings tier.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::ports::KernelLog;

use ship_ops_rpc::{
    Setting, SettingList, SettingScope as ProtoScope, SettingsService as SettingsServiceTrait,
    WatchEvent,
};

use crate::patch::patch_frame_to_proto;
use crate::service::{ResolvedSetting, SettingScope, SettingsCtx, SettingsService};

/// Wraps `SettingsService` with a per-request context factory + the shared
/// reactive registry for Watch.
pub struct SettingsWire<F> {
    make_ctx: F,
    reactive: Arc<ReactiveRegistry>,
}

impl<F> SettingsWire<F>
where
    F: Fn(&RequestContext) -> Result<SettingsCtx, ConnectError> + Send + Sync + 'static,
{
    pub fn new(kernel: Arc<KernelLog>, make_ctx: F) -> Self {
        Self {
            make_ctx,
            reactive: Arc::new(ReactiveRegistry::new(kernel)),
        }
    }

    fn make_svc(&self, rctx: &RequestContext) -> Result<SettingsService, ConnectError> {
        Ok(SettingsService::new((self.make_ctx)(rctx)?))
    }
}

/// Map the proto `SettingScope` discriminant to the crate scope. `UNSPECIFIED`
/// and unknown wire values are a hard `invalid_argument` (the scope is required).
fn map_scope(scope: buffa::EnumValue<ProtoScope>) -> Result<SettingScope, ConnectError> {
    match scope.as_known() {
        Some(ProtoScope::SETTING_SCOPE_SPACE) => Ok(SettingScope::Space),
        Some(ProtoScope::SETTING_SCOPE_ACCOUNT) => Ok(SettingScope::Account),
        Some(ProtoScope::SETTING_SCOPE_RUNTIME) => Ok(SettingScope::Runtime),
        _ => Err(ConnectError::invalid_argument(
            "scope is required (SPACE / ACCOUNT / RUNTIME)",
        )),
    }
}

fn proto_scope(scope: SettingScope) -> ProtoScope {
    match scope {
        SettingScope::Space => ProtoScope::SETTING_SCOPE_SPACE,
        SettingScope::Account => ProtoScope::SETTING_SCOPE_ACCOUNT,
        SettingScope::Runtime => ProtoScope::SETTING_SCOPE_RUNTIME,
    }
}

fn setting_to_proto(r: ResolvedSetting) -> Setting {
    let value_json = serde_json::to_vec(&r.value).unwrap_or_default();
    Setting {
        key: r.key,
        value_json,
        scope: buffa::EnumValue::from(proto_scope(r.scope)),
        is_default: r.is_default,
        ..Default::default()
    }
}

// ── Trait implementation ────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl<F> SettingsServiceTrait for SettingsWire<F>
where
    F: Fn(&RequestContext) -> Result<SettingsCtx, ConnectError> + Send + Sync + 'static,
{
    async fn get(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSettingReqView,
    ) -> ServiceResult<Setting> {
        let scope = map_scope(request.scope)?;
        let svc = self.make_svc(&ctx)?;
        let resolved = svc.get(scope, request.key).await.map_err(ConnectError::from)?;
        Ok(Response::new(setting_to_proto(resolved)))
    }

    async fn list(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedListSettingsReqView,
    ) -> ServiceResult<SettingList> {
        let scope = map_scope(request.scope)?;
        let svc = self.make_svc(&ctx)?;
        let settings: Vec<Setting> = svc
            .list(scope)
            .await
            .map_err(ConnectError::from)?
            .into_iter()
            .map(setting_to_proto)
            .collect();
        Ok(Response::new(SettingList { settings, ..Default::default() }))
    }

    async fn set(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSettingReqView,
    ) -> ServiceResult<Setting> {
        let scope = map_scope(request.scope)?;
        let svc = self.make_svc(&ctx)?;
        let value: serde_json::Value = serde_json::from_slice(request.value_json)
            .map_err(|e| ConnectError::invalid_argument(format!("invalid value JSON: {e}")))?;
        let resolved = svc
            .set(scope, request.key, value)
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(setting_to_proto(resolved)))
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWatchSettingsReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let scope = map_scope(request.scope)?;
        let svc = self.make_svc(&ctx)?;
        // One watch slot per (space-or-scope, tier). The reactive key partitions
        // by the resolved space (Space scope) or the scope name (Account/Runtime).
        let space_id = svc
            .ctx
            .space
            .as_ref()
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| scope.as_str().to_string());
        let key = WatchKey {
            space_id,
            scope: format!("settings:{}", scope.as_str()),
            query_hash: 0,
        };

        // The snapshot reads the chosen scope's resolved settings; the prefilter
        // wakes on any settings.* emission. The resolver owns a cloned context so
        // it can re-snapshot on each reaction.
        let snapshot_ctx = svc.ctx.clone();
        let spec = ResolverSpec {
            prefilter: Box::new(|env| env.event_type.starts_with("settings.")),
            snapshot: Box::new(move || {
                let svc = SettingsService::new(snapshot_ctx.clone());
                Box::pin(async move { svc.snapshot(scope).await })
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
