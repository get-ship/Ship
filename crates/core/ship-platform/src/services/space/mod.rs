//! Space service (L1) — owns space product logic the daemon used to hold inline.
//!
//! A space = the unit of isolation / coordination / config (one `.ship/`
//! directory). This service owns:
//!
//!   - **info** ([`info`]) — identity (id/root/name from the projects
//!     projection) + the effective-settings overlay (global ⟵ space). The
//!     daemon supplies its global settings snapshot; the overlay is here.
//!   - **per-space config** ([`config`]) — read the event-sourced override map
//!     and write a new override (validated, then emitted through the
//!     [`ports::SpaceEvents`] seam).
//!   - **window control** ([`windows`]) — intent (focus-vs-spawn), list,
//!     register (validate space + register actor + mint session) and close
//!     (revoke token + deregister), with the session mint/revoke seam injected
//!     as [`ports::WindowSessions`].
//!
//! The daemon handlers (`space_rpc.rs`, `command/space.rs`,
//! `auth/space_window.rs`) collapse to marshal + serialize over this service.
//! Authority checks (operator-only register, the C3 close gate) stay at the
//! transport — this service owns the mechanism, the daemon owns the policy
//! decision and supplies the kernel-attested pid.

pub mod config;
pub mod info;
pub mod ports;
pub mod windows;

pub use config::SettingValue;
pub use info::SpaceInfo;
pub use ports::{SpaceEvents, WindowSessions};
pub use windows::{ClosedWindow, RegisteredWindow, TargetSpace, WindowIntent};

/// Transport-agnostic error kind. The daemon maps each to its own wire shape
/// (REST status code, Connect error code) — L1 carries no web-framework types,
/// so the *kind* is the contract. Mirrors the variants the daemon's
/// `CommandError` distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceErrorKind {
    /// Invalid request input (→ 400).
    BadArgs,
    /// The named entity does not exist (→ 404).
    NotFound,
    /// The caller is not permitted (→ 403).
    Forbidden,
    /// A required precondition is unmet, e.g. no space identity (→ 409).
    Conflict,
    /// An internal failure (→ 500).
    Service,
}

/// Structured service error carrying a transport-agnostic [`SpaceErrorKind`] +
/// a message. The daemon's marshal layer maps `kind` to its status code.
#[derive(Debug, Clone)]
pub struct SpaceServiceError {
    pub kind: SpaceErrorKind,
    pub message: String,
}

impl SpaceServiceError {
    pub fn bad_args(msg: impl Into<String>) -> Self {
        Self { kind: SpaceErrorKind::BadArgs, message: msg.into() }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self { kind: SpaceErrorKind::NotFound, message: msg.into() }
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self { kind: SpaceErrorKind::Forbidden, message: msg.into() }
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self { kind: SpaceErrorKind::Conflict, message: msg.into() }
    }
    pub fn service(msg: impl Into<String>) -> Self {
        Self { kind: SpaceErrorKind::Service, message: msg.into() }
    }
}

impl std::fmt::Display for SpaceServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for SpaceServiceError {}

#[cfg(test)]
mod tests;
