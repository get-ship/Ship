//! Bearer token authentication for the shipd daemon.
//!
//! On startup the daemon generates a random token and writes it to
//! `~/.ship/machine/daemon/token`. All HTTP endpoints require the token in the
//! `Authorization: Bearer <token>` header. Clients read the token file
//! and include it in requests.
//!
//! Auth-domain submodules live in `apps/shipd/src/auth/` per
//! modular-runtime-04: scopes, session, attest, plus the split-out
//! `context` (AuthContext + binding minter), `middleware` (the bearer/cookie
//! middleware), and `dangerous` (the contributor-only relaxed-auth flag). Their
//! tests sit alongside. Re-exports at this root keep every historical
//! `crate::auth::X` path stable.

// Pure auth primitives live in `ship_platform::auth`. This re-export
// preserves the `crate::auth::attest` path the daemon uses.
pub use ship_auth::attest;

/// Contributor-only relaxed-auth flag (`--dangerous`). Stripped from shipped
/// (release) builds; never on by default.
pub mod dangerous;
/// The one capability gate — a pure `decide(blast, service, caller)` plus the
/// thin `caller_from_auth` adapter onto [`AuthContext`]. shipd-local; invoked by
/// the `GateFn` closure in `rpc::ops::router`.
pub mod gate;
pub mod identity;
/// Host unix socket — per-agent peer-credential auth. Unix-only: relies on
/// `SO_PEERCRED` and `UnixListener`. Windows uses a named-pipe path (future).
#[cfg(unix)]
pub mod peer_socket;
pub mod scopes;
pub mod session;

mod context;
mod middleware;
mod token;

#[cfg(test)]
mod auth_tests;

pub use context::AuthContext;
pub use middleware::{auth_middleware, project_authority_middleware};
pub use token::{generate_token, is_daemon_token, read_or_generate_token, read_token, write_token_file};

pub use crate::auth::identity::enforce_mcp_identity;

// Re-export the session surface from the split-out module.
pub use crate::auth::scopes::{
    DaemonSession, SessionRegistry, generate_session_token, hydrate_sessions, is_session_token,
    persist_session, remove_persisted_session, spawn_session_sweeper,
};

// HTTP handlers split out per mr-04 file-size enforcement. Re-exported
// at the auth root so callers (router.rs) keep using `crate::auth::attest`
// etc. without churn.
pub mod api;
pub use api::*;
