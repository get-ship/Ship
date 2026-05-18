//! Identity binding — MCP callers may only act as their own bound agent.
//!
//! MCP worker sessions hold a `caller_kind == Mcp` `AuthContext` with
//! an `agent_id` bound at session registration. Without this guard,
//! handlers that read an identity from a request body or URL path
//! would let any worker session impersonate any other agent.
//!
//! Daemon-token callers (CLI / Studio / SDK with `Scope::Full`) remain
//! free to act on behalf of any agent — they're the trusted
//! intermediary by design.

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde_json::json;
use ship_substrate::events::CallerKind;

use crate::auth::AuthContext;

/// Reject if an MCP caller's claimed agent_id (from body/path) doesn't
/// match the agent_id bound to its session. Non-MCP callers pass through.
pub fn enforce_mcp_identity(
    auth: &AuthContext,
    claimed_agent_id: Option<&str>,
) -> Result<(), Response> {
    if auth.caller_kind != CallerKind::Mcp {
        return Ok(());
    }
    let bound = auth.agent_id.as_deref();
    if claimed_agent_id == bound {
        return Ok(());
    }
    let msg = format!(
        "agent_id mismatch: MCP callers can only act as their own agent (auth={}, claimed={})",
        bound.unwrap_or("<none>"),
        claimed_agent_id.unwrap_or("<none>"),
    );
    Err((StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Scope;

    fn mcp(id: Option<&str>) -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: id.map(str::to_string),
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        }
    }

    fn cli() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        }
    }

    #[test]
    fn mcp_matching_id_passes() {
        assert!(enforce_mcp_identity(&mcp(Some("a")), Some("a")).is_ok());
    }

    #[test]
    fn mcp_mismatched_id_rejected() {
        assert!(enforce_mcp_identity(&mcp(Some("a")), Some("b")).is_err());
    }

    #[test]
    fn mcp_missing_claim_rejected() {
        // bound = "a", claimed = None — caller didn't supply an id but is
        // bound to one. Must reject to keep the binding meaningful.
        assert!(enforce_mcp_identity(&mcp(Some("a")), None).is_err());
    }

    #[test]
    fn mcp_unbound_with_claim_rejected() {
        // bound = None (e.g. Studio MCP), claimed = "b". Reject — an
        // unbound MCP session can't claim a specific identity.
        assert!(enforce_mcp_identity(&mcp(None), Some("b")).is_err());
    }

    #[test]
    fn cli_caller_passes_through() {
        assert!(enforce_mcp_identity(&cli(), Some("anything")).is_ok());
        assert!(enforce_mcp_identity(&cli(), None).is_ok());
    }
}
