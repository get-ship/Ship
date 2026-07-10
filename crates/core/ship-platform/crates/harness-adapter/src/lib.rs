//! harness-adapter — the L1 contract every third-party agent harness implements.
//!
//! `HarnessAdapter` is the seam ship-code drives and ship-harnesses implements;
//! `LaunchCommand`/`LaunchProfile` and `ServerSession` are its value types. It's
//! a contract leaf below both ship-harnesses and ship-code so each reaches it
//! without a cycle — and so harness concerns live in L1, not stranded in the
//! substrate (they used to sit alongside the L0 substrate facade only
//! to dodge that cycle).

pub mod dispatch_ports;
pub mod harness_adapter;
pub mod server_session;

pub use dispatch_ports::{LaunchCommand, LaunchProfile};
pub use harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, PushMechanism, ResumeError, SubmitMode,
};
pub use server_session::{ServerSession, SessionEvent, StubSession};
