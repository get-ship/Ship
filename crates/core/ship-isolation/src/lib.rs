//! Agent-process isolation primitives.
//!
//! **Status: experimental spike.** Lives in its own crate with no
//! dependents from `ship-substrate` / `ship-platform` / `ship-supervisor`.
//! Integration into the orchestrator is a separate branch and a separate
//! conversation. See `.artifacts/local-sandbox-spec/` for the research
//! and design intent that produced this crate.
//!
//! ## Scope
//!
//! Sandboxes the *agent process itself* — the harness binary spawned by
//! the orchestrator. Distinct from `IsolationStrategy` in
//! `ship_substrate::services::dispatch_ports`, which prepares the
//! working *directory* (git worktree) before the process runs. The two
//! compose: prepare worktree → wrap the process in an `Isolator`.
//!
//! ## Backends
//!
//! - [`backends::Noop`] — runs unsandboxed. Default; matches today.
//! - [`backends::SandboxRuntime`] — wraps the Anthropic `sandbox-runtime`
//!   npm package via a Bun subprocess. Composing/wrapping backend.
//! - [`backends::Youki`] (feature `youki-backend`, Linux only) — runs
//!   the agent inside an OCI container managed by `youki`. Forward-
//!   looking experiment.

pub mod backends;
pub mod handle;
pub mod helpers;
pub mod policy;
pub mod sidecar;

mod trait_def;

pub use handle::{ExecHandle, SandboxHandle, SandboxId};
pub use helpers::EphemeralFile;
pub use policy::{FsMount, MountMode, NetworkPolicy, SandboxPolicy, ResourceLimits};
pub use trait_def::{Isolator, IsolatorError, LaunchCommand};
