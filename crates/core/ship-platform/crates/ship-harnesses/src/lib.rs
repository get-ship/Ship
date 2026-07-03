//! ship-harnesses — L1 harness registry.
//!
//! Concrete adapter structs (ClaudeAdapter, CodexAdapter,
//! OpenCodeAdapter, PiAdapter) + the resolver helpers (adapter_for,
//! capabilities, REGISTRY). The HarnessAdapter trait lives in
//! harness_adapter (L1 leaf).
//!
//! Depends only on ship-substrate (L0) + ship-runner (L0). No ship-platform dep.

pub mod harness_adapter;
pub mod harnesses;

pub use harness_adapter::{
    adapter_for, adapter_for_or_default, capabilities,
};
pub use harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, PushMechanism, ResumeError,
};
