//! `ship-device` — the device/isolation + window service over substrate ports (L1).
//!
//! The space carve split window management out of the space service into this
//! device topic (the space service "loses window-mgmt → device"). This crate
//! owns the window-management *mechanism* the daemon used to hold inline:
//!
//!   - **window control** ([`windows`]) — intent (focus-vs-spawn), list,
//!     register (validate space + register actor + mint session) and close
//!     (revoke token + deregister). Live-window reads hit the actor projection;
//!     register/close emit `actor.created` / `actor.stopped` through the
//!     event-sourced actor-events write path. The session mint/revoke seam and
//!     the space-existence check are injected as [`ports`].
//!
//! Authority checks (operator-only register, the C3 close gate, pid attestation)
//! stay at the transport — this crate owns the mechanism, the daemon owns the
//! policy decision and supplies the kernel-attested pid.
//!
//! No sqlite/sqlx/rusqlite types and no `std::fs`/`tokio::fs` anywhere in this
//! crate: storage is event-sourced through ship-substrate's actor projection +
//! actor-events path, and the daemon-owned capabilities are injected as ports.

pub mod error;
pub mod ports;
pub mod windows;

pub use error::{DeviceError, DeviceErrorKind};
pub use ports::{SpaceRegistry, WindowSessions};
pub use windows::{ClosedWindow, RegisteredWindow, TargetSpace, WindowIntent};
