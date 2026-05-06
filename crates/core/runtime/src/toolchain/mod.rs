//! `~/.ship/runtime/` toolchain orchestration.
//!
//! - `ensure` — existence + executability checks consumed by `ship dev` /
//!   `ship publish` before they shell to bun.
//! - `install` — populate the toolchain dir from local sources (Bun
//!   binary, sui-compiler binary, pinned package.json). Future revisions
//!   will fetch from a CDN; the current implementation pulls from local
//!   paths so dev iteration doesn't need a release pipeline.

pub mod build;
pub mod ensure;
pub mod install;

pub use build::{BuildError, BuildOptions};
pub use ensure::{Result, ToolchainError, check_bun, check_node_modules, check_package_json,
    check_runtime_dir, check_sui_compiler, ensure_present};
pub use install::{InstallError, InstallOptions, install, update};

pub fn build(home: &ShipHome, opts: BuildOptions) -> std::result::Result<(), BuildError> {
    build::run(home, opts)
}

use crate::ShipHome;
