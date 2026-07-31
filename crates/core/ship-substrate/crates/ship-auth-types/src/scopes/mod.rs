//! Session registry — the in-memory session handle and its constructors.
//! The L0 seam: the `DaemonSession` value type, the `SessionRegistry` handle,
//! session-token mint/parse, and the operator/space-window constructors.
//!
//! What stays L1 in `ship_platform::auth::scopes`:
//! - `account_store` (substrate event-store writer helper),
//! - `provision_grants` / `persist_session` / `remove_persisted_session`
//!   (event-store writes),
//! - `hydrate_sessions` / `resolve_grants` / `resolve_grants_at` /
//!   `resolve_from` (projection reads).

mod session;

#[cfg(test)]
mod tests;

pub use session::{
    drop_session, generate_session_token, hex_encode, is_session_token, mint_session,
    operator_session, space_window_session, DaemonSession, SessionRegistry,
};
