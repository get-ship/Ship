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
use ship_platform::auth::peer_identity::PeerIdentity;
use ship_platform::auth::scopes::scopes_for_role;
use ship_substrate::events::CallerKind;

use crate::auth::{AuthContext, Scope};

/// Build an [`AuthContext`] from a **kernel-verified** peer identity — the
/// host-socket transport's auth source, parallel to the token/session branches
/// of `auth_middleware` but anchored in the connection's peer credentials, not
/// a presented secret. One `AuthContext` shape, one gate: the result flows to
/// `services::gate` exactly like a token-derived context.
///
/// - [`PeerIdentity::Operator`] → the operator credential (`Cli` + `Scope::Full`,
///   no restricted grants), mirroring the daemon-token branch. `role`/`grants`
///   are ignored — the operator clears the gate's grant axis.
/// - [`PeerIdentity::Agent`] → an agent caller (`Mcp`), scoped by `role`, with
///   `grants` the per-agent provisioned baseline (#85). The agent presents no
///   identity claim; `id` came from its kernel process container, and the
///   caller resolves `grants` (DB-backed) at the runtime layer.
/// - [`PeerIdentity::Unresolved`] → `None`: the host couldn't attribute the
///   peer, so the caller MUST fail closed (reject), never default to operator.
pub fn auth_context_from_peer(
    identity: &PeerIdentity,
    role: Option<&str>,
    grants: Vec<String>,
) -> Option<AuthContext> {
    match identity {
        PeerIdentity::Operator => Some(AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            // Peer-cred identities run in the daemon's home space.
            space_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
        }),
        PeerIdentity::Agent(id) => Some(AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some(id.clone()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: scopes_for_role(role),
            grants,
        }),
        PeerIdentity::Unresolved => None,
    }
}

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
            space_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
        }
    }

    fn cli() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
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

    #[test]
    fn peer_operator_is_the_operator_credential() {
        let ctx = auth_context_from_peer(&PeerIdentity::Operator, None, vec![])
            .expect("operator resolves");
        assert_eq!(ctx.caller_kind, CallerKind::Cli);
        assert_eq!(ctx.agent_id, None);
        assert!(ctx.scopes.contains(&Scope::Full));
        assert!(ctx.grants.is_empty(), "operator clears the grant axis");
    }

    #[test]
    fn peer_agent_carries_id_and_passed_grants() {
        let id = PeerIdentity::Agent("rust-lane-01".to_string());
        let ctx = auth_context_from_peer(&id, Some("commander"), vec!["ship-orchestration".into()])
            .expect("agent resolves");
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
        assert_eq!(ctx.agent_id.as_deref(), Some("rust-lane-01"));
        assert_eq!(ctx.grants, vec!["ship-orchestration".to_string()]);
    }

    #[test]
    fn peer_unresolved_fails_closed() {
        // No attribution → no context → the caller must reject the connection.
        assert!(auth_context_from_peer(&PeerIdentity::Unresolved, None, vec![]).is_none());
    }
}
