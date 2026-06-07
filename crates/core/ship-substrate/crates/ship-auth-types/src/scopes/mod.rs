//! Scope model + session registry — pure capability types and the
//! in-memory session handle. The L0 seam: role→scope tables, the
//! `Scope`/`DaemonSession` value types, the `SessionRegistry` handle,
//! session-token mint/parse, and the operator/space-window constructors.
//!
//! The file is split into cohesive submodules:
//! - [`types`]: `Scope`, `McpAuth`, `RequiredAuth` (the value types).
//! - [`roles`]: the role→scope / role→grant tables (`scopes_for_role`,
//!   `grants_for_role`, plus the `pipeline_agent_artifact_scopes` helper).
//! - [`session`]: `DaemonSession`, the `SessionRegistry` handle, the
//!   process-wide registry singleton, session-token mint/parse, the
//!   operator/space-window/mint constructors, and `hex_encode`.
//!
//! `pub use` re-exports below preserve the flat
//! `ship_auth_types::scopes::Foo` paths callers relied on before the
//! split — the submodule split is a layout change, not an API change.
//!
//! What stays L1 in `ship_platform::auth::scopes`:
//! - `account_store` (substrate event-store writer helper),
//! - `provision_grants` / `persist_session` / `remove_persisted_session`
//!   (event-store writes),
//! - `hydrate_sessions` / `resolve_grants` / `resolve_grants_at` /
//!   `resolve_from` (projection reads).
//!
//! `scopes_for_role` and `grants_for_role` live here (they are pure
//! role tables, no I/O); the projection-aware `resolve_grants` calls
//! them as the role fallback in L1.

mod roles;
mod session;
mod types;

#[cfg(test)]
mod tests;

pub use roles::{grants_for_role, scopes_for_role};
pub use session::{
    drop_session, generate_session_token, hex_encode, is_session_token, mint_session,
    operator_session, session_registry, set_session_registry, space_window_scopes,
    space_window_session, DaemonSession, SessionRegistry,
};
pub use types::{McpAuth, RequiredAuth, Scope};
