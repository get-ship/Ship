//! ConnectRPC wire layer for `ViewService`.
//!
//! `ViewWire` implements `ship_ops_rpc::ViewService` so shipd can mount it via
//! `ViewServiceExt::register`, replacing `impl ViewService for OpsImpl` that
//! lived in `apps/shipd/src/view_rpc.rs`.
//!
//! Three groups of handlers:
//!   * Session writes (`patch_session`, `append_session`, `signal_session`) —
//!     need the live `SharedViewSessionRegistry` which is shipd-internal.
//!     Injected via typed async closures captured in shipd's `ops_rpc.rs`.
//!   * Edit-mode grants (`enter_edit_mode`, `exit_edit_mode`) —
//!     call `ship_views::view_authoring::{grant,revoke}` directly; no daemon
//!     state required.
//!   * Generic tool dispatch (`tool`) — needs `ApiState` for `dispatch_tool_core`.
//!     Injected via a `ToolFn` closure.
//!
//! Agent-identity binding is preserved: `patch_session`, `append_session`,
//! `signal_session` all require an `agent_id` extracted from the auth context.
//! The `AgentFn` closure handles that extraction without coupling this crate
//! to shipd's `AuthContext` type.
//!
//! Gates: all ops carry `ship.views.write` / C1 in the proto descriptor; the
//! auth middleware rejects unauthenticated requests before handlers run. No
//! additional gate closure is needed (all six ops are C1).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::Value;
use ship_ops_rpc::{JsonResponse, ViewService as ViewServiceTrait};

// ── Closure type aliases ──────────────────────────────────────────────────────

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Extract the agent_id from a request context.
/// Shipd wires this to `ctx.extensions.get::<AuthContext>().agent_id.clone()`.
pub type AgentFn = Arc<dyn Fn(&RequestContext) -> Result<String, ConnectError> + Send + Sync>;

/// Async closure for `patch_session`: (id, agent, selector, html) → Result<()>.
/// Captures `SharedViewSessionRegistry` from shipd.
pub type PatchFn =
    Arc<dyn Fn(String, String, String, String) -> BoxFuture<Result<(), ConnectError>> + Send + Sync>;

/// Async closure for `append_session`: (id, agent, selector, html) → Result<()>.
pub type AppendFn =
    Arc<dyn Fn(String, String, String, String) -> BoxFuture<Result<(), ConnectError>> + Send + Sync>;

/// Async closure for `signal_session`: (id, agent, name, value) → Result<()>.
pub type SignalFn =
    Arc<dyn Fn(String, String, String, Value) -> BoxFuture<Result<(), ConnectError>> + Send + Sync>;

/// Async closure for `tool`: (view, session_id, op, args) → Result<String>.
/// Returns the raw JSON body from `dispatch_tool_core`. Captures `ApiState`.
pub type ToolFn =
    Arc<dyn Fn(String, String, String, Value) -> BoxFuture<Result<String, ConnectError>> + Send + Sync>;

// ── Wire struct ───────────────────────────────────────────────────────────────

/// ConnectRPC wire for `ViewService`.
pub struct ViewWire {
    agent_fn:  AgentFn,
    patch_fn:  PatchFn,
    append_fn: AppendFn,
    signal_fn: SignalFn,
    tool_fn:   ToolFn,
}

impl ViewWire {
    pub fn new(
        agent_fn:  AgentFn,
        patch_fn:  PatchFn,
        append_fn: AppendFn,
        signal_fn: SignalFn,
        tool_fn:   ToolFn,
    ) -> Self {
        Self { agent_fn, patch_fn, append_fn, signal_fn, tool_fn }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Identical view-name validation used by the REST edit-mode routes.
fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn json_ok(value: Value) -> ServiceResult<JsonResponse> {
    Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl ViewServiceTrait for ViewWire {
    async fn patch_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewPatchReqView,
    ) -> ServiceResult<JsonResponse> {
        let agent = (self.agent_fn)(&ctx)?;
        (self.patch_fn)(
            request.id.to_string(),
            agent,
            request.selector.to_string(),
            request.html.to_string(),
        )
        .await?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn append_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewAppendReqView,
    ) -> ServiceResult<JsonResponse> {
        let agent = (self.agent_fn)(&ctx)?;
        (self.append_fn)(
            request.id.to_string(),
            agent,
            request.selector.to_string(),
            request.html.to_string(),
        )
        .await?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn signal_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewSignalReqView,
    ) -> ServiceResult<JsonResponse> {
        let agent = (self.agent_fn)(&ctx)?;
        let value: Value = if request.value_json.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&request.value_json)
                .map_err(|e| ConnectError::invalid_argument(format!("value_json: {e}")))?
        };
        (self.signal_fn)(
            request.id.to_string(),
            agent,
            request.name.to_string(),
            value,
        )
        .await?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn enter_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        // Auth context required (middleware guarantees it) but agent identity
        // is not needed for edit-mode grants — any authenticated caller may
        // enter edit mode on a session they own.
        let _ = ctx;
        let view = request.view.to_string();
        let session_id = request.session_id.to_string();
        if !is_valid_view_name(&view) {
            return Err(ConnectError::invalid_argument("invalid view name"));
        }
        crate::view_authoring::grant(&session_id, &view);
        json_ok(serde_json::json!({ "view": view, "session_id": session_id, "granted": true }))
    }

    async fn exit_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        let _ = ctx;
        let view = request.view.to_string();
        let session_id = request.session_id.to_string();
        if !is_valid_view_name(&view) {
            return Err(ConnectError::invalid_argument("invalid view name"));
        }
        // Idempotent — revoking a missing grant is fine.
        crate::view_authoring::revoke(&session_id, &view);
        json_ok(serde_json::json!({ "view": view, "session_id": session_id, "granted": false }))
    }

    async fn tool(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewToolReqView,
    ) -> ServiceResult<JsonResponse> {
        let _ = ctx;
        let args_value: Value = if request.args_json.is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&request.args_json)
                .map_err(|e| ConnectError::invalid_argument(format!("args_json: {e}")))?
        };
        let json = (self.tool_fn)(
            request.view.to_string(),
            request.session_id.to_string(),
            request.op.to_string(),
            args_value,
        )
        .await?;
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }
}
