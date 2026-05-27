//! `~/.ship/runtime/` toolchain orchestration.
//!
//! - `ensure` — existence + executability checks.
//! - `install` — populate the toolchain dir from local sources. Since #163 the
//!   daemon bundles in-process via Rolldown, so the only component is the
//!   sui-compiler binary. Future revisions may fetch from a CDN.

pub mod ensure;
pub mod install;

pub use ensure::{Result, ToolchainError, check_runtime_dir, check_sui_compiler, ensure_present};
pub use install::{InstallError, InstallOptions, install, update};
