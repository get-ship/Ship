//! Identity primitives — pairing flow, token attestation, scope policy,
//! session registry, and cookie-backed browser sessions.
//!
//! Lives in `ship-platform` so any future consumer (Cloud, alternate
//! transports, MCP-side authentication) can reach the same primitives
//! without depending on the daemon. The HTTP layer that exposes these
//! over `/api/auth/*` stays in `apps/shipd/src/auth/`.

pub mod attest;
pub mod grant_diff;
pub mod pairing;
pub mod peer_identity;
pub mod projection;
pub mod scopes;
pub mod session;
pub mod sweeper;

pub use projection::{
    AuthSessionRow, AuthSessionsProjection, DaemonOwnerProjection, DaemonOwnerRow,
};

#[cfg(test)]
mod tests;
