//! Space window registration — the one handshake that turns a launched shell
//! window into a first-class, pid-bound, C2-capped session.
//!
//! A space window self-spawns (process-per-app), connects over the host unix
//! socket, and calls this once at startup. In a single step it:
//!   1. registers the window as a lifecycle actor (`ActorKind::Window`) carrying
//!      its kernel-verified pid, so the launcher's single-instance query and the
//!      reaper can see it; and
//!   2. mints a `ship_sess_` session **bound to that pid** and capped at C2
//!      (`space_window_session`) — the token the webview then presents on every
//!      Connect call.
//!
//! The pid is taken from the connection's `SO_PEERCRED` (the `PeerPid` the
//! host-socket layer threads into extensions), never from the request body — a
//! process cannot claim another's pid to the kernel. The target space MAY be
//! supplied (one daemon serves many spaces): a window can open onto any
//! REGISTERED space, validated against the projects projection. Omitting it
//! falls back to the daemon's home space (`daemon_space_id`). An unregistered
//! space id is refused — a window can't conjure a space.
//!
//! Authority: only the operator's own process may register a space window. Over
//! the host socket the peer-cred layer resolves the operator to an `is_operator`
//! context; a non-operator caller is rejected.

use axum::http::StatusCode;
use serde::Serialize;

use crate::auth::{AuthContext, SessionRegistry};

/// What the shell needs back: the bearer token for its Connect transport, plus
/// the actor id and space it was bound to (for display / single-instance).
#[derive(Debug, Clone, Serialize)]
pub struct RegisterSpaceWindowResponse {
    pub session_token: String,
    pub actor_id: String,
    pub space_id: String,
}

/// Register the calling window + mint its pid-bound, C2-capped session.
///
/// `peer_pid` is the kernel-attested pid from the host socket ([`PeerPid`]);
/// `None` means the request didn't arrive over a transport that can attest a
/// pid (TCP), so registration is refused — a window token only means anything
/// when the kernel can vouch for the process. Returns `Err((status, msg))` on
/// any refusal so the caller maps it to an HTTP response.
pub async fn register_space_window(
    registry: &SessionRegistry,
    auth: &AuthContext,
    peer_pid: Option<u32>,
    target_space_id: Option<&str>,
) -> Result<RegisterSpaceWindowResponse, (StatusCode, String)> {
    // Only the operator's own process may open a space window.
    if !crate::services::gate::caller_from_auth(auth).is_operator {
        return Err((
            StatusCode::FORBIDDEN,
            "space window registration is operator-only".to_string(),
        ));
    }
    let Some(pid) = peer_pid else {
        return Err((
            StatusCode::FORBIDDEN,
            "space window registration requires a kernel-attested pid (host socket only)"
                .to_string(),
        ));
    };
    // Resolve the target space. A supplied id must be a REGISTERED space (the
    // window can't conjure one); omitting it binds the daemon's home space.
    let space_id = match target_space_id {
        Some(id) => {
            match ship_platform::projections::projects::get(id) {
                Ok(Some(_)) => id.to_string(),
                Ok(None) => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        format!("no registered space `{id}` to bind the window to"),
                    ));
                }
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("space lookup failed: {e}"),
                    ));
                }
            }
        }
        None => {
            let Some(home) = ship_substrate::project::daemon_space_id() else {
                return Err((
                    StatusCode::CONFLICT,
                    "daemon has no space identity to bind the window to".to_string(),
                ));
            };
            home
        }
    };

    let actor_id = ship_substrate::gen_ulid();

    // `register_window` does a durable event-store append (`block_on`), so it
    // must run off this async worker — same constraint as the reaper.
    let aid = actor_id.clone();
    let sid = space_id.clone();
    let registered = tokio::task::spawn_blocking(move || {
        ship_substrate::db::actor_events::register_window(&aid, &sid, pid)
    })
    .await;
    match registered {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("register window actor: {e}"),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("register window join: {e}"),
            ));
        }
    }

    let token = ship_platform::auth::scopes::generate_session_token()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("token generation: {e}")))?;
    let session =
        ship_platform::auth::scopes::space_window_session(&space_id, pid, token.clone());
    registry.lock().await.insert(token.clone(), session);

    Ok(RegisterSpaceWindowResponse { session_token: token, actor_id, space_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::events::CallerKind;

    fn operator() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
        }
    }

    fn agent() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("worker-x".to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec![],
        }
    }

    fn empty_registry() -> SessionRegistry {
        std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()))
    }

    // multi_thread: the substrate read/write helpers use `block_on`
    // (`block_in_place`), which panics on a current-thread runtime.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn registers_window_and_mints_pid_bound_c2_session() {
        let tmp = tempfile::tempdir().unwrap();
        ship_substrate::testutil::init_project_minimal(tmp.path().to_path_buf()).unwrap();
        ship_substrate::db::ensure_db().unwrap();
        // A daemon serves one space; stamp its identity so the handler binds to it.
        ship_substrate::project::set_daemon_space_id_from_root(tmp.path());
        let space = ship_substrate::project::daemon_space_id().expect("daemon space id set");

        let registry = empty_registry();
        // Use our own pid so the registered window row reads `live`.
        let pid = std::process::id();
        let resp = register_space_window(&registry, &operator(), Some(pid), None)
            .await
            .expect("operator registers a space window");

        assert_eq!(resp.space_id, space, "no target ⟹ binds the daemon home space");
        // The minted session is in the registry, pid-bound and C2 (never operator).
        let reg = registry.lock().await;
        let session = reg.get(&resp.session_token).expect("session minted");
        assert_eq!(session.pid, Some(pid), "token bound to the registering process");
        assert!(session.pid_authorizes(Some(pid)));
        assert!(!session.pid_authorizes(Some(pid.wrapping_add(1))));
        assert!(
            !session.scopes.contains(&crate::auth::Scope::Full),
            "a space window session is never operator-class"
        );
        // A non-empty actor id was minted and `register_window` succeeded (the
        // handler propagates its error, so the `Ok` above is the proof the
        // window actor was registered — actor-row liveness is covered by the
        // substrate `find_live_window_for_space` / `register_window` tests).
        assert!(!resp.actor_id.is_empty());
    }

    #[tokio::test]
    async fn non_operator_is_refused() {
        let registry = empty_registry();
        let err = register_space_window(&registry, &agent(), Some(1234), None)
            .await
            .expect_err("agents may not register space windows");
        assert_eq!(err.0, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn missing_peer_pid_is_refused() {
        let registry = empty_registry();
        let err = register_space_window(&registry, &operator(), None, None)
            .await
            .expect_err("registration without an attested pid is refused");
        assert_eq!(err.0, StatusCode::FORBIDDEN);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unregistered_target_space_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        ship_substrate::testutil::init_project_minimal(tmp.path().to_path_buf()).unwrap();
        ship_substrate::db::ensure_db().unwrap();
        let registry = empty_registry();
        // A target space id that was never registered must be refused — a window
        // can't conjure a space.
        let err = register_space_window(
            &registry,
            &operator(),
            Some(std::process::id()),
            Some("space-does-not-exist"),
        )
        .await
        .expect_err("an unregistered target space is refused");
        assert_eq!(err.0, StatusCode::NOT_FOUND);
    }
}
