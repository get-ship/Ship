//! `ship-auth` — L1 auth policy crate.
//!
//! Carved from `ship-platform/src/auth/`. Depends only on L0 crates
//! (`ship-substrate`, `ship-auth-types`, `ship-types`). No dependency on
//! `ship-platform`.
//!
//! ## Modules
//! - `scopes` — session types, grant resolution, session persistence/hydration
//! - `projection` — `auth_sessions` + `daemon_owner` SQLite projections
//! - `pairing` — in-memory pairing registry (CLI↔Studio)
//! - `attest` — JWT attestation + daemon-owner binding
//! - `sweeper` — background session expiry sweep
//! - `peer_identity` — re-export from `ship_auth_types::peer_identity`
//! - `session` — re-export from `ship_auth_types::session`
//! - `grant_diff` — pending-grant diff helpers

pub mod attest;
pub mod grant_diff;
pub mod pairing;
pub mod peer_identity;
pub mod projection;
pub mod scopes;
pub mod session;
pub mod sweeper;

#[cfg(any(test, feature = "testutil"))]
pub mod testutil;

#[cfg(test)]
mod tests;

// Stable surface re-exports — callers that previously used
// `ship_platform::auth::*` continue to work through ship-platform's
// own re-export (`pub use ship_auth as auth`).
pub use projection::{
    AuthSessionRow, AuthSessionsProjection, DaemonOwnerProjection, DaemonOwnerRow,
    active_session, current_owner, list_active_sessions,
};
pub use scopes::{
    DaemonSession, McpAuth, RequiredAuth, Scope, SessionRegistry, account_store,
    drop_session, generate_session_token, grants_for_role, hex_encode, hydrate_sessions,
    is_session_token, mint_session, operator_session, persist_session, provision_grants,
    remove_persisted_session, resolve_grants, resolve_grants_at, scopes_for_role,
    session_registry, set_session_registry, space_window_scopes, space_window_session,
};
