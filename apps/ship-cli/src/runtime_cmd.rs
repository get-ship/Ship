//! `ship runtime ...` — thin transport over `ship_substrate::sui_compiler::*`.
//!
//! All policy lives in `ship_substrate::sui_compiler`; this file only translates
//! clap enums into call arguments and prints results. New behaviors belong
//! alongside the substrate sui_compiler module so the MCP transport can reuse
//! them without duplicating logic.

use std::path::PathBuf;

use anyhow::Result;

use ship_substrate::ShipHome;
use ship_substrate::sui_compiler::{InstallOptions, ensure_present, install, update};

pub fn run_install(sui_compiler: Option<PathBuf>) -> Result<()> {
    let home = ShipHome::from_env();
    let opts = InstallOptions { sui_compiler_source: sui_compiler, clean: false };
    install(&home, opts)?;
    println!("ship runtime: installed at {}", home.runtime_dir().display());
    Ok(())
}

pub fn run_update(sui_compiler: Option<PathBuf>) -> Result<()> {
    let home = ShipHome::from_env();
    let opts = InstallOptions { sui_compiler_source: sui_compiler, clean: true };
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
