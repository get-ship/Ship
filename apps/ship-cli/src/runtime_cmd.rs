//! `ship runtime ...` — thin transport over `crate::toolchain::*`.
//!
//! All policy lives in `crate::toolchain`; this file only translates
//! clap enums into call arguments and prints results. New behaviors belong
//! in the toolchain module (Wave-2 topology move: previously
//! `ship_substrate::sui_compiler`, now a ship-cli–local module per Rule C2).

use std::path::PathBuf;

use anyhow::Result;

use ship_substrate::ShipHome;
use crate::toolchain::{InstallOptions, ensure_present, install, update};

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
