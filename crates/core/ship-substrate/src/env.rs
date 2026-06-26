//! Typed `SHIP_*` environment-variable surface.
//!
//! The canonical definition now lives in the compiler-free `ship-paths` leaf
//! crate so leaf crates can read the typed env surface without the substrate
//! frontier. This module re-exports it verbatim, so every existing
//! `ship_substrate::env::*` and internal `crate::env::*` consumer keeps
//! compiling unchanged. See ADR `home-layout-and-scope-convention`.

pub use ship_paths::env::*;
