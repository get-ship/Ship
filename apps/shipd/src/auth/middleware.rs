//! Bearer / cookie / dev-loopback auth middleware, plus the authority
//! projection layer for crate-owned Connect wires.

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use ship_substrate::events::CallerKind;
use std::sync::Arc;

use super::context::AuthContext;
use super::scopes::{Scope, SessionRegistry, is_session_token};
use super::token::{is_daemon_token, tokens_equal};

/// Axum middleware that validates `Authorization: Bearer <token>` on every
/// request. Injects `AuthContext` into request extensions on success.
///
/// Accepts three token types:
/// 1. **Daemon token** (64-char hex) — CLI/Studio/hooks, gets `Scope::Full`
/// 2. **Session token** (`ship_sess_` prefix) — MCP agents, gets scoped access
/// 3. **SDK token** (`ship_` prefix) — external SDK callers
pub async fn auth_middleware(
    expected_token: Arc<String>,
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

    // 0.5 DEV MODE token-less loopback bypass — the local dev loop.
    //     `--dev` advertises "relaxed auth, auto-pair", but the only relaxation
    //     was auto-APPROVING a pairing the browser still had to initiate — and
    //     nothing in the shell ever calls pairInit, so every data request 401'd
    //     and the dev app couldn't load its own data. A token-less dev loop is
    //     the intended DX ("a flag → no token"): when `--dev` is set, treat a
    //     LOOPBACK peer as an authenticated paired browser session (the same
    //     identity dev auto-pair would mint) with no token/cookie. Gated on BOTH
    //     `is_dev_mode()` (which prints the "auth relaxed" boot banner) AND a
    //     loopback peer, so widening `--host` to a LAN/0.0.0.0 bind can never
    //     relax auth for a remote request — only same-machine clients.
    if super::pairing::is_dev_mode() {
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
                scopes: super::pairing::paired_session_scopes(),
                grants: vec![],
                // A dev loopback paired-session is not a pid-attested operator
                // cred — it grants no elevation (S6), same as a browser cookie.
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
                            scopes: session.scopes.clone(),
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

    let token_string = match header_token {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "missing or malformed Authorization header")
                .into_response();
        }
    };
    let token = token_string.as_str();

    // Session tokens are validated against the session registry.
    if is_session_token(token) {
        let registry = session_registry.lock().await;
        return match registry.get(token) {
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
                    scopes: session.scopes.clone(),
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

    // Daemon token and SDK tokens are validated against the expected token.
    if !tokens_equal(token, expected_token.as_str()) {
        return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
    }

    let (caller_kind, scopes) = if is_daemon_token(token) {
        // The daemon token is a local bootstrap credential carrying the dying
        // REST `Scope::Full` axis. It is NOT a pid-attested operator cred (S6):
        // `CallerKind::Cli` is a transport label only — it grants NO elevation.
        // Operator authority comes solely from a pid-attested operator session.
        (CallerKind::Cli, vec![Scope::Full])
    } else if token.starts_with("ship_") {
        (CallerKind::Sdk, vec![Scope::Full])
    } else {
        // The token equals `expected_token` but matches neither the daemon-token
        // format (64-char hex) nor an SDK prefix — an anomalous/misconfigured
        // credential. Fail closed rather than granting Cli + Full. A correct
        // daemon token always takes the first branch, so this never fires in
        // normal operation.
        return (StatusCode::UNAUTHORIZED, "unrecognized credential format").into_response();
    };

    let agent_id = header_string(&req, "x-ship-agent-id");
    let session_id = header_string(&req, "x-ship-session-id");

    let auth = AuthContext {
        caller_kind,
        agent_id,
        // Daemon/SDK tokens carry no session space; emits key None (no
        // home-space fallback on the emit path, S2).
        space_id: None,
        session_id,
        scopes,
        // No restricted grants on this path.
        grants: vec![],
        // The daemon/SDK token is NOT a pid-attested operator cred (S6): it
        // grants no operator authority. The dying `Scope::Full` it carries
        // clears the REST route-auth axis only, never the gate's operator path.
        operator_cred: false,
    };

    req.extensions_mut().insert(auth);
    next.run(req).await
}

fn header_string(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Project the authenticated caller's resolved [`Authority`] into the request
/// extensions, so crate-owned Connect wires (e.g. `ship-account`) can gate on
/// the shared `ship_types::Authority` without depending on shipd's
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
