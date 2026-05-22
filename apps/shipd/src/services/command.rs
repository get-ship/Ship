//! Write-side command registry — the mutation analog of `services::registry`.
//!
//! `call(name, args, ctx)` dispatches a named command (declared in
//! `schema::COMMANDS`) to its domain handler, which builds + emits events or
//! delegates to a platform service. REST / Connect / MCP are thin projections
//! over this, and `cargo xtask gen-ops` generates the Connect
//! `CommandOpsService` from `COMMANDS` — one source of truth for the write
//! surface, exactly as `OPS` + `registry::call` are for reads.
//!
//! Logic currently inline in route handlers is lifted into the domain
//! submodules here (and, where it's genuine domain logic, on into
//! ship-platform services) per #78. Each lift makes one more transport thin.

use serde_json::Value;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;

mod pipeline;

/// Ambient context for a command: the runtime + the authenticated caller
/// (identity + emit context). The write-side analog of passing `&ApiState` to
/// a read.
pub struct CommandCtx<'a> {
    pub state: &'a ApiState,
    pub auth: &'a AuthContext,
}

/// Structured command error carrying the HTTP status each transport maps to its
/// own wire shape (REST status code, Connect error code).
#[derive(Debug, Clone)]
pub struct CommandError {
    pub status: axum::http::StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn bad_args(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::BAD_REQUEST,
            code: "bad_args",
            message: msg.into(),
        }
    }
    pub fn unknown_command(name: &str) -> Self {
        Self {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "unknown_command",
            message: format!("unknown command `{name}`"),
        }
    }
    /// Convert a route-core `(StatusCode, String)` error into a `CommandError`.
    pub fn from_core((status, message): (axum::http::StatusCode, String)) -> Self {
        let code = if status.is_client_error() { "bad_args" } else { "service_error" };
        Self { status, code, message }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
impl std::error::Error for CommandError {}

/// Dispatch a named command. Each domain submodule's `try_dispatch` returns
/// `Ok(Some(value))` when it owns `name`, `Ok(None)` to fall through.
pub async fn call(name: &str, args: &Value, ctx: &CommandCtx<'_>) -> Result<Value, CommandError> {
    if let Some(value) = pipeline::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    Err(CommandError::unknown_command(name))
}

// ── arg helpers (shared by domain handlers) ──────────────────────────

pub(crate) fn req_str(args: &Value, key: &str) -> Result<String, CommandError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| CommandError::bad_args(format!("missing required arg `{key}`")))
}

pub(crate) fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_update_status_dispatches_through_the_registry() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![],
        };
        let ctx = CommandCtx { state: &harness.state, auth: &auth };
        let args = serde_json::json!({
            "pipeline_run_id": "run-cmd-test",
            "status": "gate_passed",
        });
        let value = call("pipeline.update_status", &args, &ctx)
            .await
            .expect("gate_passed has no required extra fields");
        assert_eq!(value["pipeline_run_id"], "run-cmd-test");
        assert_eq!(value["status"], "gate_passed");

        // Unknown command falls through to a structured error.
        let err = call("pipeline.nope", &args, &ctx).await.unwrap_err();
        assert_eq!(err.code, "unknown_command");
    }
}
