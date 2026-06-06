//! Re-export seam — the L0 browser-session cookie helpers live in
//! `ship_auth_types::session` (P2 carve). This module keeps the
//! `ship_platform::auth::session::*` import paths stable for the
//! daemon's HTTP layer and tests.
pub use ship_auth_types::session::*;
