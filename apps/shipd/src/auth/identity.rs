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
use ship_auth::peer_identity::PeerIdentity;
use ship_auth::scopes::scopes_for_role;
use ship_substrate::events::CallerKind;

use crate::auth::{AuthContext, Scope};

/// Build an [`AuthContext`] from a **kernel-verified** peer identity — the
/// host-socket transport's auth source, parallel to the token/session branches
/// of `auth_middleware` but anchored in the connection's peer credentials, not
/// a presented secret. One `AuthContext` shape, one gate: the result flows to
/// `auth::gate` exactly like a token-derived context.
///
/// - [`PeerIdentity::Operator`] → a **non-elevated** local-owner context (`Cli`
///   transport label + the dying `Scope::Full` REST axis). S6 elevation-source
///   kill: the local peer-uid no longer confers operator (`operator_cred:
///   false`). Operator authority comes SOLELY from a pid-attested operator cred
///   (a session with `is_operator_cred`), not from sharing the daemon's uid.
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
            // S6: sharing the daemon's uid is NOT operator authority. The local
            // peer-uid grants no elevation; only a pid-attested operator cred does.
            operator_cred: false,
        }),
        PeerIdentity::Agent(id) => Some(AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some(id.clone()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: scopes_for_role(role),
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

/// Reject if an MCP caller bound to a workspace tries to act on a
/// different workspace. This is the per-project enforcement teeth for
/// sandbox-minted tokens: a session scoped to project A is hard-rejected
/// against project B's resources.
///
/// The guard bites only on a *definite* mismatch (bound = Some(A),
/// claimed = Some(B), A != B). A bound session reaching a non-workspace
/// resource (claimed = None) passes, as does an unbound MCP session —
/// sandbox tokens always carry a workspace, so unbound is the
/// legacy/Studio case. Non-MCP callers pass through.
#[allow(clippy::result_large_err)] // axum Response is the contract here
pub fn enforce_mcp_workspace(
    auth: &AuthContext,
    claimed_workspace_id: Option<&str>,
) -> Result<(), Response> {
    match mcp_workspace_violation(auth, claimed_workspace_id) {
        None => Ok(()),
        Some(msg) => Err((StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()),
    }
}

/// Pure policy behind [`enforce_mcp_workspace`]: returns the violation
/// message if this MCP caller (bound to a workspace) is acting on a
/// different one, else `None`. Handlers whose return type isn't `Response`
/// call this directly and shape their own error.
pub fn mcp_workspace_violation(
    auth: &AuthContext,
    claimed_workspace_id: Option<&str>,
) -> Option<String> {
    if auth.caller_kind != CallerKind::Mcp {
        return None;
    }
    let (Some(bound), Some(claimed)) = (auth.workspace_id.as_deref(), claimed_workspace_id) else {
        return None;
    };
    if bound == claimed {
        return None;
    }
    Some(format!(
        "workspace mismatch: MCP session bound to project '{bound}' cannot act on project '{claimed}'",
    ))
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
            operator_cred: false,
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
            operator_cred: false,
        }
    }

    fn mcp_ws(workspace: Option<&str>) -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("a".into()),
            workspace_id: workspace.map(str::to_string),
            space_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
            operator_cred: false,
        }
    }

    #[test]
    fn mcp_matching_workspace_passes() {
        assert!(enforce_mcp_workspace(&mcp_ws(Some("proj-a")), Some("proj-a")).is_ok());
    }

    #[test]
    fn mcp_cross_workspace_rejected() {
        // The keystone: a session bound to project A cannot touch project B.
        assert!(enforce_mcp_workspace(&mcp_ws(Some("proj-a")), Some("proj-b")).is_err());
    }

    #[test]
    fn mcp_bound_to_unscoped_resource_passes() {
        // Resource carries no workspace (e.g. agent self-info) — a bound
        // session may still reach it; the guard only bites cross-project.
        assert!(enforce_mcp_workspace(&mcp_ws(Some("proj-a")), None).is_ok());
    }

    #[test]
    fn mcp_unbound_passes() {
        // Unbound MCP (e.g. Studio) carries no project restriction.
        // Sandbox-minted tokens always bind, so this is the legacy/trusted case.
        assert!(enforce_mcp_workspace(&mcp_ws(None), Some("proj-b")).is_ok());
    }

    #[test]
    fn cli_workspace_passes_through() {
        assert!(enforce_mcp_workspace(&cli(), Some("anything")).is_ok());
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
    fn peer_local_owner_is_not_operator_elevated() {
        // S6 (#124): sharing the daemon's uid is IDENTITY, not authority. The
        // local-owner peer context carries the dying Scope::Full REST axis but
        // grants NO operator elevation — `operator_cred` stays false, so
        // `is_operator()` is false. Operator authority comes solely from a
        // verified, pid-attested operator cred.
        let ctx = auth_context_from_peer(&PeerIdentity::Operator, None, vec![])
            .expect("local owner resolves to a context");
        assert_eq!(ctx.caller_kind, CallerKind::Cli);
        assert_eq!(ctx.agent_id, None);
        assert!(ctx.scopes.contains(&Scope::Full));
        assert!(ctx.grants.is_empty());
        assert!(
            !ctx.operator_cred,
            "the local peer-uid no longer confers operator (S6)"
        );
        assert!(
            !ctx.is_operator(),
            "the local-owner peer is not operator-elevated — only a verified cred is"
        );
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
