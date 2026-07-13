//! ship-harnesses — L1 third-party-harness driver registry.
//!
//! Concrete adapter structs (ClaudeAdapter, CodexAdapter,
//! OpenCodeAdapter, PiAdapter) + the resolver helpers (adapter_for,
//! capabilities, REGISTRY). The `AgentDriver` trait lives in the
//! `agent_driver` crate (L1 contract leaf).
//!
//! Depends only on ship-substrate (L0; the L0 dispatch gate + emit helpers
//! live directly in `ship_substrate`). No ship-platform dep.

pub mod agent_driver;
pub mod harnesses;

pub use agent_driver::{
    adapter_for, adapter_for_or_default, capabilities,
};
pub use agent_driver::{
    AgentDriver, HarnessCapabilities, HarnessEvent, PushMechanism, ResumeError,
};
