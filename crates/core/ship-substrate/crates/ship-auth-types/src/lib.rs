//! L0 auth primitives — session/registry types, kernel-verified peer identity.
//!
//! Pure types + token minting; no projection reads, no event-store writes.
//! The L1 policy (grant resolution, session persistence/hydration, the
//! expired-session sweeper, the attestation JWT verifier) stays in
//! `ship-platform::auth` and re-exports these types so existing
//! `ship_platform::auth::scopes::DaemonSession` import paths remain stable.
//!
//! ## Layer split
//!
//! - **L0 (here)**: the `DaemonSession` value type, the `SessionRegistry`
//!   handle, the kernel-attested `PeerIdentity` resolution, and the
//!   session-token generators (cookie / opaque / browser). The host-OS read
//!   (`pid → container`) lives in `ship-substrate-host`; this crate composes
//!   the kernel-verified result with the resolution policy.
//! - **L1 (`ship-platform::auth`)**: reads projections
//!   (`crate::projections::{auth,provisioned_grants}`) to compute the
//!   granted grant set, persists sessions, runs the sweeper, verifies
//!   attestation JWTs. These are not L0 — they touch the event store and
//!   the projection layer.

pub mod peer_identity;
pub mod scopes;
pub mod session;

// Root-level re-exports so callers can write `ship_auth_types::DaemonSession`
// rather than the longer module path. The re-exports are additive — the
// module paths still resolve.
pub use peer_identity::{resolve as resolve_peer_identity, PeerIdentity};
pub use scopes::{
    drop_session, generate_session_token, hex_encode, is_session_token, mint_session,
    operator_session, space_window_session, DaemonSession, SessionRegistry,
};
