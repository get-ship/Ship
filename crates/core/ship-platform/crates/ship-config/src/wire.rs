//! ConnectRPC wire layer for `ConfigService`.
//!
//! `ConfigWire<F>` wraps the service with a per-request context factory, mirror-
//! ing `ship-collection`'s `CollectionWire`. It implements the generated
//! `ship_ops_rpc::ConfigService` trait so shipd can mount it via
//! `ConfigServiceExt::register`. The handler is the settings tier (List/Get/Set);
//! Watch is the reactive surface (the StateLive replacement) over the shared
//! `ReactiveRegistry`.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::ports::{KernelLog, SpaceContext};

use ship_ops_rpc::{
    ConfigService as ConfigServiceTrait, Setting, SettingList, WatchEvent,
};

use crate::patch::{patch_frame_to_proto, settings_snapshot};
use crate::service::{ConfigService, ResolvedSetting};

/// Wraps `ConfigService` with a per-request context factory + the shared
/// reactive registry for Watch.
pub struct ConfigWire<F> {
    make_ctx: F,
    reactive: Arc<ReactiveRegistry>,
}

impl<F> ConfigWire<F>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
{
    pub fn new(kernel: Arc<KernelLog>, make_ctx: F) -> Self {
        Self {
            make_ctx,
            reactive: Arc::new(ReactiveRegistry::new(kernel)),
        }
    }

    fn make_svc(&self, rctx: &RequestContext) -> ConfigService {
        ConfigService { ctx: (self.make_ctx)(rctx) }
    }
}

fn setting_to_proto(r: ResolvedSetting) -> Setting {
    let value_json = serde_json::to_vec(&r.value).unwrap_or_default();
    Setting {
        key: r.key,
        value_json,
        is_default: r.is_default,
        ..Default::default()
    }
}

// ── Trait implementation ────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl<F> ConfigServiceTrait for ConfigWire<F>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
{
    async fn list(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListReqView,
    ) -> ServiceResult<SettingList> {
        let svc = self.make_svc(&ctx);
        let settings: Vec<Setting> = svc
            .list()
            .map_err(ConnectError::from)?
            .into_iter()
            .map(setting_to_proto)
            .collect();
        Ok(Response::new(SettingList { settings, ..Default::default() }))
    }

    async fn get(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSettingReqView,
    ) -> ServiceResult<Setting> {
        let svc = self.make_svc(&ctx);
        let resolved = svc.get(request.key).map_err(ConnectError::from)?;
        Ok(Response::new(setting_to_proto(resolved)))
    }

    async fn set(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSettingReqView,
    ) -> ServiceResult<Setting> {
        let svc = self.make_svc(&ctx);
        let value: serde_json::Value = serde_json::from_slice(request.value_json)
            .map_err(|e| ConnectError::invalid_argument(format!("invalid value JSON: {e}")))?;
        let resolved = svc
            .set(request.key, value)
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(setting_to_proto(resolved)))
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedWatchConfigReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let space_id = self.make_svc(&ctx).ctx.space_id().to_string();
        // Single watch slot per space: all daemon settings under one scope.
        let key = WatchKey { space_id, scope: "config".to_string(), query_hash: 0 };

        let spec = ResolverSpec {
            prefilter: Box::new(|env| env.event_type.starts_with("config.")),
            snapshot: Box::new(|| Box::pin(settings_snapshot())),
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
