//! `agent-driver` — the L1 contract every agent runtime implements.
//!
//! `AgentDriver` is the seam ship-code drives and every third-party harness
//! runtime (the structs in `ship-harnesses`) implements, alongside Floki's
//! in-process native driver. `LaunchCommand`/`LaunchProfile` and
//! `ServerSession` are its value types. It's a contract leaf below both
//! `ship-harnesses` and ship-code so each reaches it without a cycle, and
//! so harness concerns live in L1, out of the substrate.

pub mod agent_driver;
pub mod dispatch_ports;
pub mod server_session;

pub use dispatch_ports::{LaunchCommand, LaunchProfile};
pub use agent_driver::{
    AgentDriver, HarnessCapabilities, HarnessEvent, PushMechanism, ResumeError, SubmitMode,
};
pub use server_session::{ServerSession, SessionEvent, StubSession};
