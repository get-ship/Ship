//! Process-spawn isolation primitives.
//!
//! The trait + types every backend implements. Substrate owns the
//! abstraction because the lifecycle binds to `ActorHandle` — when an
//! actor is dropped, its sandbox is dropped too. The concrete
//! strategies (Docker, youki, srt, Fly, etc.) live in `ship-isolation`
//! at L1, alongside the capability probe and recommender.
//!
//! See `crates/core/ship-platform/crates/ship-isolation/docs/strategy.md` for the full design;
//! this module is just the abstraction surface that lets actor handles
//! own a sandbox without depending on which one.

pub mod handle;
pub mod noop;
pub mod owner;
pub mod policy;
pub mod trait_def;

pub use handle::{SandboxHandle, SandboxId};
pub use noop::Noop;
pub use owner::SandboxOwner;
pub use policy::{FsMount, MountMode, NetworkPolicy, ResourceLimits, SandboxPolicy};
pub use trait_def::{Isolator, IsolatorError, LaunchCommand};
