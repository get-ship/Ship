//! Window control surface — the daemon authoritative over windows it can't
//! itself spawn.
//!
//!   - `intent` — the focus-vs-spawn decision: consult the actor projection for
//!     a live window of a space. Hit ⟹ focus it; miss ⟹ spawn. The daemon
//!     decides; it never spawns (only the shell can touch an OS window).
//!   - `list` — enumerate window actors + pid + live state.
//!   - `register` — the one handshake that turns a launched shell window into a
//!     pid-bound, C2-capped session: validate the target space, register the
//!     window as a lifecycle actor carrying its kernel-verified pid, mint the
//!     session token via the [`WindowSessions`] port.
//!   - `close` — revoke the window's pid-bound token (instant authority cut)
//!     then close its lifecycle. The shell destroys the OS frame; revocation
//!     neuters the webview regardless.
//!
//! Authority lives at the transport (the daemon enforces operator-only on
//! register and the C3 gate on close before calling here) — this service owns
//! the mechanism, not the policy check. The kernel-attested pid is supplied by
//! the daemon (from the connection's `SO_PEERCRED`), never from a request body.
//!
//! Storage is event-sourced over substrate ports: live-window reads hit the
//! actor projection ([`actor`]); register/close emit `actor.created` /
//! `actor.stopped` through [`actor_events`]. There is no raw filesystem access.

use ship_substrate::db::actor_events;
use ship_substrate::projections::actor::{self, ActorRow};

use crate::error::DeviceError;
use crate::ports::{SpaceRegistry, WindowSessions};

/// The focus-vs-spawn decision for opening a window onto `space_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowIntent {
    /// A live window already hosts the space — the shell should focus this actor.
    Focus { actor_id: String },
    /// No live window — the shell should spawn one.
    Spawn,
}

/// Result of registering a window: the bearer token for the webview's Connect
/// transport, plus the actor id and space it was bound to.
#[derive(Debug, Clone)]
pub struct RegisteredWindow {
    pub session_token: String,
    pub actor_id: String,
    pub space_id: String,
}

/// Result of closing a window: whether the lifecycle was closed and whether a
/// token was revoked (audit distinguishes the two outcomes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedWindow {
    pub closed: bool,
    pub token_revoked: bool,
}

/// Decide focus-vs-spawn for a window onto `space_id`.
pub fn intent(space_id: &str) -> Result<WindowIntent, DeviceError> {
    let existing = actor::find_live_window_for_space(space_id)
        .map_err(|e| DeviceError::service(e.to_string()))?;
    Ok(match existing {
        Some(row) => WindowIntent::Focus { actor_id: row.id },
        None => WindowIntent::Spawn,
    })
}

/// All `window`-kind actors (live + recently-closed), newest-first.
pub fn list() -> Result<Vec<ActorRow>, DeviceError> {
    actor::list_windows().map_err(|e| DeviceError::service(e.to_string()))
}

/// How `register` should resolve the space a window binds to.
pub enum TargetSpace<'a> {
    /// A specific registered space (validated against the space registry port).
    Id(&'a str),
    /// No target supplied — bind the daemon's home space.
    Home,
}

/// Register a window + mint its pid-bound, C2-capped session.
///
/// `pid` is the kernel-attested pid (supplied by the daemon; `None` over a
/// transport that can't attest one — registration is refused, a window token is
/// meaningless without it). `home_space_id` is the daemon's home space,
/// supplied by the daemon for the `Home` fallback (`None` ⟹ the daemon has no
/// space identity to bind to → refused).
///
/// A supplied `TargetSpace::Id` must be a REGISTERED space (the window can't
/// conjure one) — validated through `registry`. On success the window is
/// registered as a lifecycle actor carrying its pid, and the session is minted
/// through `sessions`.
pub async fn register(
    sessions: &dyn WindowSessions,
    registry: &dyn SpaceRegistry,
    pid: Option<u32>,
    target: TargetSpace<'_>,
    home_space_id: Option<&str>,
) -> Result<RegisteredWindow, DeviceError> {
    let Some(pid) = pid else {
        return Err(DeviceError::forbidden(
            "window registration requires a kernel-attested pid (host socket only)",
        ));
    };

    let space_id = resolve_target_space(registry, target, home_space_id)?;

    let actor_id = ship_substrate::gen_ulid();
    // `register_window` does a durable event-store append (`block_on`), so it
    // must run off the async worker — same constraint as the reaper.
    let aid = actor_id.clone();
    let sid = space_id.clone();
    tokio::task::spawn_blocking(move || actor_events::register_window(&aid, &sid, pid))
        .await
        .map_err(|e| DeviceError::service(format!("register window join: {e}")))?
        .map_err(|e| DeviceError::service(format!("register window actor: {e}")))?;

    let session_token = sessions
        .mint_window_session(&space_id, pid)
        .await
        .map_err(|e| DeviceError::service(format!("token generation: {e}")))?;

    Ok(RegisteredWindow {
        session_token,
        actor_id,
        space_id,
    })
}

/// Resolve `target` to a concrete, registered space id.
fn resolve_target_space(
    registry: &dyn SpaceRegistry,
    target: TargetSpace<'_>,
    home_space_id: Option<&str>,
) -> Result<String, DeviceError> {
    match target {
        TargetSpace::Id(id) => match registry.space_exists(id) {
            Ok(true) => Ok(id.to_string()),
            Ok(false) => Err(DeviceError::not_found(format!(
                "no registered space `{id}` to bind the window to"
            ))),
            Err(e) => Err(DeviceError::service(format!("space lookup failed: {e}"))),
        },
        TargetSpace::Home => home_space_id.map(str::to_string).ok_or_else(|| {
            DeviceError::conflict("daemon has no space identity to bind the window to")
        }),
    }
}

/// Close a window: revoke its pid-bound token then close its lifecycle.
///
/// `actor_id` must name a `window`-kind actor. Revokes every session matching
/// the window's `(space, pid)` (instant authority cut) via `sessions`, then
/// deregisters the window actor (durable lifecycle close). The OS frame is the
/// shell's to destroy; revocation neuters the webview regardless.
pub async fn close(
    sessions: &dyn WindowSessions,
    actor_id: &str,
) -> Result<ClosedWindow, DeviceError> {
    let Some(window) =
        actor::get_actor(actor_id).map_err(|e| DeviceError::service(e.to_string()))?
    else {
        return Err(DeviceError::bad_args(format!("no window actor `{actor_id}`")));
    };
    if window.kind != "window" {
        return Err(DeviceError::bad_args(format!(
            "actor `{actor_id}` is not a window (kind={})",
            window.kind
        )));
    }

    // A window is single-instance per space; revoking by (space, pid) targets
    // exactly this window's token. Both must be present to revoke anything.
    let token_revoked = match (window.space_id.as_deref(), window.pid) {
        (Some(space_id), Some(pid)) => sessions.revoke_window_session(space_id, pid as u32).await,
        _ => false,
    };

    // Close the lifecycle (off-thread: durable event append uses block_on).
    if let Some(space_id) = window.space_id.clone() {
        let aid = actor_id.to_string();
        tokio::task::spawn_blocking(move || actor_events::deregister_window(&aid, &space_id))
            .await
            .map_err(|e| DeviceError::service(format!("deregister join: {e}")))?
            .map_err(|e| DeviceError::service(e.to_string()))?;
    }

    Ok(ClosedWindow {
        closed: true,
        token_revoked,
    })
}

#[cfg(test)]
mod tests;
