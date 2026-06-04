//! `impl OpsImpl` helpers for the canonical `AgentService` handler
//! (`agent_rpc.rs`). Extracted to keep the handler within the file cap — this is
//! NOT a parallel handler, just the shared read/write/gate plumbing.
//!
//! Planes:
//!   * `agents_typed_read` / `agents_read_json` / `agents_get_doc` — C0 reads via
//!     the sync read registry (`services::registry::call`), no gate;
//!   * `agents_gated_mutation` — C2 profile writes → the ONE gate
//!     (`services::command::call`), decoded onto `ScopedMutation`;
//!   * `agent_gated_typed` — C2/C3 orchestration control → the same gate, decoded
//!     onto the op's typed result message;
//!   * `agent_watch_stream` — the unified reactive `Watch` (agents-arc E5): a
//!     snapshot-diff stream over the agent-def `ReactiveRegistry`, snapshotting
//!     via the SAME `services::registry::call` reads (reuse, never re-implement).

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use ship_ops_rpc::{AgentDoc, JsonResponse, ScopedMutation, WatchEvent};
use ship_reactive::{ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};
use ship_platform::services::agents::watch as agent_watch;

impl OpsImpl {
    /// C0 read → sync registry dispatch, decoded onto the typed proto message.
    pub(crate) fn agents_typed_read<T: DeserializeOwned>(
        &self,
        op: &str,
        args: Value,
    ) -> ServiceResult<T> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        serde_json::from_value::<T>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("{op} decode: {e}")))
    }

    /// C0 read → sync registry dispatch, wrapped as opaque JSON.
    pub(crate) fn agents_read_json(&self, op: &str, args: Value) -> ServiceResult<JsonResponse> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    /// C0 `agents.get` → the opaque-by-design `AgentDoc`. The reader yields
    /// `{ id, scope, doc }` where `doc` is the arbitrary user def object; we
    /// keep it opaque by serializing it into the `doc_json` string field
    /// (same open-payload convention as CollectionItem.data_json). Only the
    /// id + resolved scope are typed.
    pub(crate) fn agents_get_doc(&self, args: Value) -> ServiceResult<AgentDoc> {
        let value = crate::services::registry::call("agents.get", &args, self.state())
            .map_err(|e| ConnectError::internal(format!("agents.get: {e}")))?;
        let id = value.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
        let scope = value.get("scope").and_then(Value::as_str).unwrap_or_default().to_string();
        let doc_json = value.get("doc").map(|d| d.to_string()).unwrap_or_else(|| "null".to_string());
        Ok(Response::new(AgentDoc { id, scope, doc_json, ..Default::default() }))
    }

    /// C2 profile write → the one gate (`services::command::call`), decoded onto
    /// `ScopedMutation`. Named `agents_gated_mutation` to avoid collision with
    /// the same-shaped helper in `rules_rpc`.
    pub(crate) async fn agents_gated_mutation(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: Value,
    ) -> ServiceResult<ScopedMutation> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value =
            command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        serde_json::from_value::<ScopedMutation>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))
    }

    /// C2/C3 control write → the one gate (`services::command::call`), decoded
    /// onto the op's typed result message. The command arms thread STRUCTURED
    /// values (no JSON-string re-wrap); the proto message's snake_case aliases
    /// decode that object directly.
    pub(crate) async fn agent_gated_typed<T: DeserializeOwned>(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: Value,
    ) -> ServiceResult<T> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value =
            command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        serde_json::from_value::<T>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))
    }

    /// The unified reactive `Watch` (agents-arc E5) — open a snapshot-diff stream
    /// over the agent-def `ReactiveRegistry`. `scope` is the decoded oneof: `None`
    /// = all agents (snapshot `{summaries, catalogs}`, broad config-namespace
    /// prefilter), `Some(id)` = one agent (snapshot `{detail, wire, settings}`,
    /// prefilter only that agent's `agent.def.updated`). The `WatchKey` carries
    /// the connection's bound `space_id` so cross-space subscribers never share a
    /// cohort; dedup + fanout come from the registry. The snapshot reuses the
    /// existing C0 reads (`services::registry::call`) — never a re-implementation.
    pub(crate) fn agent_watch_stream(
        &self,
        ctx: &RequestContext,
        scope: Option<String>,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let registry = self
            .state()
            .agent_reactive()
            .cloned()
            .ok_or_else(|| ConnectError::unavailable("agent watch unavailable: no active space"))?;
        let space_id = ctx
            .extensions
            .get::<AuthContext>()
            .and_then(|a| a.space_id.clone())
            .unwrap_or_default();
        let state = self.state().clone();

        let (scope_key, spec): (String, ResolverSpec) = match scope {
            None => {
                let st = state.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(agent_watch::all_scope_relevant),
                    snapshot: Box::new(move || {
                        let st = st.clone();
                        Box::pin(async move { all_scope_snapshot(&st) })
                    }),
                };
                (agent_watch::SCOPE_ALL.to_string(), spec)
            }
            Some(id) => {
                let st = state.clone();
                let id_for_filter = id.clone();
                let id_for_snap = id.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(move |env| {
                        agent_watch::agent_scope_relevant(env, &id_for_filter)
                    }),
                    snapshot: Box::new(move || {
                        let st = st.clone();
                        let id = id_for_snap.clone();
                        Box::pin(async move { agent_scope_snapshot(&st, &id) })
                    }),
                };
                (agent_watch::scope_agent(&id), spec)
            }
        };

        let key = WatchKey { space_id, scope: scope_key, query_hash: 0 };
        let mut rx = registry.watch(key, EventFilter::default(), spec);
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

/// Map a reactive `PatchFrame` to the proto `WatchEvent`. Mirrors ship-space /
/// ship-collection's `patch_frame_to_proto` (same reactive types, same shared
/// `WatchEvent` from reactive.proto).
fn patch_frame_to_proto(frame: ship_reactive::PatchFrame) -> WatchEvent {
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

/// Snapshot for the all-agents watch: `{summaries, catalogs}`, composed from the
/// existing C0 registry reads. A read failure degrades to `null` for that field
/// (the resolver diffs, so a transient miss self-heals on the next event).
fn all_scope_snapshot(state: &crate::ship_runtime::ApiState) -> Value {
    json!({
        "summaries": read_or_null(state, "agents.summaries", Value::Null),
        "catalogs": read_or_null(state, "agents.catalogs", Value::Null),
    })
}

/// Snapshot for the single-agent watch: `{detail, wire, settings}` for `id`.
fn agent_scope_snapshot(state: &crate::ship_runtime::ApiState, id: &str) -> Value {
    let args = json!({ "id": id });
    json!({
        "detail": read_or_null(state, "agents.detail", args.clone()),
        "wire": read_or_null(state, "agents.wire", args.clone()),
        "settings": read_or_null(state, "agents.settings", args),
    })
}

fn read_or_null(state: &crate::ship_runtime::ApiState, op: &str, args: Value) -> Value {
    crate::services::registry::call(op, &args, state).unwrap_or(Value::Null)
}
