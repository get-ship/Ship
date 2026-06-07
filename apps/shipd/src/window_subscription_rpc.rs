//! `WindowSubscriptionService` handlers — the live-data multiplex (ADR 0001).
//!
//! ONE Connect server-stream per webview plus a unary control plane, keyed on
//! `window_id` and demuxed by `subscription_id`. The `Stream` handler is
//! `live_read::live_read_inner` generalized from one-actor-per-op to
//! one-actor-per-window: a single kernel actor (subscribing `config`) wakes a
//! re-resolve of **every** `Subscription` in the window's bucket, each emitted
//! as a `MultiplexEvent.data` tagged by its `subscription_id`. The per-sub
//! resolvers are the same builders the per-op `<Op>Live` streams call
//! (`window_subscriptions::resolver_for`).
//!
//! This collapses the per-`<data live>` connection fan-out that exhausted the
//! browser's ~6/origin HTTP/1.1 budget (measured: shell home held 12 sockets)
//! down to one stream per window.

use std::sync::Arc;
use std::time::Duration;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::Value;
use ship_ops_rpc::{
    Heartbeat, MultiplexData, MultiplexEvent, OpenResponse, OwnedOpenRequestView,
    OwnedPatchRequestView, OwnedStreamRequestView, OwnedSubscribeRequestView,
    OwnedUnsubscribeRequestView, PatchAck, SubscribeAck, UnsubscribeAck,
    WindowSubscriptionService,
};
use ship_substrate::events::ActorConfig;

use crate::ship_runtime::ApiState;
use crate::window_subscriptions::{resolver_for, WindowSubscriptionRegistry};

/// Server heartbeat interval, advertised in `OpenResponse.heartbeat_ms` and the
/// cadence of the top-level `MultiplexEvent` heartbeat on the stream. The idle
/// reaper's timeout is sized off this (registry default = 2× + slack).
const HEARTBEAT_MS: u32 = 15_000;

/// Daemon-side `WindowSubscriptionService`. Holds `ApiState`; the per-window
/// state lives in `state.window_subscriptions()`.
pub struct WindowSubscriptionImpl {
    state: ApiState,
}

impl WindowSubscriptionImpl {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }

    fn registry(&self) -> &Arc<WindowSubscriptionRegistry> {
        self.state.window_subscriptions()
    }
}

impl WindowSubscriptionService for WindowSubscriptionImpl {
    #[allow(refining_impl_trait)]
    async fn open(
        &self,
        _ctx: RequestContext,
        _request: OwnedOpenRequestView,
    ) -> ServiceResult<OpenResponse> {
        let session_id = format!("sess.{}", ship_substrate::gen_ulid());
        let window_id = self.registry().open(session_id.clone());
        Ok(Response::new(OpenResponse {
            window_id,
            session_id,
            heartbeat_ms: HEARTBEAT_MS,
            ..Default::default()
        }))
    }

    #[allow(refining_impl_trait)]
    async fn subscribe(
        &self,
        _ctx: RequestContext,
        request: OwnedSubscribeRequestView,
    ) -> ServiceResult<SubscribeAck> {
        let window_id = request.window_id.to_string();
        let domain = request.domain.to_string();
        let op = request.op.to_string();
        let args: Value = if request.args_json.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(request.args_json)
                .map_err(|e| ConnectError::invalid_argument(format!("args_json: {e}")))?
        };

        let resolve = resolver_for(&domain, &op, &args).ok_or_else(|| {
            ConnectError::invalid_argument(format!(
                "multiplex has no resolver for {domain}.{op}"
            ))
        })?;

        // Resolve the initial value before handing the closure to the registry
        // so the ack can carry it (the client renders immediately, then the
        // stream pushes updates).
        let initial = resolve(&self.state);
        let initial_value = serde_json::to_vec(&initial).unwrap_or_default();

        let sub_id = self
            .registry()
            .subscribe(&window_id, domain, op, args, resolve)
            .ok_or_else(|| {
                ConnectError::not_found(format!("unknown window '{window_id}' — Open first"))
            })?;

