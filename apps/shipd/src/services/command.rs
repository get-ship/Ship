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

mod agent;
mod annotations;
mod git;
mod mesh;
mod pipeline;
mod project;
mod surfaces;
mod views;

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
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::NOT_FOUND,
            code: "not_found",
            message: msg.into(),
        }
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::FORBIDDEN,
            code: "forbidden",
            message: msg.into(),
        }
    }
    pub fn service(msg: impl Into<String>) -> Self {
        Self {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            code: "service_error",
            message: msg.into(),
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
    // The one capability gate (spec: .artifacts/auth-gate-synthesis.md). Every
    // write surface — REST and Connect — reaches commands through here, so this
    // is the single enforcement point. Unknown names fall through ungated to the
    // dispatch chain, which returns `unknown_command`.
    if let Some(desc) = crate::services::schema::COMMANDS.iter().find(|c| c.name == name) {
        use crate::services::gate;
        let blast = gate::BlastRadius::from_descriptor(desc.blast_radius);
        let domain = gate::classify(name);
        let caller = gate::caller_from_auth(ctx.auth);
        if let gate::Decision::Deny(msg) = gate::decide(blast, &domain, &caller) {
            return Err(CommandError::forbidden(msg));
        }
    }
    if let Some(value) = pipeline::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = project::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = mesh::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = git::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = agent::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = surfaces::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = views::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    if let Some(value) = annotations::try_dispatch(name, args, ctx).await? {
        return Ok(value);
    }
    Err(CommandError::unknown_command(name))
}

// ── arg helpers (shared by domain handlers) ──────────────────────────

/// The caller's agent id (identity-bound commands require it).
pub(crate) fn require_agent(ctx: &CommandCtx<'_>) -> Result<String, CommandError> {
    ctx.auth
        .agent_id
        .clone()
        .ok_or_else(|| CommandError::forbidden("agent identity required"))
}

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

/// String-array arg (e.g. `paths`, `depends_on`).
pub(crate) fn str_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Bridge an interim axum-`Response` core into a command `Value`/error. The #78
/// service-lift removes this once those cores return typed values directly.
pub(crate) async fn response_to_value(
    resp: axum::response::Response,
) -> Result<Value, CommandError> {
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .map_err(|e| CommandError::service(e.to_string()))?;
    if status.is_success() {
        Ok(serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    } else {
        Err(CommandError {
            status,
            code: if status.is_client_error() { "bad_args" } else { "service_error" },
            message: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }
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
            grants: vec![],
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gate_denies_ungranted_agent_a_restricted_command() {
        let harness = crate::test_harness::TestDaemon::new().await;
        // A worker-role MCP agent: no orchestration grant, not the operator.
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("worker-1".to_string()),
            workspace_id: None,
            session_id: None,
            scopes: ship_platform::auth::scopes::scopes_for_role(Some("worker")),
            grants: vec![],
        };
        let ctx = CommandCtx { state: &harness.state, auth: &auth };
        let args = serde_json::json!({ "agent": "rust-lane" });
        // agent.dispatch is a restricted (capability-defining) op — denied
        // before any dispatch logic runs.
        let err = call("agent.dispatch", &args, &ctx).await.unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gate_allows_agent_with_minted_orchestration_grant() {
        let harness = crate::test_harness::TestDaemon::new().await;
        // A commander-role agent: grants_for_role mints ship-orchestration into
        // the session, so the gate lets agent.dispatch past (it fails later on
        // missing args, NOT on the gate — proving the grant cleared it).
        let auth = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("commander-1".to_string()),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
            grants: ship_platform::auth::scopes::grants_for_role(Some("commander")),
        };
        let ctx = CommandCtx { state: &harness.state, auth: &auth };
        let args = serde_json::json!({ "agent": "rust-lane" });
        // The grant cleared the gate: whether dispatch then succeeds or fails
        // downstream, it must NOT be a FORBIDDEN gate denial.
        if let Err(e) = call("agent.dispatch", &args, &ctx).await {
            assert_ne!(
                e.status,
                axum::http::StatusCode::FORBIDDEN,
                "minted orchestration grant should clear the gate"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn agent_grant_upgrades_live_session_and_is_operator_only() {
        use ship_platform::auth::scopes::DaemonSession;
        let harness = crate::test_harness::TestDaemon::new().await;
        // A live worker session with no restricted grants.
        let token = "sess-grant-test";
        {
            let mut reg = harness.state.session_registry().lock().await;
            reg.insert(
                token.to_string(),
                DaemonSession {
                    token: token.to_string(),
                    agent_id: "worker-x".to_string(),
                    scopes: vec![],
                    grants: vec![],
                    workspace_id: None,
                    created_at: std::time::Instant::now(),
                    expires_at_unix: None,
                    ephemeral: true,
                },
            );
        }
        let args = serde_json::json!({ "agent": "worker-x", "grants": ["ship-orchestration"] });

        // Operator approves → the live session is upgraded.
        let op = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec![],
        };
        let v = call("agent.grant", &args, &CommandCtx { state: &harness.state, auth: &op })
            .await
            .expect("operator may grant");
        assert_eq!(v["sessions_upgraded"], 1);
        {
            let reg = harness.state.session_registry().lock().await;
            assert!(reg.get(token).unwrap().grants.iter().any(|g| g == "ship-orchestration"));
        }

        // A non-operator agent calling agent.grant is denied (C2 gate).
        let agent = AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some("x".to_string()),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec!["ship-orchestration".to_string()],
        };
        let err = call("agent.grant", &args, &CommandCtx { state: &harness.state, auth: &agent })
            .await
            .unwrap_err();
        assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);
    }
}
