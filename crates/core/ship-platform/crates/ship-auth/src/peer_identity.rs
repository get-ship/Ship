//! Re-export seam — the L0 peer-identity types and resolver live in
//! `ship_auth_types::peer_identity` (P2 carve). This module keeps the
//! `ship_platform::auth::peer_identity::*` import paths stable for the
//! rest of the platform and the daemon.
pub use ship_auth_types::peer_identity::*;
