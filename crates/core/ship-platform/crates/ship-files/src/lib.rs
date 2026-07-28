//! `ship-files` — L1 filesystem helpers carved from ship-platform/services.
//!
//! Provides [`files::FilesService`], JSONC utilities, and path-tolerant
//! RFC 6902 support. Depends only on `ship-substrate` (L0) and std — no
//! ship-platform dependency.

pub mod files;
pub mod jsonc;
pub mod path_tolerant;