        Ok(Response::new(SubscribeAck {
            subscription_id: sub_id,
            initial_value,
            ..Default::default()
        }))
    }

    #[allow(refining_impl_trait)]
    async fn unsubscribe(
        &self,
        _ctx: RequestContext,
        request: OwnedUnsubscribeRequestView,
    ) -> ServiceResult<UnsubscribeAck> {
        self.registry()
            .unsubscribe(request.window_id, request.subscription_id);
        Ok(Response::new(UnsubscribeAck::default()))
    }

    #[allow(refining_impl_trait)]
    async fn patch(
        &self,
        _ctx: RequestContext,
        _request: OwnedPatchRequestView,
    ) -> ServiceResult<PatchAck> {
        // The multiplex is server-authoritative: every value is re-resolved from
        // source on each kernel event, so there is no per-sub mutable value to
        // optimistically patch. Until an optimistic-patch path exists, refuse
        // honestly rather than ack a no-op.
        Err(ConnectError::unimplemented(
            "WindowSubscriptionService.Patch is not implemented (values are server-resolved)",
        ))
    }

    #[allow(refining_impl_trait)]
    async fn stream(
        &self,
        _ctx: RequestContext,
        request: OwnedStreamRequestView,
    ) -> ServiceResult<ServiceStream<MultiplexEvent>> {
        let window_id = request.window_id.to_string();
        let bucket = self.registry().bucket(&window_id).ok_or_else(|| {
            ConnectError::not_found(format!("unknown window '{window_id}' — Open first"))
        })?;

        let state = self.state.clone();
        let registry = self.registry().clone();

        let actor_id = format!("win-mux-{}-{}", window_id, ship_substrate::gen_ulid());
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: vec!["config".to_string()],
            see_directed_events: false,
        };
        let mailbox = {
            let mut kernel = state.kernel().lock().await;
            kernel
                .spawn_actor(&actor_id, config)
                .map(|(_store, mb)| mb)
                .map_err(|e| ConnectError::internal(e.to_string()))?
        };
        let guard = crate::ship_runtime::ActorGuard {
            actor_id,
            kernel: state.kernel().clone(),
        };
        let drop_guard = WindowDropGuard {
            registry,
            window_id: window_id.clone(),
        };

        let stream = async_stream::stream! {
            let _guard = guard;
            // Drop the window's bucket from the registry when the stream ends
            // (window closed / client gone), backed by the idle reaper.
            let _drop_guard = drop_guard;
            let mut mb = mailbox;
            let mut version: u32 = 0;

            // Emit the current value of every subscription present at stream
            // start. Subscriptions added later get their first value from the
            // unary SubscribeAck and their updates from the re-resolve loop.
            bucket.touch();
            for (sub_id, resolve) in bucket.resolvers() {
                let value = resolve(&state);
                if let Ok(bytes) = serde_json::to_vec(&value) {
                    version += 1;
                    yield Ok::<MultiplexEvent, ConnectError>(data_event(&sub_id, bytes, version));
                }
            }

            let mut hb = tokio::time::interval(Duration::from_millis(HEARTBEAT_MS as u64));
            hb.tick().await; // skip the immediate first tick

            loop {
                tokio::select! {
                    ev = mb.recv() => {
                        let Some(_ev) = ev else { break };
                        // Coalesce a burst of events into one re-resolve sweep.
                        while mb.try_recv().is_ok() {}
                        bucket.touch();
                        for (sub_id, resolve) in bucket.resolvers() {
                            let value = resolve(&state);
                            if let Ok(bytes) = serde_json::to_vec(&value) {
                                version += 1;
                                yield Ok(data_event(&sub_id, bytes, version));
                            }
                        }
                    }
                    _ = hb.tick() => {
                        bucket.touch();
                        yield Ok(heartbeat_event());
                    }
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}

/// A `MultiplexEvent.data` frame tagged by `subscription_id`.
fn data_event(sub_id: &str, value: Vec<u8>, version: u32) -> MultiplexEvent {
    MultiplexEvent {
        subscription_id: sub_id.to_string(),
        body: MultiplexData {
            value,
            version,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

/// A top-level heartbeat frame (empty `subscription_id`).
fn heartbeat_event() -> MultiplexEvent {
    MultiplexEvent {
        subscription_id: String::new(),
        body: Heartbeat::default().into(),
        ..Default::default()
    }
}

/// Drops a window's bucket from the registry when its `Stream` ends — the
/// primary GC for closed windows; the idle reaper is the safety net.
struct WindowDropGuard {
    registry: Arc<WindowSubscriptionRegistry>,
    window_id: String,
}

impl Drop for WindowDropGuard {
    fn drop(&mut self) {
        self.registry.drop_window(&self.window_id);
    }
}

#[cfg(test)]
#[path = "window_subscription_rpc_tests.rs"]
mod tests;
