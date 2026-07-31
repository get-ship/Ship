//! Identity binding — MCP callers may only act as their own bound agent.
//!
//! MCP worker sessions hold a `caller_kind == Mcp` `AuthContext` with
//! an `agent_id` bound at session registration. Without this guard,
//! handlers that read an identity from a request body or URL path
//! would let any worker session impersonate any other agent.
//!
//! CLI / Studio / SDK callers remain free to act on behalf of any agent —
//! they're the trusted intermediary by design.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use ship_auth::peer_identity::PeerIdentity;
use ship_substrate::events::CallerKind;

use crate::auth::AuthContext;

/// Build an [`AuthContext`] from a **kernel-verified** peer identity — the
/// host-socket transport's auth source, parallel to the token/session branches
/// of `auth_middleware` but anchored in the connection's peer credentials, not
/// a presented secret. One `AuthContext` shape, one gate: the result flows to
/// `auth::gate` exactly like a token-derived context.
///
/// - [`PeerIdentity::LocalOwner`] → a **capped, non-operator** local-owner context
///   (`Cli` transport label, no elevated grants). Sharing the daemon's uid is
///   identity, not authority: the local owner carries no grants and no operator
///   cred. Operator authority comes SOLELY from the Hub, never from a local
///   connection.
/// - [`PeerIdentity::Agent`] → an agent caller (`Mcp`) with `grants` the
///   per-agent provisioned baseline (#85). The agent presents no identity claim;
///   `id` came from its kernel process container, and the caller resolves
///   `grants` (DB-backed) at the runtime layer.
/// - [`PeerIdentity::Unresolved`] → `None`: the host couldn't attribute the
///   peer, so the caller MUST fail closed (reject), never default to operator.
pub fn auth_context_from_peer(identity: &PeerIdentity, grants: Vec<String>) -> Option<AuthContext> {
    match identity {
        PeerIdentity::LocalOwner => Some(AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            // Peer-cred identities run in the daemon's home space.
            space_id: None,
            session_id: None,
            // Capped non-operator: sharing the daemon's uid grants no elevation.
            grants: vec![],
            operator_cred: false,
        }),
        PeerIdentity::Agent(id) => Some(AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some(id.clone()),
            session_id: None,
            space_id: None,
            grants,
            operator_cred: false,
        }),
        PeerIdentity::Unresolved => None,
    }
}

/// Reject if an MCP caller's claimed agent_id (from body/path) doesn't
/// match the agent_id bound to its session. Non-MCP callers pass through.
#[allow(clippy::result_large_err)] // axum Response is the contract here
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

    fn mcp(id: Option<&str>) -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: id.map(str::to_string),
            space_id: None,
            session_id: None,
            grants: vec![],
            operator_cred: false,
        }
    }

    fn cli() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            space_id: None,
            session_id: None,
            grants: vec![],
            operator_cred: false,
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
    fn peer_local_owner_is_capped_non_operator() {
        // Sharing the daemon's uid is IDENTITY, not authority. The local-owner
        // peer context carries no elevated grants and no operator cred, so
        // `is_operator()` is false. Operator authority comes solely from the Hub.
        let ctx = auth_context_from_peer(&PeerIdentity::LocalOwner, vec![])
            .expect("local owner resolves to a context");
        assert_eq!(ctx.caller_kind, CallerKind::Cli);
        assert_eq!(ctx.agent_id, None);
        assert!(ctx.grants.is_empty());
        assert!(
            !ctx.operator_cred,
            "the local peer-uid confers no operator authority"
        );
        assert!(
            !ctx.is_operator(),
            "the local-owner peer is not operator-elevated — only the Hub is"
        );
    }

    #[test]
    fn peer_agent_carries_id_and_passed_grants() {
        let id = PeerIdentity::Agent("rust-lane-01".to_string());
        let ctx =
            auth_context_from_peer(&id, vec!["ship-orchestration".into()]).expect("agent resolves");
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
        assert_eq!(ctx.agent_id.as_deref(), Some("rust-lane-01"));
        assert_eq!(ctx.grants, vec!["ship-orchestration".to_string()]);
    }

    #[test]
    fn peer_unresolved_fails_closed() {
        // No attribution → no context → the caller must reject the connection.
        assert!(auth_context_from_peer(&PeerIdentity::Unresolved, vec![]).is_none());
    }
}
