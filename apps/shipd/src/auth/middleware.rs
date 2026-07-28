//! Cookie / dev-loopback / session-token auth middleware, plus the
//! authority-projection layer for crate-owned Connect wires.
//!
//! The host unix socket's peer-cred layer (see `peer_socket`) runs OUTSIDE
//! this middleware and inserts `AuthContext` directly — that path is
//! honored unconditionally. The middleware below handles the rest:
//!
//! 1. Session token (`ship_sess_` prefix) — MCP agents, provisioned grants,
//!    pid-attested when bound
//! 2. Browser session cookie — paired browser session, no pid binding
//! 3. `--unsafe-skip-auth` loopback bypass — contributor dev only,
//!    compiled out of release builds

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use ship_substrate::events::CallerKind;

use super::context::AuthContext;
use super::scopes::{SessionRegistry, is_session_token};

/// Axum middleware that authenticates a request via session token, browser
/// cookie, or the loopback bypass. Injects `AuthContext` into request
/// extensions on success. The peer-cred layer (host socket) inserts the
/// context BEFORE this runs and is honored as-is.
pub async fn auth_middleware(
    session_registry: SessionRegistry,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // 0. Already authenticated by an outer layer? The host unix socket
    //    resolves identity from the connection's kernel peer credentials and
    //    inserts `AuthContext` before this middleware runs (no token is
    //    presented on that transport). Honor it — one gate, two auth sources.
    //    On TCP nothing inserts `AuthContext` ahead of this layer, so this is
    //    a no-op there.
    if req.extensions().get::<AuthContext>().is_some() {
        return next.run(req).await;
    }

    // `--unsafe-skip-auth` token-less loopback bypass — the contributor dev override.
    //     Treat a LOOPBACK peer as an authenticated capped client (the broad C2
    //     authoring set, never the operator) with no token/cookie. Gated on BOTH
    //     `is_unsafe_skip_auth_mode()` (which prints the "auth relaxed" boot banner)
    //     AND a loopback peer, so widening `--host` to a LAN/0.0.0.0 bind can
    //     never relax auth for a remote request — only same-machine clients.
    //     Compiled out of release builds: a shipped daemon can never relax auth.
    #[cfg(debug_assertions)]
    if super::unsafe_skip_auth::is_unsafe_skip_auth_mode() {
        let loopback = req
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|c| c.0.ip().is_loopback())
            .unwrap_or(false);
        if loopback {
            let auth = AuthContext {
                caller_kind: CallerKind::Cli,
                agent_id: None,
                space_id: None,
                session_id: None,
                // Broad C2 authoring set — never the operator (operator authority
                // lives only in the Hub, never in a local connection).
                grants: ["agents", "skills", "mcp", "create", "view"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                operator_cred: false,
            };
            req.extensions_mut().insert(auth);
            return next.run(req).await;
        }
    }

    // 1. Cookie check — browser sessions.
    if let Some(cookie_header) = req.headers().get("cookie").and_then(|v| v.to_str().ok())
        && let Some(session_id) = super::session::parse_session_cookie(cookie_header)
            && super::session::is_browser_session_id(&session_id) {
                match super::session::lookup_browser_session(&session_registry, &session_id).await {
                    Some(session) => {
                        let auth = AuthContext {
                            caller_kind: CallerKind::Cli,
                            agent_id: None,
                            space_id: session.space_id.clone(),
                            session_id: None,
                            grants: session.grants.clone(),
                            // A browser cookie is NOT a pid-attested operator
                            // cred — it grants no elevation (S6).
                            operator_cred: false,
                        };
                        req.extensions_mut().insert(auth);
                        return next.run(req).await;
                    }
                    None => {
                        return (StatusCode::UNAUTHORIZED, "invalid or expired session cookie")
                            .into_response();
                    }
                }
            }

    let header_token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Query-param tokens are never accepted: no live route needs them (the
    // former iframe raw-serve flow `/api/v1/view/*/raw` was removed and nothing
    // serves that path). Every endpoint authenticates via the
    // `Authorization: Bearer` header or the `ship_session` cookie. A `?token=`
    // is ignored — and warned, so a stale caller is visible — since it would
    // otherwise leak the token into access logs.
    let query_has_token = req
        .uri()
        .query()
        .map(|q| q.split('&').any(|p| p.starts_with("token=")))
        .unwrap_or(false);
    if query_has_token {
        tracing::warn!(
            "auth: ignored ?token= on {} (use Authorization header or cookie)",
            req.uri().path()
        );
    }

    let token = match header_token {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "missing or malformed Authorization header")
                .into_response();
        }
    };

    // Session tokens are validated against the session registry.
    if is_session_token(&token) {
        let registry = session_registry.lock().await;
        return match registry.get(&token) {
            Some(session) => {
                // Pid-binding: a pid-bound token (a space window OR an operator
                // cred, S6 #124) is bound to its registering process. The kernel
                // pid arrives via the peer-cred layer (UDS only); on TCP it's
                // absent → a bound token fails closed. An unbound session
                // (agent/legacy) is not pid-restricted. This check runs BEFORE the
                // operator cred is carried below, so an operator cred reaching the
                // AuthContext has a verified pid.
                let peer_pid = req
                    .extensions()
                    .get::<super::peer_socket::PeerPid>()
                    .and_then(|p| p.0);
                if !session.pid_authorizes(peer_pid) {
                    return (
                        StatusCode::FORBIDDEN,
                        "session token bound to a different process",
                    )
                        .into_response();
                }
                // Identity is bound to the session at registration time and
                // cannot be overridden by client headers. x-ship-session-id
                // remains freely settable — it's the agent's own session context,
                // not a privilege claim.
                let session_id = header_string(&req, "x-ship-session-id");

                let auth = AuthContext {
                    caller_kind: CallerKind::Mcp,
                    agent_id: Some(session.agent_id.clone()),
                    // The session's space — for a space-window token this is the
                    // window's own space, so its emits land in that space.
                    space_id: session.space_id.clone(),
                    session_id,
                    // The minted restricted grants — the gate's grant axis.
                    grants: session.grants.clone(),
                    // Operator authority comes SOLELY from a pid-attested
                    // operator cred (S6). `pid_authorizes` was already enforced
                    // above, so an operator-cred session that reaches here has a
                    // verified pid — carry the elevation. Every other session
                    // (space window, agent, paired Studio) leaves this `false`.
                    operator_cred: session.is_operator_cred,
                };
                drop(registry);
                req.extensions_mut().insert(auth);
                next.run(req).await
            }
            None => {
                (StatusCode::UNAUTHORIZED, "invalid session token").into_response()
            }
        };
    }

    // No on-disk daemon token — the only Bearer tokens the daemon accepts
    // are session tokens (above). Any other presented credential is
    // rejected: the cut removed the legacy 64-hex daemon token and the
    // `ship_`-prefix SDK token path.
    (StatusCode::UNAUTHORIZED, "invalid or unsupported token").into_response()
}

fn header_string(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Project the authenticated caller's resolved [`Authority`] into the request
/// extensions, so crate-owned Connect wires (e.g. `ship-account`) can gate on
/// the shared `agent_schema::Authority` without depending on shipd's
/// `AuthContext`. Mirrors how `rpc::settings` passes `is_operator` into
/// `ship-config`: the authority is resolved in shipd (the SOLE minter) and
/// handed across the crate boundary as the shared type. Runs INSIDE
/// `auth_middleware` (which has already populated `AuthContext`); a request with
/// no `AuthContext` inserts nothing, and the wire fails closed (unauthenticated).
pub async fn project_authority_middleware(mut req: Request<Body>, next: Next) -> Response {
    if let Some(authority) = req
        .extensions()
        .get::<AuthContext>()
        .map(AuthContext::authority)
    {
        req.extensions_mut().insert(authority);
    }
    next.run(req).await
}
