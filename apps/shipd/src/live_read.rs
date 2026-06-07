//! Shared server-streaming helper for the typed live-read ops (#138).
//!
//! Each per-domain `<Op>Live` rpc calls `live_read_with(resolve)`,
//! which resolves the op via the supplied closure and re-resolves +
//! re-pushes the full value on every `config.*` kernel event — the exact
//! freshness loop the expr-SSE ran (full-value `set`, no deltas), now typed and
//! keyed by the rpc method instead of a parsed expr string.

use connectrpc::{ConnectError, Response, ServiceResult, ServiceStream};
use ship_ops_rpc::JsonResponse;
use ship_substrate::events::ActorConfig;

use crate::ops_rpc::OpsImpl;
use crate::window_subscriptions::ResolveFn;

impl OpsImpl {
    /// Open a live read backed by a resolver from the live-resolver registry.
    /// Takes the shared `ResolveFn` (`Arc<dyn Fn(&ApiState) -> Value>`) so the
    /// per-op `<Op>Live` streams and the multiplex feed this one loop from the
    /// same `(domain, op)` table.
    pub(crate) async fn live_read_with(
        &self,
        resolve: ResolveFn,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read_inner(self.state().clone(), resolve).await
    }

    async fn live_read_inner(
        &self,
        state: crate::ship_runtime::ApiState,
        resolve: ResolveFn,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let initial = resolve(&state);

        let actor_id = format!("live-read-{}", ship_substrate::gen_ulid());
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
        let guard = crate::ship_runtime::ActorGuard { actor_id, kernel: state.kernel().clone() };

        let stream = async_stream::stream! {
            let _guard = guard;
            let mut mb = mailbox;
            if let Ok(json) = serde_json::to_string(&initial) {
                yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
            }
            while let Some(_ev) = mb.recv().await {
                // Coalesce a burst of events into one re-resolve.
                while mb.try_recv().is_ok() {}
                let value = resolve(&state);
                if let Ok(json) = serde_json::to_string(&value) {
                    yield Ok(JsonResponse { json, ..Default::default() });
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}
