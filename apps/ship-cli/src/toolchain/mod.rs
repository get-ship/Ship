//! `~/.ship/runtime/` sui-compiler provisioning.
//!
//! Moved from `ship_substrate::sui_compiler` (Wave-2 topology cleanup):
//! toolchain provisioning is app-concern logic with a single consumer
//! (ship-cli), so Rule C2 applies — module, not a separate crate or a
//! substrate concern. ship-substrate carries zero toolchain knowledge.
//!
//! - `ensure` — existence + executability checks.
//! - `install` — populate the runtime dir from local sources.

pub mod ensure;
pub mod install;

pub use ensure::{Result, ToolchainError, check_runtime_dir, check_sui_compiler, ensure_present};
pub use install::{InstallError, InstallOptions, install, update};
