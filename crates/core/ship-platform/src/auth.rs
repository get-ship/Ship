//! Auth primitives — session lifecycle, attestation, pairing, scope policy,
//! peer identity, and grant resolution.
//!
//! Implementation is in the `ship-auth` crate (L1, no ship-platform dep).
//! This module is a thin re-export seam so all existing `ship_platform::auth::*`
//! import paths remain stable.

pub use ship_auth::attest;
pub use ship_auth::grant_diff;
pub use ship_auth::pairing;
pub use ship_auth::peer_identity;
pub use ship_auth::projection;
pub use ship_auth::scopes;
pub use ship_auth::session;
pub use ship_auth::sweeper;

pub use ship_auth::{
    AuthSessionRow, AuthSessionsProjection, DaemonOwnerProjection, DaemonOwnerRow,
    active_session, current_owner, list_active_sessions,
};
