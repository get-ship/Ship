//! Project layout, identity, discovery, registry, global state, and init.
//!
//! L0 surface (discovery, paths, identity, etc.) re-exported from ship-substrate.
//! `init_project` and `init_project_with_registry` are L1 — they live here
//! because they depend on config schemas and the package registry.

pub use ship_substrate::project::*;

mod init;
pub use init::{InitWithRegistryResult, init_project, init_project_with_registry};
