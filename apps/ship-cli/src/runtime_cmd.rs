//! `ship runtime ...` — thin transport over `ship_application::toolchain::*`.
//!
//! All policy lives in the runtime crate; this file only translates clap
//! enums into call arguments and prints results. New behaviors belong in
//! `crates/core/ship-application/src/toolchain/` so the MCP transport can reuse
//! them without duplicating logic.

use std::path::PathBuf;

use anyhow::Result;

use ship_substrate::ShipHome;
use ship_application::toolchain::{InstallOptions, ensure_present, install, update};

pub fn run_install(bun: Option<PathBuf>, sui_compiler: Option<PathBuf>) -> Result<()> {
    let home = ShipHome::from_env();
    let opts = InstallOptions {
        bun_source: bun,
        sui_compiler_source: sui_compiler,
        bundler_ts_source: None,
        clean: false,
    };
    install(&home, opts)?;
    println!("ship runtime: installed at {}", home.runtime_dir().display());
    Ok(())
}

pub fn run_update(bun: Option<PathBuf>, sui_compiler: Option<PathBuf>) -> Result<()> {
    let home = ShipHome::from_env();
    let opts = InstallOptions {
        bun_source: bun,
        sui_compiler_source: sui_compiler,
        bundler_ts_source: None,
        clean: true,
    };
    update(&home, opts)?;
    println!("ship runtime: updated at {}", home.runtime_dir().display());
    Ok(())
}

pub fn run_doctor() -> Result<()> {
    let home = ShipHome::from_env();
    match ensure_present(&home) {
        Ok(()) => {
            println!("ship runtime: healthy — {} ✓", home.runtime_dir().display());
            Ok(())
        }
        Err(e) => Err(anyhow::Error::new(e)),
    }
}
