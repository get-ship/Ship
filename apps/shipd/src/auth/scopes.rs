//! shipd-side re-exports of the auth session surface.
//!
//! Session-registry primitives + grant resolution live in `ship_auth::scopes`.
//! This module re-exports them so existing `crate::auth::scopes::Foo` import
//! paths continue to resolve.

pub use ship_auth::scopes::{
    DaemonSession, SessionRegistry, drop_session, generate_session_token, hydrate_sessions,
    is_session_token, mint_session, persist_session, remove_persisted_session,
};
pub use ship_auth::sweeper::spawn_session_sweeper;
