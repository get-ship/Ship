//! `impl OpsImpl` helpers for the canonical `AgentService` handler
//! (`agent_rpc.rs`). Extracted to keep the handler within the file cap — this is
//! NOT a parallel handler, just the shared read/write/gate plumbing.
//!
//! Three planes:
//!   * `agents_typed_read` / `agents_read_json` / `agents_get_doc` — C0 reads via
//!     the sync read registry (`services::registry::call`), no gate;
//!   * `agents_gated_mutation` — C2 profile writes → the ONE gate
//!     (`services::command::call`), decoded onto `ScopedMutation`;
//!   * `agent_gated_typed` — C2/C3 orchestration control → the same gate, decoded
//!     onto the op's typed result message.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde::de::DeserializeOwned;
use serde_json::Value;

use ship_ops_rpc::{AgentDoc, JsonResponse, ScopedMutation};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

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
}
