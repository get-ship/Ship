//! Ship-specific isolation strategies.
//!
//! The abstraction (the `Isolator` trait + `SandboxPolicy` + the
//! lifetime-bound `SandboxHandle`) lives in `isolation_port`
//! so an `ProcessHandle` can own its sandbox. This crate provides the
//! concrete L1 pieces:
//!
//! - **Backends** — Docker / Podman, youki rootless, sandbox-runtime,
//!   sidecar, etc. Each is gated by a Cargo feature.
//! - **Capabilities** — host probe for which primitives are usable.
//! - **Tier + Strategy** — the user-facing taxonomy.
//! - **Recommender** — maps `(capabilities, overrides) → strategy` with
//!   per-OS defaults and honest downgrade reasons.
//! - **Registry** — `IsolationStrategy → Arc<dyn Isolator>` factory map.
//! - **Settings** — env-var precedence + "auto" sentinel resolution +
//!   launch-wrap helpers.
//!
//! See `docs/strategy.md` for the design intent.

pub mod backends;
pub mod wire;
pub mod capabilities;
pub mod decision;
pub mod decision_ops;
pub mod harness;
pub mod helpers;
pub mod net;
pub mod recommender;
pub mod runtime;
pub mod settings;
pub mod sidecar;
pub mod strategies_config;
pub mod tier;

pub use capabilities::{Capabilities, HostOs};
pub use helpers::EphemeralFile;
pub use recommender::{
    RecommendInput, RecommenderError, ResolutionReason, ResolvedIsolation, recommend,
};
pub use tier::{IsolationProfile, IsolationStrategy, IsolationTier};

// Re-export the substrate-owned abstraction so existing callers using
// `ship_isolation::Isolator` etc. don't change. These are the same
// types — substrate is the canonical location now because ProcessHandle
// owns the sandbox lifetime.
pub use isolation_port::{
    FsMount, Isolator, IsolatorError, LaunchCommand, MountMode, NetworkPolicy,
    ResourceLimits, SandboxHandle, SandboxId, SandboxPolicy,
};

/// `policy` was a top-level module here pre-L0-move. Callers that
/// imported `ship_isolation::policy::SandboxPolicy` still work via this
/// re-export; new code should use `ship_isolation::SandboxPolicy` (or
/// `isolation_port::SandboxPolicy`) directly.
pub mod policy {
    pub use isolation_port::{
        FsMount, MountMode, NetworkPolicy, ResourceLimits, SandboxPolicy,
    };
}
