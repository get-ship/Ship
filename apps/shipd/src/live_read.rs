//! Shared server-streaming helper for the typed live-read ops (#138).
//!
//! This is the typed replacement for the `/_sui/data?expr=` string-dispatch
//! path. Each per-domain `<Op>Live` rpc calls `live_read(registry_name, args)`,
//! which resolves the op through the sync read registry and re-resolves +
//! re-pushes the full value on every `config.*` kernel event — the exact
//! freshness loop the expr-SSE ran (full-value `set`, no deltas), now typed and
//! keyed by the rpc method instead of a parsed expr string. The expr evaluator
//! (`views::sui_data`) is deleted once every consumer is on these streams.
//!
//! Namespaces match the expr path's v1 behavior: every live read wakes on
//! `config` (the coarse namespace `subscribe_namespaces` returned). Narrowing
//! to per-op namespaces is a follow-up — a single change here + per-call arg.

use connectrpc::{ConnectError, Response, ServiceResult, ServiceStream};
use serde_json::Value;
use ship_ops_rpc::JsonResponse;
use ship_substrate::events::ActorConfig;

use crate::ops_rpc::OpsImpl;

impl OpsImpl {
    /// Open a live read: stream the registry op's full value now and again on
    /// every `config` kernel event. `registry_name` is the dotted op
    /// (`git.overview`, `dashboard.state`); `args` is the JSON object the
    /// registry consumes (empty object for nullary reads).
    pub(crate) async fn live_read(
        &self,
        registry_name: &'static str,
        args: Value,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let state = self.state().clone();
        let resolve = move |st: &crate::ship_runtime::ApiState| -> Value {
            crate::services::registry::call(registry_name, &args, st).unwrap_or(Value::Null)
        };

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
